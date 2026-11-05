//! Biomes d'une schématique : biome le plus présent (formats Sponge 2 et 3) et couleurs
//! d'herbe, de feuillage et d'eau qui vont avec. Sans biome connu : « plaines », comme
//! les arènes du jeu. Toute la map prend la teinte de son biome dominant.

use super::nbt::{Compound, Nbt};

/// Couleurs 0xRRGGBB des faces teintées selon le biome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Teintes {
    pub herbe: u32,
    pub feuillage: u32,
    pub eau: u32,
}

const EAU: u32 = 0x3f76e4;

/// (biomes, herbe, feuillage, eau) des biomes du monde normal.
const TABLE: [(&[&str], u32, u32, u32); 17] = [
    (
        &["plains", "sunflower_plains", "beach"],
        0x91bd59,
        0x77ab2f,
        EAU,
    ),
    (&["forest", "flower_forest"], 0x79c05a, 0x59ae30, EAU),
    (
        &["birch_forest", "old_growth_birch_forest"],
        0x88bb67,
        0x6ba941,
        EAU,
    ),
    (&["dark_forest"], 0x507a32, 0x59ae30, EAU),
    (&["taiga"], 0x86b783, 0x68a464, 0x287082),
    (
        &["old_growth_pine_taiga", "old_growth_spruce_taiga"],
        0x86b87f,
        0x68a55f,
        EAU,
    ),
    (
        &[
            "snowy_plains",
            "snowy_taiga",
            "snowy_beach",
            "ice_spikes",
            "frozen_river",
            "grove",
            "snowy_slopes",
        ],
        0x80b497,
        0x60a17b,
        0x3d57d6,
    ),
    (&["swamp"], 0x6a7039, 0x6a7039, 0x617b64),
    (&["mangrove_swamp"], 0x6a7039, 0x8db127, 0x3a7a6a),
    (
        &["jungle", "sparse_jungle", "bamboo_jungle"],
        0x59c93c,
        0x30bb0b,
        EAU,
    ),
    (
        &["savanna", "savanna_plateau", "windswept_savanna", "desert"],
        0xbfb755,
        0xaea42a,
        EAU,
    ),
    (
        &["badlands", "eroded_badlands", "wooded_badlands"],
        0x90814d,
        0x9e814d,
        EAU,
    ),
    (&["cherry_grove"], 0xb6db61, 0xb6db61, 0x5db7ef),
    (&["meadow"], 0x83bb6d, 0x63a948, 0x0e4ecf),
    (
        &[
            "windswept_hills",
            "windswept_forest",
            "windswept_gravelly_hills",
            "stony_shore",
        ],
        0x8ab689,
        0x6da36b,
        EAU,
    ),
    (&["mushroom_fields"], 0x55c93f, 0x2bbb0f, EAU),
    (
        &["warm_ocean", "lukewarm_ocean", "deep_lukewarm_ocean"],
        0x8eb971,
        0x71a74d,
        0x45adf2,
    ),
];

impl Teintes {
    /// Couleurs du biome (nom avec ou sans « minecraft: ») ; « plaines » s'il est inconnu.
    pub fn du_biome(biome: Option<&str>) -> Self {
        let nom = biome.map(|b| b.strip_prefix("minecraft:").unwrap_or(b));
        let (_, herbe, feuillage, eau) = TABLE
            .iter()
            .find(|(noms, ..)| nom.is_some_and(|n| noms.contains(&n)))
            .unwrap_or(&TABLE[0]);
        Self {
            herbe: *herbe,
            feuillage: *feuillage,
            eau: *eau,
        }
    }
}

impl Default for Teintes {
    fn default() -> Self {
        Self::du_biome(None)
    }
}

/// Biome le plus présent de la schématique (Sponge 3 : `Biomes`, Sponge 2 : `BiomePalette`
/// et `BiomeData`), sans « minecraft: ».
pub fn biome_dominant(s: &Compound) -> Option<String> {
    let (palette, donnees) = match s.compound("Biomes") {
        Some(b) => (b.compound("Palette")?, b.get("Data")?),
        None => (s.compound("BiomePalette")?, s.get("BiomeData")?),
    };
    let Nbt::Octets(donnees) = donnees else {
        return None;
    };
    let mut comptes = vec![0u32; palette.0.len() + 1];
    let mut lu = 0;
    while lu < donnees.len() {
        let (mut valeur, mut decalage) = (0usize, 0);
        loop {
            let o = donnees.get(lu).map_or(0, |&o| o as u8);
            lu += 1;
            valeur |= ((o & 0x7f) as usize) << decalage;
            if o & 0x80 == 0 || decalage > 28 {
                break;
            }
            decalage += 7;
        }
        if let Some(c) = comptes.get_mut(valeur) {
            *c += 1;
        }
    }
    let (plus, _) = comptes.iter().enumerate().max_by_key(|(_, c)| **c)?;
    palette
        .0
        .iter()
        .find(|(_, i)| i.entier() == Some(plus as i64))
        .map(|(nom, _)| nom.strip_prefix("minecraft:").unwrap_or(nom).to_owned())
}

#[cfg(test)]
mod essais {
    use super::{Teintes, biome_dominant};
    use crate::minecraft::nbt::{Compound, Nbt};

    #[test]
    fn biome_le_plus_present() {
        let c = |v: Vec<(&str, Nbt)>| {
            Nbt::Compound(Compound(
                v.into_iter().map(|(k, n)| (k.to_owned(), n)).collect(),
            ))
        };
        let palette = c(vec![
            ("minecraft:plains", Nbt::Entier(0)),
            ("minecraft:swamp", Nbt::Entier(1)),
        ]);
        let Nbt::Compound(s) = c(vec![(
            "Biomes",
            c(vec![
                ("Palette", palette),
                ("Data", Nbt::Octets(vec![1, 0, 1, 1])),
            ]),
        )]) else {
            unreachable!()
        };
        assert_eq!(biome_dominant(&s).as_deref(), Some("swamp"));
        assert_eq!(biome_dominant(&Compound(Vec::new())), None);
    }

    #[test]
    fn couleurs_par_biome() {
        assert_eq!(Teintes::du_biome(None).herbe, 0x91bd59);
        assert_eq!(Teintes::du_biome(Some("minecraft:swamp")).eau, 0x617b64);
        assert_eq!(Teintes::du_biome(Some("inconnu")), Teintes::default());
    }
}
