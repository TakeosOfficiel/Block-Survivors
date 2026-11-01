//! Petits dessins pixel art faits au lancement (src/ui/iconesObjets.ts, ornements.ts) :
//! icônes 16 × 16 des bagues et amulettes (anneau ou chaîne de métal, gemme sertie,
//! contour sombre) et losange d'or serti d'une émeraude des écrans.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bs_donnees::Donnees;
use bs_donnees::equipement::{FormeIcone, IconeObjet, Metal};

pub(crate) const TAILLE: usize = 16;
pub(crate) const CONTOUR: u32 = 0x1a1206;
/// Côté du losange (impair : un trait de 1 pixel passe par sa rangée du milieu).
pub const COTE_JOYAU: usize = 13;

/// Images dessinées au lancement : icône de chaque objet d'équipement, pièces dessinées
/// des armures, losange.
#[derive(Resource)]
pub struct Dessins {
    pub objets: Vec<Handle<Image>>,
    /// Bouclier, cape et talisman de chaque armure de monde (ordre des données).
    pub armures: Vec<[Handle<Image>; 3]>,
    pub joyau: Handle<Image>,
}

pub fn creer(
    mut commandes: Commands,
    mut images: ResMut<Assets<Image>>,
    dj: Res<crate::lancement::DonneesJeu>,
) {
    let d: &Donnees = dj.d;
    let objets = d
        .equipement
        .objets
        .values()
        .map(|o| image(&mut images, TAILLE, &icone_objet(&o.icone)))
        .collect();
    let armures = d
        .armures
        .armures
        .values()
        .map(|a| {
            crate::icones_armures::dessiner(&a.couleurs).map(|px| image(&mut images, TAILLE, &px))
        })
        .collect();
    let joyau = image(&mut images, COTE_JOYAU, &joyau());
    commandes.insert_resource(Dessins {
        objets,
        armures,
        joyau,
    });
}

/// Image carrée de `cote` pixels 0xRRGGBB (u32::MAX : transparent).
fn image(images: &mut Assets<Image>, cote: usize, px: &[u32]) -> Handle<Image> {
    let rgba = px
        .iter()
        .flat_map(|&c| {
            if c == u32::MAX {
                [0; 4]
            } else {
                [(c >> 16) as u8, (c >> 8) as u8, c as u8, 255]
            }
        })
        .collect();
    let mut image = Image::new(
        Extent3d {
            width: cote as u32,
            height: cote as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    images.add(image)
}

/// Mélange deux couleurs 0xRRGGBB.
fn melanger(couleur: u32, vers: u32, part: f32) -> u32 {
    let canal = |decalage: u32| {
        let a = ((couleur >> decalage) & 0xff) as f32;
        let b = ((vers >> decalage) & 0xff) as f32;
        ((a + (b - a) * part).round() as u32) << decalage
    };
    canal(16) | canal(8) | canal(0)
}

/// Grille 16 × 16 (u32::MAX : transparent), puis contour sombre autour du dessin.
pub(crate) fn icone_objet(icone: &IconeObjet) -> Vec<u32> {
    const VIDE: u32 = u32::MAX;
    let mut grille = vec![VIDE; TAILLE * TAILLE];
    let mut poser = |x: i32, y: i32, c: u32| {
        if (0..TAILLE as i32).contains(&x) && (0..TAILLE as i32).contains(&y) {
            grille[y as usize * TAILLE + x as usize] = c;
        }
    };
    let [clair, moyen, sombre] = match icone.metal {
        Metal::Fer => [0xf0f0f0, 0xb8b8b8, 0x6a6a6a],
        Metal::Or => [0xfff2a0, 0xe8b830, 0x946a10],
        Metal::Cuivre => [0xffc08a, 0xd0784a, 0x7a3a20],
        Metal::Netherite => [0x8a7a7a, 0x4e4444, 0x262020],
    };
    let gemme = [
        melanger(icone.gemme, 0xffffff, 0.45),
        icone.gemme,
        melanger(icone.gemme, 0, 0.45),
    ];
    match icone.forme {
        FormeIcone::Bague => {
            // Anneau vu de trois quarts
            for y in 0..TAILLE as i32 {
                for x in 0..TAILLE as i32 {
                    let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
                    let d = ((fx - 8.0) / 5.5).powi(2) + ((fy - 10.5) / 4.5).powi(2);
                    if d > 1.0 || d <= 0.5 {
                        continue;
                    }
                    let c = if y < 9 {
                        clair
                    } else if x > 8 && y > 11 {
                        sombre
                    } else {
                        moyen
                    };
                    poser(x, y, c);
                }
            }
            // Chaton et gemme en losange
            for x in 6..=9 {
                poser(x, 7, moyen);
            }
            losange(&mut poser, 7.5, 4.0, 3.0, gemme);
        }
        FormeIcone::Pendentif => {
            // Chaîne en V, maillons alternés
            for k in 0..=7 {
                let c = if k % 2 == 1 { sombre } else { clair };
                let dx = (k as f32 * 0.7).round() as i32;
                poser(2 + dx, 1 + k, c);
                poser(13 - dx, 1 + k, c);
            }
            // Monture de métal et pendentif
            for y in 8..TAILLE as i32 {
                for x in 0..TAILLE as i32 {
                    let d =
                        (x as f32 + 0.5 - 8.0).abs() / 3.6 + (y as f32 + 0.5 - 12.0).abs() / 4.0;
                    if d > 1.0 || d <= 0.7 {
                        continue;
                    }
                    let c = if y < 11 {
                        clair
                    } else if y > 13 {
                        sombre
                    } else {
                        moyen
                    };
                    poser(x, y, c);
                }
            }
            losange(&mut poser, 7.5, 11.5, 2.0, gemme);
        }
    }
    // Contour : chaque case vide qui touche un pixel dessiné
    let mut sortie = grille.clone();
    let plein = |x: i32, y: i32| {
        (0..TAILLE as i32).contains(&x)
            && (0..TAILLE as i32).contains(&y)
            && grille[y as usize * TAILLE + x as usize] != VIDE
    };
    for y in 0..TAILLE as i32 {
        for x in 0..TAILLE as i32 {
            let i = y as usize * TAILLE + x as usize;
            if grille[i] == VIDE
                && (plein(x + 1, y) || plein(x - 1, y) || plein(x, y + 1) || plein(x, y - 1))
            {
                sortie[i] = CONTOUR;
            }
        }
    }
    sortie
}

/// Gemme en losange de rayon `r` : reflet en haut à gauche, ombre en bas à droite, éclat.
fn losange(poser: &mut impl FnMut(i32, i32, u32), cx: f32, cy: f32, r: f32, c: [u32; 3]) {
    for y in (cy - r).floor() as i32..=(cy + r).ceil() as i32 {
        for x in (cx - r).floor() as i32..=(cx + r).ceil() as i32 {
            let dx = x as f32 - cx;
            let dy = y as f32 + 0.5 - cy;
            if dx.abs() + dy.abs() > r {
                continue;
            }
            let couleur = if dx + dy < -0.9 {
                c[0]
            } else if dx + dy > 0.9 {
                c[2]
            } else {
                c[1]
            };
            poser(x, y, couleur);
        }
    }
    poser(
        (cx - 1.0).round() as i32,
        (cy - 1.0).round() as i32,
        0xffffff,
    );
}

/// Losange d'or (contour sombre, biseau éclairé en haut à gauche) serti d'une émeraude.
fn joyau() -> Vec<u32> {
    let c = (COTE_JOYAU as i32 - 1) / 2;
    let mut px = Vec::with_capacity(COTE_JOYAU * COTE_JOYAU);
    for y in 0..COTE_JOYAU as i32 {
        for x in 0..COTE_JOYAU as i32 {
            let anneau = c - ((x - c).abs() + (y - c).abs());
            let haut_gauche = x + y < 2 * c;
            px.push(match anneau {
                a if a < 0 => u32::MAX,
                0 => CONTOUR,
                1 if haut_gauche => 0xffe58a,
                1 => 0x8a6418,
                2 => 0xe0b040,
                _ if (x == c - 1 && y == c - 1) || (x == c && y == c - 2) => 0xc8ffd8,
                _ if haut_gauche || (x + y == 2 * c && x < c) => 0x17dd62,
                _ => 0x0a8a3a,
            });
        }
    }
    px
}
