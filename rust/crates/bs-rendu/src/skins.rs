//! Skins importées par le joueur (port de src/assets/skinsImportees.ts), ramenées au format
//! du jeu avant d'être gardées : patron carré 64×64 ou HD (128, 256, 512, 1024). Une image
//! carrée d'une autre taille est remise à la taille HD la plus proche (pixels nets) ;
//! l'ancien format 2:1 (64×32 et ses HD), sans membres gauches, est converti comme le fait
//! le jeu (membres gauches = droits en miroir).
//!
//! Gardée aussi légère que possible : une skin « HD » qui n'est qu'une skin agrandie
//! (blocs de pixels identiques) est gardée à sa vraie taille, sans perte ; une skin encore
//! trop lourde est réduite de moitié jusqu'à tenir. Gardée en PNG dans une data URL, comme
//! le jeu TS (même sauvegarde).

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::{ImageFormat, RgbaImage};

/// Taille du patron du jeu (les skins HD en sont des multiples).
const PATRON: u32 = 64;
/// Plus grande skin gardée (16 fois le patron).
const TAILLE_MAX: u32 = 1024;
/// Poids maximal de la skin dans la sauvegarde (caractères de la data URL).
const POIDS_MAX: usize = 1_200_000;
const ENTETE: &str = "data:image/png;base64,";

/// Skin gardée : data URL, taille, et `reduite` si des détails ont été perdus pour tenir.
#[derive(Debug, Clone)]
pub struct SkinNormalisee {
    pub adresse: String,
    pub taille: u32,
    pub reduite: bool,
}

/// Ramène l'image (fichier PNG, JPEG…) au patron du jeu, ou dit pourquoi elle est refusée.
pub fn normaliser(octets: &[u8]) -> Result<SkinNormalisee, String> {
    let image = image::load_from_memory(octets)
        .map_err(|_| "Skin refusée : l'image est illisible.".to_owned())?
        .to_rgba8();
    let (l, h) = image.dimensions();
    let carre = l == h;
    let ancien = l == h * 2;
    if (!carre && !ancien) || l < PATRON {
        return Err(format!(
            "Skin refusée : l'image fait {l}×{h}. Il faut une skin du jeu, carrée (64×64, ou \
             HD : 128, 256, 512, 1024…) ou à l'ancien format (64×32 et ses HD)."
        ));
    }
    // Taille HD la plus proche : 64 × une puissance de 2, au plus TAILLE_MAX
    let puissance = (f64::from(l) / f64::from(PATRON)).log2().round().max(0.0) as u32;
    let mut taille = (PATRON << puissance.min(4)).clamp(PATRON, TAILLE_MAX);
    let mut toile = dessiner_patron(&image, taille, ancien);
    // Agrandie sans détail en plus : gardée à sa vraie taille (rendu identique)
    let reelle = taille_reelle(&toile);
    if reelle < taille {
        taille = reelle;
        toile = redimensionner(&toile, taille, taille);
    }
    let mut adresse = data_url(&toile);
    let mut reduite = false;
    while adresse.len() > POIDS_MAX && taille > PATRON {
        reduite = true;
        taille /= 2;
        toile = dessiner_patron(&image, taille, ancien);
        adresse = data_url(&toile);
    }
    if adresse.len() > POIDS_MAX {
        return Err("Skin refusée : l'image est trop lourde pour être gardée.".into());
    }
    Ok(SkinNormalisee {
        adresse,
        taille,
        reduite,
    })
}

/// Image Bevy d'une skin gardée (data URL), ou `None` si elle est illisible.
pub fn image_skin(adresse: &str) -> Option<Image> {
    let (_, donnees) = adresse.split_once(";base64,")?;
    let octets = STANDARD.decode(donnees).ok()?;
    let rgba = image::load_from_memory(&octets).ok()?.to_rgba8();
    let (l, h) = rgba.dimensions();
    let mut image = Image::new(
        Extent3d {
            width: l,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba.into_raw(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    Some(image)
}

fn data_url(toile: &RgbaImage) -> String {
    let mut png = Vec::new();
    toile
        .write_to(&mut std::io::Cursor::new(&mut png), ImageFormat::Png)
        .expect("encodage PNG en mémoire");
    format!("{ENTETE}{}", STANDARD.encode(png))
}

/// Copie de `source` à la taille donnée, pixels nets (plus proche voisin, comme
/// `drawImage` sans lissage).
fn redimensionner(source: &RgbaImage, largeur: u32, hauteur: u32) -> RgbaImage {
    let (sl, sh) = source.dimensions();
    RgbaImage::from_fn(largeur, hauteur, |x, y| {
        let sx = ((u64::from(x) * 2 + 1) * u64::from(sl) / (u64::from(largeur) * 2)) as u32;
        let sy = ((u64::from(y) * 2 + 1) * u64::from(sh) / (u64::from(hauteur) * 2)) as u32;
        *source.get_pixel(sx.min(sl - 1), sy.min(sh - 1))
    })
}

/// Plus petite taille (64 × une puissance de 2) où chaque bloc de pixels est uni.
fn taille_reelle(toile: &RgbaImage) -> u32 {
    let taille = toile.width();
    let mut facteur = taille / PATRON;
    while facteur > 1 {
        if blocs_unis(toile, facteur) {
            return taille / facteur;
        }
        facteur /= 2;
    }
    taille
}

fn blocs_unis(toile: &RgbaImage, facteur: u32) -> bool {
    let taille = toile.width();
    (0..taille).all(|y| {
        (0..taille)
            .all(|x| toile.get_pixel(x, y) == toile.get_pixel(x - x % facteur, y - y % facteur))
    })
}

/// Patron carré de `taille` px ; l'ancien format reçoit ses membres gauches.
fn dessiner_patron(image: &RgbaImage, taille: u32, ancien: bool) -> RgbaImage {
    if !ancien {
        return redimensionner(image, taille, taille);
    }
    // Ancien format : la moitié haute telle quelle, puis jambe et bras gauches
    let haut = redimensionner(image, taille, taille / 2);
    let mut toile = RgbaImage::new(taille, taille);
    image::imageops::replace(&mut toile, &haut, 0, 0);
    let k = taille / PATRON;
    membre_miroir(&mut toile, k, (0, 16), (16, 48)); // jambe droite → jambe gauche
    membre_miroir(&mut toile, k, (40, 16), (32, 48)); // bras droit → bras gauche
    toile
}

/// Membre de 4×12×4 (patron du jeu, en px de 64) recopié en miroir de (u, v) vers
/// (du, dv) : chaque face retournée, faces droite et gauche échangées.
fn membre_miroir(toile: &mut RgbaImage, k: u32, (u, v): (u32, u32), (du, dv): (u32, u32)) {
    // [x source, y, largeur, hauteur, x destination] des 6 faces, en px du patron
    const FACES: [[u32; 5]; 6] = [
        [4, 0, 4, 4, 4],    // dessus
        [8, 0, 4, 4, 8],    // dessous
        [0, 4, 4, 12, 8],   // côté extérieur → l'autre côté
        [4, 4, 4, 12, 4],   // devant
        [8, 4, 4, 12, 0],   // côté intérieur → l'autre côté
        [12, 4, 4, 12, 12], // dos
    ];
    let source = toile.clone();
    for [sx, sy, l, h, dx] in FACES {
        for y in 0..h * k {
            for x in 0..l * k {
                let p = *source.get_pixel((u + sx) * k + x, (v + sy) * k + y);
                toile.put_pixel((du + dx + l) * k - 1 - x, (dv + sy) * k + y, p);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn png(image: &RgbaImage) -> Vec<u8> {
        let mut v = Vec::new();
        image
            .write_to(&mut std::io::Cursor::new(&mut v), ImageFormat::Png)
            .unwrap();
        v
    }

    fn relire(s: &SkinNormalisee) -> RgbaImage {
        let i = image_skin(&s.adresse).unwrap();
        RgbaImage::from_raw(i.width(), i.height(), i.data.unwrap()).unwrap()
    }

    #[test]
    fn formats_refuses() {
        assert!(normaliser(b"pas une image").is_err());
        assert!(normaliser(&png(&RgbaImage::new(64, 48))).is_err());
        assert!(normaliser(&png(&RgbaImage::new(32, 32))).is_err());
    }

    #[test]
    fn hd_agrandie_gardee_a_sa_vraie_taille() {
        // Skin 64×64 bruitée, agrandie ×4 : gardée en 64×64, sans perte
        let petite = RgbaImage::from_fn(64, 64, |x, y| {
            Rgba([(x * 4) as u8, (y * 4) as u8, (x ^ y) as u8, 255])
        });
        let grande = redimensionner(&petite, 256, 256);
        let s = normaliser(&png(&grande)).unwrap();
        assert_eq!(s.taille, 64);
        assert!(!s.reduite);
        assert_eq!(relire(&s), petite);
        // Taille voisine : 200×200 devient 256 (détails réels gardés)
        let bruit = RgbaImage::from_fn(200, 200, |x, y| {
            Rgba([(x * 7 + y) as u8, (y * 3) as u8, 9, 255])
        });
        assert_eq!(normaliser(&png(&bruit)).unwrap().taille, 256);
    }

    #[test]
    fn ancien_format_recoit_ses_membres_gauches() {
        let mut ancien = RgbaImage::new(64, 32);
        // Devant de la jambe droite : colonne gauche rouge, le reste bleu
        for y in 20..32 {
            for x in 4..8 {
                let c = if x == 4 {
                    [255, 0, 0, 255]
                } else {
                    [0, 0, 255, 255]
                };
                ancien.put_pixel(x, y, Rgba(c));
            }
        }
        ancien.put_pixel(0, 0, Rgba([1, 2, 3, 255]));
        let s = normaliser(&png(&ancien)).unwrap();
        let t = relire(&s);
        assert_eq!(t.dimensions(), (64, 64));
        // Devant de la jambe gauche (20, 52) : retourné, le rouge passe à droite
        assert_eq!(t.get_pixel(23, 52), &Rgba([255, 0, 0, 255]));
        assert_eq!(t.get_pixel(20, 52), &Rgba([0, 0, 255, 255]));
        assert_eq!(t.get_pixel(0, 0), &Rgba([1, 2, 3, 255]));
    }
}
