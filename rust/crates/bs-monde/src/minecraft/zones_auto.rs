//! Zones d'ennemis par défaut d'une map importée qui n'en déclare aucune (ni `.json`,
//! ni panneau `[ennemis]`) : la zone jouable est découpée en carrés de 12 blocs, et un
//! carré au sol fait surtout de blocs du Nether prend la population de l'arène
//! « nether » ; un carré surtout sous terre (sol et plafond de pierre) celle de la
//! « mine ». Les carrés voisins de même nature sur une ligne forment une seule zone.
//! Rien n'est ajouté pour la nature de l'arène de base de la map (même population).

use bs_donnees::Donnees;
use bs_donnees::arenes::{Arene, ZoneEnnemis};

use super::modeles::BlocCuit;
use super::schematique::Schematique;
use super::zone::Zone;

/// Côté (blocs) des carrés classés.
const TUILE: usize = 12;
/// Part des cases praticables d'un carré qu'il faut pour le classer.
const PART_MIN: f32 = 0.5;
/// Hauteur (blocs au-dessus des pieds) jusqu'où l'on cherche un plafond.
const PLAFOND_MAX: i32 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Nature {
    Nether,
    Mine,
}

impl Nature {
    fn arene(self) -> &'static str {
        match self {
            Nature::Nether => "nether",
            Nature::Mine => "mine",
        }
    }
}

fn du_nether(nom: &str) -> bool {
    matches!(
        nom,
        "netherrack"
            | "soul_sand"
            | "soul_soil"
            | "basalt"
            | "polished_basalt"
            | "magma_block"
            | "glowstone"
            | "crimson_nylium"
            | "warped_nylium"
            | "nether_wart_block"
            | "warped_wart_block"
            | "ancient_debris"
            | "nether_gold_ore"
            | "nether_quartz_ore"
            | "shroomlight"
    ) || nom.contains("nether_brick")
        || nom.contains("blackstone")
}

fn de_pierre(nom: &str) -> bool {
    matches!(
        nom,
        "stone"
            | "cobblestone"
            | "mossy_cobblestone"
            | "gravel"
            | "granite"
            | "diorite"
            | "andesite"
            | "tuff"
            | "calcite"
            | "dripstone_block"
            | "smooth_basalt"
            | "dirt"
            | "coarse_dirt"
            | "rooted_dirt"
            | "clay"
            | "infested_stone"
    ) || nom.contains("deepslate")
        || nom.ends_with("_ore")
}

/// Zones des carrés du Nether ou souterrains ; vide si l'arène `base` les a déjà.
pub fn zones_par_defaut(
    d: &Donnees,
    s: &Schematique,
    blocs: &[BlocCuit],
    zone: &Zone,
    base: &Arene,
) -> Vec<ZoneEnnemis> {
    let nom = |x: i32, y: i32, z: i32| {
        s.bloc(x, y, z)
            .map_or("air", |i| blocs[i as usize].nom.as_str())
    };
    let nature_case = |ix: usize, iz: usize| {
        let (x, z) = (zone.x0 + ix as i32, zone.z0 + iz as i32);
        let y = zone.niveaux[iz * zone.largeur + ix];
        let sol = nom(x, y - 1, z);
        if du_nether(sol) {
            return Some(Nature::Nether);
        }
        let plafond = (2..=PLAFOND_MAX)
            .map(|h| nom(x, y + h, z))
            .find(|n| *n != "air" && !n.ends_with("cave_air"));
        (de_pierre(sol) && plafond.is_some_and(de_pierre)).then_some(Nature::Mine)
    };
    let (tx, tz) = (
        zone.largeur.div_ceil(TUILE),
        zone.profondeur.div_ceil(TUILE),
    );
    let mut natures = vec![None; tx * tz];
    for (k, n) in natures.iter_mut().enumerate() {
        let (cx, cz) = ((k % tx) * TUILE, (k / tx) * TUILE);
        let (mut praticables, mut nether, mut mine) = (0, 0, 0);
        for iz in cz..(cz + TUILE).min(zone.profondeur) {
            for ix in cx..(cx + TUILE).min(zone.largeur) {
                if zone.cases[iz * zone.largeur + ix] != 1 {
                    continue;
                }
                praticables += 1;
                match nature_case(ix, iz) {
                    Some(Nature::Nether) => nether += 1,
                    Some(Nature::Mine) => mine += 1,
                    None => {}
                }
            }
        }
        let part = |c: i32| praticables > 0 && c as f32 >= praticables as f32 * PART_MIN;
        *n = if part(nether) {
            Some(Nature::Nether)
        } else if part(mine) {
            Some(Nature::Mine)
        } else {
            None
        };
    }
    let (dl, dp) = (zone.largeur as f32 / 2.0, zone.profondeur as f32 / 2.0);
    let mut zones = Vec::new();
    for z in 0..tz {
        let mut x = 0;
        while x < tx {
            let Some(n) = natures[z * tx + x] else {
                x += 1;
                continue;
            };
            let debut = x;
            while x < tx && natures[z * tx + x] == Some(n) {
                x += 1;
            }
            let Some(a) = d.arenes.arenes.iter().find(|a| a.id == n.arene()) else {
                continue;
            };
            let memes = |a: &Arene, b: &Arene| {
                a.ennemis.len() == b.ennemis.len()
                    && a.ennemis
                        .iter()
                        .all(|e| b.ennemis.iter().any(|f| f.ennemi == e.ennemi))
            };
            if memes(a, base) {
                continue;
            }
            let x1 = (x * TUILE).min(zone.largeur) as f32;
            let z1 = ((z + 1) * TUILE).min(zone.profondeur) as f32;
            zones.push(ZoneEnnemis {
                nom: a.nom.clone(),
                rect: [
                    (debut * TUILE) as f32 - dl,
                    (z * TUILE) as f32 - dp,
                    x1 - dl,
                    z1 - dp,
                ],
                ennemis: a.ennemis.clone(),
            });
        }
    }
    zones
}
