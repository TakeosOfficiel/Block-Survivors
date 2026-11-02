//! Équipe (coopération, src/reseau/hote.ts et combattant.ts) : l'hôte simule les autres
//! joueurs dans sa propre partie. Chacun a son état, ses bonus, son arsenal, ses talents,
//! son équipement, ses systèmes d'armes et son champ de flux ; ils sont rangés dans un
//! `Combattant` et échangés avec ceux de la partie le temps de le faire jouer
//! (`echanger`, deux fois rend l'état d'origine), si bien que tous les systèmes écrits
//! pour un joueur servent tels quels. Les ennemis visent le plus proche joueur debout
//! qui ne choisit pas ses cartes ; l'XP et le niveau sont communs, chacun choisit ses
//! cartes ; un joueur à terre est relevé par un allié resté près de lui ; la partie est
//! perdue quand toute l'équipe est à terre. Seul, la partie est exactement la même.

use std::mem::swap;

use bs_donnees::entites::Personnage;
use bs_monde::ChampDeFlux;

use crate::combat::{Arsenal, SystemesArmes};
use crate::equipement::Equipement;
use crate::etat::{EtatJoueur, EtatPartie, Modificateurs, Statistiques};
use crate::joueur::Joueur;
use crate::partie::Partie;
use crate::passifs::Passifs;
use crate::talents::Talents;

/// Règles de la coopération (src/donnees/multijoueur.ts `REGLES_MULTIJOUEUR`).
pub mod regles {
    /// Joueurs au plus dans une partie.
    pub const JOUEURS_MAX: usize = 4;
    /// Par joueur en plus : rythme des apparitions, PV des ennemis, PV des boss.
    pub const APPARITIONS: f32 = 0.6;
    pub const PV_ENNEMIS: f32 = 0.25;
    pub const PV_BOSS: f32 = 0.7;
    /// Réanimation : s à terre avant de ne plus pouvoir être relevé, rayon de l'allié,
    /// s à rester près de lui, part des PV rendue, s d'invincibilité.
    pub const A_TERRE: f32 = 30.0;
    pub const RAYON_REANIMATION: f32 = 1.6;
    pub const DUREE_REANIMATION: f32 = 3.0;
    pub const PV_REANIMATION: f32 = 0.3;
    pub const PROTECTION_REANIMATION: f32 = 2.0;
    /// Choix des cartes : s laissées (plus une marge), protection entretenue pendant le choix.
    pub const DELAI_CHOIX: f32 = 20.0;
    pub const MARGE_CHOIX: f32 = 10.0;
    pub const PROTECTION_CHOIX: f32 = 0.25;
    /// Les ennemis n'apparaissent pas à moins de ce rayon d'un joueur distant.
    pub const RAYON_VUE: f32 = 20.0;

    /// Multiplicateur pour `n` joueurs.
    pub fn facteur(par_joueur: f32, n: usize) -> f32 {
        1.0 + par_joueur * n.saturating_sub(1) as f32
    }
}

/// Un autre joueur, simulé par l'hôte. Les champs du haut sont échangés avec ceux de la
/// partie pendant qu'il joue.
pub struct Combattant {
    pub etat_joueur: EtatJoueur,
    pub modificateurs: Modificateurs,
    pub personnage: &'static Personnage,
    pub index_personnage: usize,
    /// Son bilan (les totaux d'équipe, XP, émeraudes, coffres, boss, sont partagés).
    pub stats: Statistiques,
    pub joueur: Joueur,
    pub passifs: Passifs,
    pub arsenal: Arsenal,
    pub talents: Talents,
    pub equipement: Equipement,
    pub armes: SystemesArmes,
    pub flux: ChampDeFlux,
    vies_traitees: u32,
    // --- Non échangés ---
    /// Cartes (niveaux et objets de coffre) qu'il lui reste à choisir.
    pub cartes_en_attente: u32,
    /// s passées à choisir.
    pub temps_choix: f32,
    /// En train de choisir : protégé, immobile, ignoré des ennemis.
    pub en_choix: bool,
    /// Parti en cours de partie : à terre pour de bon, hors du terrain.
    pub parti: bool,
}

/// Les autres joueurs et ce qui ne vaut qu'à plusieurs.
#[derive(Default)]
pub struct Equipe {
    pub membres: Vec<Combattant>,
    /// Combattant dont l'état est dans la partie (0 : l'hôte).
    actif: usize,
    /// L'hôte choisit ses cartes (la partie ne s'arrête plus pour lui).
    pub choix_hote: bool,
    /// Par joueur (0 : l'hôte), s passées près de lui par un allié pendant qu'il est à terre.
    pub reanimations: Vec<f32>,
    /// Rythme des apparitions selon le nombre de joueurs.
    pub facteur_apparitions: f32,
    /// Choix de coffre déjà distribués aux autres.
    coffres_vus: u32,
}

impl Equipe {
    /// Partie seule : l'hôte, rythme normal.
    pub fn nouvelle() -> Self {
        Self {
            reanimations: vec![0.0],
            facteur_apparitions: 1.0,
            ..Self::default()
        }
    }
}

/// Totaux d'équipe gardés d'un bilan à l'autre lors d'un échange.
fn partager(depuis: &Statistiques, vers: &mut Statistiques) {
    vers.xp_totale = depuis.xp_totale;
    vers.emeraudes_ramassees = depuis.emeraudes_ramassees;
    vers.coffres_ouverts = depuis.coffres_ouverts;
    vers.boss_vaincus = depuis.boss_vaincus;
}

impl Partie {
    /// Nombre de joueurs (l'hôte compris).
    pub fn nombre_joueurs(&self) -> usize {
        1 + self.equipe.membres.len()
    }

    /// Fait passer le combattant `k` (≥ 1) dans la partie, ou l'en retire s'il y est.
    pub fn echanger(&mut self, k: usize) {
        if k == 0 {
            return;
        }
        let c = &mut self.equipe.membres[k - 1];
        let e = &mut self.etat;
        swap(&mut e.joueur, &mut c.etat_joueur);
        swap(&mut e.modificateurs, &mut c.modificateurs);
        swap(&mut e.personnage, &mut c.personnage);
        swap(&mut e.index_personnage, &mut c.index_personnage);
        swap(&mut e.stats, &mut c.stats);
        partager(&c.stats, &mut e.stats);
        swap(&mut self.joueur, &mut c.joueur);
        swap(&mut self.passifs, &mut c.passifs);
        swap(&mut self.arsenal, &mut c.arsenal);
        swap(&mut self.talents, &mut c.talents);
        swap(&mut self.equipement, &mut c.equipement);
        swap(&mut self.armes, &mut c.armes);
        swap(&mut self.flux, &mut c.flux);
        swap(&mut self.vies_traitees, &mut c.vies_traitees);
        self.equipe.actif = if self.equipe.actif == k { 0 } else { k };
    }

    /// Fait jouer `f` au combattant `k` (0 : l'hôte).
    pub fn avec_combattant<R>(&mut self, k: usize, f: impl FnOnce(&mut Self) -> R) -> R {
        self.echanger(k);
        let r = f(self);
        self.echanger(k);
        r
    }

    /// État du joueur `k` (0 : l'hôte), hors d'un échange.
    pub fn etat_joueur(&self, k: usize) -> &EtatJoueur {
        if k == self.equipe.actif {
            &self.etat.joueur
        } else if k == 0 {
            &self.equipe.membres[self.equipe.actif - 1].etat_joueur
        } else {
            &self.equipe.membres[k - 1].etat_joueur
        }
    }

    pub fn etat_joueur_mut(&mut self, k: usize) -> &mut EtatJoueur {
        if k == self.equipe.actif {
            &mut self.etat.joueur
        } else if k == 0 {
            &mut self.equipe.membres[self.equipe.actif - 1].etat_joueur
        } else {
            &mut self.equipe.membres[k - 1].etat_joueur
        }
    }

    fn en_choix(&self, k: usize) -> bool {
        if k == 0 {
            self.equipe.choix_hote
        } else {
            self.equipe.membres[k - 1].en_choix
        }
    }

    /// Le plus proche des joueurs debout qui ne choisissent pas leurs cartes.
    pub fn plus_proche_joueur(&self, x: f32, z: f32) -> Option<usize> {
        let mut meilleur = None;
        let mut meilleure_d2 = f32::INFINITY;
        for k in 0..self.nombre_joueurs() {
            let j = self.etat_joueur(k);
            if j.mort || self.en_choix(k) {
                continue;
            }
            let d2 = (j.x - x) * (j.x - x) + (j.z - z) * (j.z - z);
            if d2 < meilleure_d2 {
                meilleure_d2 = d2;
                meilleur = Some(k);
            }
        }
        meilleur
    }

    /// Un joueur distant debout voit ce point (les ennemis n'y apparaissent pas).
    pub fn vu_par_un_autre(&self, x: f32, z: f32, rayon: f32) -> bool {
        let portee = regles::RAYON_VUE + rayon;
        (1..self.nombre_joueurs()).any(|k| {
            let j = self.etat_joueur(k);
            !j.mort && (j.x - x) * (j.x - x) + (j.z - z) * (j.z - z) < portee * portee
        })
    }

    /// Toute l'équipe est à terre.
    pub fn equipe_vaincue(&self) -> bool {
        (0..self.nombre_joueurs()).all(|k| self.etat_joueur(k).mort)
    }

    /// Un joueur debout est à moins de `rayon` de (x, z).
    pub fn joueur_a_portee(&self, x: f32, z: f32, rayon: f32) -> bool {
        (0..self.nombre_joueurs()).any(|k| {
            let j = self.etat_joueur(k);
            !j.mort && (j.x - x) * (j.x - x) + (j.z - z) * (j.z - z) <= rayon * rayon
        })
    }

    /// Coup d'un ennemi au joueur `k`.
    pub fn blesser_joueur_k(&mut self, k: usize, degats: f32, boss: bool) -> bool {
        self.avec_combattant(k, |p| p.blesser_joueur(degats, boss))
    }

    /// Ajoute un joueur (personnage `index_personnage`) près du départ ; son rang.
    pub fn ajouter_combattant(&mut self, index_personnage: usize) -> usize {
        let d = self.d;
        let neuf = EtatPartie::nouvel(d, self.etat.arene, index_personnage);
        let mut stats = neuf.stats;
        partager(&self.etat.stats, &mut stats);
        let k = self.nombre_joueurs();
        self.equipe.membres.push(Combattant {
            etat_joueur: neuf.joueur,
            modificateurs: neuf.modificateurs,
            personnage: neuf.personnage,
            index_personnage,
            stats,
            joueur: Joueur::default(),
            passifs: Passifs::nouveaux(neuf.personnage.bonus),
            arsenal: Arsenal::nouveau(),
            talents: Talents::nouveaux(d.armes.len()),
            equipement: Equipement::nouveau(d),
            armes: SystemesArmes::nouveaux(d),
            flux: ChampDeFlux::nouveau(&self.carte),
            vies_traitees: 0,
            cartes_en_attente: 0,
            temps_choix: 0.0,
            en_choix: false,
            parti: false,
        });
        self.avec_combattant(k, |p| {
            // Autour du point de départ, chacun de son côté
            let angle = k as f32 * std::f32::consts::FRAC_PI_2;
            let (x, z) = (
                p.etat.joueur.x + 1.5 * bs_commun::maths::sin(angle),
                p.etat.joueur.z + 1.5 * bs_commun::maths::cos(angle),
            );
            let (x, z) = p.carte.resoudre_collision(x, z, p.d.regles.joueur.rayon);
            p.etat.joueur.x = x;
            p.etat.joueur.z = z;
            p.preparer_joueur();
            // Même niveau que l'équipe
            let hote = p.etat_joueur(0);
            let (niveau, xp, xp_niveau) = (hote.niveau, hote.xp, hote.xp_niveau);
            let j = &mut p.etat.joueur;
            j.niveau = niveau;
            j.xp = xp;
            j.xp_niveau = xp_niveau;
        });
        let n = self.nombre_joueurs();
        self.equipe.reanimations.resize(n, 0.0);
        self.equipe.facteur_apparitions = regles::facteur(regles::APPARITIONS, n);
        self.ennemis.multiplicateur_pv = regles::facteur(regles::PV_ENNEMIS, n);
        self.ennemis.multiplicateur_pv_boss = regles::facteur(regles::PV_BOSS, n);
        k
    }

    /// Un joueur est parti : à terre pour de bon, hors du terrain, plus une cible.
    pub fn retirer_combattant(&mut self, k: usize) {
        let Some(c) = self.equipe.membres.get_mut(k.wrapping_sub(1)) else {
            return;
        };
        c.parti = true;
        c.cartes_en_attente = 0;
        c.joueur.commandes = Default::default();
        let j = &mut c.etat_joueur;
        j.mort = true;
        j.temps_mort = regles::A_TERRE;
        j.x = 1e4;
        j.z = 1e4;
    }

    /// Les autres joueurs : choix des cartes, déplacement, totem.
    pub(crate) fn maj_membres(&mut self, dt: f32) {
        if self.equipe.choix_hote {
            let j = &mut self.etat.joueur;
            j.protection = j.protection.max(regles::PROTECTION_CHOIX);
        }
        for k in 1..self.nombre_joueurs() {
            let c = &mut self.equipe.membres[k - 1];
            if c.parti {
                continue;
            }
            // Protégé le temps de choisir ; au-delà (joueur absent), plus de bouclier.
            // À terre, il ne choisit pas : ses cartes attendent qu'il soit relevé
            let choisit = c.cartes_en_attente > 0 && !c.etat_joueur.mort;
            c.temps_choix = if choisit { c.temps_choix + dt } else { 0.0 };
            c.en_choix = choisit && c.temps_choix < regles::DELAI_CHOIX + regles::MARGE_CHOIX;
            if c.en_choix {
                let j = &mut c.etat_joueur;
                j.protection = j.protection.max(regles::PROTECTION_CHOIX);
                c.joueur.commandes = Default::default();
            }
            self.avec_combattant(k, |p| {
                p.maj_joueur(dt);
                let (jx, jz) = (p.etat.joueur.x, p.etat.joueur.z);
                p.flux.maj(&p.carte, jx, jz);
                p.traiter_resurrection();
            });
        }
    }

    /// Armes et talents des autres joueurs (après ceux de l'hôte).
    pub(crate) fn maj_armes_membres(&mut self, dt: f32) {
        for k in 1..self.nombre_joueurs() {
            let c = &self.equipe.membres[k - 1];
            if c.parti {
                continue;
            }
            let tire = !c.en_choix;
            self.avec_combattant(k, |p| p.maj_armes(dt, tire));
        }
    }

    /// Fin du pas : niveau commun, objets de coffre pour tous, réanimations.
    pub(crate) fn finir_pas_equipe(&mut self, dt: f32) {
        // Niveau commun : l'XP va à l'hôte, les autres le suivent et choisissent leurs cartes
        let (niveau, xp, xp_niveau) = {
            let j = &self.etat.joueur;
            (j.niveau, j.xp, j.xp_niveau)
        };
        let coffre = self.etat.choix_coffre.restants;
        let nouveaux = coffre.saturating_sub(self.equipe.coffres_vus);
        self.equipe.coffres_vus = coffre;
        for c in &mut self.equipe.membres {
            let j = &mut c.etat_joueur;
            if !c.parti {
                c.cartes_en_attente += niveau.saturating_sub(j.niveau) + nouveaux;
            }
            j.niveau = niveau;
            j.xp = xp;
            j.xp_niveau = xp_niveau;
        }
        self.reanimer(dt);
    }

    /// Un joueur à terre se relève quand un allié debout reste assez longtemps près de lui.
    fn reanimer(&mut self, dt: f32) {
        let r2 = regles::RAYON_REANIMATION * regles::RAYON_REANIMATION;
        for k in 0..self.nombre_joueurs() {
            let j = self.etat_joueur(k);
            let parti = k > 0 && self.equipe.membres[k - 1].parti;
            if !j.mort || j.temps_mort >= regles::A_TERRE || parti {
                self.equipe.reanimations[k] = 0.0;
                continue;
            }
            let (x, z) = (j.x, j.z);
            let aide = (0..self.nombre_joueurs()).any(|m| {
                let a = self.etat_joueur(m);
                m != k && !a.mort && (a.x - x) * (a.x - x) + (a.z - z) * (a.z - z) <= r2
            });
            let t = &mut self.equipe.reanimations[k];
            *t = if aide { *t + dt } else { (*t - dt).max(0.0) };
            if *t < regles::DUREE_REANIMATION {
                continue;
            }
            *t = 0.0;
            let j = self.etat_joueur_mut(k);
            j.mort = false;
            j.temps_mort = 0.0;
            j.pv = libm::ceilf(j.pv_max * regles::PV_REANIMATION).max(1.0);
            j.protection = regles::PROTECTION_REANIMATION;
            self.ev.gerbe("totem", x, 1.2, z);
            self.ev.son("totem");
        }
    }
}
