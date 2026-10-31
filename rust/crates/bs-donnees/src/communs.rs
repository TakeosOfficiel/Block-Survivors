//! Types partagés par les tables : textures, apparence des entités, ajouts aux
//! statistiques des armes et aux modificateurs du joueur.
//!
//! Les noms des champs restent ceux de la version TypeScript (camelCase dans les
//! fichiers RON) : mêmes noms partout, et les identifiants du contenu (parties de
//! modèles, gerbes…) gardent leur forme.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Chemin d'une texture vanilla, relatif à assets/minecraft/textures/, sans « .png ».
pub type Texture = String;
/// Table ordonnée par identifiant (l'ordre du fichier est celui des listes du jeu).
pub type Table<T> = IndexMap<String, T>;
pub type Vec3 = [f32; 3];

pub(crate) fn faux(b: &bool) -> bool {
    !*b
}
pub(crate) fn nul(x: &f32) -> bool {
    *x == 0.0
}

/// Valeur interpolée linéairement entre le début et la fin de la montée en difficulté.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Intervalle {
    pub debut: f32,
    pub fin: f32,
}

/// Apparence d'une entité : modèle, skin, couleurs de secours, ombre.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Apparence {
    pub modele: String,
    pub skin: Texture,
    /// Couleur unie par partie du modèle si le skin est introuvable.
    pub couleurs_secours: IndexMap<String, u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub couleur_secours_par_defaut: Option<u32>,
    pub rayon_ombre: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objet_en_main: Option<Texture>,
}

/// Champs d'`Apparence`, rangés à part par l'importeur (en TypeScript, ils sont à plat).
pub const CHAMPS_APPARENCE: [&str; 6] = [
    "modele",
    "skin",
    "couleursSecours",
    "couleurSecoursParDefaut",
    "rayonOmbre",
    "objetEnMain",
];

/// Déclare une structure de nombres dont chaque champ vaut 0 s'il est absent (ajouts,
/// parts en plus) et n'est pas écrit s'il vaut 0.
macro_rules! nombres_facultatifs {
    ($(#[$doc:meta])* $nom:ident { $($champ:ident),* $(,)? }) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct $nom {
            $(#[serde(default, skip_serializing_if = "nul")] pub $champ: f32,)*
        }
    };
}

/// Statistiques d'une arme ; chaque mécanique n'utilise que celles qui la concernent.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatsArme {
    pub degats: f32,
    /// s entre deux déclenchements (orbite : délai avant de retoucher un ennemi)
    pub cadence: f32,
    pub nombre: f32,
    pub portee: f32,
    /// blocs/s (orbite : rad/s)
    pub vitesse: f32,
    pub zone: f32,
    pub duree: f32,
    pub perforation: f32,
    /// blocs/s donnés aux ennemis touchés (négatif : les attire)
    pub recul: f32,
    pub ralenti: f32,
    pub etourdi: f32,
}

nombres_facultatifs! {
    /// Ajouts (ou parts en plus) aux statistiques d'une arme ; absent = 0.
    ModifArme { degats, cadence, nombre, portee, vitesse, zone, duree, perforation, recul, ralenti, etourdi }
}

nombres_facultatifs! {
    /// Ajouts aux modificateurs globaux du joueur ; absent = 0.
    Modifs {
        degats, cadence, zone, portee, projectiles, allies, vitesse, pv_max, armure, regeneration, vies,
        collecte, critique, degats_critiques,
        degats_melee, zone_melee, degats_projectiles, vitesse_projectiles, degats_feu, cadence_feu,
        degats_allies, vitesse_allies, pv_allies, duree_allies, recul,
        degats_brulure, duree_brulure, cumuls_brulure, degats_poison, duree_ralenti, degats_ralentis,
        regeneration_part, soins_objets,
        degats_morts_vivants, degats_arthropodes, degats_nether, degats_recus, emeraudes, chance_brulure,
        degats_sol,
    }
}

/// Nom d'une statistique d'arme (libellés et statistiques affichées sur les cartes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StatArme {
    Degats,
    Cadence,
    Nombre,
    Portee,
    Vitesse,
    Zone,
    Duree,
    Perforation,
    Recul,
    Ralenti,
    Etourdi,
}
