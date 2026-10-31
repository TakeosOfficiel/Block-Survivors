//! Sons, particules (images et gerbes) et états des ennemis (brûlure, poison…).

use serde::{Deserialize, Serialize};

use crate::communs::{Table, Texture, Vec3, faux};

/// Famille de sons, chacune avec son curseur de volume dans les options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CategorieSon {
    Joueur,
    Armes,
    Creatures,
    Interface,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Son {
    pub categorie: CategorieSon,
    /// Chemin vanilla relatif à assets/minecraft/sounds/, sans « .ogg ».
    pub chemin: String,
    /// Empreinte SHA-1 du fichier dans l'index d'assets.
    pub hachage: String,
    pub volume: f32,
    /// Hauteur tirée entre ces bornes à chaque lecture.
    pub hauteur: [f32; 2],
    /// Délai minimal entre deux lectures (s).
    pub intervalle_min: f32,
    pub simultanes: u32,
}

pub type Sons = Table<Son>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationParticule {
    /// Parcourt la série au fil de la vie.
    Age,
    /// Garde la première image.
    Fixe,
    /// Un quart de l'icône donnée à l'émission.
    Morceau,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Particule {
    pub images: Vec<Texture>,
    pub animation: AnimationParticule,
    pub taille: [f32; 2],
    pub duree: [f32; 2],
    /// blocs/s² vers le bas (négatif : monte).
    pub gravite: f32,
    /// Part de la vitesse perdue par seconde.
    pub frottement: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gris: Option<[f32; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teintes: Option<Vec<u32>>,
    #[serde(default, skip_serializing_if = "faux")]
    pub retrecit: bool,
}

/// Une salve de particules d'un même type ; une gerbe en enchaîne plusieurs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Salve {
    pub particule: String,
    pub nombre: u32,
    pub vitesse: f32,
    pub dispersion: f32,
    pub montee: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Particules {
    pub particules: Table<Particule>,
    pub gerbes: Table<Vec<Salve>>,
}

/// État d'un ennemi.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Etat {
    /// Particules émises toutes les `periode` s tant qu'il dure.
    pub gerbe: String,
    pub periode: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub couleur: Option<Vec3>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cumuls_max: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tic: Option<f32>,
}
