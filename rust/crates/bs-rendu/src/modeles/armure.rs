//! Armure de la Collection portée par le héros (src/modeles/armure.ts et la partie armures
//! de src/ui/iconesObjets.ts) : un modèle par pièce visible (`armure_*` dans
//! donnees/modeles.ron), posé exactement comme le joueur à chaque image — ses parties
//! reprennent les rotations des parties du joueur du même nom (table calculée une fois).
//!
//! Textures : casque, plastron, bottes sur la planche du jeu `entity/equipment/humanoid/
//! <matériau>`, jambières sur `humanoid_leggings` ; le cuir teint est multiplié par la
//! teinte du monde puis recouvert de son calque `_overlay`. Bouclier et cape sont dessinés
//! aux couleurs du monde (bordure, champ, emblème en losange). Le talisman ne se voit pas.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bs_donnees::Donnees;
use bs_donnees::arenes::Monde;
use bs_donnees::equipement::{Armure, CouleursArmure, Materiau, Piece};

use super::compilation::ModeleCompile;
use crate::textures::Textures;

/// Ordre des emplacements (comme la Collection).
pub const PIECES: [Piece; 7] = [
    Piece::Casque,
    Piece::Plastron,
    Piece::Jambieres,
    Piece::Bottes,
    Piece::Bouclier,
    Piece::Cape,
    Piece::Talisman,
];

/// Couleur des parties d'une pièce sans texture.
pub const SECOURS_ARMURE: u32 = 0x9a9aa0;

/// Modèle de chaque pièce visible.
pub fn modele(piece: Piece) -> Option<&'static str> {
    match piece {
        Piece::Casque => Some("armure_casque"),
        Piece::Plastron => Some("armure_plastron"),
        Piece::Jambieres => Some("armure_jambieres"),
        Piece::Bottes => Some("armure_bottes"),
        Piece::Bouclier => Some("armure_bouclier"),
        Piece::Cape => Some("armure_cape"),
        Piece::Talisman => None,
    }
}

fn materiau(m: Materiau) -> &'static str {
    match m {
        Materiau::Leather => "leather",
        Materiau::Chainmail => "chainmail",
        Materiau::Iron => "iron",
        Materiau::Gold => "gold",
        Materiau::Diamond => "diamond",
        Materiau::Netherite => "netherite",
    }
}

/// Planche du jeu de la pièce (jambières sur la seconde).
fn texture_jeu(a: &Armure, piece: Piece) -> String {
    let planche = if piece == Piece::Jambieres {
        "humanoid_leggings"
    } else {
        "humanoid"
    };
    format!("entity/equipment/{planche}/{}", materiau(a.materiau))
}

/// Pièces visibles portées : (emplacement, monde).
pub fn visibles(portees: &[Option<Monde>; 7]) -> impl Iterator<Item = (Piece, Monde)> + '_ {
    PIECES
        .iter()
        .zip(portees)
        .filter_map(|(p, m)| Some((*p, (*m)?)))
        .filter(|(p, _)| modele(*p).is_some())
}

/// Textures du jeu à charger pour l'armure portée.
pub fn demander_textures(
    d: &Donnees,
    portees: &[Option<Monde>; 7],
    serveur: &AssetServer,
    t: &mut Textures,
) {
    for (piece, monde) in visibles(portees) {
        let Some(a) = d.armures.armures.get(&monde) else {
            continue;
        };
        if matches!(piece, Piece::Bouclier | Piece::Cape) {
            continue;
        }
        let base = texture_jeu(a, piece);
        if a.teinte.is_some() {
            t.demander(serveur, &format!("{base}_overlay"));
        }
        t.demander(serveur, &base);
    }
}

/// Texture posée sur la pièce (`None` : couleur de secours).
pub fn skin(
    d: &Donnees,
    piece: Piece,
    monde: Monde,
    serveur: &AssetServer,
    textures: &Textures,
    images: &mut Assets<Image>,
) -> Option<Handle<Image>> {
    let a = d.armures.armures.get(&monde)?;
    let dessin = match piece {
        Piece::Bouclier => bouclier(&a.couleurs),
        Piece::Cape => cape(&a.couleurs),
        _ => {
            let base = texture_jeu(a, piece);
            let poignee = textures.obtenir(serveur, &base)?;
            let Some(teinte) = a.teinte else {
                return Some(poignee);
            };
            let calque = textures
                .obtenir(serveur, &format!("{base}_overlay"))
                .and_then(|p| images.get(&p).cloned());
            let image = images.get(&poignee)?;
            teinter(image, calque.as_ref(), teinte)?
        }
    };
    Some(images.add(dessin))
}

/// Index, dans le modèle du joueur, de chaque partie de la pièce (absente : `None`).
pub fn correspondance(joueur: &ModeleCompile, piece: &ModeleCompile) -> Vec<Option<usize>> {
    piece
        .def
        .parties
        .keys()
        .map(|nom| joueur.def.parties.get_index_of(nom))
        .collect()
}

/// Planche RGBA dessinée à la main.
struct Planche {
    largeur: usize,
    pixels: Vec<u8>,
}

impl Planche {
    fn nouvelle(largeur: usize, hauteur: usize) -> Self {
        Self {
            largeur,
            pixels: vec![0; largeur * hauteur * 4],
        }
    }

    fn rect(&mut self, x: i32, y: i32, l: i32, h: i32, c: u32) {
        let hauteur = (self.pixels.len() / 4 / self.largeur) as i32;
        for py in y.max(0)..(y + h).min(hauteur) {
            for px in x.max(0)..(x + l).min(self.largeur as i32) {
                let i = (py as usize * self.largeur + px as usize) * 4;
                self.pixels[i..i + 4].copy_from_slice(&[
                    (c >> 16) as u8,
                    (c >> 8) as u8,
                    c as u8,
                    255,
                ]);
            }
        }
    }

    /// Face de blason : bordure, champ, emblème en losange dans le rectangle donné.
    fn blason(&mut self, x: i32, y: i32, l: i32, h: i32, c: &CouleursArmure) {
        self.rect(x, y, l, h, c.bordure);
        self.rect(x + 1, y + 1, l - 2, h - 2, c.champ);
        let (cx, cy) = (x as f32 + l as f32 / 2.0, y as f32 + h as f32 / 2.0);
        let r = (l.min(h) / 4).max(1);
        for dy in -r..=r {
            let demi = r - dy.abs();
            let gx = (cx - demi as f32 - 0.5).floor() as i32;
            let gy = (cy + dy as f32 - 0.5).floor() as i32;
            self.rect(gx, gy, demi * 2 + 1, 1, c.embleme);
        }
    }

    fn image(self) -> Image {
        let hauteur = self.pixels.len() / 4 / self.largeur;
        image(self.largeur as u32, hauteur as u32, self.pixels)
    }
}

fn image(largeur: u32, hauteur: u32, pixels: Vec<u8>) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: largeur,
            height: hauteur,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    image
}

/// Bouclier : boîte de 1 × 12 × 10 px sur le bras gauche, planche 64 × 32 (faces de
/// 10 × 12 en (0, 10) et (11, 10)).
fn bouclier(c: &CouleursArmure) -> Image {
    let mut p = Planche::nouvelle(64, 32);
    p.rect(0, 0, 32, 32, c.bordure);
    p.blason(0, 10, 10, 12, c);
    p.blason(11, 10, 10, 12, c);
    p.image()
}

/// Cape : boîte de 10 × 16 × 1 px dans le dos, faces en (1, 1) et (12, 1).
fn cape(c: &CouleursArmure) -> Image {
    let mut p = Planche::nouvelle(64, 32);
    p.rect(0, 0, 32, 32, c.champ);
    for x in [1, 12] {
        p.blason(x, 1, 10, 16, c);
        // Col et ourlet de la couleur de bordure
        p.rect(x, 1, 10, 2, c.bordure);
        p.rect(x, 15, 10, 2, c.bordure);
    }
    p.image()
}

/// Cuir teint : texture × teinte (transparence gardée), puis calque non teint par-dessus.
fn teinter(base: &Image, calque: Option<&Image>, teinte: u32) -> Option<Image> {
    let (l, h) = (base.width(), base.height());
    let source = base.data.as_ref()?;
    if source.len() != (l * h * 4) as usize {
        return None;
    }
    let t = [(teinte >> 16) as u8, (teinte >> 8) as u8, teinte as u8];
    let mut pixels = source.clone();
    for px in pixels.chunks_exact_mut(4) {
        for k in 0..3 {
            px[k] = ((u16::from(px[k]) * u16::from(t[k]) + 127) / 255) as u8;
        }
    }
    let calque = calque
        .filter(|c| c.width() == l && c.height() == h)
        .and_then(|c| c.data.as_ref());
    if let Some(calque) = calque.filter(|c| c.len() == pixels.len()) {
        for (px, c) in pixels.chunks_exact_mut(4).zip(calque.chunks_exact(4)) {
            // Composition « par-dessus » (alpha du calque)
            let a = u16::from(c[3]);
            for k in 0..3 {
                px[k] = ((u16::from(c[k]) * a + u16::from(px[k]) * (255 - a) + 127) / 255) as u8;
            }
            px[3] = (a + u16::from(px[3]) * (255 - a) / 255) as u8;
        }
    }
    Some(image(l, h, pixels))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bouclier_et_cape_aux_couleurs_du_monde() {
        let c = CouleursArmure {
            champ: 0x112233,
            bordure: 0x445566,
            embleme: 0x778899,
        };
        let b = Planche::nouvelle(64, 32);
        assert_eq!(b.pixels.len(), 64 * 32 * 4);
        let i = bouclier(&c);
        let px = |im: &Image, x: usize, y: usize| {
            let d = im.data.as_ref().unwrap();
            let k = (y * 64 + x) * 4;
            [d[k], d[k + 1], d[k + 2], d[k + 3]]
        };
        // Bordure du blason, champ, emblème au centre
        assert_eq!(px(&i, 0, 10), [0x44, 0x55, 0x66, 255]);
        assert_eq!(px(&i, 1, 11), [0x11, 0x22, 0x33, 255]);
        assert_eq!(px(&i, 4, 15), [0x77, 0x88, 0x99, 255]);
        // Hors de la moitié gauche : transparent
        assert_eq!(px(&i, 40, 5), [0; 4]);
        let k = cape(&c);
        assert_eq!(px(&k, 5, 2), [0x44, 0x55, 0x66, 255]);
        assert_eq!(px(&k, 0, 0), [0x11, 0x22, 0x33, 255]);
    }

    #[test]
    fn cuir_teint_garde_la_transparence() {
        let base = image(1, 2, vec![255, 255, 255, 255, 10, 20, 30, 0]);
        let t = teinter(&base, None, 0x804020).unwrap();
        let d = t.data.unwrap();
        assert_eq!(&d[0..4], &[0x80, 0x40, 0x20, 255]);
        assert_eq!(d[7], 0);
    }
}
