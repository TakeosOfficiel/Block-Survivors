//! Mise en page des écrans en jeu (docs/CHARTE-UI.md) : bandeau de pause resserré au
//! centre (« Reprendre » vert en premier, avec le focus), confirmation avant de
//! recommencer ou d'abandonner, statistiques dans un panneau à gouttière. Le bilan défile
//! à la molette (`Defilant`) ; un indicateur dit qu'il reste de quoi lire dessous.
//! L'écran de fin est dans `fin`.

use bevy::prelude::*;
use bs_regles::partie::Partie;

use super::Action;
use super::bilan::{self, Contexte};
use super::charte::{self, AIDE, IVOIRE, OR_SOURD, OR_VIF, Relief};
use super::style::{self, plein};
use crate::hud::police::TextePixel;

/// Au-dessus du HUD et des cartes.
pub(super) const PLAN: i32 = 20;
const NOIR: (u8, u8, u8) = (6, 8, 14);
/// Largeur du bandeau de pause et des panneaux.
const LARGEUR: f32 = 760.0;

/// Zone qui défile à la molette.
#[derive(Component)]
pub struct Defilant;

/// Indicateur « encore du contenu dessous » d'une zone défilante.
#[derive(Component)]
pub struct Indicateur(Entity);

pub(super) fn noir(alpha: f32) -> Color {
    Color::srgba_u8(NOIR.0, NOIR.1, NOIR.2, (alpha * 255.0) as u8)
}

/// Racine plein écran, contenu centré en colonne.
pub(super) fn racine(commandes: &mut Commands, fond: impl Bundle) -> Entity {
    commandes
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                padding: UiRect::all(Val::Px(24.0)),
                ..plein()
            },
            GlobalZIndex(PLAN),
            fond,
        ))
        .id()
}

pub(super) fn rangee(commandes: &mut Commands, parent: Entity) -> Entity {
    let n = Node {
        align_items: AlignItems::Center,
        column_gap: Val::Px(12.0),
        ..default()
    };
    commandes.spawn((n, ChildOf(parent))).id()
}

/// Bouton en relief de la charte, libellé en police du jeu ; `focus` : liseré d'or vif
/// (l'action d'Entrée).
pub(super) fn bouton(
    commandes: &mut Commands,
    parent: Entity,
    texte: &str,
    largeur: f32,
    (relief, focus): (Relief, bool),
    marque: impl Bundle,
) -> Entity {
    let hauteur = if focus {
        charte::H_PRINCIPAL
    } else {
        charte::H_SECONDAIRE
    };
    let b = charte::bouton_relief(commandes, parent, relief, hauteur, marque);
    commandes
        .entity(b)
        .entry::<Node>()
        .and_modify(move |mut n| {
            n.width = Val::Px(largeur);
        });
    if focus {
        commandes
            .entity(b)
            .insert(Outline::new(Val::Px(2.0), Val::Px(2.0), OR_VIF));
    }
    let taille = if focus { 2.5 } else { 2.0 };
    commandes.spawn((TextePixel::nouveau(texte, IVOIRE, taille), ChildOf(b)));
    b
}

/// Panneau charbon du bilan : zone défilante avec gouttière à droite, indicateur dessous.
pub(super) fn panneau(commandes: &mut Commands, c: &Contexte, parent: Entity, p: &Partie) {
    let cadre = charte::panneau(
        commandes,
        parent,
        Node {
            width: Val::Px(LARGEUR + 40.0),
            max_width: Val::Percent(96.0),
            min_height: Val::ZERO,
            flex_shrink: 1.0,
            padding: UiRect::new(Val::Px(20.0), Val::Px(8.0), Val::Px(16.0), Val::Px(8.0)),
            ..default()
        },
    );
    let zone = commandes
        .spawn((
            Defilant,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                min_height: Val::ZERO,
                // Gouttière : le texte ne touche pas le bord droit
                padding: UiRect::new(Val::ZERO, Val::Px(16.0), Val::ZERO, Val::Px(8.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition::default(),
            ChildOf(cadre),
        ))
        .id();
    bilan::bilan(commandes, c, zone, p);
    let indicateur = commandes
        .spawn((
            Indicateur(zone),
            Node {
                align_self: AlignSelf::Stretch,
                justify_content: JustifyContent::Center,
                padding: UiRect::top(Val::Px(6.0)),
                border: UiRect::top(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(charte::BORD),
            Visibility::Hidden,
            ChildOf(cadre),
        ))
        .id();
    let t = TextePixel::nouveau("⌄ Molette : la suite plus bas ⌄", OR_VIF, 1.5);
    commandes.spawn((t, ChildOf(indicateur)));
}

/// Montre l'indicateur tant que sa zone n'est pas défilée jusqu'en bas.
pub fn indiquer(
    mut indicateurs: Query<(&Indicateur, &mut Visibility)>,
    zones: Query<(&ScrollPosition, &ComputedNode), With<Defilant>>,
) {
    for (ind, mut v) in &mut indicateurs {
        let Ok((s, n)) = zones.get(ind.0) else {
            continue;
        };
        let reste = (n.content_size.y - n.size.y) * n.inverse_scale_factor - s.0.y;
        let voulu = if reste > 4.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        v.set_if_neq(voulu);
    }
}

/// Bandeau sombre centré, filets d'or sourd en haut et en bas.
fn bandeau(commandes: &mut Commands, parent: Entity) -> Entity {
    commandes
        .spawn((
            Node {
                width: Val::Px(LARGEUR),
                max_width: Val::Percent(96.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                padding: UiRect::axes(Val::Px(24.0), Val::Px(24.0)),
                border: UiRect::vertical(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(charte::PANNEAU),
            BorderColor::all(OR_SOURD),
            ChildOf(parent),
        ))
        .id()
}

/// Pause : résumé, « Reprendre » (vert, focus), Statistiques, Options, puis Recommencer et
/// Menu principal (confirmés). `coop` : pas de « Recommencer » (on ne recommence pas seul).
pub fn pause(commandes: &mut Commands, c: &Contexte, p: &Partie, coop: bool) -> Entity {
    let r = racine(commandes, BackgroundColor(noir(0.55)));
    let bande = bandeau(commandes, r);
    style::titre(commandes, bande, "Pause", OR_VIF, 5.0);
    bilan::pastilles(commandes, c, bande, p);
    bouton(
        commandes,
        bande,
        "Reprendre",
        300.0,
        (Relief::PRINCIPAL, true),
        Action::Reprendre,
    );
    let ligne = rangee(commandes, bande);
    let secondaire = (Relief::SECONDAIRE, false);
    bouton(
        commandes,
        ligne,
        "Statistiques",
        200.0,
        secondaire,
        Action::Statistiques,
    );
    bouton(
        commandes,
        ligne,
        "Options",
        200.0,
        secondaire,
        Action::Options,
    );
    let ligne = rangee(commandes, bande);
    if !coop {
        bouton(
            commandes,
            ligne,
            "Recommencer",
            200.0,
            secondaire,
            Action::Recommencer,
        );
    }
    bouton(
        commandes,
        ligne,
        "Menu principal",
        200.0,
        secondaire,
        Action::Menu,
    );
    bilan::texte(commandes, bande, "Échap ou Entrée : reprendre", AIDE, 1.5);
    r
}

/// Confirmation de `action` (Recommencer ou Menu) : la partie en cours sera perdue.
pub fn confirmer(commandes: &mut Commands, action: Action) -> Entity {
    let r = racine(commandes, BackgroundColor(noir(0.7)));
    let bande = bandeau(commandes, r);
    let (titre, detail) = if action == Action::Menu {
        (
            "Abandonner la partie ?",
            "Retour au menu principal : la partie en cours est perdue.",
        )
    } else {
        (
            "Recommencer la partie ?",
            "Une nouvelle partie démarre : celle-ci est perdue.",
        )
    };
    style::titre(commandes, bande, titre, IVOIRE, 3.0);
    bilan::texte(commandes, bande, detail, AIDE, 2.0);
    let ligne = rangee(commandes, bande);
    bouton(
        commandes,
        ligne,
        "Annuler",
        220.0,
        (Relief::SECONDAIRE, true),
        Action::Retour,
    );
    let rouge = Relief {
        fond: Color::srgb_u8(0x6e, 0x2a, 0x2a),
        bord: charte::ROUGE,
        fond_survol: Color::srgb_u8(0x86, 0x34, 0x34),
        bord_survol: Color::srgb_u8(0xff, 0x9a, 0x9a),
        tranche: Color::srgb_u8(0x45, 0x18, 0x18),
    };
    bouton(commandes, ligne, "Confirmer", 220.0, (rouge, false), action);
    bilan::texte(commandes, bande, "Échap ou Entrée : annuler", AIDE, 1.5);
    r
}

/// Statistiques de la pause : panneau du bilan, Retour (focus).
pub fn statistiques(commandes: &mut Commands, c: &Contexte, p: &Partie) -> Entity {
    let r = racine(commandes, BackgroundColor(noir(0.8)));
    style::titre(commandes, r, "Statistiques", OR_VIF, 4.0);
    panneau(commandes, c, r, p);
    bouton(
        commandes,
        r,
        "Retour",
        220.0,
        (Relief::SECONDAIRE, true),
        Action::Retour,
    );
    r
}
