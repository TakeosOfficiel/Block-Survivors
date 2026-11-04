//! Règles communes à toutes les arènes : joueur, apparitions, calendriers des
//! boss, progression, plafonds, états, multijoueur.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::communs::{Intervalle, Table};
use crate::effets::Etat;
use crate::entites::Joueur;
use crate::progression::Progression;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Apparitions {
    /// Durée (s) pour passer des valeurs « debut » aux valeurs « fin ».
    pub duree_montee: f32,
    pub intervalle: Intervalle,
    pub taille_groupe: Intervalle,
    pub max_simultanes: Intervalle,
    pub multiplicateur_pv: Intervalle,
    pub distance_min: f32,
    pub dispersion_groupe: f32,
    /// Surface marchable (cases) à partir de laquelle une map reçoit tous ses mobs ;
    /// en dessous, le plafond `max_simultanes` baisse au prorata (absent du jeu TS).
    #[serde(default = "surface_reference")]
    pub surface_reference: f32,
    /// Part minimale du plafond gardée sur une toute petite map.
    #[serde(default = "facteur_surface_min")]
    pub facteur_surface_min: f32,
}

/// Un peu moins que les arènes du jeu (60 × 60, environ 3 400 cases marchables).
fn surface_reference() -> f32 {
    3000.0
}

fn facteur_surface_min() -> f32 {
    0.3
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CalendrierBoss {
    pub premier_passage: f32,
    pub intervalle: f32,
    pub avertissement: f32,
    /// PV multipliés comme ceux des ennemis ordinaires à ce moment (mini-boss).
    pub pv_selon_temps: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Plafonds {
    /// Délai des armes : multiplicateur minimal.
    pub delai_min: f32,
    pub projectiles_bonus: f32,
    pub critique: f32,
    pub regeneration: f32,
    pub bouclier: f32,
    pub armure: f32,
    pub surplus_allie: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResistanceBoss {
    pub ralenti: f32,
    pub ralenti_max: f32,
    pub enracinement: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ParJoueur {
    pub apparitions: f32,
    pub pv_ennemis: f32,
    pub pv_boss: f32,
}

impl ParJoueur {
    /// Multiplicateur d'un effet de difficulté pour `n` joueurs.
    pub fn facteur(par_joueur: f32, n: u32) -> f32 {
        1.0 + par_joueur * n.saturating_sub(1) as f32
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Reanimation {
    pub a_terre: f32,
    pub rayon: f32,
    pub duree: f32,
    pub pv: f32,
    pub protection: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChoixCartes {
    pub delai: f32,
    pub marge: f32,
    pub protection: f32,
    pub periode_bouclier: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Reseau {
    pub instantanes_par_seconde: f32,
    pub rayon_envoi: f32,
    pub rayon_vue: f32,
    pub silence_entrees: f32,
    pub chiffres_max: u32,
    pub periode_stats: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prediction {
    pub correction: f32,
    pub teleportation: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Multijoueur {
    pub joueurs_max: u32,
    pub par_joueur: ParJoueur,
    pub reanimation: Reanimation,
    pub choix_cartes: ChoixCartes,
    pub reseau: Reseau,
    pub prediction: Prediction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Regles {
    pub joueur: Joueur,
    pub apparitions: Apparitions,
    pub boss: CalendrierBoss,
    pub mini_boss: CalendrierBoss,
    pub progression: Progression,
    pub plafonds: Plafonds,
    /// Mécaniques qui profitent des « +1 projectile » permanents.
    pub profite_projectiles: IndexMap<String, bool>,
    pub etats: Table<Etat>,
    pub resistance_boss: ResistanceBoss,
    pub multijoueur: Multijoueur,
}

/// Textes divers : astuces des écrans de chargement, noms des skins.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Textes {
    pub astuces: Vec<String>,
    pub skins: IndexMap<String, String>,
}
