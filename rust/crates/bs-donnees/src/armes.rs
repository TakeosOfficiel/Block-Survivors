//! Armes et pouvoirs du joueur : statistiques, mécanique, rangs et éveils.
//! Une mécanique = un comportement de combat (bs-regles) ; ajouter une arme qui
//! réutilise une mécanique existante = ajouter une entrée dans donnees/armes.ron.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::communs::{ModifArme, StatArme, StatsArme, Texture};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FamilleArme {
    Melee,
    Projectile,
    Feu,
    Allie,
}

/// Comportement d'une arme et ses réglages propres (voir l'en-tête de src/donnees/armes.ts
/// pour l'usage des statistiques par mécanique).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Mecanique {
    /// Frappe en arc de cercle devant le joueur.
    Balayage { angle: f32, effet: Vec<Texture> },
    /// Projectiles en ligne droite vers l'ennemi le plus proche.
    Tir { projectile: String, dispersion: f32 },
    /// Traverse tout ; revient au joueur à partir du rang `loyaute`.
    Retour {
        projectile: String,
        dispersion: f32,
        loyaute: f32,
    },
    /// Objets qui tournent en permanence autour du joueur.
    Orbite { projectile: String },
    /// Explose au premier ennemi touché ; 4 éclats dès le rang `eclats_au_rang`.
    Explosif {
        projectile: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gerbe: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        son: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        eclats_au_rang: Option<f32>,
    },
    /// Éclairs sur des ennemis au hasard autour du joueur.
    Foudre { couleur: u32 },
    /// Alliés qui combattent seuls un temps limité.
    Invocation {
        allie: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        brulure_au_rang: Option<f32>,
    },
    /// Fissures en éventail vers le plus gros groupe.
    Fissure { ecart: f32, vitesse: f32 },
    /// Arme géante qui tombe du ciel sur le plus gros groupe.
    Chute { image: Texture, taille: f32 },
    /// Le joueur tourne sur lui-même et frappe tout autour.
    Tourbillon {
        intervalle: f32,
        effet: Vec<Texture>,
    },
    /// Coup de bouclier en cône ; renvoie les projectiles dès le rang `renvoi`.
    Heurt { angle: f32, renvoi: f32 },
    /// Tornade en entonnoir qui avance vers les ennemis.
    Tornade {
        texture: Texture,
        feu: Texture,
        brulure: Texture,
        vitesse: f32,
        intervalle: f32,
    },
    /// Anneau d'éclats qui s'étend ; laisse un sol ralentissant dès `sol_au_rang`.
    Nova {
        texture: Texture,
        expansion: f32,
        sol: Texture,
        sol_au_rang: f32,
        duree_sol: f32,
    },
    /// Bloc qui tombe sur un groupe, puis feu au sol.
    Meteore {
        texture: Texture,
        feu: Texture,
        degats_sol: f32,
    },
    /// Orbes qui tournent puis foncent ; blessent au contact dès `contact_au_rang`.
    Orbes {
        texture: Texture,
        contact_au_rang: f32,
        contact: f32,
    },
    /// Flèche chercheuse qui rebondit d'ennemi en ennemi.
    Ricochet {
        projectile: String,
        perte: f32,
        distance_rebond: f32,
    },
    /// Flèches qui tombent du ciel sur le plus gros groupe.
    Pluie {
        projectile: String,
        impacts_par_seconde: f32,
    },
    /// Piège posé sous le joueur ; enracine dès `enracine_au_rang`.
    Piege {
        image: Texture,
        intervalle: f32,
        enracine_au_rang: f32,
    },
}

impl Mecanique {
    /// Nom de la mécanique, tel qu'écrit dans les données.
    pub fn nom(&self) -> &'static str {
        match self {
            Self::Balayage { .. } => "balayage",
            Self::Tir { .. } => "tir",
            Self::Retour { .. } => "retour",
            Self::Orbite { .. } => "orbite",
            Self::Explosif { .. } => "explosif",
            Self::Foudre { .. } => "foudre",
            Self::Invocation { .. } => "invocation",
            Self::Fissure { .. } => "fissure",
            Self::Chute { .. } => "chute",
            Self::Tourbillon { .. } => "tourbillon",
            Self::Heurt { .. } => "heurt",
            Self::Tornade { .. } => "tornade",
            Self::Nova { .. } => "nova",
            Self::Meteore { .. } => "meteore",
            Self::Orbes { .. } => "orbes",
            Self::Ricochet { .. } => "ricochet",
            Self::Pluie { .. } => "pluie",
            Self::Piege { .. } => "piege",
        }
    }

    /// Projectile utilisé, s'il y en a un.
    pub fn projectile(&self) -> Option<&str> {
        match self {
            Self::Tir { projectile, .. }
            | Self::Retour { projectile, .. }
            | Self::Orbite { projectile }
            | Self::Explosif { projectile, .. }
            | Self::Ricochet { projectile, .. }
            | Self::Pluie { projectile, .. } => Some(projectile),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NiveauArme {
    /// Texte de la carte d'amélioration.
    pub description: String,
    /// Ajouts aux statistiques (cumulés rang après rang).
    pub modif: ModifArme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeEtat {
    Brulure,
    Poison,
    Ralenti,
    Etourdi,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EtatEveil {
    #[serde(rename = "type")]
    pub genre: TypeEtat,
    /// Dégâts par seconde, part de vitesse retirée ou durée (étourdi, s).
    pub valeur: f32,
    pub duree: f32,
}

/// Éveil au rang maximal : une variante exclusive (l'autre n'est plus proposée).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Eveil {
    pub nom: String,
    pub description: String,
    pub icone: Texture,
    /// Ajouts aux statistiques de base (avant les modificateurs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ajout: Option<ModifArme>,
    /// Parts en plus des statistiques finales.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<ModifArme>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etat: Option<EtatEveil>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Arme {
    pub nom: String,
    pub description: String,
    pub icone: Texture,
    pub mecanique: Mecanique,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub famille: Option<FamilleArme>,
    /// Statistiques au rang 1.
    pub stats: StatsArme,
    /// Améliorations des rangs 2, 3… ; rang maximal = 1 + longueur.
    pub niveaux: Vec<NiveauArme>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub son: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub libelles: Option<IndexMap<StatArme, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stats_affichees: Option<Vec<StatArme>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eveils: Option<[Eveil; 2]>,
    /// Couleur signature des lueurs de l'arme (0xRRGGBB) ; sans elle, celle de sa mécanique.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teinte: Option<u32>,
}
