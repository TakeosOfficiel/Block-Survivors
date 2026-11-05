//! Ce que les fichiers de modèles ne disent pas (src/minecraft/proprietes.ts) : teinte
//! de biome des faces teintées, collisions et effets au sol de chaque bloc. Couleurs du
//! biome de la map (voir `biomes`) ; bouleau, sapin et quelques blocs ont leur couleur fixe.

use super::biomes::Teintes;
use super::modeles::{BlocCuit, Liquide};

/// D'où vient la couleur des faces teintées d'un bloc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Teinte {
    Fixe(u32),
    Herbe,
    Feuillage,
    Eau,
}

impl Teinte {
    /// Couleur 0xRRGGBB dans un biome de ces couleurs.
    pub fn couleur(self, t: &Teintes) -> u32 {
        match self {
            Teinte::Fixe(c) => c,
            Teinte::Herbe => t.herbe,
            Teinte::Feuillage => t.feuillage,
            Teinte::Eau => t.eau,
        }
    }
}

/// Teinte des faces teintées du bloc.
pub fn teinte_de(nom: &str) -> Teinte {
    match nom {
        "birch_leaves" => Teinte::Fixe(0x80a755),
        "spruce_leaves" => Teinte::Fixe(0x619961),
        "oak_leaves" | "jungle_leaves" | "acacia_leaves" | "dark_oak_leaves"
        | "mangrove_leaves" | "vine" => Teinte::Feuillage,
        "lily_pad" => Teinte::Fixe(0x208030),
        "water" | "water_cauldron" | "bubble_column" => Teinte::Eau,
        "redstone_wire" => Teinte::Fixe(0xa00000),
        "melon_stem" | "pumpkin_stem" | "attached_melon_stem" | "attached_pumpkin_stem" => {
            Teinte::Fixe(0xbfb030)
        }
        _ => Teinte::Herbe,
    }
}

/// `Plein` bloque la marche et les tirs ; `Liquide` seulement la marche.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Collision {
    Vide,
    Plein,
    Liquide,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Physique {
    pub collision: Collision,
    /// Multiplicateur de vitesse (sol ou bloc traversé).
    pub ralentit: Option<f32>,
    /// Seules les araignées ignorent ce ralentissement.
    pub toile: bool,
    /// Dégâts par seconde (sol ou bloc traversé).
    pub degats: Option<f32>,
}

impl Physique {
    const fn de(collision: Collision) -> Self {
        Self {
            collision,
            ralentit: None,
            toile: false,
            degats: None,
        }
    }
}

/// `nom` vaut `mot` ou finit par « _mot ».
fn fin_de_mot(nom: &str, mot: &str) -> bool {
    nom == mot || nom.strip_suffix(mot).is_some_and(|d| d.ends_with('_'))
}

/// Blocs qu'on traverse en jeu, malgré un modèle épais.
fn traversable(nom: &str) -> bool {
    const CONTIENT: [&str; 21] = [
        "structure_void",
        "propagule",
        "torch",
        "button",
        "lever",
        "pressure_plate",
        "tripwire",
        "redstone_wire",
        "ladder",
        "vine",
        "lichen",
        "scaffolding",
        "cobweb",
        "petals",
        "leaf_litter",
        "wildflowers",
        "lily_pad",
        "sugar_cane",
        "kelp",
        "seagrass",
        "pointed_dripstone",
    ];
    const FINIT: [&str; 9] = [
        "banner",
        "portal",
        "_sapling",
        "bush",
        "fern",
        "roots",
        "sprouts",
        "carpet",
        "amethyst_bud",
    ];
    const EGAL: [&str; 7] = [
        "light",
        "air",
        "end_gateway",
        "short_grass",
        "tall_grass",
        "snow",
        "small_dripleaf",
    ];
    CONTIENT.iter().any(|m| nom.contains(m))
        || FINIT.iter().any(|m| nom.ends_with(m))
        || EGAL.contains(&nom)
        || ["rail", "sign", "fire"].iter().any(|m| fin_de_mot(nom, m))
}

fn effets(nom: &str) -> Physique {
    let mut p = Physique::de(Collision::Vide);
    match nom {
        "cobweb" => {
            p.ralentit = Some(0.3);
            p.toile = true;
        }
        "sweet_berry_bush" => {
            p.ralentit = Some(0.5);
            p.degats = Some(1.0);
        }
        "soul_sand" | "honey_block" => p.ralentit = Some(0.6),
        "magma_block" | "fire" | "soul_fire" | "campfire" | "soul_campfire" => p.degats = Some(2.0),
        "lava" => p.degats = Some(8.0),
        _ => {}
    }
    p
}

pub fn physique_de(b: &BlocCuit) -> Physique {
    let e = effets(&b.nom);
    let avec = |collision| Physique { collision, ..e };
    if b.liquide.is_some() {
        return avec(Collision::Liquide);
    }
    if b.nom == "barrier" || b.inconnu {
        return Physique::de(Collision::Plein);
    }
    if traversable(&b.nom) {
        return avec(Collision::Vide);
    }
    if b.nom.ends_with("fence_gate") && b.propriete("open") == Some("true") {
        return Physique::de(Collision::Vide);
    }
    // D'après la forme : rien, des plans sans épaisseur ou une fine couche → on passe
    let modeles: Vec<_> = b.parties.iter().flatten().map(|v| &v.modele).collect();
    let boites: Vec<_> = modeles.iter().filter_map(|m| m.boite).collect();
    if boites.is_empty() || modeles.iter().all(|m| m.plat || m.quads.is_empty()) {
        return avec(Collision::Vide);
    }
    let haut = boites
        .iter()
        .map(|(_, max)| max[1])
        .fold(f32::MIN, f32::max);
    let bas = boites
        .iter()
        .map(|(min, _)| min[1])
        .fold(f32::MAX, f32::min);
    if haut - bas <= 3.0 {
        return avec(Collision::Vide);
    }
    avec(Collision::Plein)
}

/// Le liquide du bloc pour l'affichage (eau des blocs « waterlogged » comprise).
pub fn liquide_affiche(b: &BlocCuit) -> Option<Liquide> {
    if b.a_de_l_eau() {
        Some(Liquide::Eau)
    } else {
        b.liquide
    }
}
