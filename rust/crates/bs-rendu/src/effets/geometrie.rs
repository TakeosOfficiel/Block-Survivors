//! Géométries des objets du jeu (src/moteur/geometrie.ts, src/effets/peintures.ts) :
//! plans (de face ou au sol), flèche en croix, modèle cubique fusionné, et skin peinte
//! de la fusée d'artifice d'après les pixels de l'item.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bs_donnees::blocs::Modele;

use crate::modeles::compilation::PIXELS_PAR_BLOC;
use crate::modeles::geometrie::{Maillage, cube};

/// Rectangle de texture [x, y, largeur, hauteur] (pixels) en UV (u0, v0, u1, v1),
/// v vers le bas.
pub fn rectangle_uv([x, y, l, h]: [f32; 4], [lt, ht]: [f32; 2]) -> [f32; 4] {
    [x / lt, y / ht, (x + l) / lt, (y + h) / ht]
}

/// Plan vertical (normale +Z) centré, `l` × `h` blocs : tourné vers la caméra par
/// l'orientation de son instance.
pub fn plan(l: f32, h: f32, [u0, v0, u1, v1]: [f32; 4], couleur: [f32; 4]) -> Maillage {
    let (a, b) = (l / 2.0, h / 2.0);
    let mut m = Maillage::default();
    m.quad(
        [
            Vec3::new(-a, b, 0.0),
            Vec3::new(a, b, 0.0),
            Vec3::new(a, -b, 0.0),
            Vec3::new(-a, -b, 0.0),
        ],
        Vec3::Z,
        [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
        couleur,
    );
    m
}

/// Plan couché au sol (normale +Y), côté 1, centré.
pub fn plan_sol() -> Maillage {
    let mut m = Maillage::default();
    m.quad(
        [
            Vec3::new(-0.5, 0.0, -0.5),
            Vec3::new(0.5, 0.0, -0.5),
            Vec3::new(0.5, 0.0, 0.5),
            Vec3::new(-0.5, 0.0, 0.5),
        ],
        Vec3::Y,
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        [1.0; 4],
    );
    m
}

/// Flèche : deux plans en croix (vertical x = 0, horizontal y = 0), de l'arrière (−Z)
/// à la pointe (+Z), comme `creerGeometrieCroisee`.
pub fn croisee(longueur: f32, largeur: f32, uv: [f32; 4], couleur: [f32; 4]) -> Maillage {
    let [u0, v0, u1, v1] = uv;
    let (z0, z1, e) = (-longueur / 2.0, longueur / 2.0, largeur / 2.0);
    let uvs = [[u0, v1], [u1, v1], [u1, v0], [u0, v0]];
    let mut m = Maillage::default();
    m.quad(
        [
            Vec3::new(0.0, -e, z0),
            Vec3::new(0.0, -e, z1),
            Vec3::new(0.0, e, z1),
            Vec3::new(0.0, e, z0),
        ],
        Vec3::X,
        uvs,
        couleur,
    );
    m.quad(
        [
            Vec3::new(-e, 0.0, z0),
            Vec3::new(-e, 0.0, z1),
            Vec3::new(e, 0.0, z1),
            Vec3::new(e, 0.0, z0),
        ],
        Vec3::Y,
        uvs,
        couleur,
    );
    m
}

/// Modèle cubique fusionné (parties immobiles), couché dans le sens du vol : le haut
/// du modèle vers +Z, centré en hauteur (trident, fusée lancés).
pub fn modele_couche(def: &Modele, couleur: [f32; 4]) -> Maillage {
    modele_tourne(def, couleur, Quat::from_rotation_x(FRAC_PI_2), 1.0)
}

/// Modèle cubique fusionné dressé, centré en hauteur, haut de `hauteur` blocs
/// (cloche en orbite).
pub fn modele_debout(def: &Modele, couleur: [f32; 4], hauteur: f32) -> Maillage {
    let m = modele_tourne(def, couleur, Quat::IDENTITY, 1.0);
    let (bas, haut) = bornes(&m);
    let echelle = hauteur / (haut - bas).max(1e-3);
    modele_tourne(def, couleur, Quat::IDENTITY, echelle)
}

/// Bas et haut (y) d'un maillage.
fn bornes(m: &Maillage) -> (f32, f32) {
    m.positions
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), p| (a.min(p[1]), b.max(p[1])))
}

/// Parties fusionnées, centrées en hauteur, tournées par `couche`, mises à l'échelle.
fn modele_tourne(def: &Modele, couleur: [f32; 4], couche: Quat, echelle: f32) -> Maillage {
    let mut m = Maillage::default();
    for cube_def in def.parties.values() {
        let debut = m.positions.len();
        cube(&mut m, cube_def, def.taille_texture, couleur);
        // Ordre de la version TS : Z, puis Y, puis X
        let [rx, ry, rz] = cube_def.rotation.unwrap_or([0.0; 3]);
        let rotation = Quat::from_euler(EulerRot::XYZ, rx, ry, rz);
        let pivot = Vec3::from_array(cube_def.pivot) / PIXELS_PAR_BLOC;
        for k in debut..m.positions.len() {
            m.positions[k] = (rotation * Vec3::from_array(m.positions[k]) + pivot).to_array();
            m.normales[k] = (rotation * Vec3::from_array(m.normales[k])).to_array();
        }
    }
    let (bas, haut) = bornes(&m);
    let milieu = Vec3::new(0.0, (bas + haut) / 2.0, 0.0);
    for p in &mut m.positions {
        *p = (couche * (Vec3::from_array(*p) - milieu) * echelle).to_array();
    }
    for n in &mut m.normales {
        *n = (couche * Vec3::from_array(*n)).to_array();
    }
    m
}

/// Disque blanc à bord net (ombre de secours sans la texture du jeu), RGBA.
pub fn disque(cote: usize) -> Vec<u8> {
    let mut pixels = vec![0u8; cote * cote * 4];
    let r = cote as f32 / 2.0;
    for y in 0..cote {
        for x in 0..cote {
            let (dx, dy) = (x as f32 + 0.5 - r, y as f32 + 0.5 - r);
            if dx * dx + dy * dy <= r * r {
                pixels[(y * cote + x) * 4..][..4].copy_from_slice(&[255; 4]);
            }
        }
    }
    pixels
}

type Couleur = [u8; 3];

/// Faces d'un cube dans le patron : droite, avant, gauche, arrière, dessus, dessous.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Face {
    Cote,
    Dessus,
    Dessous,
}

/// Remplit le patron d'un cube (même dépliage que les modèles) pixel par pixel ;
/// `couleur(face, colonne, ligne)`, colonne continue tout autour des côtés.
fn peindre_cube(
    pixels: &mut [u8],
    largeur: usize,
    [u, v]: [usize; 2],
    [l, h, p]: [usize; 3],
    couleur: impl Fn(Face, usize, usize) -> Couleur,
) {
    let faces = [
        (Face::Cote, u, v + p, p, h),
        (Face::Cote, u + p, v + p, l, h),
        (Face::Cote, u + p + l, v + p, p, h),
        (Face::Cote, u + 2 * p + l, v + p, l, h),
        (Face::Dessus, u + p, v, l, p),
        (Face::Dessous, u + p + l, v, l, p),
    ];
    for (face, x0, y0, fl, fh) in faces {
        for y in 0..fh {
            for x in 0..fl {
                let colonne = if face == Face::Cote { x0 - u + x } else { x };
                let [r, g, b] = couleur(face, colonne, y);
                let i = ((y0 + y) * largeur + x0 + x) * 4;
                pixels[i..i + 4].copy_from_slice(&[r, g, b, 255]);
            }
        }
    }
}

/// Skin 32 × 32 de la fusée (tube à rayures en spirale, coiffe rouge, tuyère noircie),
/// couleurs lues dans l'item 16 × 16 `item` (RGBA) ou couleurs de secours.
pub fn peindre_fusee(item: Option<&[u8]>) -> Vec<u8> {
    let lire = |x: usize, y: usize, secours: Couleur| -> Couleur {
        match item {
            Some(px) if px.len() >= 16 * 16 * 4 && px[(y * 16 + x) * 4 + 3] > 0 => {
                let i = (y * 16 + x) * 4;
                [px[i], px[i + 1], px[i + 2]]
            }
            _ => secours,
        }
    };
    let rouge_clair = lire(8, 3, [0xd6, 0x2a, 0x2a]);
    let rouge = lire(8, 2, [0xb9, 0x29, 0x29]);
    let rouge_fonce = lire(7, 3, [0x99, 0x29, 0x29]);
    let rouge_noir = lire(9, 3, [0x6d, 0x17, 0x17]);
    let blanc = lire(7, 7, [0xf4, 0xf4, 0xf4]);
    let blanc_ombre = lire(7, 6, [0xb2, 0xcc, 0xd1]);
    let meche = lire(8, 13, [0x2d, 0x14, 0x09]);
    let mut px = vec![0u8; 32 * 32 * 4];
    peindre_cube(&mut px, 32, [0, 0], [4, 9, 4], |f, c, l| match f {
        Face::Dessus => rouge_fonce,
        Face::Dessous => rouge_noir,
        Face::Cote => match (c + l) % 6 {
            0 => blanc,
            1 => blanc_ombre,
            5 => rouge_fonce,
            _ => rouge,
        },
    });
    peindre_cube(&mut px, 32, [0, 13], [3, 2, 3], |f, _, l| match f {
        Face::Dessus => rouge,
        Face::Dessous => rouge_fonce,
        Face::Cote if l == 0 => rouge_clair,
        Face::Cote => rouge,
    });
    peindre_cube(&mut px, 32, [16, 0], [2, 2, 2], |f, c, _| {
        if f == Face::Dessous {
            rouge_fonce
        } else if c % 2 == 0 {
            rouge_clair
        } else {
            rouge
        }
    });
    peindre_cube(&mut px, 32, [16, 4], [1, 1, 1], |_, _, _| rouge_clair);
    peindre_cube(&mut px, 32, [16, 6], [2, 1, 2], |_, _, _| meche);
    px
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangle_uv_en_fractions() {
        assert_eq!(
            rectangle_uv([16.0, 0.0, 16.0, 32.0], [64.0, 32.0]),
            [0.25, 0.0, 0.5, 1.0]
        );
    }

    #[test]
    fn disque_plein_au_centre_vide_aux_coins() {
        let px = disque(16);
        assert_eq!(px.len(), 16 * 16 * 4);
        assert_eq!(px[(8 * 16 + 8) * 4 + 3], 255);
        assert_eq!(px[3], 0);
    }

    #[test]
    fn fusee_peinte_sans_item() {
        let px = peindre_fusee(None);
        assert_eq!(px.len(), 32 * 32 * 4);
        // Coin du patron du tube (face avant) : opaque ; hors patron : transparent
        assert_eq!(px[(4 * 32 + 4) * 4 + 3], 255);
        assert_eq!(px[(31 * 32 + 31) * 4 + 3], 0);
    }

    #[test]
    fn croisee_deux_plans() {
        let m = croisee(1.0, 0.5, [0.0, 0.0, 1.0, 1.0], [1.0; 4]);
        assert_eq!(m.positions.len(), 8);
        assert_eq!(m.indices.len(), 12);
    }
}
