//! Blocs du décor (textures, forme, effets au sol) et modèles cubiques des
//! personnages et créatures (parties, animations).

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::communs::{Texture, Vec3, faux};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormeBloc {
    /// Bloc plein (par défaut).
    Cube,
    /// Deux plans croisés en diagonale (fleurs, herbes, toiles), jamais solide.
    Croix,
    /// Cube légèrement abaissé (eau, lave), infranchissable à pied.
    Liquide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ondulation {
    Feuillage,
    Plante,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Faces {
    pub haut: Texture,
    pub cotes: Texture,
    pub bas: Texture,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationBloc {
    pub images_par_seconde: f32,
    #[serde(default, skip_serializing_if = "faux")]
    pub aller_retour: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Bloc {
    pub nom: String,
    pub faces: Faces,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forme: Option<FormeBloc>,
    /// Bloque le passage (vrai par défaut pour les cubes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solide: Option<bool>,
    /// Texture à trous (feuilles, toiles, fleurs).
    #[serde(default, skip_serializing_if = "faux")]
    pub decoupe: bool,
    #[serde(default, skip_serializing_if = "faux")]
    pub lumineux: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ondule: Option<Ondulation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teinte_haut: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teinte: Option<u32>,
    #[serde(default, skip_serializing_if = "faux")]
    pub rotation_aleatoire: bool,
    /// Multiplicateur de vitesse sur ce bloc (sable des âmes) ou dedans (toile).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ralentit: Option<f32>,
    /// Toile d'araignée : son ralentissement ne gêne pas les araignées.
    #[serde(default, skip_serializing_if = "faux")]
    pub toile: bool,
    /// Dégâts infligés au joueur qui marche dessus (magma).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub degats: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation: Option<AnimationBloc>,
    pub couleur_secours: u32,
}

impl Bloc {
    /// Bloque le passage : cubes et liquides par défaut, jamais les croix.
    pub fn bloque_le_passage(&self) -> bool {
        match self.forme.unwrap_or(FormeBloc::Cube) {
            FormeBloc::Croix => false,
            FormeBloc::Liquide => true,
            FormeBloc::Cube => self.solide.unwrap_or(true),
        }
    }
}

/// Bloc tiré au hasard parmi une liste pondérée.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChoixBloc {
    pub bloc: String,
    pub poids: f32,
}

// ---------------------------------------------------------------------------
// Modèles cubiques
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cube {
    /// Largeur, hauteur, profondeur en pixels de texture.
    pub taille: Vec3,
    /// Articulation, en pixels depuis les pieds.
    pub pivot: Vec3,
    /// Centre de la boîte par rapport à l'articulation.
    pub centre: Vec3,
    /// Coin haut-gauche du patron dans la texture.
    pub uv: [f32; 2],
    #[serde(default, skip_serializing_if = "faux")]
    pub miroir: bool,
    /// Pose de base (rad), appliquée dans l'ordre Z, Y, X.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Vec3>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gonflement: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Axe {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MembreMarche {
    pub partie: String,
    pub axe: Axe,
    pub amplitude: f32,
    pub dephasage: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequence: Option<f32>,
    /// |sin| au lieu de cos : mouvement toujours du même côté.
    #[serde(default, skip_serializing_if = "faux")]
    pub absolu: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Marche {
    /// rad de cycle par bloc parcouru
    pub frequence: f32,
    pub vitesse_reference: f32,
    pub lissage: f32,
    pub membres: Vec<MembreMarche>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MembreRepos {
    pub partie: String,
    /// 1 = côté droit, -1 = côté gauche
    pub signe: i8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Repos {
    pub amplitude: f32,
    pub frequence_z: f32,
    pub frequence_x: f32,
    pub membres: Vec<MembreRepos>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tournoiement {
    pub partie: String,
    pub vitesse: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Flottement {
    pub hauteur: f32,
    pub amplitude: f32,
    pub frequence: f32,
    #[serde(default, skip_serializing_if = "faux")]
    pub bonds: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MembreAttaque {
    pub partie: String,
    pub amplitude: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attaque {
    pub duree: f32,
    pub membres: Vec<MembreAttaque>,
}

/// Où un modèle tient un objet, dans le repère d'une partie.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Main {
    pub partie: String,
    pub position: Vec3,
    pub rotation: Vec3,
    pub taille: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Modele {
    pub taille_texture: [f32; 2],
    pub parties: IndexMap<String, Cube>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portrait: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main: Option<Main>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marche: Option<Marche>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repos: Option<Repos>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tournoiement: Option<Vec<Tournoiement>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flottement: Option<Flottement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attaque: Option<Attaque>,
}

impl Modele {
    /// Toutes les parties nommées par les animations, la main et le portrait.
    pub fn parties_citees(&self) -> impl Iterator<Item = &str> {
        let marche = self
            .marche
            .iter()
            .flat_map(|m| m.membres.iter().map(|x| x.partie.as_str()));
        let repos = self
            .repos
            .iter()
            .flat_map(|r| r.membres.iter().map(|x| x.partie.as_str()));
        let tour = self
            .tournoiement
            .iter()
            .flatten()
            .map(|t| t.partie.as_str());
        let attaque = self
            .attaque
            .iter()
            .flat_map(|a| a.membres.iter().map(|x| x.partie.as_str()));
        let main = self.main.iter().map(|m| m.partie.as_str());
        let portrait = self.portrait.iter().flatten().map(String::as_str);
        marche
            .chain(repos)
            .chain(tour)
            .chain(attaque)
            .chain(main)
            .chain(portrait)
    }
}
