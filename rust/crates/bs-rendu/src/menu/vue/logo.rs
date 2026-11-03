//! Logo de l'accueil, dessiné en police du jeu (aucune image) : des lettres en relief
//! comme des blocs, la face en bandes (claire en haut, plus sombre en bas), une
//! épaisseur de plusieurs pixels de police vers le bas, le tout cerclé de noir.
//! BLOCK en pierre, SURVIVORS comme un bloc d'herbe (dessus vert, terre dessous) ;
//! `BS_LOGO=2` (pierre et or) ou `3` (or et pierre) pour comparer.

use bevy::prelude::*;

use super::{Pinceau, place};
use crate::hud::police::TextePixel;

/// Une matière : bandes de la face (de haut en bas, avec la part de hauteur où chacune
/// s'arrête) et teinte de l'épaisseur.
struct Matiere {
    bandes: &'static [(Color, f32)],
    flanc: Color,
}

const NOIR: Color = Color::srgb_u8(0x10, 0x0c, 0x0a);

const PIERRE: Matiere = Matiere {
    bandes: &[
        (Color::srgb_u8(0xf2, 0xf0, 0xea), 0.3),
        (Color::srgb_u8(0xc4, 0xc2, 0xbb), 0.7),
        (Color::srgb_u8(0x96, 0x94, 0x8e), 1.0),
    ],
    flanc: Color::srgb_u8(0x55, 0x53, 0x50),
};

const HERBE: Matiere = Matiere {
    bandes: &[
        (Color::srgb_u8(0x9c, 0xe0, 0x4a), 0.28),
        (Color::srgb_u8(0x6a, 0xb0, 0x2e), 0.45),
        (Color::srgb_u8(0xb0, 0x7a, 0x48), 0.75),
        (Color::srgb_u8(0x8a, 0x5a, 0x34), 1.0),
    ],
    flanc: Color::srgb_u8(0x4a, 0x2e, 0x1a),
};

const OR: Matiere = Matiere {
    bandes: &[
        (Color::srgb_u8(0xff, 0xec, 0x8a), 0.3),
        (Color::srgb_u8(0xff, 0xc8, 0x3a), 0.65),
        (Color::srgb_u8(0xe0, 0x8e, 0x1a), 1.0),
    ],
    flanc: Color::srgb_u8(0x7a, 0x3e, 0x0c),
};

/// Un mot : ses lettres une à une, écartées d'un pixel de police (le gras les
/// collerait sinon).
fn mot(p: &mut Pinceau, parent: Entity, t: &str, m: &Matiere, taille: f32, profondeur: u32) {
    let r = p.noeud(
        parent,
        Node {
            column_gap: Val::ZERO,
            ..default()
        },
    );
    let mut lettre = [0u8; 4];
    for c in t.chars() {
        ligne(p, r, c.encode_utf8(&mut lettre), m, taille, profondeur);
    }
}

/// Une lettre, en gras (chaque dessin doublé d'un pixel vers la droite, comme le gras
/// du jeu) : ombre portée, silhouette noire, épaisseur (`profondeur` pixels de police
/// vers le bas), puis la face en bandes, sans trait entre elle et son épaisseur.
fn ligne(p: &mut Pinceau, parent: Entity, t: &str, m: &Matiere, taille: f32, profondeur: u32) {
    let px = |k: f32| Some(Val::Px(k * taille));
    let l = p.noeud(
        parent,
        Node {
            margin: UiRect::new(
                Val::ZERO,
                Val::Px(taille),
                Val::ZERO,
                Val::Px(taille * profondeur as f32),
            ),
            ..default()
        },
    );
    // Dessin contouré en (x, y) pixels de police ; sans bord, il perd la marge d'un
    // pixel du contour et se décale d'autant
    let dessin =
        |p: &mut Pinceau, parent: Entity, couleur: Color, contour: bool, (x, y): (f32, f32)| {
            for gras in [0.0, 1.0] {
                let texte = TextePixel::nouveau(t, couleur, taille);
                let (texte, marge) = if contour {
                    (texte.contour(), 0.0)
                } else {
                    (texte.sans_bord(), 1.0)
                };
                let n = Node {
                    flex_shrink: 0.0,
                    ..place(px(x + gras + marge), None, px(y + marge), None)
                };
                p.c.spawn((texte, n, ChildOf(parent)));
            }
        };
    // Place de la ligne dans le flux (invisible)
    p.c.spawn((
        TextePixel::nouveau(t, Color::NONE, taille).contour(),
        Visibility::Hidden,
        ChildOf(l),
    ));
    let prof = profondeur as f32;
    dessin(
        p,
        l,
        Color::srgba(0.0, 0.0, 0.0, 0.45),
        true,
        (1.0, prof + 1.5),
    );
    for k in 0..=profondeur {
        dessin(p, l, NOIR, true, (0.0, k as f32));
    }
    for k in 1..=profondeur {
        dessin(p, l, m.flanc, false, (0.0, k as f32));
    }
    // Face : bandes rognées, de la plus basse (pleine) à la plus haute
    for &(couleur, fin) in m.bandes.iter().rev() {
        let r = p.noeud(
            l,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(fin * 100.0),
                overflow: Overflow::clip(),
                ..place(Some(Val::ZERO), None, Some(Val::ZERO), None)
            },
        );
        dessin(p, r, couleur, false, (0.0, 0.0));
    }
}

/// Logo sur deux lignes de même largeur.
pub(super) fn construire(p: &mut Pinceau, parent: Entity) {
    let l = p.noeud(
        parent,
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: Val::Px(6.0),
            margin: UiRect::bottom(Val::Px(30.0)),
            ..default()
        },
    );
    let variante = std::env::var("BS_LOGO").unwrap_or_default();
    let (haut, bas) = match variante.as_str() {
        "2" => (&PIERRE, &OR),
        "3" => (&OR, &PIERRE),
        _ => (&PIERRE, &HERBE),
    };
    mot(p, l, "BLOCK", haut, 10.0, 2);
    mot(p, l, "SURVIVORS", bas, 5.55, 2);
}
