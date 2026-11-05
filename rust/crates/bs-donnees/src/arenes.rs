//! Arènes construites à la main (sol, murs, décor, populations d'ennemis, ambiance)
//! et structures de décor en blocs. Les arènes importées (cartes/*.schem) sont lues
//! au lancement à partir de leur schématique.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::blocs::ChoixBloc;
use crate::communs::{Intervalle, Table, Texture};

/// Rectangle [x0, z0, x1, z1] en blocs, origine au centre de l'arène.
pub type Rectangle = [f32; 4];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Monde {
    Prairie,
    Mine,
    Nether,
    Champignons,
}

/// Remplacement du sol : un rectangle, ou des taches rondes placées au hasard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum TacheSol {
    Rectangle {
        bloc: String,
        rect: Rectangle,
    },
    Taches {
        bloc: String,
        nombre: f32,
        /// min, max (blocs)
        rayon: [f32; 2],
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<Rectangle>,
    },
}

/// Décor : une structure à un endroit précis, ou dispersée au hasard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ElementDecor {
    Structure {
        structure: String,
        x: f32,
        z: f32,
    },
    Dispersion {
        structure: String,
        nombre: f32,
        /// blocs libres exigés autour
        espacement: f32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<Rectangle>,
    },
}

impl ElementDecor {
    pub fn structure(&self) -> &str {
        match self {
            Self::Structure { structure, .. } | Self::Dispersion { structure, .. } => structure,
        }
    }
}

/// Ennemi pouvant apparaître à partir d'un moment de la partie, avec un poids de tirage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChoixEnnemi {
    pub ennemi: String,
    /// s de jeu avant la première apparition
    pub depuis: f32,
    pub poids: f32,
}

/// Zone de l'arène avec sa propre population d'ennemis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZoneEnnemis {
    pub nom: String,
    pub rect: Rectangle,
    pub ennemis: Vec<ChoixEnnemi>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exterieur {
    pub bloc: String,
    pub assombrissement: f32,
    pub taille: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Lumiere {
    pub ambiante: f32,
    pub soleil: f32,
    pub couleur_ambiante: u32,
    pub couleur_soleil: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub x: f32,
    pub z: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Arene {
    pub id: String,
    pub monde: Monde,
    pub nom: String,
    pub description: String,
    pub icone: Texture,
    /// Zone jouable en blocs, centrée sur l'origine.
    pub largeur: f32,
    pub profondeur: f32,
    pub hauteur_mur: f32,
    pub sol: Vec<ChoixBloc>,
    /// Appliquées dans l'ordre, après le sol de base.
    pub taches: Vec<TacheSol>,
    pub murs: Vec<ChoixBloc>,
    pub decor: Vec<ElementDecor>,
    /// Rayon laissé libre autour du point de départ.
    pub zone_depart: f32,
    pub exterieur: Option<Exterieur>,
    pub ciel: u32,
    pub brouillard: Intervalle,
    pub lumiere: Lumiere,
    /// Boss dans l'ordre d'arrivée (puis en boucle).
    pub boss: Vec<String>,
    pub mini_boss: Vec<String>,
    /// Population hors zones.
    pub ennemis: Vec<ChoixEnnemi>,
    /// Zones à population propre (la première qui contient le point l'emporte).
    pub zones: Vec<ZoneEnnemis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depart: Option<Point>,
    /// Lieu d'arrivée des boss et mini-boss (panneau `[boss]` d'une map), sinon hors champ.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arrivee_boss: Option<Point>,
    /// Biome des teintes d'une map importée (« swamp »…), à la place de celui du fichier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biome: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Arenes {
    pub arenes: Vec<Arene>,
    pub par_defaut: String,
}

/// Structure de décor : couches de bas en haut, lignes du nord au sud, un caractère
/// par bloc d'ouest en est (« . » = vide, les autres renvoient à `legende`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Structure {
    pub couches: Vec<Vec<String>>,
    pub legende: IndexMap<char, String>,
}

pub type Structures = Table<Structure>;
