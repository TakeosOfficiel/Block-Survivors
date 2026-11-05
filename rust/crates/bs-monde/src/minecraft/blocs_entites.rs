//! Blocs dessinés par une entité dans Minecraft (coffres) : leur modèle vanilla
//! est vide, on le refait ici en boîtes texturées avec la texture d'entité (64 × 64,
//! dépliée comme les modèles d'entités), tournées selon `facing`.

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

/// Taille des textures d'entité (pixels) : les UV des modèles vont de 0 à 16.
const TAILLE_TEXTURE: f32 = 64.0;

fn modele(texture: String, boites: &[Boite]) -> ModeleResolu {
    let echelle = 16.0 / TAILLE_TEXTURE;
    let elements = boites
        .iter()
        .map(|b| {
            let faces: Map<String, Value> = b
                .faces
                .iter()
                .map(|(d, z)| {
                    let uv = z.map(|v| v * echelle);
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
    texture.map(|t| (coffre(t), ry))
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
    }
}
