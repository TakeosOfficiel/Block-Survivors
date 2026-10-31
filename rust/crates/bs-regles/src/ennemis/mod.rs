//! Ennemis : pool unique, apparition, recherches (plus proche, au hasard, plus gros
//! groupe), puis dans les sous-modules les coups et les états (`coups`), le
//! déplacement et les comportements (`deplacement`). Port de src/entites/ennemis.ts.
//!
//! Un ennemi est désigné par son index dans le pool : les index ne bougent qu'à la
//! phase de retrait de `maj_ennemis` (échange avec le dernier), jamais pendant les
//! coups. Ce qui garde un ennemi d'un pas à l'autre retient aussi son `id`.

mod coups;
mod deplacement;

use bs_donnees::Donnees;
use bs_donnees::entites::{Comportement, Ennemi as DefinitionEnnemi};

use crate::grille::GrilleSpatiale;
use crate::partie::Partie;

pub use coups::Nature;

/// Ennemis simultanés au maximum.
pub const CAPACITE: usize = 600;
const CELLULE_GRILLE: f32 = 2.0;
/// s entre deux vérifications de la ligne de vue d'un ennemi.
const PERIODE_VUE: f32 = 0.25;
/// Ennemis examinés pour trouver le plus gros groupe.
const ESSAIS_GROUPE: usize = 12;

/// Brûlure, poison ou saignement : cumuls, dégâts par cumul et par seconde, temps restant.
#[derive(Debug, Clone, Copy, Default)]
pub struct DegatsPeriodiques {
    pub cumuls: f32,
    pub degats_par_seconde: f32,
    /// s restantes (remises à neuf à chaque application).
    pub temps: f32,
    /// s avant les prochains dégâts.
    pub tic: f32,
    /// Arme créditée des dégâts.
    pub arme: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Ennemi {
    /// Identifiant unique de l'apparition (les index du pool changent).
    pub id: u32,
    pub index_type: usize,
    pub def: &'static DefinitionEnnemi,
    pub x: f32,
    pub z: f32,
    /// Recul (blocs/s), amorti avec le temps.
    pub vx: f32,
    pub vz: f32,
    /// Direction voulue par le comportement (longueur 0 ou 1).
    pub dir_x: f32,
    pub dir_z: f32,
    pub angle: f32,
    pub vitesse_sol: f32,
    pub pv: f32,
    pub pv_max: f32,
    /// s avant la prochaine attaque.
    pub minuterie: f32,
    /// s de surcouche rouge restantes.
    pub blessure: f32,
    /// s de mèche écoulées (creeper).
    pub meche: f32,
    /// −1 : vivant ; sinon s depuis la mort.
    pub mort: f32,
    pub a_exploser: bool,
    /// Ligne de vue directe vers le joueur (sinon : suit le champ de flux).
    pub voit_joueur: bool,
    /// Tireur : aucun bloc n'arrête son tir vers le joueur.
    pub ligne_de_tir: bool,
    pub minuterie_vue: f32,
    /// Allié invoqué qui l'a attiré (index dans les invocations) et son identifiant.
    pub cible_allie: Option<(usize, u32)>,
    /// Par arme : temps de jeu avant lequel elle ne peut plus le toucher.
    pub immunites: Vec<f32>,
    pub brulure: DegatsPeriodiques,
    pub poison: DegatsPeriodiques,
    pub saignement: DegatsPeriodiques,
    /// Part de vitesse perdue (0 à 1) et s restantes.
    pub ralenti: f32,
    pub ralenti_temps: f32,
    /// s restantes sans pouvoir marcher.
    pub enracine: f32,
    /// s restantes sans marcher ni attaquer.
    pub etourdi: f32,
    /// Dégâts en plus subis des alliés (0,15 = +15 %) et s restantes.
    pub marque: f32,
    pub marque_temps: f32,
    pub minuterie_etats: f32,
    // Talents : marque de l'Archère, cumuls contre un boss, piège, morsures
    pub marque_chasseur: f32,
    pub cumuls_talent: f32,
    pub temps_cumuls: f32,
    /// Temps de jeu jusqu'auquel l'ennemi compte comme pris dans un piège.
    pub piege: f64,
    pub dans_piege: bool,
    pub morsure_a: u32,
    pub morsure_b: u32,
    pub temps_morsure_a: f64,
    pub temps_morsure_b: f64,
    pub encercle: f32,
}

impl Ennemi {
    fn nouveau(def: &'static DefinitionEnnemi, armes: usize) -> Self {
        Self {
            id: 0,
            index_type: 0,
            def,
            x: 0.0,
            z: 0.0,
            vx: 0.0,
            vz: 0.0,
            dir_x: 0.0,
            dir_z: 0.0,
            angle: 0.0,
            vitesse_sol: 0.0,
            pv: 1.0,
            pv_max: 1.0,
            minuterie: 0.0,
            blessure: 0.0,
            meche: 0.0,
            mort: -1.0,
            a_exploser: false,
            voit_joueur: true,
            ligne_de_tir: true,
            minuterie_vue: 0.0,
            cible_allie: None,
            immunites: vec![0.0; armes],
            brulure: DegatsPeriodiques::default(),
            poison: DegatsPeriodiques::default(),
            saignement: DegatsPeriodiques::default(),
            ralenti: 0.0,
            ralenti_temps: 0.0,
            enracine: 0.0,
            etourdi: 0.0,
            marque: 0.0,
            marque_temps: 0.0,
            minuterie_etats: 0.0,
            marque_chasseur: 0.0,
            cumuls_talent: 0.0,
            temps_cumuls: 0.0,
            piege: 0.0,
            dans_piege: false,
            morsure_a: 0,
            morsure_b: 0,
            temps_morsure_a: -1.0,
            temps_morsure_b: -1.0,
            encercle: 0.0,
        }
    }

    #[inline]
    pub fn vivant(&self) -> bool {
        self.mort < 0.0
    }

    #[inline]
    pub fn boss(&self) -> bool {
        self.def.boss.is_some()
    }

    /// Efface tous les états (apparition).
    fn effacer_etats(&mut self) {
        self.brulure.cumuls = 0.0;
        self.poison.cumuls = 0.0;
        self.saignement.cumuls = 0.0;
        self.marque_chasseur = 0.0;
        self.cumuls_talent = 0.0;
        self.temps_cumuls = 0.0;
        self.piege = 0.0;
        self.dans_piege = false;
        self.morsure_a = 0;
        self.morsure_b = 0;
        self.temps_morsure_a = -1.0;
        self.temps_morsure_b = -1.0;
        self.encercle = 0.0;
        self.ralenti = 0.0;
        self.ralenti_temps = 0.0;
        self.enracine = 0.0;
        self.etourdi = 0.0;
        self.marque = 0.0;
        self.marque_temps = 0.0;
        self.minuterie_etats = 0.0;
    }
}

/// Pool des ennemis et leur grille (index des vivants, reconstruite à chaque pas).
#[derive(Debug, Clone)]
pub struct Ennemis {
    /// Actifs dans [0, nombre).
    pub pool: Vec<Ennemi>,
    pub nombre: usize,
    grille: GrilleSpatiale,
    voisins: Vec<u32>,
    groupe: Vec<u32>,
    /// Plus grand rayon d'ennemi (marge des recherches dans la grille).
    rayon_max: f32,
    prochain_id: u32,
    /// Multijoueur : ennemis, et boss ou mini-boss, plus robustes selon le nombre de joueurs.
    pub multiplicateur_pv: f32,
    pub multiplicateur_pv_boss: f32,
}

impl Ennemis {
    pub fn nouveaux(d: &'static Donnees, demi_cote: f32) -> Self {
        let premier = &d.ennemis[0];
        let armes = d.armes.len();
        Self {
            pool: (0..CAPACITE)
                .map(|_| Ennemi::nouveau(premier, armes))
                .collect(),
            nombre: 0,
            grille: GrilleSpatiale::nouvelle(demi_cote, CELLULE_GRILLE, CAPACITE),
            voisins: vec![0; 64],
            groupe: vec![0; 64],
            rayon_max: d.ennemis.values().map(|e| e.rayon).fold(0.0, f32::max),
            prochain_id: 1,
            multiplicateur_pv: 1.0,
            multiplicateur_pv_boss: 1.0,
        }
    }

    #[inline]
    pub fn actifs(&self) -> &[Ennemi] {
        &self.pool[..self.nombre]
    }

    /// Écrit dans `sortie` les index des ennemis vivants qui touchent le cercle (leur
    /// propre rayon compris) ; renvoie leur nombre.
    pub fn chercher(&mut self, x: f32, z: f32, rayon: f32, sortie: &mut [u32]) -> usize {
        let candidats = self
            .grille
            .requeter(x, z, rayon + self.rayon_max, &mut self.voisins);
        let mut n = 0;
        for k in 0..candidats {
            if n >= sortie.len() {
                break;
            }
            let i = self.voisins[k];
            let e = &self.pool[i as usize];
            let (dx, dz) = (e.x - x, e.z - z);
            let portee = rayon + e.def.rayon;
            if e.vivant() && dx * dx + dz * dz <= portee * portee {
                sortie[n] = i;
                n += 1;
            }
        }
        n
    }

    /// Premier boss vivant, ou aucun.
    pub fn boss_actif(&self) -> Option<usize> {
        self.actifs().iter().position(|e| e.boss() && e.vivant())
    }

    /// Ennemi vivant le plus proche à moins de `portee`.
    pub fn plus_proche(&self, x: f32, z: f32, portee: f32) -> Option<usize> {
        let mut meilleur = None;
        let mut meilleure_d2 = portee * portee;
        for (i, e) in self.actifs().iter().enumerate() {
            if !e.vivant() {
                continue;
            }
            let d2 = (e.x - x) * (e.x - x) + (e.z - z) * (e.z - z);
            if d2 < meilleure_d2 {
                meilleure_d2 = d2;
                meilleur = Some(i);
            }
        }
        meilleur
    }

    /// Grille des ennemis vivants, utilisée par les armes et au pas suivant.
    pub fn reconstruire_grille(&mut self) {
        self.grille.vider();
        for i in 0..self.nombre {
            let e = &self.pool[i];
            if e.vivant() {
                let (x, z) = (e.x, e.z);
                self.grille.inserer(i, x, z);
            }
        }
    }

    /// Retire l'ennemi `i` (échange avec le dernier actif).
    fn liberer(&mut self, i: usize) {
        self.nombre -= 1;
        self.pool.swap(i, self.nombre);
    }
}

impl Partie {
    /// Fait apparaître un ennemi (index de type) ; faux si le pool est plein.
    pub fn apparaitre(
        &mut self,
        index_type: usize,
        x: f32,
        z: f32,
        multiplicateur_pv: f32,
    ) -> bool {
        let en = &mut self.ennemis;
        if en.nombre >= CAPACITE {
            return false;
        }
        let def = &self.d.ennemis[index_type];
        let limite_x = self.etat.arene.largeur / 2.0 - def.rayon;
        let limite_z = self.etat.arene.profondeur / 2.0 - def.rayon;
        let multiplicateur = if def.boss.is_some() || def.mini_boss {
            en.multiplicateur_pv_boss
        } else {
            en.multiplicateur_pv
        };
        let id = en.prochain_id;
        en.prochain_id += 1;
        let minuterie_vue = self.alea.tirer() as f32 * PERIODE_VUE;
        let j = &self.etat.joueur;
        let e = &mut en.pool[en.nombre];
        en.nombre += 1;
        e.id = id;
        e.index_type = index_type;
        e.def = def;
        e.x = x.clamp(-limite_x, limite_x);
        e.z = z.clamp(-limite_z, limite_z);
        e.vx = 0.0;
        e.vz = 0.0;
        e.dir_x = 0.0;
        e.dir_z = 0.0;
        e.angle = bs_commun::maths::atan2(j.x - e.x, j.z - e.z);
        e.vitesse_sol = 0.0;
        e.pv_max = libm::roundf(def.pv * multiplicateur_pv * multiplicateur);
        e.pv = e.pv_max;
        e.minuterie = 0.5;
        e.blessure = 0.0;
        e.meche = 0.0;
        e.mort = -1.0;
        e.a_exploser = false;
        e.voit_joueur = true;
        e.ligne_de_tir = true;
        e.cible_allie = None;
        // Étale les vérifications de la ligne de vue
        e.minuterie_vue = minuterie_vue;
        e.immunites.fill(0.0);
        e.effacer_etats();
        true
    }

    /// Ennemi vivant tiré au hasard à moins de `portee` (tirage uniforme en un passage).
    pub fn ennemi_au_hasard(&mut self, x: f32, z: f32, portee: f32) -> Option<usize> {
        let mut choisi = None;
        let mut vus = 0.0;
        let p2 = portee * portee;
        for i in 0..self.ennemis.nombre {
            let e = &self.ennemis.pool[i];
            if !e.vivant() || (e.x - x) * (e.x - x) + (e.z - z) * (e.z - z) > p2 {
                continue;
            }
            vus += 1.0;
            if self.alea.tirer() * vus < 1.0 {
                choisi = Some(i);
            }
        }
        choisi
    }

    /// Centre du plus gros groupe d'ennemis à moins de `portee` de (x, z) : parmi quelques
    /// ennemis tirés au hasard, celui qui a le plus de voisins dans `rayon`.
    pub fn plus_gros_groupe(
        &mut self,
        x: f32,
        z: f32,
        portee: f32,
        rayon: f32,
    ) -> Option<(f32, f32)> {
        let mut meilleur: Option<(usize, f32, f32)> = None;
        let mut groupe = std::mem::take(&mut self.ennemis.groupe);
        for _ in 0..ESSAIS_GROUPE {
            let Some(i) = self.ennemi_au_hasard(x, z, portee) else {
                break;
            };
            let (ex, ez) = (self.ennemis.pool[i].x, self.ennemis.pool[i].z);
            let voisins = self.ennemis.chercher(ex, ez, rayon, &mut groupe);
            if meilleur.is_none_or(|(m, _, _)| voisins > m) {
                meilleur = Some((voisins, ex, ez));
            }
        }
        self.ennemis.groupe = groupe;
        meilleur.map(|(_, x, z)| (x, z))
    }

    /// Repousse sans les blesser les ennemis vivants dans un cercle (résurrection du joueur).
    pub fn repousser_ennemis(&mut self, x: f32, z: f32, rayon: f32, force: f32) {
        for e in &mut self.ennemis.pool[..self.ennemis.nombre] {
            let (dx, dz) = (e.x - x, e.z - z);
            let d = bs_commun::maths::longueur(dx, dz);
            if !e.vivant() || d > rayon + e.def.rayon {
                continue;
            }
            let k = force * (1.0 - e.def.resistance_recul.unwrap_or(0.0))
                / if d == 0.0 { 1.0 } else { d };
            e.vx += dx * k;
            e.vz += dz * k;
        }
    }

    /// Fin de partie gagnée : toute la horde tombe, sans butin ni statistiques.
    pub fn terrasser_tous(&mut self) {
        for e in &mut self.ennemis.pool[..self.ennemis.nombre] {
            if e.vivant() {
                e.mort = 0.0;
                e.meche = 0.0;
                e.blessure = coups::DUREE_BLESSURE;
            }
        }
    }

    /// Un pas : états, comportement et déplacement, retrait des morts, grille.
    pub fn maj_ennemis(&mut self, dt: f32) {
        // 1. Comportement et déplacement (la grille date de la fin du pas précédent :
        //    ses index sont valides, seuls des ajouts ont pu avoir lieu depuis)
        let n = self.ennemis.nombre;
        for i in 0..n {
            if self.ennemis.pool[i].vivant() {
                self.maj_etats(i, dt);
            }
            let e = &mut self.ennemis.pool[i];
            if !e.vivant() {
                e.mort += dt;
                e.vitesse_sol = 0.0;
                continue;
            }
            self.deplacer(i, dt);
        }

        // 2. Retrait des ennemis explosés ou morts depuis assez longtemps
        for i in (0..n).rev() {
            let e = &self.ennemis.pool[i];
            if e.a_exploser {
                self.exploser(i);
                self.ennemis.liberer(i);
            } else if e.mort >= DUREE_MORT {
                let (x, z) = (e.x, e.z);
                self.ev.gerbe("mort", x, 0.6, z);
                self.ennemis.liberer(i);
            }
        }

        // 3. Grille des ennemis vivants
        self.ennemis.reconstruire_grille();
    }

    /// Explosion (creeper) : dégâts au joueur décroissants avec la distance.
    fn exploser(&mut self, i: usize) {
        let e = &self.ennemis.pool[i];
        let Comportement::Explosif { rayon, degats, .. } = e.def.comportement else {
            return;
        };
        let (x, z, boss) = (e.x, e.z, e.boss());
        self.ev.gerbe("explosion", x, 0.8, z);
        self.ev.son("explosion");
        self.ev.secousse(0.45);
        let j = &self.etat.joueur;
        let distance = bs_commun::maths::longueur(j.x - x, j.z - z);
        if distance < rayon {
            self.blesser_joueur(libm::ceilf(degats * (1.0 - distance / rayon)), boss);
        }
        self.blesser_allies_zone(x, z, rayon, degats);
    }
}

/// Chute sur le côté avant disparition (s).
pub const DUREE_MORT: f32 = 0.6;
