//! Partie : regroupe l'état, le monde et tous les systèmes, et les fait avancer d'un pas
//! dans l'ordre du jeu TypeScript (src/jeu.ts, `simuler`). Les systèmes sont écrits
//! comme des blocs `impl Partie` dans leurs modules.

use bs_commun::Mulberry;
use bs_donnees::Donnees;
use bs_monde::{CarteMonde, ChampDeFlux, generer_monde};

use crate::butin::Butin;
use crate::cartes::Cartes;
use crate::catalogue::Catalogue;
use crate::combat::{Arsenal, SystemesArmes};
use crate::directeurs::{DirecteurApparitions, DirecteurBoss, SorteBoss};
use crate::ennemis::Ennemis;
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
    vies_traitees: u32,
}

impl Partie {
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
        let monde = generer_monde(d, arene);
        let carte = monde.carte;
        let mut etat = EtatPartie::nouvel(d, arene, index_personnage);
        let flux = ChampDeFlux::nouveau(&carte);
        let demi_cote = arene.largeur.max(arene.profondeur) / 2.0 + 1.0;
        let vue = VueCamera::nouvelle(etat.joueur.x, etat.joueur.z, 1280.0 / 720.0);
        // Bonus permanents du personnage (sauvegarde vierge : ni talents ni Collection)
        let passifs = Passifs::nouveaux(etat.personnage.bonus);
        etat.modificateurs = passifs.calculer(d, None);
        etat.joueur.pv_max += etat.modificateurs.pv_max;
        etat.joueur.pv = etat.joueur.pv_max;
        let archere = d
            .personnages
            .personnages
            .get_index(index_personnage)
            .is_some_and(|(id, _)| id == "archer");
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
            talents: Talents::nouveaux(d.armes.len(), archere),
            cartes: Cartes::nouvelles(d),
            equipement: Equipement::nouveau(d),
            butin: Butin::default(),
            armes: SystemesArmes::nouveaux(d),
            directeur: DirecteurApparitions::default(),
            directeur_boss: DirecteurBoss::nouveau(SorteBoss::Boss, &d.regles.boss),
            directeur_mini_boss: DirecteurBoss::nouveau(SorteBoss::MiniBoss, &d.regles.mini_boss),
            prolongation: false,
            vies_traitees: 0,
        };
        for id in &partie.etat.personnage.armes_depart {
            if let Some(index) = d.armes.get_index_of(id) {
                partie.ameliorer_arme(index);
            }
        }
        partie
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
        self.maj_directeur(dt);
        self.maj_directeurs_boss();
        // Reconstruit aussi la grille utilisée par les armes
        self.maj_ennemis(dt);
        self.recompenser_boss();
        self.maj_arsenal(dt);
        self.maj_talents(dt);
        self.maj_orbes(dt);
        self.maj_objets(dt);
        self.maj_coffres(dt);
        // Systèmes d'armes, dans l'ordre du jeu TS
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
        self.maj_projectiles_ennemis(dt);
        self.traiter_resurrection();
    }

    /// Un pas du robot : la simulation, puis la caméra qui suit le joueur.
    pub fn pas(&mut self, dt: f32) {
        self.simuler(dt);
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        self.vue.suivre(jx, jz, dt);
    }

    /// Défaite si le joueur est tombé ; victoire à la fin du temps, sauf boss en vie.
    fn verifier_fin(&mut self) {
        let duree = self.d.regles.progression.duree_partie;
        if self.etat.joueur.mort {
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
    fn traiter_resurrection(&mut self) {
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
