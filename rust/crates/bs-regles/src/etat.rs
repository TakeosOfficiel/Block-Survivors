//! État d'une partie : temps, joueur, statistiques, modificateurs. Source de vérité de
//! la simulation, sans rien d'affichage (port de src/etat.ts).

use bs_donnees::arenes::Arene;
use bs_donnees::entites::Personnage;
use bs_donnees::{Donnees, Modifs};

/// Modificateurs globaux du joueur, cumulés depuis ses bonus (passifs, équipement, talents).
pub type Modificateurs = Modifs;

/// Modificateurs de départ, avant tout bonus (port de `creerModificateurs`).
pub fn modificateurs_de_base() -> Modificateurs {
    Modifs {
        degats: 1.0,
        cadence: 1.0,
        zone: 1.0,
        portee: 1.0,
        vitesse: 1.0,
        collecte: 1.0,
        // Comme les coups critiques du jeu : ×1,5
        critique: 0.05,
        degats_critiques: 1.5,
        recul: 1.0,
        ..Modifs::default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EtatJoueur {
    /// Position au sol (blocs).
    pub x: f32,
    pub z: f32,
    /// Vitesse (blocs/s).
    pub vx: f32,
    pub vz: f32,
    /// Orientation (rad), 0 = face vers +Z.
    pub angle: f32,
    pub pv: f32,
    pub pv_max: f32,
    /// s restantes sans pouvoir être blessé (après un coup).
    pub invulnerabilite: f32,
    /// s d'invincibilité après une résurrection (totem).
    pub protection: f32,
    /// PV d'absorption (cœurs dorés), pris avant les PV.
    pub bouclier: f32,
    /// Totems déjà consommés.
    pub vies_utilisees: u32,
    pub mort: bool,
    /// s écoulées depuis la mort.
    pub temps_mort: f32,
    pub niveau: u32,
    /// XP accumulée dans le niveau en cours.
    pub xp: f32,
    /// XP nécessaire pour passer au niveau suivant.
    pub xp_niveau: f32,
}

/// Bilan chiffré de la partie (écrans de pause et de fin, robot).
#[derive(Debug, Clone, PartialEq)]
pub struct Statistiques {
    /// Ennemis tués, par index de type.
    pub tues_par_type: Vec<u32>,
    /// Par index d'arme : dégâts, coups, critiques, plus gros coup, ennemis achevés.
    pub degats_par_arme: Vec<f64>,
    pub coups_par_arme: Vec<u32>,
    pub critiques_par_arme: Vec<u32>,
    pub plus_gros_coup_par_arme: Vec<f32>,
    pub tues_par_arme: Vec<u32>,
    /// Temps de jeu où chaque arme a été obtenue (−1 : jamais).
    pub acquisition_arme: Vec<f64>,
    pub degats_subis: f32,
    pub pv_soignes: f32,
    pub xp_totale: f32,
    pub emeraudes_ramassees: u32,
    pub coffres_ouverts: u32,
    pub boss_vaincus: u32,
}

/// Fin de partie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Issue {
    Victoire,
    Defaite,
}

/// Choix d'objets de coffre en attente ; `ender` : ceux d'un coffre de l'Ender, proposés d'abord.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChoixCoffre {
    pub restants: u32,
    pub ender: u32,
}

#[derive(Debug, Clone)]
pub struct EtatPartie {
    pub arene: &'static Arene,
    pub index_personnage: usize,
    pub personnage: &'static Personnage,
    /// Secondes de jeu écoulées.
    pub temps_jeu: f64,
    pub ennemis_tues: u32,
    /// Montées de niveau dont les cartes n'ont pas encore été choisies.
    pub niveaux_en_attente: u32,
    pub relances_restantes: u32,
    /// Émeraudes ramassées (elles ouvrent les coffres).
    pub emeraudes: u32,
    pub choix_coffre: ChoixCoffre,
    pub issue: Option<Issue>,
    /// Vrai une fois la partie gagnée puis continuée.
    pub sans_fin: bool,
    pub stats: Statistiques,
    pub modificateurs: Modificateurs,
    pub joueur: EtatJoueur,
}

impl EtatPartie {
    pub fn nouvel(d: &'static Donnees, arene: &'static Arene, index_personnage: usize) -> Self {
        let personnage = &d.personnages.personnages[index_personnage];
        let progression = &d.regles.progression;
        let (x, z) = arene.depart.as_ref().map_or((0.0, 0.0), |p| (p.x, p.z));
        let armes = d.armes.len();
        Self {
            arene,
            index_personnage,
            personnage,
            temps_jeu: 0.0,
            ennemis_tues: 0,
            niveaux_en_attente: 0,
            relances_restantes: progression.relances,
            emeraudes: 0,
            choix_coffre: ChoixCoffre::default(),
            issue: None,
            sans_fin: false,
            stats: Statistiques {
                tues_par_type: vec![0; d.ennemis.len()],
                degats_par_arme: vec![0.0; armes],
                coups_par_arme: vec![0; armes],
                critiques_par_arme: vec![0; armes],
                plus_gros_coup_par_arme: vec![0.0; armes],
                tues_par_arme: vec![0; armes],
                acquisition_arme: vec![-1.0; armes],
                degats_subis: 0.0,
                pv_soignes: 0.0,
                xp_totale: 0.0,
                emeraudes_ramassees: 0,
                coffres_ouverts: 0,
                boss_vaincus: 0,
            },
            modificateurs: modificateurs_de_base(),
            joueur: EtatJoueur {
                x,
                z,
                vx: 0.0,
                vz: 0.0,
                angle: 0.0,
                pv: personnage.pv,
                pv_max: personnage.pv,
                invulnerabilite: 0.0,
                protection: 0.0,
                bouclier: 0.0,
                vies_utilisees: 0,
                mort: false,
                temps_mort: 0.0,
                niveau: 1,
                xp: 0.0,
                xp_niveau: progression.xp_premier_niveau,
            },
        }
    }

    /// Minutes jouées au-delà de la durée de la partie en mode sans fin (0 sinon).
    pub fn minutes_sans_fin(&self, duree_partie: f32) -> f32 {
        if self.sans_fin {
            (self.temps_jeu as f32 - duree_partie).max(0.0) / 60.0
        } else {
            0.0
        }
    }
}

/// XP nécessaire pour passer du niveau `niveau` au suivant.
pub fn xp_pour_niveau(d: &Donnees, niveau: u32) -> f32 {
    let p = &d.regles.progression;
    let mut xp = p.xp_premier_niveau;
    let mut n = 1;
    for palier in &p.paliers {
        let jusqu = palier.jusqu_au_niveau.unwrap_or(u32::MAX);
        while n < niveau && n < jusqu {
            xp += palier.increment;
            n += 1;
        }
    }
    xp
}

/// Ajoute de l'XP ; chaque niveau franchi s'ajoute aux cartes en attente.
pub fn gagner_xp(d: &Donnees, etat: &mut EtatPartie, valeur: f32) {
    let j = &mut etat.joueur;
    j.xp += valeur;
    etat.stats.xp_totale += valeur;
    while j.xp >= j.xp_niveau {
        j.xp -= j.xp_niveau;
        j.niveau += 1;
        j.xp_niveau = xp_pour_niveau(d, j.niveau);
        etat.niveaux_en_attente += 1;
    }
}
