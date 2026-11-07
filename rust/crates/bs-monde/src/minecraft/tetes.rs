//! Têtes posées (squelette, zombie, creeper, joueur, piglin, dragon) : leur modèle
//! vanilla est vide, on refait celui de l'entité en boîtes de la texture d'entité.
//! Les boîtes sont écrites comme dans les modèles d'entités de Minecraft (pixels, y
//! vers le bas, face vers -z), puis retournées et placées dans le bloc comme le fait le
//! rendu des têtes : au sol au centre, au mur contre le bloc au sud.

use serde_json::{Map, Value, json};

use super::cuisson::ModeleResolu;

/// Boîte d'un modèle d'entité : coin, taille, coin de sa texture dépliée, gonflement
/// (calque du chapeau) et, pour les oreilles, rotation autour de z (pivot, degrés).
struct Cube {
    de: [f32; 3],
    taille: [f32; 3],
    uv: [f32; 2],
    gonfle: f32,
    rotation_z: Option<([f32; 3], f32)>,
}

const fn cube(de: [f32; 3], taille: [f32; 3], uv: [f32; 2]) -> Cube {
    Cube {
        de,
        taille,
        uv,
        gonfle: 0.0,
        rotation_z: None,
    }
}

const TETE: Cube = cube([-4.0, -8.0, -4.0], [8.0; 3], [0.0, 0.0]);
const CHAPEAU: Cube = Cube {
    gonfle: 0.5,
    ..cube([-4.0, -8.0, -4.0], [8.0; 3], [32.0, 0.0])
};

const PIGLIN: [Cube; 6] = [
    cube([-5.0, -8.0, -4.0], [10.0, 8.0, 8.0], [0.0, 0.0]),
    cube([-2.0, -4.0, -5.0], [4.0, 4.0, 1.0], [31.0, 1.0]),
    cube([2.0, -2.0, -5.0], [1.0, 2.0, 1.0], [2.0, 4.0]),
    cube([-3.0, -2.0, -5.0], [1.0, 2.0, 1.0], [2.0, 0.0]),
    Cube {
        rotation_z: Some(([4.5, -6.0, 0.0], -30.0)),
        ..cube([4.5, -6.0, -2.0], [1.0, 5.0, 4.0], [51.0, 6.0])
    },
    Cube {
        rotation_z: Some(([-4.5, -6.0, 0.0], 30.0)),
        ..cube([-5.5, -6.0, -2.0], [1.0, 5.0, 4.0], [39.0, 6.0])
    },
];

const DRAGON: [Cube; 7] = [
    cube([-6.0, -1.0, -24.0], [12.0, 5.0, 16.0], [176.0, 44.0]),
    cube([-8.0, -8.0, -10.0], [16.0; 3], [112.0, 30.0]),
    cube([-5.0, -12.0, -4.0], [2.0, 4.0, 6.0], [0.0, 0.0]),
    cube([-5.0, -3.0, -22.0], [2.0, 2.0, 4.0], [112.0, 0.0]),
    cube([3.0, -12.0, -4.0], [2.0, 4.0, 6.0], [0.0, 0.0]),
    cube([3.0, -3.0, -22.0], [2.0, 2.0, 4.0], [112.0, 0.0]),
    cube([-6.0, 4.0, -24.0], [12.0, 4.0, 16.0], [176.0, 65.0]),
];

/// Forme d'une tête : texture, taille de la texture, boîtes, échelle, et point du bloc
/// (en seizièmes, posée au sol) où va l'origine du modèle d'entité.
struct Forme {
    texture: &'static str,
    taille_texture: [f32; 2],
    cubes: &'static [Cube],
    echelle: f32,
    origine: [f32; 3],
}

const AU_SOL: [f32; 3] = [8.0, 0.0, 8.0];

fn forme(nom: &str) -> Option<Forme> {
    let simple = |texture, hauteur| Forme {
        texture,
        taille_texture: [64.0, hauteur],
        cubes: std::slice::from_ref(&TETE),
        echelle: 1.0,
        origine: AU_SOL,
    };
    const AVEC_CHAPEAU: [Cube; 2] = [TETE, CHAPEAU];
    Some(match nom {
        "skeleton_skull" => simple("skeleton/skeleton", 32.0),
        "wither_skeleton_skull" => simple("skeleton/wither_skeleton", 32.0),
        "creeper_head" => simple("creeper/creeper", 32.0),
        "zombie_head" => Forme {
            cubes: &AVEC_CHAPEAU,
            ..simple("zombie/zombie", 64.0)
        },
        "player_head" => Forme {
            cubes: &AVEC_CHAPEAU,
            ..simple("player/wide/steve", 64.0)
        },
        "piglin_head" => Forme {
            cubes: &PIGLIN,
            ..simple("piglin/piglin", 64.0)
        },
        // Tête de 16 px réduite aux trois quarts, posée sur sa mâchoire
        "dragon_head" => Forme {
            texture: "enderdragon/dragon",
            taille_texture: [256.0, 256.0],
            cubes: &DRAGON,
            echelle: 0.75,
            origine: [8.0, 6.0, 9.5],
        },
        _ => return None,
    })
}

/// Modèle de la tête `nom` (sans « _wall »), face au nord, au sol ou au mur.
pub(super) fn tete(nom: &str, mur: bool) -> Option<ModeleResolu> {
    let f = forme(nom)?;
    let o = if mur {
        [f.origine[0], f.origine[1] + 4.0, f.origine[2] + 4.0]
    } else {
        f.origine
    };
    let k = f.echelle;
    let e = [16.0 / f.taille_texture[0], 16.0 / f.taille_texture[1]];
    let elements = f
        .cubes
        .iter()
        .map(|c| {
            let [x, y, z] = c.de.map(|v| v - c.gonfle);
            let [l, h, p] = c.taille.map(|v| v + 2.0 * c.gonfle);
            // Retournée en x et en y, comme le rendu des têtes
            let de = [o[0] - (x + l) * k, o[1] - (y + h) * k, o[2] + z * k];
            let a = [o[0] - x * k, o[1] - y * k, o[2] + (z + p) * k];
            let [u, v] = c.uv;
            let [l, h, p] = c.taille;
            let zones = [
                ("up", [u + p, v, u + p + l, v + p]),
                ("down", [u + p + l, v, u + p + 2.0 * l, v + p]),
                ("east", [u, v + p, u + p, v + p + h]),
                ("north", [u + p, v + p, u + p + l, v + p + h]),
                ("west", [u + p + l, v + p, u + 2.0 * p + l, v + p + h]),
                (
                    "south",
                    [u + 2.0 * p + l, v + p, u + 2.0 * p + 2.0 * l, v + p + h],
                ),
            ];
            let faces: Map<String, Value> = zones
                .iter()
                .map(|(d, z)| {
                    let uv = [z[0] * e[0], z[1] * e[1], z[2] * e[0], z[3] * e[1]];
                    ((*d).to_owned(), json!({ "texture": "#t", "uv": uv }))
                })
                .collect();
            let mut element = json!({ "from": de, "to": a, "faces": faces });
            if let Some((pivot, angle)) = c.rotation_z {
                let origine = [
                    o[0] - pivot[0] * k,
                    o[1] - pivot[1] * k,
                    o[2] + pivot[2] * k,
                ];
                element["rotation"] = json!({ "origin": origine, "axis": "z", "angle": angle });
            }
            element
        })
        .collect();
    Some(ModeleResolu {
        textures: Map::from_iter([(
            "t".to_owned(),
            Value::String(format!("entity/{}", f.texture)),
        )]),
        elements,
    })
}

/// Quart de tour d'une tête posée au sol : `rotation` 0 regarde au sud, 4 à l'ouest…
pub(super) fn rotation_sol(rotation: Option<&str>) -> i32 {
    let r: i32 = rotation.and_then(|r| r.parse().ok()).unwrap_or(0);
    (180 + (r + 2) / 4 % 4 * 90) % 360
}
