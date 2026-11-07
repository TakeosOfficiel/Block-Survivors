//! Titres hors du HUD : chaque `TextePixel` qui n'est pas sous la racine du HUD perd
//! son image de police pixel et devient un texte Bevy, de même couleur, taille
//! proportionnelle, ombre ou contour rendus par une ombre portée, coupure et alignement
//! repris : en Luckiest Guy pour les titres, en Inter demi-gras pour les petits textes
//! et les paragraphes (plus lisibles). `suivre` recopie ensuite ses changements (`ecrire`, `colorer`).

use bevy::prelude::*;
use bevy::ui::ContentSize;

use super::PolicesMenu;
use crate::hud::Hud;
use crate::hud::police::{Bord, ImageTexte, TextePixel};

/// Pixels de police par pixel de police pixel, titres (hauteur des capitales comparable).
const ECHELLE: f32 = 9.5;
/// De même pour les petits textes et paragraphes, avec leur taille minimale.
const ECHELLE_TEXTE: f32 = 8.0;
const MIN_TEXTE: f32 = 13.0;
/// En dessous (pixels d'écran par pixel de police), c'est un petit texte.
const TAILLE_TITRE: f32 = 2.0;

/// Titre redessiné en Luckiest Guy.
#[derive(Component)]
pub struct TitreLisse;

fn ombre(t: &TextePixel) -> Option<TextShadow> {
    let (alpha, decalage) = match t.bord {
        Bord::Ombre => (0.55, 0.8),
        Bord::Contour => (0.85, 1.0),
        Bord::Aucun => return None,
    };
    Some(TextShadow {
        offset: Vec2::new(0.0, (t.taille * decalage).max(1.0)),
        color: Color::srgba(0.0, 0.0, 0.0, alpha),
    })
}

fn police(t: &TextePixel, polices: &PolicesMenu) -> TextFont {
    let titre = t.largeur_max.is_none() && t.taille >= TAILLE_TITRE;
    let (font, font_size) = if titre {
        (&polices.logo, t.taille * ECHELLE)
    } else {
        (&polices.grasse, (t.taille * ECHELLE_TEXTE).max(MIN_TEXTE))
    };
    TextFont {
        font: font.clone(),
        font_size,
        ..default()
    }
}

/// Vrai si l'entité est sous la racine du HUD.
fn dans_hud(e: Entity, hud: Option<&Hud>, parents: &Query<&ChildOf>) -> bool {
    let Some(hud) = hud else {
        return false;
    };
    let mut e = e;
    loop {
        if e == hud.racine {
            return true;
        }
        match parents.get(e) {
            Ok(p) => e = p.parent(),
            Err(_) => return false,
        }
    }
}

pub fn convertir(
    mut commandes: Commands,
    hud: Option<Res<Hud>>,
    polices: Res<PolicesMenu>,
    parents: Query<&ChildOf>,
    mut nouveaux: Query<(Entity, &TextePixel, &mut Node), Added<TextePixel>>,
) {
    for (e, t, mut node) in &mut nouveaux {
        if dans_hud(e, hud.as_deref(), &parents) {
            continue;
        }
        node.width = Val::Auto;
        node.height = Val::Auto;
        if let Some(m) = t.largeur_max {
            node.max_width = Val::Px(m);
        }
        let alignement = if t.a_gauche {
            Justify::Left
        } else {
            Justify::Center
        };
        let mut c = commandes.entity(e);
        c.remove::<(ImageNode, ImageTexte, ContentSize)>().insert((
            TitreLisse,
            Text::new(t.texte.clone()),
            police(t, &polices),
            TextColor(t.couleur),
            TextLayout::new_with_justify(alignement),
        ));
        if let Some(o) = ombre(t) {
            c.insert(o);
        }
    }
}

pub fn suivre(
    polices: Res<PolicesMenu>,
    mut titres: Query<
        (&TextePixel, &mut Text, &mut TextColor, &mut TextFont),
        (Changed<TextePixel>, With<TitreLisse>),
    >,
) {
    for (t, mut texte, mut couleur, mut font) in &mut titres {
        if texte.0 != t.texte {
            texte.0.clone_from(&t.texte);
        }
        if couleur.0 != t.couleur {
            couleur.0 = t.couleur;
        }
        let voulue = police(t, &polices);
        if font.font_size != voulue.font_size {
            *font = voulue;
        }
    }
}
