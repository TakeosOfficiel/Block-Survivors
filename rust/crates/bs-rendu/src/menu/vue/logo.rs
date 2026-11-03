//! Logo de l'accueil en Luckiest Guy (`polices`) : deux lignes de même largeur, BLOCK en
//! pierre claire, SURVIVORS en or. Chaque ligne est un empilement de textes : une
//! ombre douce, un contour noir (copies décalées en cercle), l'épaisseur (copies vers
//! le bas, teinte du flanc), puis la face en bandes rognées (claire en haut).

use bevy::prelude::*;

use super::{Pinceau, place};

/// Une matière : bandes de la face (de haut en bas, avec la part de hauteur où chacune
/// s'arrête) et teinte de l'épaisseur.
struct Matiere {
    bandes: &'static [(Color, f32)],
    flanc: Color,
}

const NOIR: Color = Color::srgb_u8(0x12, 0x0d, 0x0a);

const PIERRE: Matiere = Matiere {
    bandes: &[
        (Color::srgb_u8(0xff, 0xfd, 0xf6), 0.42),
        (Color::srgb_u8(0xe2, 0xdf, 0xd6), 0.62),
        (Color::srgb_u8(0xc2, 0xbe, 0xb4), 1.0),
    ],
    flanc: Color::srgb_u8(0x5c, 0x58, 0x52),
};

const OR: Matiere = Matiere {
    bandes: &[
        (Color::srgb_u8(0xff, 0xe7, 0x7a), 0.45),
        (Color::srgb_u8(0xff, 0xc4, 0x2e), 0.65),
        (Color::srgb_u8(0xf0, 0x9a, 0x14), 1.0),
    ],
    flanc: Color::srgb_u8(0x86, 0x44, 0x0c),
};

/// Directions du contour (huit, plus huit à mi-chemin pour l'arrondir).
const CERCLE: usize = 16;

/// Une ligne : `taille` (px) de police, `profondeur` et `bord` (contour) en px.
fn ligne(
    p: &mut Pinceau,
    parent: Entity,
    t: &str,
    m: &Matiere,
    taille: f32,
    (profondeur, bord): (f32, f32),
) {
    let police = p.ic.polices.logo.clone();
    let l = p.noeud(
        parent,
        Node {
            padding: UiRect::all(Val::Px(bord)),
            margin: UiRect::bottom(Val::Px(profondeur - taille * 0.3)),
            ..default()
        },
    );
    let texte = |couleur: Color| {
        (
            Text::new(t),
            TextFont {
                font: police.clone(),
                font_size: taille,
                ..default()
            },
            TextColor(couleur),
            TextLayout::new_with_no_wrap(),
        )
    };
    // Place dans le flux (invisible)
    p.c.spawn((texte(Color::NONE), ChildOf(l)));
    let a = |x: f32, y: f32| place(Some(Val::Px(bord + x)), None, Some(Val::Px(bord + y)), None);
    // Ombre douce sous le tout
    p.c.spawn((
        texte(Color::srgba(0.0, 0.0, 0.0, 0.35)),
        a(bord * 0.6, profondeur + bord * 1.8),
        ChildOf(l),
    ));
    // Contour noir autour de la face, du milieu et du fond de l'épaisseur
    for y in [0.0, profondeur / 2.0, profondeur] {
        for i in 0..CERCLE {
            let (s, c) = (i as f32 * std::f32::consts::TAU / CERCLE as f32).sin_cos();
            p.c.spawn((texte(NOIR), a(c * bord, y + s * bord), ChildOf(l)));
        }
    }
    // Épaisseur, du fond vers la face, un pixel à la fois
    let mut y = profondeur;
    while y > 0.0 {
        p.c.spawn((texte(m.flanc), a(0.0, y), ChildOf(l)));
        y -= 1.0;
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
        p.c.spawn((texte(couleur), a(0.0, 0.0), ChildOf(r)));
    }
}

/// Logo sur deux lignes de même largeur.
pub(super) fn construire(p: &mut Pinceau, parent: Entity) {
    let l = p.noeud(
        parent,
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            margin: UiRect::new(Val::Px(-6.0), Val::ZERO, Val::Px(-14.0), Val::Px(26.0)),
            ..default()
        },
    );
    ligne(p, l, "BLOCK", &PIERRE, 132.0, (9.0, 5.0));
    ligne(p, l, "SURVIVORS", &OR, 74.0, (6.0, 4.0));
}
