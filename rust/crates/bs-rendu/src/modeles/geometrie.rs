//! Géométrie des modèles (src/moteur/geometrie.ts) : boîte d'une partie avec le patron
//! de skin Minecraft, et objet tenu en main extrudé depuis son sprite (plaque d'un
//! pixel d'épaisseur, rebord coloré par chaque pixel du bord).

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bs_donnees::blocs::Cube;

use super::compilation::PIXELS_PAR_BLOC;

/// Maillage en construction (quadrilatères).
#[derive(Debug, Default, Clone)]
pub struct Maillage {
    pub positions: Vec<[f32; 3]>,
    pub normales: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub couleurs: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl Maillage {
    /// Coins en tour (un sens ou l'autre) : les triangles sont orientés pour que leur
    /// face avant regarde du côté de `normale`.
    pub(crate) fn quad(
        &mut self,
        coins: [Vec3; 4],
        normale: Vec3,
        uvs: [[f32; 2]; 4],
        couleur: [f32; 4],
    ) {
        let b = self.positions.len() as u32;
        let direct = (coins[1] - coins[0])
            .cross(coins[2] - coins[0])
            .dot(normale)
            > 0.0;
        self.positions.extend(coins.map(|c| c.to_array()));
        self.normales.extend([normale.to_array(); 4]);
        self.uvs.extend(uvs);
        self.couleurs.extend([couleur; 4]);
        if direct {
            self.indices.extend([b, b + 1, b + 2, b, b + 2, b + 3]);
        } else {
            self.indices.extend([b, b + 3, b + 2, b, b + 2, b + 1]);
        }
    }

    pub fn en_mesh(&self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normales.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.couleurs.clone())
        .with_inserted_indices(Indices::U32(self.indices.clone()))
    }
}

/// Faces de la boîte comme `THREE.BoxGeometry` : normale, puis position du coin (u, v)
/// (v = 1 en haut) en demi-tailles. Ordre : +X, −X, +Y, −Y, +Z, −Z.
const FACES: [(Vec3, fn(f32, f32) -> Vec3); 6] = [
    (Vec3::X, |u, v| Vec3::new(1.0, 2.0 * v - 1.0, 1.0 - 2.0 * u)),
    (Vec3::NEG_X, |u, v| {
        Vec3::new(-1.0, 2.0 * v - 1.0, 2.0 * u - 1.0)
    }),
    (Vec3::Y, |u, v| Vec3::new(2.0 * u - 1.0, 1.0, 1.0 - 2.0 * v)),
    (Vec3::NEG_Y, |u, v| {
        Vec3::new(2.0 * u - 1.0, -1.0, 2.0 * v - 1.0)
    }),
    (Vec3::Z, |u, v| Vec3::new(2.0 * u - 1.0, 2.0 * v - 1.0, 1.0)),
    (Vec3::NEG_Z, |u, v| {
        Vec3::new(1.0 - 2.0 * u, 2.0 * v - 1.0, -1.0)
    }),
];

/// Boîte d'une partie, relative à son articulation, avec le patron de skin habituel
/// (côtés, dessus, dessous, avant, arrière ; `miroir` pour les membres gauches).
pub fn cube(m: &mut Maillage, c: &Cube, taille_texture: [f32; 2], couleur: [f32; 4]) {
    let [l, h, p] = c.taille;
    let [u, v] = c.uv;
    let g = 2.0 * c.gonflement.unwrap_or(0.0);
    let demi = Vec3::new(l + g, h + g, p + g) / (2.0 * PIXELS_PAR_BLOC);
    let centre = Vec3::from_array(c.centre) / PIXELS_PAR_BLOC;
    // [x, y, largeur, hauteur] en pixels, dans l'ordre des faces
    let gauche = [u + p + l, v + p, p, h];
    let droite = [u, v + p, p, h];
    let rectangles = [
        if c.miroir { droite } else { gauche },
        if c.miroir { gauche } else { droite },
        [u + p, v, l, p],
        [u + p + l, v, l, p],
        [u + p, v + p, l, h],
        [u + 2.0 * p + l, v + p, l, h],
    ];
    for (k, ((normale, coin), [x, y, largeur, hauteur])) in FACES.iter().zip(rectangles).enumerate()
    {
        // Le dessous est retourné verticalement par rapport aux autres faces, comme en jeu
        let retourne = k == 3;
        let uv = |su: f32, sv: f32| {
            let su = if c.miroir { 1.0 - su } else { su };
            let ligne = if retourne { sv } else { 1.0 - sv };
            [
                (x + su * largeur) / taille_texture[0],
                (y + ligne * hauteur) / taille_texture[1],
            ]
        };
        let sommets = [(0.0, 1.0), (1.0, 1.0), (1.0, 0.0), (0.0, 0.0)];
        m.quad(
            sommets.map(|(a, b)| centre + coin(a, b) * demi),
            *normale,
            sommets.map(|(a, b)| uv(a, b)),
            couleur,
        );
    }
}

/// Objet tenu, extrudé depuis son sprite RGBA (`largeur` × `hauteur`) : plaque dans le
/// plan x = 0 centrée sur l'origine, droite du sprite vers +Z, haut vers +Y ; `pixel` =
/// côté d'un pixel en blocs.
pub fn objet(pixels: &[u8], largeur: usize, hauteur: usize, pixel: f32) -> Maillage {
    objet_epais(pixels, largeur, hauteur, pixel, pixel)
}

/// Comme [`objet`], d'épaisseur `epaisseur` blocs (sorts plus massifs qu'un objet tenu).
pub fn objet_epais(
    pixels: &[u8],
    largeur: usize,
    hauteur: usize,
    pixel: f32,
    epaisseur: f32,
) -> Maillage {
    let opaque = |x: isize, y: isize| {
        x >= 0
            && y >= 0
            && (x as usize) < largeur
            && (y as usize) < hauteur
            && pixels[(y as usize * largeur + x as usize) * 4 + 3] >= 128
    };
    let mut m = Maillage::default();
    let blanc = [1.0; 4];
    let e = epaisseur / 2.0;
    let z = |colonne: f32| (colonne - largeur as f32 / 2.0) * pixel;
    let y = |ligne: f32| (hauteur as f32 / 2.0 - ligne) * pixel;
    let (z0, z1, y0, y1) = (z(0.0), z(largeur as f32), y(hauteur as f32), y(0.0));
    // Avant (+X) et arrière (−X) : tout le sprite, découpé par la transparence
    m.quad(
        [
            Vec3::new(e, y1, z1),
            Vec3::new(e, y1, z0),
            Vec3::new(e, y0, z0),
            Vec3::new(e, y0, z1),
        ],
        Vec3::X,
        [[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
        blanc,
    );
    m.quad(
        [
            Vec3::new(-e, y1, z0),
            Vec3::new(-e, y1, z1),
            Vec3::new(-e, y0, z1),
            Vec3::new(-e, y0, z0),
        ],
        Vec3::NEG_X,
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        blanc,
    );
    // Rebords : UV au centre du pixel, qui colore toute la tranche
    for ligne in 0..hauteur as isize {
        for colonne in 0..largeur as isize {
            if !opaque(colonne, ligne) {
                continue;
            }
            let uv = [
                (colonne as f32 + 0.5) / largeur as f32,
                (ligne as f32 + 0.5) / hauteur as f32,
            ];
            let (za, zb) = (z(colonne as f32), z(colonne as f32 + 1.0));
            let (ya, yb) = (y(ligne as f32 + 1.0), y(ligne as f32));
            let v = Vec3::new;
            if !opaque(colonne, ligne - 1) {
                m.quad(
                    [v(-e, yb, za), v(-e, yb, zb), v(e, yb, zb), v(e, yb, za)],
                    Vec3::Y,
                    [uv; 4],
                    blanc,
                );
            }
            if !opaque(colonne, ligne + 1) {
                m.quad(
                    [v(e, ya, za), v(e, ya, zb), v(-e, ya, zb), v(-e, ya, za)],
                    Vec3::NEG_Y,
                    [uv; 4],
                    blanc,
                );
            }
            if !opaque(colonne - 1, ligne) {
                m.quad(
                    [v(-e, yb, za), v(e, yb, za), v(e, ya, za), v(-e, ya, za)],
                    Vec3::NEG_Z,
                    [uv; 4],
                    blanc,
                );
            }
            if !opaque(colonne + 1, ligne) {
                m.quad(
                    [v(e, yb, zb), v(-e, yb, zb), v(-e, ya, zb), v(e, ya, zb)],
                    Vec3::Z,
                    [uv; 4],
                    blanc,
                );
            }
        }
    }
    m
}
