//! Entités : joueur et personnages, ennemis, alliés invoqués, projectiles,
//! objets ramassables et coffres.

use serde::{Deserialize, Serialize};

use crate::communs::{Apparence, Modifs, Table, Texture, faux};

/// Réglages communs à tous les personnages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Joueur {
    /// s d'invulnérabilité après un coup
    pub invulnerabilite: f32,
    pub rayon: f32,
    /// rapidité d'atteinte de la vitesse cible (1/s)
    pub reactivite: f32,
    pub vitesse_rotation: f32,
    /// rayon d'attraction des orbes d'XP (blocs)
    pub collecte: f32,
    pub rayon_ramassage: f32,
}

/// Personnage jouable : apparence, statistiques, armes, cartes exclusives.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Personnage {
    pub nom: String,
    pub role: String,
    pub description: String,
    pub apparence: Apparence,
    /// 20 = 10 cœurs
    pub pv: f32,
    pub vitesse: f32,
    pub armes_depart: Vec<String>,
    /// Attaques de la classe : seules elles sont proposées si l'arsenal est complet.
    pub arsenal: Vec<String>,
    pub arsenal_complet: bool,
    pub bonus: Modifs,
    pub cartes_exclusives: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Personnages {
    pub personnages: Table<Personnage>,
    pub par_defaut: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Comportement {
    /// Va au contact et frappe.
    Melee {
        degats: f32,
        portee: f32,
        cadence: f32,
    },
    /// Garde ses distances et tire des projectiles (salves de `nombre`, écartées de `dispersion` rad).
    Archer {
        projectile: String,
        degats: f32,
        distance_ideale: f32,
        portee: f32,
        cadence: f32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        nombre: Option<f32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        dispersion: Option<f32>,
    },
    /// S'approche, gonfle puis explose (annulé si le joueur s'éloigne).
    Explosif {
        distance_amorcage: f32,
        distance_annulation: f32,
        duree: f32,
        rayon: f32,
        degats: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FamilleEnnemi {
    MortVivant,
    Arthropode,
    Nether,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouleurBarre {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Boss {
    pub couleur_barre: CouleurBarre,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SonsEnnemi {
    pub blessure: String,
    pub mort: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Ennemi {
    pub nom: String,
    pub apparence: Apparence,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub famille: Option<FamilleEnnemi>,
    pub pv: f32,
    pub vitesse: f32,
    pub rayon: f32,
    pub comportement: Comportement,
    pub xp: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub echelle: Option<f32>,
    /// Part du recul ignorée (0 = normal, 1 = inébranlable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resistance_recul: Option<f32>,
    /// Objet toujours lâché à la mort.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub butin: Option<String>,
    /// Vole : traverse les obstacles (bords de l'arène exceptés).
    #[serde(default, skip_serializing_if = "faux")]
    pub ignore_obstacles: bool,
    /// Rayon contre le décor, s'il diffère de `rayon` (boss plus larges qu'un passage).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rayon_decor: Option<f32>,
    #[serde(default, skip_serializing_if = "faux")]
    pub traverse_toiles: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boss: Option<Boss>,
    #[serde(default, skip_serializing_if = "faux")]
    pub mini_boss: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sons: Option<SonsEnnemi>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hauteur_sons: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TirAllie {
    pub projectile: String,
    pub salve: f32,
    pub ecart: f32,
    pub distance: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Aura {
    pub intervalle: f32,
    pub enracinement: f32,
    pub delai: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlocsAllie {
    pub empilement: Vec<Texture>,
    pub racines: Texture,
}

/// Allié invoqué par une arme.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Allie {
    pub nom: String,
    pub apparence: Apparence,
    pub vitesse: f32,
    pub rayon: f32,
    pub cadence_coups: f32,
    pub detection: f32,
    pub son_coup: String,
    pub son_blessure: String,
    pub son_mort: String,
    pub pv: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armure: Option<f32>,
    /// Nombre maximal en même temps.
    pub quota: f32,
    /// Profite des « +1 allié » permanents.
    pub renforts: bool,
    /// Distance à laquelle il détourne les ennemis du joueur.
    pub provocation: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vol: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tir: Option<TirAllie>,
    #[serde(default, skip_serializing_if = "faux")]
    pub soutien: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aura: Option<Aura>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocs: Option<BlocsAllie>,
    #[serde(default, skip_serializing_if = "faux")]
    pub remplacement: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Allies {
    pub allies: Table<Allie>,
    /// Créatures des talents de l'Invocateur, hors armes (ravageur, crocs).
    pub creatures_talents: Table<Apparence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormeProjectile {
    Croisee,
    Relief,
    Modele,
    FaceCamera,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Projectile {
    pub nom: String,
    pub forme: FormeProjectile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modele: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peinture: Option<String>,
    pub texture: Texture,
    pub taille_texture: [f32; 2],
    /// Rectangle [x, y, largeur, hauteur] affiché, dans la texture.
    pub uv: [f32; 4],
    pub longueur: f32,
    pub largeur: f32,
    pub vitesse: f32,
    pub duree: f32,
    pub rayon: f32,
    pub hauteur: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation_propre: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trainee: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decalage_trainee: Option<f32>,
    pub couleur_secours: u32,
}

/// Objet ramassable (soins, émeraudes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Objet {
    pub nom: String,
    pub icone: Texture,
    pub soin: f32,
    /// Tas d'émeraudes : entre [min, max] au hasard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emeraudes: Option<[f32; 2]>,
    /// Probabilité d'être lâché par un ennemi ordinaire.
    pub chance: f32,
    pub taille: f32,
    pub couleur_secours: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coffre {
    pub nom: String,
    pub titre: String,
    pub apparence: Apparence,
    pub payant: bool,
    pub choix: f32,
    pub son: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TuesCoffre {
    pub premier: f32,
    pub intervalle: f32,
    pub croissance: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrixCoffre {
    pub base: f32,
    pub increment: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReglesCoffres {
    pub capacite: f32,
    pub tues: TuesCoffre,
    pub prix: PrixCoffre,
    pub distance: [f32; 2],
    pub rayon_ouverture: f32,
    pub duree_ouverture: f32,
    pub reste_ouvert: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coffres {
    pub coffres: Table<Coffre>,
    pub regles: ReglesCoffres,
}
