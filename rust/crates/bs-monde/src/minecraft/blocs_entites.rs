//! Blocs dessinés par une entité dans Minecraft (coffres, têtes, bannières) : leur modèle vanilla
//! est vide, on le refait ici en boîtes texturées avec la texture d'entité (dépliée
//! comme les modèles d'entités), tournées selon `facing` ou `rotation`.

use serde_json::{Map, Value, json};

use super::bannieres::banniere;
use super::cuisson::{Direction, ModeleResolu};
use super::schematique::Schematique;
use super::tetes::{rotation_sol, tete};

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

/// Moitié d'un coffre double : où est l'autre moitié (vue du coffre tourné au nord).
#[derive(Clone, Copy, PartialEq)]
enum Moitie {
    Seul,
    /// `type=left` : l'autre moitié est à l'est.
    Gauche,
    /// `type=right` : l'autre moitié est à l'ouest.
    Droite,
}

/// Coffre, serrure au nord : couvercle (5 px) sur une caisse (9 px visibles). Dans la
/// texture, tout est retourné (de haut en bas et de gauche à droite) et le premier
/// côté déplié est l'est. Une moitié de coffre double fait 15 px de large,
/// collée à l'autre moitié, sans la face qui la touche.
fn coffre(texture: &str, moitie: Moitie) -> ModeleResolu {
    use Direction::*;
    let (l, p) = (if moitie == Moitie::Seul { 14.0 } else { 15.0 }, 14.0);
    // Côtés dépliés : est (p), arrière sud (l), ouest (p), avant nord (l)
    let cotes = |y_bas: f32, y_haut: f32| {
        let mut x = 0.0;
        let mut c = Vec::new();
        for (d, largeur) in [(Est, p), (Sud, l), (Ouest, p), (Nord, l)] {
            let colle =
                (d == Est && moitie == Moitie::Gauche) || (d == Ouest && moitie == Moitie::Droite);
            if !colle {
                c.push((d, [x, y_haut, x + largeur, y_bas]));
            }
            x += largeur;
        }
        c
    };
    let mut caisse = cotes(33.0, 42.0);
    caisse.push((Bas, [p, 19.0, p + l, 19.0 + p]));
    let mut couvercle = cotes(14.0, 19.0);
    couvercle.push((Haut, [p + l, 0.0, p + 2.0 * l, p]));
    // Serrure : 2 px au milieu d'un coffre seul, 1 px de chaque moitié contre l'autre
    let ls = if moitie == Moitie::Seul { 2.0 } else { 1.0 };
    let serrure = vec![
        (Nord, [ls + 2.0, 5.0, 2.0 * ls + 2.0, 1.0]),
        (Est, [0.0, 5.0, 1.0, 1.0]),
        (Ouest, [ls + 1.0, 5.0, ls + 2.0, 1.0]),
        (Haut, [ls + 1.0, 0.0, 2.0 * ls + 1.0, 1.0]),
        (Bas, [1.0, 0.0, ls + 1.0, 1.0]),
    ];
    // Retournement de gauche à droite (le haut en bas est déjà dans les zones)
    let miroir = |faces: Vec<(Direction, Zone)>| -> Vec<(Direction, Zone)> {
        faces
            .into_iter()
            .map(|(d, [u0, v0, u1, v1])| (d, [u1, v0, u0, v1]))
            .collect()
    };
    let (caisse, couvercle, serrure) = (miroir(caisse), miroir(couvercle), miroir(serrure));
    let ([x0, x1], [s0, s1]) = match moitie {
        Moitie::Seul => ([1.0, 15.0], [7.0, 9.0]),
        Moitie::Gauche => ([1.0, 16.0], [15.0, 16.0]),
        Moitie::Droite => ([0.0, 15.0], [0.0, 1.0]),
    };
    let suffixe = match moitie {
        Moitie::Seul => "",
        Moitie::Gauche => "_left",
        Moitie::Droite => "_right",
    };
    modele(
        format!("entity/chest/{texture}{suffixe}"),
        [64.0, 64.0],
        &[
            Boite {
                de: [x0, 0.0, 1.0],
                a: [x1, 9.0, 15.0],
                faces: caisse,
            },
            Boite {
                de: [x0, 9.0, 1.0],
                a: [x1, 14.0, 15.0],
                faces: couvercle,
            },
            Boite {
                de: [s0, 7.0, 0.0],
                a: [s1, 11.0, 1.0],
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
    if let Some(t) = texture {
        let moitie = match propriete("type").as_deref() {
            Some("left") if t != "ender" => Moitie::Gauche,
            Some("right") if t != "ender" => Moitie::Droite,
            _ => Moitie::Seul,
        };
        return Some((coffre(t, moitie), ry));
    }
    let mur = nom.replacen("_wall", "", 1);
    if let Some(couleur) = mur.strip_suffix("_banner") {
        return Some(if mur != nom {
            (banniere(couleur, true, propriete("motifs").as_deref()), ry)
        } else {
            let ry = rotation_sol(propriete("rotation").as_deref());
            (banniere(couleur, false, propriete("motifs").as_deref()), ry)
        });
    }
    let modele = tete(&mur, mur != nom)?;
    if mur != nom {
        Some((modele, ry))
    } else {
        Some((modele, rotation_sol(propriete("rotation").as_deref())))
    }
}

/// Coffre de la palette qui peut former un coffre double : son orientation.
fn coffre_seul(etat: &str) -> Option<&str> {
    let (nom, reste) = etat.split_once('[')?;
    let nom = nom.strip_prefix("waxed_").unwrap_or(nom);
    let coffre = matches!(nom, "chest" | "trapped_chest") || nom.ends_with("copper_chest");
    if !coffre || reste.contains("type=") {
        return None;
    }
    let f = reste
        .split(['[', ',', ']'])
        .find_map(|p| p.strip_prefix("facing="))?;
    Some(f)
}

/// Les structures de Bedrock ne disent pas quels coffres forment un coffre double :
/// deux coffres voisins pareils, côte à côte face au même côté, sont réunis comme
/// Minecraft le fait quand on les pose (`type=left` pour celui dont l'autre moitié
/// est à sa droite vue de face, c'est-à-dire un quart de tour horaire plus loin).
pub(super) fn apparier_coffres(s: &mut Schematique) {
    let orientations: Vec<Option<String>> = s
        .palette
        .iter()
        .map(|e| coffre_seul(e).map(str::to_owned))
        .collect();
    if orientations.iter().all(Option::is_none) {
        return;
    }
    let variante = |s: &mut Schematique, b: u16, t: &str| -> u16 {
        let e = s.palette[b as usize].replacen(']', &format!(",type={t}]"), 1);
        match s.palette.iter().position(|p| *p == e) {
            Some(k) => k as u16,
            None => {
                s.palette.push(e);
                (s.palette.len() - 1) as u16
            }
        }
    };
    let (l, p) = (s.largeur, s.profondeur);
    for i in 0..s.blocs.len() {
        let b = s.blocs[i];
        let Some(Some(f)) = orientations.get(b as usize) else {
            continue;
        };
        let (x, z) = ((i % l) as i32, (i / l % p) as i32);
        // Un quart de tour horaire à partir de l'orientation
        let (dx, dz) = match f.as_str() {
            "north" => (1, 0),
            "east" => (0, 1),
            "south" => (-1, 0),
            _ => (0, -1),
        };
        let y = (i / (l * p)) as i32;
        let voisin = |x: i32, z: i32| s.bloc(x, y, z) == Some(b);
        let (j, k) = if voisin(x + dx, z + dz) {
            (x + dx, z + dz)
        } else if voisin(x - dx, z - dz) {
            (x - dx, z - dz)
        } else {
            continue;
        };
        let gauche = (j, k) == (x + dx, z + dz);
        let a = variante(s, b, if gauche { "left" } else { "right" });
        let c = variante(s, b, if gauche { "right" } else { "left" });
        s.blocs[i] = a;
        s.blocs[(y as usize * p + k as usize) * l + j as usize] = c;
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
        let (m, _) = modele_bloc_entite("red_wall_banner", |k| {
            (k == "motifs").then(|| "cross:blue".to_owned())
        })
        .unwrap();
        assert_eq!(m.textures["c0"], "entity/banner/base");
        assert_eq!(m.textures["c1"], "entity/banner/cross");
        // Barre, puis le tissu et un calque de motif
        assert_eq!(m.elements.len(), 3);
    }

    #[test]
    fn coffres_doubles_de_bedrock() {
        let mut s = Schematique {
            largeur: 3,
            hauteur: 1,
            profondeur: 1,
            palette: vec!["air".into(), "chest[facing=north]".into()],
            blocs: vec![1, 1, 1],
            panneaux: Vec::new(),
            version_donnees: 0,
            biome: None,
            grille_biomes: None,
        };
        apparier_coffres(&mut s);
        let etats: Vec<&str> = s
            .blocs
            .iter()
            .map(|&b| s.palette[b as usize].as_str())
            .collect();
        assert_eq!(
            etats,
            [
                "chest[facing=north,type=left]",
                "chest[facing=north,type=right]",
                "chest[facing=north]"
            ]
        );
    }
}
