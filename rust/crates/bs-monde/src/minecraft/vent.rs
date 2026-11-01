//! Ondulation au vent d'un bloc d'une map importée (src/minecraft/maillage.ts) : chaque
//! sommet porte (centre x, pied y, centre z, mode). Le feuillage frémit, le sommet des
//! plantes se balance ; le shader des blocs fait le reste.

use super::modeles::BlocCuit;

pub const VENT_AUCUN: f32 = 0.0;
pub const VENT_FEUILLAGE: f32 = 1.0;
pub const VENT_PLANTE: f32 = 2.0;

/// Plantes qui se balancent (fleurs, herbes, pousses, cultures, lianes…).
const PLANTES: &[&str] = &[
    "short_grass",
    "grass",
    "tall_grass",
    "fern",
    "large_fern",
    "dandelion",
    "poppy",
    "blue_orchid",
    "allium",
    "azure_bluet",
    "oxeye_daisy",
    "cornflower",
    "lily_of_the_valley",
    "wither_rose",
    "torchflower",
    "sunflower",
    "lilac",
    "rose_bush",
    "peony",
    "sweet_berry_bush",
    "dead_bush",
    "bush",
    "firefly_bush",
    "short_dry_grass",
    "tall_dry_grass",
    "wildflowers",
    "pink_petals",
    "leaf_litter",
    "cactus_flower",
    "open_eyeblossom",
    "closed_eyeblossom",
    "wheat",
    "carrots",
    "potatoes",
    "beetroots",
    "seagrass",
    "tall_seagrass",
    "kelp",
    "kelp_plant",
    "vine",
    "cave_vines",
    "cave_vines_plant",
    "weeping_vines",
    "weeping_vines_plant",
    "twisting_vines",
    "twisting_vines_plant",
    "hanging_roots",
    "brown_mushroom",
    "red_mushroom",
    "crimson_roots",
    "warped_roots",
    "nether_sprouts",
    "spore_blossom",
    "pitcher_plant",
    "small_dripleaf",
];

/// Mode de vent d'un bloc.
pub fn mode_vent(b: &BlocCuit) -> f32 {
    if b.nom.ends_with("_leaves") {
        VENT_FEUILLAGE
    } else if b.nom.ends_with("_sapling")
        || b.nom.ends_with("_tulip")
        || PLANTES.contains(&b.nom.as_str())
    {
        VENT_PLANTE
    } else {
        VENT_AUCUN
    }
}

/// Vent d'un bloc posé en `o` (coin du bloc, en coordonnées du jeu).
pub fn vent_du_bloc(b: &BlocCuit, mode: f32, o: [f32; 3]) -> [f32; 4] {
    if mode == VENT_AUCUN {
        return [0.0; 4];
    }
    // Moitié haute d'une grande plante : le pied est un bloc plus bas
    let haute = b.propriete("half") == Some("upper");
    let pied = o[1] - if haute { 1.0 } else { 0.0 };
    [o[0] + 0.5, pied, o[2] + 0.5, mode]
}
