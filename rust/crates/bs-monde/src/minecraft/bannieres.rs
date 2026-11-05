//! Bannières : tissu de la couleur de fond, puis un calque par motif (bande, croix,
//! creeper…), chacun teint de sa couleur, comme Minecraft les superpose. Les motifs
//! viennent de l'entité du bloc (`patterns` de Java 1.20.5+, `Patterns` des versions
//! plus anciennes et de Bedrock, en codes courts) ; ils sont rangés dans l'état du bloc
//! (`motifs=stripe_top:blue+cross:red`) pour que chaque bannière ait son modèle.

use serde_json::{Map, Value, json};

use super::cuisson::ModeleResolu;
use super::nbt::{Compound, Nbt};

/// Teintures de Java, dans l'ordre de leurs numéros (Bedrock : ordre inverse).
pub const TEINTURES: [&str; 16] = [
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "light_gray",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black",
];

/// Couleur 0xRRGGBB des teintures sur une texture (même ordre que `TEINTURES`).
const COULEURS: [u32; 16] = [
    0xf9fffe, 0xf9801d, 0xc74ebd, 0x3ab3da, 0xfed83d, 0x80c71f, 0xf38baa, 0x474f52, 0x9d9d97,
    0x169c9c, 0x8932b8, 0x3c44aa, 0x835432, 0x5e7c16, 0xb02e26, 0x1d1d21,
];

/// Codes courts des motifs (Java avant 1.20.5, Bedrock) et leur nom.
const CODES: [(&str, &str); 43] = [
    ("b", "base"),
    ("bs", "stripe_bottom"),
    ("ts", "stripe_top"),
    ("ls", "stripe_left"),
    ("rs", "stripe_right"),
    ("cs", "stripe_center"),
    ("ms", "stripe_middle"),
    ("drs", "stripe_downright"),
    ("dls", "stripe_downleft"),
    ("ss", "small_stripes"),
    ("cr", "cross"),
    ("sc", "straight_cross"),
    ("bt", "triangle_bottom"),
    ("tt", "triangle_top"),
    ("bts", "triangles_bottom"),
    ("tts", "triangles_top"),
    ("ld", "diagonal_left"),
    ("rd", "diagonal_up_right"),
    ("lud", "diagonal_up_left"),
    ("rud", "diagonal_right"),
    ("mc", "circle"),
    ("mr", "rhombus"),
    ("vh", "half_vertical"),
    ("hh", "half_horizontal"),
    ("vhr", "half_vertical_right"),
    ("hhb", "half_horizontal_bottom"),
    ("bl", "square_bottom_left"),
    ("br", "square_bottom_right"),
    ("tl", "square_top_left"),
    ("tr", "square_top_right"),
    ("sku", "skull"),
    ("flo", "flower"),
    ("cre", "creeper"),
    ("moj", "mojang"),
    ("bri", "bricks"),
    ("gra", "gradient"),
    ("gru", "gradient_up"),
    ("cbo", "curly_border"),
    ("bo", "border"),
    ("glb", "globe"),
    ("pig", "piglin"),
    ("flw", "flow"),
    ("gus", "guster"),
];

fn nom_du_code(code: &str) -> Option<&'static str> {
    CODES.iter().find(|(c, _)| *c == code).map(|(_, n)| *n)
}

/// Motifs d'une entité de bannière de Java (`Data` d'un .schem ou entité à plat).
pub fn motifs_java(e: &Compound) -> Option<String> {
    let e = e.compound("Data").unwrap_or(e);
    let mut motifs = Vec::new();
    if let Some(l) = e.get("patterns").and_then(Nbt::liste) {
        for m in l {
            let Nbt::Compound(m) = m else { continue };
            let nom = m.get("pattern").and_then(Nbt::chaine).unwrap_or("");
            let couleur = m.get("color").and_then(Nbt::chaine).unwrap_or("white");
            let nom = nom.strip_prefix("minecraft:").unwrap_or(nom);
            motifs.push(format!("{nom}:{couleur}"));
        }
    }
    motifs.extend(anciens_motifs(e, false));
    (!motifs.is_empty()).then(|| motifs.join("+"))
}

/// Motifs d'une entité de bannière de Bedrock.
pub fn motifs_bedrock(e: &Compound) -> Option<String> {
    let motifs: Vec<String> = anciens_motifs(e, true).collect();
    (!motifs.is_empty()).then(|| motifs.join("+"))
}

/// `Patterns` en codes courts ; numéros de teinture dans l'ordre inverse pour Bedrock.
fn anciens_motifs(e: &Compound, inverse: bool) -> impl Iterator<Item = String> + '_ {
    let liste = e.get("Patterns").and_then(Nbt::liste).unwrap_or(&[]);
    liste.iter().filter_map(move |m| {
        let Nbt::Compound(m) = m else { return None };
        let nom = nom_du_code(m.get("Pattern").and_then(Nbt::chaine)?)?;
        let c = m
            .get("Color")
            .and_then(Nbt::entier)
            .unwrap_or(0)
            .clamp(0, 15) as usize;
        let couleur = TEINTURES[if inverse { 15 - c } else { c }];
        Some(format!("{nom}:{couleur}"))
    })
}

/// État de bannière avec ses motifs.
pub fn avec_motifs(etat: &str, motifs: &str) -> String {
    match etat.strip_suffix(']') {
        Some(debut) => format!("{debut},motifs={motifs}]"),
        None => format!("{etat}[motifs={motifs}]"),
    }
}

fn couleur(teinture: &str) -> u32 {
    TEINTURES
        .iter()
        .position(|t| *t == teinture)
        .map_or(0xffffff, |k| COULEURS[k])
}

/// Bannière tournée au nord, sur un mât de bois : debout (mât de 42 px, tissu de
/// 20 × 40 px devant) ou au mur (contre le bloc au sud, le tissu pendant sur le bloc du
/// dessous). Tissu et motifs prennent les textures d'entité, teintes de leur couleur ;
/// chaque calque est un peu plus épais que le précédent pour passer devant.
pub(super) fn banniere(fond: &str, mur: bool, motifs: Option<&str>) -> ModeleResolu {
    let bois = |de: [f32; 3], a: [f32; 3], uv: [f32; 4]| {
        let faces: Map<String, Value> = ["north", "south", "east", "west", "up", "down"]
            .iter()
            .map(|d| ((*d).to_owned(), json!({ "texture": "#bois", "uv": uv })))
            .collect();
        json!({ "from": de, "to": a, "faces": faces })
    };
    let (z, haut) = if mur { (13.0, 14.0) } else { (6.0, 42.0) };
    let mut elements = vec![bois(
        [-2.0, haut, z],
        [18.0, haut + 2.0, z + 3.0],
        [0.0, 0.0, 16.0, 2.0],
    )];
    if !mur {
        elements.push(bois(
            [7.0, 0.0, 7.0],
            [9.0, 42.0, 9.0],
            [7.0, 0.0, 9.0, 16.0],
        ));
    }
    let mut textures = Map::from_iter([("bois".to_owned(), json!("block/oak_planks"))]);
    let calques = std::iter::once(("base", fond)).chain(
        motifs
            .unwrap_or("")
            .split('+')
            .filter_map(|m| m.split_once(':')),
    );
    // Texture d'entité de 64 × 64 : tissu de 20 × 40 × 1 en (0, 0)
    let uv = |r: [f32; 4]| r.map(|v| v / 4.0);
    for (k, (motif, teinture)) in calques.enumerate() {
        let cle = format!("c{k}");
        textures.insert(cle.clone(), json!(format!("entity/banner/{motif}")));
        let t = format!("#{cle}");
        let c = couleur(teinture);
        let mut faces = Map::from_iter([
            (
                "north".to_owned(),
                json!({ "texture": t, "uv": uv([1.0, 1.0, 21.0, 41.0]), "couleur": c }),
            ),
            (
                "south".to_owned(),
                json!({ "texture": t, "uv": uv([22.0, 1.0, 42.0, 41.0]), "couleur": c }),
            ),
        ]);
        if k == 0 {
            for (d, r) in [
                ("east", [0.0, 1.0, 1.0, 41.0]),
                ("west", [21.0, 1.0, 22.0, 41.0]),
                ("up", [1.0, 0.0, 21.0, 1.0]),
                ("down", [21.0, 0.0, 41.0, 1.0]),
            ] {
                faces.insert(
                    d.to_owned(),
                    json!({ "texture": t, "uv": uv(r), "couleur": c }),
                );
            }
        }
        let e = 0.02 * k as f32;
        elements.push(json!({
            "from": [-2.0, haut - 40.0, z - e],
            "to": [18.0, haut, z + 1.0 + e],
            "faces": faces,
        }));
    }
    ModeleResolu { textures, elements }
}

#[cfg(test)]
mod essais {
    use super::*;

    fn motif(cles: Vec<(&str, Nbt)>) -> Nbt {
        Nbt::Compound(Compound(
            cles.into_iter().map(|(k, v)| (k.to_owned(), v)).collect(),
        ))
    }

    #[test]
    fn motifs_de_java_et_de_bedrock() {
        let nouveau = Compound(vec![(
            "patterns".into(),
            Nbt::Liste(vec![motif(vec![
                ("pattern", Nbt::Chaine("minecraft:stripe_top".into())),
                ("color", Nbt::Chaine("light_blue".into())),
            ])]),
        )]);
        assert_eq!(
            motifs_java(&nouveau).as_deref(),
            Some("stripe_top:light_blue")
        );
        let ancien = |couleur| {
            Compound(vec![(
                "Patterns".into(),
                Nbt::Liste(vec![
                    motif(vec![
                        ("Pattern", Nbt::Chaine("cre".into())),
                        ("Color", Nbt::Entier(couleur)),
                    ]),
                    motif(vec![("Pattern", Nbt::Chaine("inconnu".into()))]),
                ]),
            )])
        };
        assert_eq!(motifs_java(&ancien(14)).as_deref(), Some("creeper:red"));
        assert_eq!(motifs_bedrock(&ancien(0)).as_deref(), Some("creeper:black"));
        assert_eq!(motifs_bedrock(&Compound::default()), None);
        assert_eq!(
            avec_motifs("red_banner[rotation=4]", "cross:blue"),
            "red_banner[rotation=4,motifs=cross:blue]"
        );
    }
}
