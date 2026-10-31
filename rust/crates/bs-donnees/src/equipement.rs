//! Équipement de partie (bagues et amulettes des coffres) et armures de la
//! Collection (permanentes, une par monde).

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::arenes::Monde;
use crate::communs::{Modifs, Table};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Emplacement {
    Bague,
    Amulette,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rarete {
    Commune,
    Rare,
    Epique,
    Legendaire,
}

/// Effet propre d'un objet (mécanique dans le combat), ses nombres au niveau 1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum EffetObjet {
    /// Onde qui repousse et étourdit autour du joueur.
    Cloche {
        intervalle: f32,
        rayon: f32,
        force: f32,
        etourdi: f32,
    },
    /// Un ennemi tué sur `chance` lâche un éclat qui explose.
    Filon {
        chance: f32,
        rayon: f32,
        degats: f32,
    },
    /// Un ennemi tué sur `chance` empoisonne autour de lui.
    Spores {
        chance: f32,
        rayon: f32,
        poison: f32,
        duree: f32,
    },
    /// Dégâts en plus sous `seuil` des PV max.
    Bastion { seuil: f32, bonus: f32 },
    /// Perdre `seuil` des PV max d'un coup : nova de feu et protection.
    Magma {
        seuil: f32,
        rayon: f32,
        degats: f32,
        protection: f32,
        recharge: f32,
    },
    /// Explosion de spores périodique autour du joueur.
    Couronne {
        intervalle: f32,
        rayon: f32,
        degats: f32,
        poison: f32,
        duree: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormeIcone {
    Bague,
    Pendentif,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Metal {
    Fer,
    Or,
    Cuivre,
    Netherite,
}

/// Dessin pixel art de l'icône.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IconeObjet {
    pub forme: FormeIcone,
    pub metal: Metal,
    pub gemme: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjetEquipement {
    pub nom: String,
    pub emplacement: Emplacement,
    pub rarete: Rarete,
    /// Monde où il se trouve (aucun : tous les mondes).
    pub monde: Option<Monde>,
    /// Texte de l'effet à chaque niveau (`ReglesEquipement::niveaux`).
    pub description: Vec<String>,
    /// Bonus au niveau 1, multipliés par le facteur du niveau.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modif: Option<Modifs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effet: Option<EffetObjet>,
    pub icone: IconeObjet,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NomRarete {
    pub nom: String,
}

pub type PoidsRaretes = IndexMap<Rarete, f32>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TirageBois {
    pub poids: PoidsRaretes,
    pub par_coffre: PoidsRaretes,
    pub minimum: PoidsRaretes,
    pub maximum: PoidsRaretes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TirageEnder {
    pub poids: PoidsRaretes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmeraudesRefus {
    pub bois: f32,
    pub ender: f32,
}

/// Règles de tirage des objets dans les coffres.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReglesEquipement {
    pub bagues: u32,
    pub amulettes: u32,
    /// Facteur des effets par niveau (le dernier est le maximum).
    pub niveaux: Vec<f32>,
    pub propositions: u32,
    pub bois: TirageBois,
    pub ender: TirageEnder,
    pub chance_renfort: f32,
    pub emeraudes_consolation: f32,
    pub emeraudes_refus: EmeraudesRefus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Equipement {
    pub objets: Table<ObjetEquipement>,
    pub raretes: IndexMap<Rarete, NomRarete>,
    pub regles: ReglesEquipement,
}

// ---------------------------------------------------------------------------
// Collection : armures de monde
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Piece {
    Casque,
    Plastron,
    Jambieres,
    Bottes,
    Bouclier,
    Cape,
    Talisman,
}

/// Matériau des textures du jeu (entity/equipment/humanoid/<matériau>).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Materiau {
    Leather,
    Chainmail,
    Iron,
    Gold,
    Diamond,
    Netherite,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CouleursArmure {
    pub champ: u32,
    pub bordure: u32,
    pub embleme: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PieceArmure {
    pub nom: String,
    pub description: String,
    pub modif: Modifs,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ensemble {
    pub description: String,
    pub modif: Modifs,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Armure {
    pub nom: String,
    pub monde: Monde,
    pub materiau: Materiau,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teinte: Option<u32>,
    pub couleurs: CouleursArmure,
    pub pieces: IndexMap<Piece, PieceArmure>,
    pub ensemble: Ensemble,
}

/// Butin d'armure d'un boss selon son rang dans la partie.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ButinBoss {
    pub chance: f32,
    pub chance_seconde: f32,
    pub poids: IndexMap<Piece, f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Armures {
    pub armures: IndexMap<Monde, Armure>,
    pub noms_mondes: IndexMap<Monde, String>,
    pub noms_pieces: IndexMap<Piece, String>,
    pub butin_boss: Vec<ButinBoss>,
}
