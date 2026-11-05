//! Blocs dessinés par une entité dans Minecraft (coffres, têtes) : leur modèle vanilla
//! est vide, on le refait ici en boîtes texturées avec la texture d'entité (dépliée
//! comme les modèles d'entités), tournées selon `facing` ou `rotation`.

use serde_json::{Map, Value, json};

use super::cuisson::{Direction, ModeleResolu};

/// Rectangle de texture en pixels (x0, y0, x1, y1) ; y0 > y1 retourne l'image.
type Zone = [f32; 4];

/// Boîte (en seizièmes) et zone de texture de chacune de ses faces visibles.
struct Boite {
    de: [f32; 3],
    a: [f32; 3],
    faces: Vec<(Direction, Zone)>,
}

/// Modèle d'une texture d'entité de `taille` pixels (largeur, hauteur) : les UV des
/// modèles vont de 0 à 16 sur toute la texture.
fn modele(texture: String, taille: [f32; 2], boites: &[Boite]) -> ModeleResolu {
    let elements = boites
        .iter()
        .map(|b| {
            let faces: Map<String, Value> = b
                .faces
                .iter()
                .map(|(d, z)| {
                    let e = [16.0 / taille[0], 16.0 / taille[1]];
                    let uv = [z[0] * e[0], z[1] * e[1], z[2] * e[0], z[3] * e[1]];
                    (d.nom().to_owned(), json!({ "texture": "#t", "uv": uv }))
                })
                .collect();
            json!({ "from": b.de, "to": b.a, "faces": faces })
        })
        .collect();
    ModeleResolu {
        textures: Map::from_iter([("t".to_owned(), Value::String(texture))]),
        elements,
    }
}

/// Quart de tour (degrés, autour de y) d'un bloc tourné vers `facing` (nord par défaut).
fn rotation(facing: Option<&str>) -> i32 {
    match facing {
        Some("east") => 90,
        Some("south") => 180,
        Some("west") => 270,
        _ => 0,
    }
}

/// Coffre simple, serrure au nord : couvercle (5 px) sur une caisse (9 px visibles).
/// Dans la texture, les côtés sont à l'envers (le haut de l'image est le bas du bloc).
fn coffre(texture: &str) -> ModeleResolu {
    use Direction::*;
    let cotes = |y_bas: f32, y_haut: f32| {
        // Ouest, arrière (sud), est, avant (nord), de 14 px de large
        [(Ouest, 0.0), (Sud, 14.0), (Est, 28.0), (Nord, 42.0)]
            .map(|(d, x)| (d, [x, y_haut, x + 14.0, y_bas]))
    };
    let mut caisse = cotes(33.0, 42.0).to_vec();
    caisse.push((Bas, [14.0, 19.0, 28.0, 33.0]));
    let mut couvercle = cotes(14.0, 19.0).to_vec();
    couvercle.push((Haut, [28.0, 0.0, 42.0, 14.0]));
    let serrure = vec![
        (Nord, [4.0, 5.0, 6.0, 1.0]),
        (Ouest, [0.0, 5.0, 1.0, 1.0]),
        (Est, [3.0, 5.0, 4.0, 1.0]),
        (Haut, [1.0, 0.0, 3.0, 1.0]),
        (Bas, [3.0, 0.0, 5.0, 1.0]),
    ];
    modele(
        format!("entity/chest/{texture}"),
        [64.0, 64.0],
        &[
            Boite {
                de: [1.0, 0.0, 1.0],
                a: [15.0, 9.0, 15.0],
                faces: caisse,
            },
            Boite {
                de: [1.0, 9.0, 1.0],
                a: [15.0, 14.0, 15.0],
                faces: couvercle,
            },
            Boite {
                de: [7.0, 7.0, 0.0],
                a: [9.0, 11.0, 1.0],
                faces: serrure,
            },
        ],
    )
}

/// Tête de 8 px, face au nord, posée au sol (au centre) ou au mur (contre le bloc au
/// sud). La texture est celle de l'entité : tête dépliée en haut à gauche.
fn tete(texture: &str, hauteur_texture: f32, mur: bool) -> ModeleResolu {
    use Direction::*;
    let faces = vec![
        (Haut, [8.0, 0.0, 16.0, 8.0]),
        (Bas, [16.0, 0.0, 24.0, 8.0]),
        (Est, [0.0, 8.0, 8.0, 16.0]),
        (Nord, [8.0, 8.0, 16.0, 16.0]),
        (Ouest, [16.0, 8.0, 24.0, 16.0]),
        (Sud, [24.0, 8.0, 32.0, 16.0]),
    ];
    let (de, a) = if mur {
        ([4.0, 4.0, 8.0], [12.0, 12.0, 16.0])
    } else {
        ([4.0, 0.0, 4.0], [12.0, 8.0, 12.0])
    };
    modele(
        format!("entity/{texture}"),
        [64.0, hauteur_texture],
        &[Boite { de, a, faces }],
    )
}

/// Texture et hauteur de texture de la tête `nom` (sans « _wall »).
fn texture_tete(nom: &str) -> Option<(&'static str, f32)> {
    Some(match nom {
        "skeleton_skull" => ("skeleton/skeleton", 32.0),
        "wither_skeleton_skull" => ("skeleton/wither_skeleton", 32.0),
        "creeper_head" => ("creeper/creeper", 32.0),
        "zombie_head" => ("zombie/zombie", 64.0),
        "player_head" => ("player/wide/steve", 64.0),
        _ => return None,
    })
}

/// Quart de tour d'une tête posée au sol : `rotation` 0 regarde au sud, 4 à l'ouest…
fn rotation_sol(rotation: Option<&str>) -> i32 {
    let r: i32 = rotation.and_then(|r| r.parse().ok()).unwrap_or(0);
    (180 + (r + 2) / 4 % 4 * 90) % 360
}

/// Modèle et rotation (degrés autour de y) d'un bloc à entité, s'il en a un ici.
pub(super) fn modele_bloc_entite(
    nom: &str,
    propriete: impl Fn(&str) -> Option<String>,
) -> Option<(ModeleResolu, i32)> {
    let facing = propriete("facing");
    let ry = rotation(facing.as_deref());
    let texture = match nom.strip_prefix("waxed_").unwrap_or(nom) {
        "chest" => Some("normal"),
        "trapped_chest" => Some("trapped"),
        "ender_chest" => Some("ender"),
        "copper_chest" => Some("copper"),
        "exposed_copper_chest" => Some("copper_exposed"),
        "weathered_copper_chest" => Some("copper_weathered"),
        "oxidized_copper_chest" => Some("copper_oxidized"),
        _ => None,
    };
    if let Some(t) = texture {
        return Some((coffre(t), ry));
    }
    let mur = nom.replacen("_wall", "", 1);
    let (t, h) = texture_tete(&mur)?;
    if mur != nom {
        Some((tete(t, h, true), ry))
    } else {
        let ry = rotation_sol(propriete("rotation").as_deref());
        Some((tete(t, h, false), ry))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coffres_tournes() {
        let (m, ry) = modele_bloc_entite("waxed_oxidized_copper_chest", |k| {
            (k == "facing").then(|| "east".to_owned())
        })
        .unwrap();
        assert_eq!(ry, 90);
        assert_eq!(m.elements.len(), 3);
        assert!(modele_bloc_entite("red_bed", |_| None).is_none());
        let (_, ry) = modele_bloc_entite("creeper_head", |k| {
            (k == "rotation").then(|| "4".to_owned())
        })
        .unwrap();
        assert_eq!(ry, 270);
        assert!(modele_bloc_entite("zombie_wall_head", |_| None).is_some());
    }
}
