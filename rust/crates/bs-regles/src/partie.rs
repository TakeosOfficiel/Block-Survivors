//! Partie : regroupe l'état, le monde et tous les systèmes, et les fait avancer d'un pas
//! dans l'ordre du jeu TypeScript (src/jeu.ts, `simuler`). Les systèmes sont écrits
//! comme des blocs `impl Partie` dans leurs modules.

use bs_commun::Mulberry;
use bs_donnees::Donnees;
use bs_donnees::arenes::Arene;
use bs_monde::{CarteMonde, ChampDeFlux, generer_monde};

use crate::butin::Butin;
use crate::cartes::Cartes;
use crate::catalogue::Catalogue;
use crate::combat::{Arsenal, SystemesArmes};
use crate::directeurs::{DirecteurApparitions, DirecteurBoss, SorteBoss};
use crate::ennemis::Ennemis;
use crate::equipe::Equipe;
use crate::equipement::Equipement;
use crate::etat::{EtatPartie, Issue};
use crate::evenements::Evenements;
use crate::joueur::{Joueur, TOTEM_RAYON_RECUL, TOTEM_RECUL};
use crate::passifs::Passifs;
use crate::projectiles_ennemis::ProjectilesEnnemis;
use crate::ramassage::Ramassables;
use crate::talents::Talents;
use crate::vue::VueCamera;

/// Hauteur approchée d'un modèle d'ennemi (blocs), pour placer les chiffres de dégâts.
/// Visuel seulement : le jeu TS la tire du modèle compilé.
const HAUTEUR_MODELE: f32 = 1.8;

pub struct Partie {
    pub d: &'static Donnees,
    pub c: Catalogue,
    pub etat: EtatPartie,
    pub carte: CarteMonde,
    pub flux: ChampDeFlux,
    pub vue: VueCamera,
    pub alea: Mulberry,
    pub ev: Evenements,
    /// Par type d'ennemi : hauteur des chiffres de dégâts et des gerbes.
    pub hauteurs: Vec<f32>,
    pub joueur: Joueur,
    pub ennemis: Ennemis,
    pub projectiles_ennemis: ProjectilesEnnemis,
    pub ramassables: Ramassables,
    pub passifs: Passifs,
    pub arsenal: Arsenal,
    pub talents: Talents,
    /// Cartes de niveau proposées.
    pub cartes: Cartes,
    /// Bagues et amulette portées.
    pub equipement: Equipement,
    /// Pièces d'armure gagnées sur les boss.
    pub butin: Butin,
    /// États des systèmes d'armes (projectiles, orbites, éclairs…).
    pub armes: SystemesArmes,
    pub directeur: DirecteurApparitions,
    pub directeur_boss: DirecteurBoss,
    pub directeur_mini_boss: DirecteurBoss,
    /// Fin de partie atteinte avec un boss en vie : il faut le vaincre pour gagner.
    pub prolongation: bool,
    /// Totems déjà traités (recul des ennemis).
    pub(crate) vies_traitees: u32,
    /// Les autres joueurs (coopération) ; vide en solo.
    pub equipe: Equipe,
    /// Partie d'un invité : seul l'hôte décide des coups (voir `simuler_invite`).
    pub invite: bool,
}

impl Partie {
    /// Côté du bloc du sol de l'arène (son extérieur, sinon la terre) : éclats des
    /// éboulis.
    pub fn texture_sol(&self) -> &'static str {
        let d = self.d;
        let bloc = self
            .etat
            .arene
            .exterieur
            .as_ref()
            .map_or("dirt", |e| e.bloc.as_str());
        d.blocs
            .get(bloc)
            .map_or("block/dirt", |b| b.faces.cotes.as_str())
    }

    /// Nouvelle partie dans l'arène `index_arene` avec le personnage `index_personnage`.
    /// `evenements` : enregistrer les retours visuels et sonores (inutile pour le robot).
    pub fn nouvelle(
        d: &'static Donnees,
        index_arene: usize,
        index_personnage: usize,
        graine: u32,
        evenements: bool,
    ) -> Self {
        let arene = &d.arenes.arenes[index_arene];
        let carte = generer_monde(d, arene).carte;
        Self::avec_arene(d, arene, carte, index_personnage, graine, evenements)
    }

    /// Nouvelle partie dans une arène déjà construite (map Minecraft importée) avec sa
    /// carte de collision.
    pub fn avec_arene(
        d: &'static Donnees,
        arene: &'static Arene,
        carte: CarteMonde,
        index_personnage: usize,
        graine: u32,
        evenements: bool,
    ) -> Self {
        let etat = EtatPartie::nouvel(d, arene, index_personnage);
        let flux = ChampDeFlux::nouveau(&carte);
        let demi_cote = arene.largeur.max(arene.profondeur) / 2.0 + 1.0;
        let vue = VueCamera::nouvelle(etat.joueur.x, etat.joueur.z, 1280.0 / 720.0);
        let passifs = Passifs::nouveaux(etat.personnage.bonus);
        let talents = Talents::nouveaux(d.armes.len());
        let mut partie = Self {
            d,
            c: Catalogue::nouveau(d),
            etat,
            carte,
            flux,
            vue,
            alea: Mulberry::nouveau(graine),
            ev: Evenements::nouveaux(evenements),
            hauteurs: d
                .ennemis
                .values()
                .map(|e| HAUTEUR_MODELE * e.echelle.unwrap_or(1.0) + 0.3)
                .collect(),
            joueur: Joueur::default(),
            ennemis: Ennemis::nouveaux(d, demi_cote),
            projectiles_ennemis: ProjectilesEnnemis::nouveaux(),
            ramassables: Ramassables::nouveaux(d),
            passifs,
            arsenal: Arsenal::nouveau(),
            talents,
            cartes: Cartes::nouvelles(d),
            equipement: Equipement::nouveau(d),
            butin: Butin::default(),
            armes: SystemesArmes::nouveaux(d),
            directeur: DirecteurApparitions::default(),
            directeur_boss: DirecteurBoss::nouveau(SorteBoss::Boss, &d.regles.boss),
            directeur_mini_boss: DirecteurBoss::nouveau(SorteBoss::MiniBoss, &d.regles.mini_boss),
            prolongation: false,
            vies_traitees: 0,
            equipe: Equipe::nouvelle(),
            invite: false,
        };
        partie.preparer_joueur();
        partie
    }

    /// Bonus permanents du personnage (sauvegarde vierge : ni talents ni Collection),
    /// mécaniques de sa classe et armes de départ du joueur en jeu.
    pub(crate) fn preparer_joueur(&mut self) {
        let d = self.d;
        let etat = &mut self.etat;
        etat.modificateurs = self.passifs.calculer(d, None);
        etat.joueur.pv_max += etat.modificateurs.pv_max;
        etat.joueur.pv = etat.joueur.pv_max;
        let classe = d
            .personnages
            .personnages
            .get_index(etat.index_personnage)
            .map_or("", |(id, _)| id.as_str());
        let talents = &mut self.talents;
        talents.classes.dernier = (etat.joueur.x, etat.joueur.z);
        if classe == "guerrier" {
            talents.classes.guerrier = Some(crate::classes::Guerrier::nouveau(d));
        }
        if classe == "archer" {
            talents.classes.archere = Some(crate::classes::Archere::nouvelle(d));
        }
        if classe == "mage" {
            talents.classes.mage = Some(crate::classes::Mage::nouveau(d));
        }
        if classe == "invocateur" {
            talents.classes.invocateur = Some(crate::classes::Invocateur::nouveau(d));
        }
        for id in &self.etat.personnage.armes_depart {
            if let Some(index) = d.armes.get_index_of(id) {
                self.ameliorer_arme(index);
            }
        }
    }

    /// Un pas de simulation, dans l'ordre de `simuler` du jeu TS.
    pub fn simuler(&mut self, dt: f32) {
        if self.etat.issue.is_none() {
            self.etat.temps_jeu += f64::from(dt);
            self.verifier_fin();
        }
        self.maj_joueur(dt);
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        self.flux.maj(&self.carte, jx, jz);
        let seul = self.equipe.membres.is_empty();
        if !seul {
            self.maj_membres(dt);
        }
        self.maj_directeur(dt * self.equipe.facteur_apparitions);
        self.maj_directeurs_boss();
        // Reconstruit aussi la grille utilisée par les armes
        self.maj_ennemis(dt);
        self.recompenser_boss();
        let tire = !self.equipe.choix_hote;
        if tire {
            self.maj_arsenal(dt);
        }
        self.maj_talents(dt);
        self.maj_orbes(dt);
        self.maj_objets(dt);
        self.maj_coffres(dt);
        self.maj_systemes_armes(dt);
        if !seul {
            self.maj_armes_membres(dt);
        }
        self.maj_projectiles_ennemis(dt);
        self.traiter_resurrection();
        if !seul {
            self.finir_pas_equipe(dt);
        }
    }

    /// Arsenal, talents et systèmes d'armes du joueur en jeu (un autre de l'équipe).
    pub(crate) fn maj_armes(&mut self, dt: f32, tire: bool) {
        if tire {
            self.maj_arsenal(dt);
        }
        self.maj_talents(dt);
        self.maj_systemes_armes(dt);
    }

    /// Systèmes d'armes, dans l'ordre du jeu TS.
    fn maj_systemes_armes(&mut self, dt: f32) {
        self.maj_projectiles_joueur(dt);
        self.maj_invocations(dt);
        self.maj_foudre(dt);
        self.maj_fissures(dt);
        self.maj_chutes(dt);
        self.maj_tourbillon(dt);
        self.maj_tornades(dt);
        self.maj_novas(dt);
        self.maj_meteores(dt);
        self.maj_orbes_arcaniques(dt);
        self.maj_ricochets(dt);
        self.maj_pluies(dt);
        self.maj_pieges(dt);
    }

    /// Un pas du robot : la simulation, puis la caméra qui suit le joueur.
    pub fn pas(&mut self, dt: f32) {
        self.simuler(dt);
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        self.vue.suivre(jx, jz, dt);
    }

    /// Défaite si toute l'équipe est tombée ; victoire à la fin du temps, sauf boss en vie.
    fn verifier_fin(&mut self) {
        let duree = self.d.regles.progression.duree_partie;
        if self.equipe_vaincue() {
            self.etat.issue = Some(Issue::Defaite);
        } else if !self.etat.sans_fin && self.etat.temps_jeu >= f64::from(duree) {
            // Pas de victoire tant qu'un boss est en vie : il faut le vaincre
            if self.ennemis.boss_actif().is_some() {
                self.prolongation = true;
                return;
            }
            if !self.prolongation {
                self.etat.temps_jeu = f64::from(duree);
            }
            self.prolongation = false;
            self.etat.issue = Some(Issue::Victoire);
            self.terrasser_tous();
            self.ev.son("victoire");
            self.ev.secousse(0.5);
        }
    }

    /// Totem consommé : les ennemis proches sont repoussés.
    pub(crate) fn traiter_resurrection(&mut self) {
        let j = &self.etat.joueur;
        if j.vies_utilisees == self.vies_traitees {
            return;
        }
        self.vies_traitees = j.vies_utilisees;
        let (x, z) = (j.x, j.z);
        self.repousser_ennemis(x, z, TOTEM_RAYON_RECUL, TOTEM_RECUL);
        self.ev.gerbe("totem", x, 1.2, z);
    }
}
