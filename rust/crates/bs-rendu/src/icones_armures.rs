//! Icônes des armures de la Collection (src/ui/iconesObjets.ts) : bouclier, cape et
//! talisman dessinés aux couleurs de leur monde (16 × 16, contour sombre) ; casque,
//! plastron, jambières et bottes sont les icônes du jeu (`item/<matériau>_<pièce>`), le
//! cuir multiplié par la teinte du monde sous son calque non teint.

use bevy::prelude::*;
use bs_donnees::Donnees;
use bs_donnees::arenes::Monde;
use bs_donnees::equipement::{CouleursArmure, FormeIcone, IconeObjet, Metal, Piece};

use crate::dessins::{CONTOUR, Dessins, TAILLE, icone_objet};

/// Icône d'une pièce : image, calque posé dessus, teinte de l'image.
pub struct IconePiece {
    pub image: Handle<Image>,
    pub calque: Option<Handle<Image>>,
    pub teinte: Color,
}

/// Icône de `piece` de l'armure de `monde` ; `charger` donne la poignée d'une icône du
/// jeu.
pub fn icone(
    d: &Donnees,
    dessins: &Dessins,
    monde: Monde,
    piece: Piece,
    mut charger: impl FnMut(&str) -> Handle<Image>,
) -> IconePiece {
    let k = d.armures.armures.get_index_of(&monde).unwrap_or(0);
    let a = &d.armures.armures[k];
    let dessinee = |i: usize| IconePiece {
        image: dessins.armures[k][i].clone(),
        calque: None,
        teinte: Color::WHITE,
    };
    let nom = match piece {
        Piece::Casque => "helmet",
        Piece::Plastron => "chestplate",
        Piece::Jambieres => "leggings",
        Piece::Bottes => "boots",
        Piece::Bouclier => return dessinee(0),
        Piece::Cape => return dessinee(1),
        Piece::Talisman => return dessinee(2),
    };
    let chemin = format!(
        "item/{}_{nom}",
        crate::modeles::armure::materiau(a.materiau)
    );
    match a.teinte {
        Some(t) => IconePiece {
            image: charger(&chemin),
            calque: Some(charger(&format!("{chemin}_overlay"))),
            teinte: Color::srgb_u8((t >> 16) as u8, (t >> 8) as u8, t as u8),
        },
        None => IconePiece {
            image: charger(&chemin),
            calque: None,
            teinte: Color::WHITE,
        },
    }
}

/// Bouclier, cape et talisman (pixels 0xRRGGBB, u32::MAX : transparent).
pub fn dessiner(c: &CouleursArmure) -> [Vec<u32>; 3] {
    let talisman = icone_objet(&IconeObjet {
        forme: FormeIcone::Pendentif,
        metal: Metal::Or,
        gemme: c.embleme,
    });
    [bouclier(c), cape(c), talisman]
}

struct Toile(Vec<u32>);

impl Toile {
    fn nouvelle() -> Self {
        Self(vec![u32::MAX; TAILLE * TAILLE])
    }

    fn rect(&mut self, x: i32, y: i32, l: i32, couleur: u32) {
        for px in x.max(0)..(x + l).min(TAILLE as i32) {
            if (0..TAILLE as i32).contains(&y) {
                self.0[y as usize * TAILLE + px as usize] = couleur;
            }
        }
    }

    /// Losange de l'emblème centré en (8, cy).
    fn embleme(&mut self, cy: i32, couleur: u32) {
        for dy in -2..=2i32 {
            let demi = 2 - dy.abs();
            self.rect(8 - demi, cy + dy, demi * 2, couleur);
        }
    }
}

/// Écu : rectangle à pointe arrondie, contour sombre.
fn bouclier(c: &CouleursArmure) -> Vec<u32> {
    let mut t = Toile::nouvelle();
    for y in 1..15 {
        let retrait = if y > 10 { (y - 10) as f32 * 1.2 } else { 0.0 };
        let x0 = (2.0 + retrait).round() as i32;
        let x1 = (14.0 - retrait).round() as i32;
        t.rect(x0 - 1, y, x1 - x0 + 2, CONTOUR);
        t.rect(x0, y, x1 - x0, c.bordure);
        if y > 1 && y < 13 {
            t.rect(x0 + 1, y, (x1 - x0 - 2).max(0), c.champ);
        }
    }
    t.rect(2, 0, 12, CONTOUR);
    t.embleme(6, c.embleme);
    t.0
}

/// Cape qui s'évase, col en haut.
fn cape(c: &CouleursArmure) -> Vec<u32> {
    let mut t = Toile::nouvelle();
    for y in 2..15 {
        let demi = 3 + ((y - 2) as f32 * 0.35).round() as i32;
        t.rect(8 - demi - 1, y, demi * 2 + 2, CONTOUR);
        let couleur = if !(4..=12).contains(&y) {
            c.bordure
        } else {
            c.champ
        };
        t.rect(8 - demi, y, demi * 2, couleur);
    }
    t.rect(4, 1, 8, CONTOUR);
    t.embleme(8, c.embleme);
    t.0
}
