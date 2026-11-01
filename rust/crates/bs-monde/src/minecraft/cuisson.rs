//! Cuisson d'un modèle de bloc vanilla en quadrilatères (fin de src/minecraft/modeles.ts) :
//! éléments (boîtes en seizièmes), rotation d'élément avec « rescale », rotation d'état
//! par quarts de tour, UV par défaut, tournées ou verrouillées (« uvlock »).
//!
//! Conventions Minecraft : x vers l'est, y vers le haut, z vers le sud ; coordonnées
//! des modèles en seizièmes, UV en pixels (v vers le bas).

use serde_json::{Map, Value};

use super::modeles::chemin_ressource;

pub type V3 = [f32; 3];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Bas,
    Haut,
    Nord,
    Sud,
    Ouest,
    Est,
}

impl Direction {
    pub const TOUTES: [Direction; 6] = [
        Direction::Bas,
        Direction::Haut,
        Direction::Nord,
        Direction::Sud,
        Direction::Ouest,
        Direction::Est,
    ];

    /// Nom dans les fichiers de Minecraft.
    pub fn nom(self) -> &'static str {
        ["down", "up", "north", "south", "west", "east"][self as usize]
    }

    pub fn de_nom(n: &str) -> Option<Self> {
        Self::TOUTES.into_iter().find(|d| d.nom() == n)
    }

    pub fn vecteur(self) -> [i32; 3] {
        [
            [0, -1, 0],
            [0, 1, 0],
            [0, 0, -1],
            [0, 0, 1],
            [-1, 0, 0],
            [1, 0, 0],
        ][self as usize]
    }

    fn vecteur_f(self) -> V3 {
        self.vecteur().map(|v| v as f32)
    }

    /// Direction la plus proche d'un vecteur.
    pub fn de_vecteur(v: V3) -> Self {
        let mut meilleure = Direction::Haut;
        let mut produit = f32::NEG_INFINITY;
        for d in Self::TOUTES {
            let n = d.vecteur_f();
            let p = v[0] * n[0] + v[1] * n[1] + v[2] * n[2];
            if p > produit {
                produit = p;
                meilleure = d;
            }
        }
        meilleure
    }
}

/// Quadrilatère cuit : coins haut-gauche, haut-droit, bas-droit, bas-gauche vus de face.
#[derive(Debug, Clone)]
pub struct Quad {
    /// Repère du bloc (0 à 1).
    pub positions: [V3; 4],
    /// Pixels de la texture (0 à 16, v vers le bas).
    pub uv: [[f32; 2]; 4],
    pub texture: String,
    pub normale: V3,
    /// Face masquée si le voisin dans cette direction est un bloc plein opaque.
    pub cullface: Option<Direction>,
    /// Teinte de biome (herbe, feuillage, eau…).
    pub teinte: bool,
    /// Faux : éclairée comme une face du dessus (plantes).
    pub ombre: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ModeleCuit {
    pub quads: Vec<Quad>,
    /// Un seul cube de 0 à 16, sans rotation : peut cacher les faces de ses voisins.
    pub cube_plein: bool,
    /// Boîte englobante des éléments (seizièmes) : sert à deviner les collisions.
    pub boite: Option<(V3, V3)>,
    /// Tous les éléments sont des plans sans épaisseur (plantes en croix, cultures).
    pub plat: bool,
}

/// Modèle et ses parents fusionnés.
#[derive(Debug, Clone, Default)]
pub struct ModeleResolu {
    pub elements: Vec<Value>,
    pub textures: Map<String, Value>,
}

fn v3(v: Option<&Value>) -> Option<V3> {
    let a = v?.as_array()?;
    Some([0, 1, 2].map(|k| a.get(k).and_then(Value::as_f64).unwrap_or(0.0) as f32))
}

/// Coins (TL, TR, BR, BL) d'une face d'élément, en seizièmes.
fn coins_face(d: Direction, f: V3, t: V3) -> [V3; 4] {
    match d {
        Direction::Nord => [
            [t[0], t[1], f[2]],
            [f[0], t[1], f[2]],
            [f[0], f[1], f[2]],
            [t[0], f[1], f[2]],
        ],
        Direction::Sud => [
            [f[0], t[1], t[2]],
            [t[0], t[1], t[2]],
            [t[0], f[1], t[2]],
            [f[0], f[1], t[2]],
        ],
        Direction::Ouest => [
            [f[0], t[1], f[2]],
            [f[0], t[1], t[2]],
            [f[0], f[1], t[2]],
            [f[0], f[1], f[2]],
        ],
        Direction::Est => [
            [t[0], t[1], t[2]],
            [t[0], t[1], f[2]],
            [t[0], f[1], f[2]],
            [t[0], f[1], t[2]],
        ],
        Direction::Haut => [
            [f[0], t[1], f[2]],
            [t[0], t[1], f[2]],
            [t[0], t[1], t[2]],
            [f[0], t[1], t[2]],
        ],
        Direction::Bas => [
            [f[0], f[1], t[2]],
            [t[0], f[1], t[2]],
            [t[0], f[1], f[2]],
            [f[0], f[1], f[2]],
        ],
    }
}

/// UV d'un point selon la direction de sa face (« uvlock »).
fn uv_projete(d: Direction, p: V3) -> [f32; 2] {
    match d {
        Direction::Haut => [p[0], p[2]],
        Direction::Bas => [p[0], 16.0 - p[2]],
        Direction::Nord => [16.0 - p[0], 16.0 - p[1]],
        Direction::Sud => [p[0], 16.0 - p[1]],
        Direction::Ouest => [p[2], 16.0 - p[1]],
        Direction::Est => [16.0 - p[2], 16.0 - p[1]],
    }
}

fn uv_par_defaut(d: Direction, f: V3, t: V3) -> [f32; 4] {
    match d {
        Direction::Bas => [f[0], 16.0 - t[2], t[0], 16.0 - f[2]],
        Direction::Haut => [f[0], f[2], t[0], t[2]],
        Direction::Nord => [16.0 - t[0], 16.0 - t[1], 16.0 - f[0], 16.0 - f[1]],
        Direction::Sud => [f[0], 16.0 - t[1], t[0], 16.0 - f[1]],
        Direction::Ouest => [f[2], 16.0 - t[1], t[2], 16.0 - f[1]],
        Direction::Est => [16.0 - t[2], 16.0 - t[1], 16.0 - f[2], 16.0 - f[1]],
    }
}

/// Rotation directe (main droite) de `angle` degrés autour de l'axe 0, 1 ou 2.
fn tourner(p: V3, axe: usize, angle: f32, centre: V3) -> V3 {
    let a = angle.to_radians();
    let (s, c) = a.sin_cos();
    let [x, y, z] = [0, 1, 2].map(|k| p[k] - centre[k]);
    let r = match axe {
        0 => [x, y * c - z * s, y * s + z * c],
        1 => [x * c + z * s, y, -x * s + z * c],
        _ => [x * c - y * s, x * s + y * c, z],
    };
    [0, 1, 2].map(|k| r[k] + centre[k])
}

const CENTRE: V3 = [8.0; 3];
const ORIGINE: V3 = [0.0; 3];

/// Rotation d'état : x puis y, par quarts de tour (sens horaire vu de l'axe).
fn tourner_etat(p: V3, rx: i32, ry: i32, centre: V3) -> V3 {
    let mut q = p;
    if rx != 0 {
        q = tourner(q, 0, -rx as f32, centre);
    }
    if ry != 0 {
        q = tourner(q, 1, -ry as f32, centre);
    }
    q
}

fn resoudre_texture(reference: &str, textures: &Map<String, Value>) -> Option<String> {
    let mut r = reference.to_owned();
    for _ in 0..8 {
        let Some(var) = r.strip_prefix('#') else {
            break;
        };
        r = match textures.get(var)? {
            Value::String(s) => s.clone(),
            v => v.get("sprite")?.as_str()?.to_owned(),
        };
    }
    (!r.starts_with('#')).then(|| chemin_ressource(&r).to_owned())
}

pub fn cuire_modele(modele: &ModeleResolu, rx: i32, ry: i32, uvlock: bool) -> ModeleCuit {
    let mut quads = Vec::new();
    let mut boite: Option<(V3, V3)> = None;
    let mut plat = !modele.elements.is_empty();
    for e in &modele.elements {
        let (Some(f), Some(t)) = (v3(e.get("from")), v3(e.get("to"))) else {
            continue;
        };
        boite = Some(match boite {
            None => (f, t),
            Some((a, b)) => (
                [0, 1, 2].map(|k| a[k].min(f[k])),
                [0, 1, 2].map(|k| b[k].max(t[k])),
            ),
        });
        if f[0] != t[0] && f[1] != t[1] && f[2] != t[2] {
            plat = false;
        }
        // Rotation de l'élément (et mise à l'échelle « rescale » des deux autres axes)
        let r = e.get("rotation");
        let angle = r
            .and_then(|r| r.get("angle"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0) as f32;
        let axe = match r.and_then(|r| r.get("axis")).and_then(Value::as_str) {
            Some("x") => 0,
            Some("z") => 2,
            _ => 1,
        };
        let origine = v3(r.and_then(|r| r.get("origin"))).unwrap_or(CENTRE);
        let rescale = r.and_then(|r| r.get("rescale")).and_then(Value::as_bool) == Some(true);
        let echelle = if rescale && angle != 0.0 {
            1.0 / angle.abs().to_radians().cos()
        } else {
            1.0
        };
        let tourne = r.is_some() && angle != 0.0;
        let transformer = |p: V3| {
            let mut q = p;
            if tourne {
                q = tourner(q, axe, angle, origine);
                if echelle != 1.0 {
                    q = [0, 1, 2].map(|k| {
                        if k == axe {
                            q[k]
                        } else {
                            origine[k] + (q[k] - origine[k]) * echelle
                        }
                    });
                }
            }
            tourner_etat(q, rx, ry, CENTRE)
        };
        let ombre = e.get("shade").and_then(Value::as_bool) != Some(false)
            && e.get("shade_direction_override").is_none();
        let Some(Value::Object(faces)) = e.get("faces") else {
            continue;
        };
        for (nom, face) in faces {
            let Some(d) = Direction::de_nom(nom) else {
                continue;
            };
            let Some(texture) = face
                .get("texture")
                .and_then(Value::as_str)
                .and_then(|r| resoudre_texture(r, &modele.textures))
            else {
                continue;
            };
            let coins = coins_face(d, f, t).map(transformer);
            let mut normale = d.vecteur_f();
            if tourne {
                normale = tourner(normale, axe, angle, ORIGINE);
            }
            normale = tourner_etat(normale, rx, ry, ORIGINE);
            let finale = Direction::de_vecteur(normale);
            let uv = if uvlock {
                coins.map(|p| uv_projete(finale, p))
            } else {
                let [u0, v0, u1, v1] = match face.get("uv").and_then(Value::as_array) {
                    Some(a) if a.len() == 4 => {
                        [0, 1, 2, 3].map(|k| a[k].as_f64().unwrap_or(0.0) as f32)
                    }
                    _ => uv_par_defaut(d, f, t),
                };
                let base = [[u0, v0], [u1, v0], [u1, v1], [u0, v1]];
                // Rotation de la texture sur la face, par quarts de tour (sens horaire)
                let rot = face.get("rotation").and_then(Value::as_i64).unwrap_or(0);
                let k = ((rot / 90) % 4 + 4) % 4;
                [0, 1, 2, 3].map(|i| base[((i - k + 4) % 4) as usize])
            };
            let cullface = face
                .get("cullface")
                .and_then(Value::as_str)
                .and_then(Direction::de_nom)
                .map(|c| Direction::de_vecteur(tourner_etat(c.vecteur_f(), rx, ry, ORIGINE)));
            quads.push(Quad {
                positions: coins.map(|p| p.map(|v| v / 16.0)),
                uv,
                texture,
                normale,
                cullface,
                teinte: face.get("tintindex").is_some(),
                ombre,
            });
        }
    }
    let cube_plein = match modele.elements.as_slice() {
        [e] => {
            let angle = e
                .get("rotation")
                .and_then(|r| r.get("angle"))
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            angle == 0.0
                && v3(e.get("from")) == Some([0.0; 3])
                && v3(e.get("to")) == Some([16.0; 3])
                && e.get("faces")
                    .and_then(Value::as_object)
                    .is_some_and(|f| f.len() == 6)
        }
        _ => false,
    };
    ModeleCuit {
        quads,
        cube_plein,
        boite,
        plat,
    }
}
