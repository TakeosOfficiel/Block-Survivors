//! Structures de Bedrock (.mcstructure) : NBT petit-boutiste, blocs rangés x puis y
//! puis z (z le plus rapide), une palette d'états Bedrock (`name` et `states`) et une
//! seconde couche (eau des blocs immergés). Les noms et états sont traduits vers ceux de
//! Java pour les blocs courants ; un état inconnu est laissé de côté (le modèle prend
//! alors sa première variante compatible).

use super::nbt::{Compound, Nbt};
use super::schematique::{Palette, Panneau, Schematique, texte_de};

/// Volume maximal accepté (blocs).
const VOLUME_MAX: usize = 1 << 27;

/// Noms Bedrock → Java (anciens noms compris).
const NOMS: [(&str, &str); 42] = [
    ("skull", "skeleton_skull"),
    ("bed", "red_bed"),
    ("grass", "grass_block"),
    ("tallgrass", "short_grass"),
    ("snow", "snow_block"),
    ("snow_layer", "snow"),
    ("flowing_water", "water"),
    ("flowing_lava", "lava"),
    ("lit_pumpkin", "jack_o_lantern"),
    ("brick_block", "bricks"),
    ("stonebrick", "stone_bricks"),
    ("yellow_flower", "dandelion"),
    ("red_flower", "poppy"),
    ("deadbush", "dead_bush"),
    ("waterlily", "lily_pad"),
    ("web", "cobweb"),
    ("reeds", "sugar_cane"),
    ("fence_gate", "oak_fence_gate"),
    ("wooden_door", "oak_door"),
    ("trapdoor", "oak_trapdoor"),
    ("wooden_button", "oak_button"),
    ("wooden_pressure_plate", "oak_pressure_plate"),
    ("standing_sign", "oak_sign"),
    ("wall_sign", "oak_wall_sign"),
    ("mob_spawner", "spawner"),
    ("invisible_bedrock", "barrier"),
    ("noteblock", "note_block"),
    ("magma", "magma_block"),
    ("nether_brick", "nether_bricks"),
    ("red_nether_brick", "red_nether_bricks"),
    ("quartz_ore", "nether_quartz_ore"),
    ("slime", "slime_block"),
    ("end_bricks", "end_stone_bricks"),
    ("melon_block", "melon"),
    ("golden_rail", "powered_rail"),
    ("light_block", "light"),
    ("structure_void", "air"),
    ("unknown", "air"),
    ("frame", "air"),
    ("glow_frame", "air"),
    ("stone_block_slab", "smooth_stone_slab"),
    ("double_stone_block_slab", "smooth_stone_slab"),
];

/// Faces numérotées de Bedrock (`facing_direction`).
const FACES: [&str; 6] = ["down", "up", "north", "south", "west", "east"];
/// `direction` (0 à 3) de Bedrock.
const DIRECTIONS: [&str; 4] = ["south", "west", "north", "east"];
/// `weirdo_direction` des escaliers.
const ESCALIERS: [&str; 4] = ["east", "west", "south", "north"];

fn texte(states: &Compound, cle: &str) -> Option<String> {
    match states.get(cle)? {
        Nbt::Chaine(s) => Some(s.clone()),
        v => v.entier().map(|n| n.to_string()),
    }
}

fn indice<'a>(table: &[&'a str], states: &Compound, cle: &str) -> Option<&'a str> {
    let n = states.get(cle)?.entier()?;
    table.get(usize::try_from(n).ok()?).copied()
}

/// État Java (« oak_stairs[facing=east,half=bottom] ») d'une entrée de palette Bedrock.
pub(super) fn etat_java(nom: &str, states: &Compound) -> String {
    let nom = nom.strip_prefix("minecraft:").unwrap_or(nom);
    let mut nom = NOMS
        .iter()
        .find(|(b, _)| *b == nom)
        .map_or(nom, |(_, j)| j)
        .to_owned();
    let mut p: Vec<(&str, String)> = Vec::new();
    if let Some(base) = nom.strip_suffix("_double_slab") {
        nom = format!("{base}_slab");
        p.push(("type", "double".into()));
    } else if nom.starts_with("double_stone_block_slab") {
        p.push(("type", "double".into()));
    } else if let Some(m) = texte(states, "minecraft:vertical_half") {
        p.push(("type", m));
    }
    if let Some(a) = texte(states, "pillar_axis") {
        p.push(("axis", a));
    }
    if nom.ends_with("_stairs") {
        if let Some(f) = indice(&ESCALIERS, states, "weirdo_direction") {
            p.push(("facing", f.into()));
        }
        let haut = states.get("upside_down_bit").and_then(Nbt::entier) == Some(1);
        p.push(("half", if haut { "top" } else { "bottom" }.into()));
        p.push(("shape", "straight".into()));
    } else if let Some(f) = texte(states, "minecraft:cardinal_direction") {
        p.push(("facing", f));
    } else if let Some(f) = indice(&FACES, states, "facing_direction") {
        p.push(("facing", f.into()));
    } else if let Some(f) = indice(&DIRECTIONS, states, "direction") {
        p.push(("facing", f.into()));
    }
    if let Some(h) = states.get("head_piece_bit").and_then(Nbt::entier) {
        p.push(("part", if h == 1 { "head" } else { "foot" }.into()));
    }
    if let Some(h) = states.get("upper_block_bit").and_then(Nbt::entier) {
        p.push(("half", if h == 1 { "upper" } else { "lower" }.into()));
    }
    for (b, j) in [("open_bit", "open"), ("hanging", "hanging")] {
        if let Some(v) = states.get(b).and_then(Nbt::entier) {
            p.push((j, (v == 1).to_string()));
        }
    }
    if let Some(r) = texte(states, "ground_sign_direction") {
        p.push(("rotation", r));
    }
    if nom == "snow"
        && let Some(h) = states.get("height").and_then(Nbt::entier)
    {
        p.push(("layers", (h + 1).to_string()));
    }
    if nom == "torch" || nom.ends_with("_torch") {
        let f = texte(states, "torch_facing_direction");
        if let Some(f @ ("north" | "south" | "east" | "west")) = f.as_deref() {
            nom = nom.replace("torch", "wall_torch");
            p.push(("facing", f.into()));
        }
    }
    // Têtes : face 1 (vers le haut) au sol, sinon au mur
    if nom.ends_with("_skull") || (nom.ends_with("_head") && nom != "piston_head") {
        let k = p.iter().position(|(c, _)| *c == "facing");
        if let Some(k) = k {
            if p[k].1 == "up" || p[k].1 == "down" {
                p.remove(k);
            } else if let Some((base, fin)) = nom.rsplit_once('_') {
                nom = format!("{base}_wall_{fin}");
            }
        }
    }
    if p.is_empty() {
        return nom;
    }
    let paires: Vec<String> = p.iter().map(|(k, v)| format!("{k}={v}")).collect();
    format!("{nom}[{}]", paires.join(","))
}

fn liste_entiers(n: Option<&Nbt>) -> Vec<i64> {
    n.and_then(Nbt::entiers).unwrap_or_default()
}

pub(super) fn lire_mcstructure(racine: &Compound) -> Result<Schematique, String> {
    let taille = liste_entiers(racine.get("size"));
    let [largeur, hauteur, profondeur] = match taille[..] {
        [x, y, z] if x > 0 && y > 0 && z > 0 => [x, y, z].map(|v| v as usize),
        _ => return Err("Structure Bedrock invalide : size manquant.".into()),
    };
    let volume = largeur * hauteur * profondeur;
    if volume > VOLUME_MAX {
        return Err(format!(
            "Structure Bedrock trop grande ({largeur} × {hauteur} × {profondeur} blocs)."
        ));
    }
    let structure = racine
        .compound("structure")
        .ok_or("Structure Bedrock invalide : structure manquante.")?;
    let couches = structure
        .get("block_indices")
        .and_then(Nbt::liste)
        .unwrap_or(&[]);
    let principale = liste_entiers(couches.first());
    let eau = liste_entiers(couches.get(1));
    let defaut = structure
        .compound("palette")
        .and_then(|p| p.compound("default"))
        .ok_or("Structure Bedrock invalide : palette manquante.")?;
    let etats: Vec<String> = defaut
        .get("block_palette")
        .and_then(Nbt::liste)
        .unwrap_or(&[])
        .iter()
        .map(|e| match e {
            Nbt::Compound(e) => etat_java(
                e.get("name").and_then(Nbt::chaine).unwrap_or("air"),
                e.compound("states").unwrap_or(&Compound::default()),
            ),
            _ => "air".into(),
        })
        .collect();
    let mut palette = Palette::nouvelle();
    let passage: Vec<u16> = etats.iter().map(|e| palette.ajouter(e)).collect();
    // Index des états immergés (« waterlogged=true » ajouté), créés au besoin
    let mut immerges: Vec<Option<u16>> = vec![None; etats.len()];
    let mut blocs = vec![0u16; volume];
    let positions = defaut.compound("block_position_data");
    let mut panneaux = Vec::new();
    for (i, &k) in principale.iter().enumerate().take(volume) {
        let (x, reste) = (i / (hauteur * profondeur), i % (hauteur * profondeur));
        let (y, z) = (reste / profondeur, reste % profondeur);
        let k = usize::try_from(k).ok().filter(|&k| k < etats.len());
        let dans_l_eau = eau
            .get(i)
            .and_then(|&k| etats.get(usize::try_from(k).ok()?))
            .is_some_and(|e| e.starts_with("water"));
        let b = match k {
            Some(k) if passage[k] != 0 && dans_l_eau => *immerges[k].get_or_insert_with(|| {
                let e = &etats[k];
                palette.ajouter(&match e.strip_suffix(']') {
                    Some(debut) => format!("{debut},waterlogged=true]"),
                    None => format!("{e}[waterlogged=true]"),
                })
            }),
            Some(k) if passage[k] != 0 => passage[k],
            _ if dans_l_eau => palette.ajouter("water"),
            _ => 0,
        };
        let etat = k.map(|k| &etats[k]);
        blocs[(y * profondeur + z) * largeur + x] = b;
        if etat.is_some_and(|e| e.contains("sign"))
            && let Some(Nbt::Compound(donnees)) = positions
                .and_then(|p| p.compound(&i.to_string()))
                .and_then(|p| p.get("block_entity_data"))
        {
            let textes = ["FrontText", "BackText", "Text"]
                .iter()
                .filter_map(|c| donnees.get(c).map(|v| ((*c).to_owned(), v.clone())))
                .collect();
            panneaux.push(Panneau {
                x: x as i32,
                y: y as i32,
                z: z as i32,
                texte: texte_de(&Nbt::Compound(Compound(textes))),
            });
        }
    }
    Ok(Schematique {
        largeur,
        hauteur,
        profondeur,
        palette: palette.etats,
        blocs,
        panneaux,
        version_donnees: 0,
        biome: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minecraft::schematique::lire_schematique;

    /// Petit écrivain NBT petit-boutiste (juste ce qu'il faut pour le test).
    fn ecrire(v: &Nbt, o: &mut Vec<u8>) {
        match v {
            Nbt::Entier(n) => o.extend(n.to_le_bytes()),
            Nbt::Octet(n) => o.push(*n as u8),
            Nbt::Chaine(s) => {
                o.extend((s.len() as u16).to_le_bytes());
                o.extend(s.as_bytes());
            }
            Nbt::Liste(l) => {
                o.push(l.first().map_or(1, type_de));
                o.extend((l.len() as i32).to_le_bytes());
                l.iter().for_each(|x| ecrire(x, o));
            }
            Nbt::Compound(c) => {
                for (nom, x) in &c.0 {
                    o.push(type_de(x));
                    ecrire(&Nbt::Chaine(nom.clone()), o);
                    ecrire(x, o);
                }
                o.push(0);
            }
            _ => unreachable!(),
        }
    }

    fn type_de(v: &Nbt) -> u8 {
        match v {
            Nbt::Octet(_) => 1,
            Nbt::Entier(_) => 3,
            Nbt::Chaine(_) => 8,
            Nbt::Liste(_) => 9,
            Nbt::Compound(_) => 10,
            _ => unreachable!(),
        }
    }

    fn c(paires: Vec<(&str, Nbt)>) -> Nbt {
        Nbt::Compound(Compound(
            paires.into_iter().map(|(k, v)| (k.into(), v)).collect(),
        ))
    }

    fn entiers(v: &[i32]) -> Nbt {
        Nbt::Liste(v.iter().map(|&n| Nbt::Entier(n)).collect())
    }

    fn bloc(nom: &str, etats: Vec<(&str, Nbt)>) -> Nbt {
        c(vec![
            ("name", Nbt::Chaine(nom.into())),
            ("states", c(etats)),
        ])
    }

    #[test]
    fn structure_bedrock() {
        let palette = Nbt::Liste(vec![
            bloc("minecraft:air", vec![]),
            bloc("minecraft:grass", vec![]),
            bloc(
                "minecraft:oak_stairs",
                vec![
                    ("weirdo_direction", Nbt::Entier(3)),
                    ("upside_down_bit", Nbt::Octet(1)),
                ],
            ),
            bloc("minecraft:water", vec![("liquid_depth", Nbt::Entier(0))]),
            bloc(
                "minecraft:standing_sign",
                vec![("ground_sign_direction", Nbt::Entier(4))],
            ),
        ]);
        // 2 × 1 × 2, ordre x puis z : (0,0), (0,1), (1,0), (1,1)
        let panneau = c(vec![(
            "block_entity_data",
            c(vec![
                ("id", Nbt::Chaine("Sign".into())),
                ("FrontText", c(vec![("Text", Nbt::Chaine("[boss]".into()))])),
            ]),
        )]);
        let racine = c(vec![
            ("format_version", Nbt::Entier(1)),
            ("size", entiers(&[2, 1, 2])),
            (
                "structure",
                c(vec![
                    (
                        "block_indices",
                        Nbt::Liste(vec![entiers(&[1, 2, 4, 0]), entiers(&[-1, 3, -1, 3])]),
                    ),
                    (
                        "palette",
                        c(vec![(
                            "default",
                            c(vec![
                                ("block_palette", palette),
                                ("block_position_data", c(vec![("2", panneau)])),
                            ]),
                        )]),
                    ),
                ]),
            ),
            ("structure_world_origin", entiers(&[0, 0, 0])),
        ]);
        let mut octets = vec![10, 0, 0];
        let Nbt::Compound(r) = &racine else {
            unreachable!()
        };
        // Racine : type, nom vide, contenu
        for (nom, x) in &r.0 {
            octets.push(type_de(x));
            ecrire(&Nbt::Chaine(nom.clone()), &mut octets);
            ecrire(x, &mut octets);
        }
        octets.push(0);
        let s = lire_schematique(&octets).unwrap();
        assert_eq!((s.largeur, s.hauteur, s.profondeur), (2, 1, 2));
        let nom = |x, z| s.palette[s.bloc(x, 0, z).unwrap() as usize].as_str();
        assert_eq!(nom(0, 0), "grass_block");
        assert_eq!(
            nom(0, 1),
            "oak_stairs[facing=north,half=top,shape=straight,waterlogged=true]"
        );
        assert_eq!(nom(1, 0), "oak_sign[rotation=4]");
        assert_eq!(nom(1, 1), "water");
        assert_eq!(s.panneaux.len(), 1);
        assert_eq!((s.panneaux[0].x, s.panneaux[0].z), (1, 0));
        assert_eq!(s.panneaux[0].texte, "[boss]");
    }
}

#[cfg(test)]
mod tests_tetes {
    use super::*;

    fn face(f: i32) -> Compound {
        Compound(vec![("facing_direction".into(), Nbt::Entier(f))])
    }

    #[test]
    fn tetes_au_sol_et_au_mur() {
        assert_eq!(etat_java("minecraft:zombie_head", &face(1)), "zombie_head");
        assert_eq!(
            etat_java("minecraft:skeleton_skull", &face(2)),
            "skeleton_wall_skull[facing=north]"
        );
    }
}
