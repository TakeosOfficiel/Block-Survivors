//! Éléments des pages « à l'air libre » (Talents, Multijoueur) : aucun panneau ni case
//! carrée. Du texte posé, des liens qui passent à l'or au survol, des champs de saisie
//! en pilule, des pastilles rondes et le grand bouton vert arrondi en police du logo.

use bevy::prelude::*;

use super::vue::Lien;
use super::{ActionMenu, Icones};
use crate::ecrans::charte::{self, IVOIRE, OR_VIF, Relief};

/// Texte du menu (police du logo si gras et grand, Inter sinon).
pub fn ecrire(
    commandes: &mut Commands,
    ic: &Icones,
    parent: Entity,
    t: &str,
    (gras, taille, couleur): (bool, f32, Color),
) -> Entity {
    let t = ic.polices.texte(t, gras, taille, couleur);
    commandes.spawn((t, ChildOf(parent))).id()
}

/// Rangée centrée verticalement, éléments écartés de `ecart`.
pub fn rangee(commandes: &mut Commands, parent: Entity, ecart: f32) -> Entity {
    commandes
        .spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(ecart),
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(parent),
        ))
        .id()
}

/// Colonne, éléments espacés de `ecart`.
pub fn colonne(commandes: &mut Commands, parent: Entity, ecart: f32) -> Entity {
    commandes
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(ecart),
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(parent),
        ))
        .id()
}

/// Lien sans cadre (libellé gras, or au survol) ; sans action, simple texte.
pub fn lien(
    commandes: &mut Commands,
    ic: &Icones,
    parent: Entity,
    t: &str,
    (taille, couleur): (f32, Color),
    action: Option<ActionMenu>,
) -> Entity {
    let b = rangee(commandes, parent, 10.0);
    commandes.entity(b).entry::<Node>().and_modify(|mut n| {
        n.padding = UiRect::vertical(Val::Px(4.0));
    });
    if let Some(a) = action {
        commandes.entity(b).insert((Button, Lien(couleur), a));
    }
    ecrire(commandes, ic, b, t, (true, taille, couleur));
    b
}

/// Champ de saisie en pilule (un clic commence la saisie) ; `actif` : en cours.
pub fn champ(
    commandes: &mut Commands,
    ic: &Icones,
    parent: Entity,
    t: &str,
    actif: bool,
    action: ActionMenu,
) -> Entity {
    let c = commandes
        .spawn((
            Button,
            action,
            Node {
                width: Val::Px(240.0),
                height: Val::Px(42.0),
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(20.0)),
                border: UiRect::all(Val::Px(1.5)),
                border_radius: BorderRadius::MAX,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
            BorderColor::all(if actif {
                OR_VIF
            } else {
                Color::srgba(1.0, 1.0, 1.0, 0.2)
            }),
            ChildOf(parent),
        ))
        .id();
    let couleur = if actif { IVOIRE } else { charte::AIDE };
    ecrire(commandes, ic, c, t, (false, charte::T_CORPS, couleur));
    c
}

/// Pastille ronde pleine (présence, puce).
pub fn point(commandes: &mut Commands, parent: Entity, cote: f32, couleur: Color) -> Entity {
    commandes
        .spawn((
            Node {
                width: Val::Px(cote),
                height: Val::Px(cote),
                border_radius: BorderRadius::MAX,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(couleur),
            ChildOf(parent),
        ))
        .id()
}

/// Grand bouton vert arrondi en police du logo ; éteint (gris, sans action) si `None`.
pub fn bouton_vert(
    commandes: &mut Commands,
    ic: &Icones,
    parent: Entity,
    t: &str,
    action: Option<ActionMenu>,
) -> Entity {
    let h = charte::H_PRINCIPAL + 8.0;
    let b = charte::bouton_relief(commandes, parent, Relief::PRINCIPAL, h, ());
    commandes.entity(b).entry::<Node>().and_modify(|mut n| {
        n.border_radius = BorderRadius::MAX;
        n.padding = UiRect::horizontal(Val::Px(36.0));
        n.flex_shrink = 0.0;
    });
    let couleur = match action {
        Some(a) => {
            commandes.entity(b).insert(a);
            IVOIRE
        }
        None => {
            commandes.entity(b).remove::<(Button, Relief)>().insert((
                BackgroundColor(charte::CREUX),
                BorderColor::all(charte::BORD),
            ));
            charte::AIDE
        }
    };
    commandes.spawn((
        Text::new(t),
        TextFont {
            font: ic.polices.logo.clone(),
            font_size: 30.0,
            ..default()
        },
        TextColor(couleur),
        TextShadow {
            offset: Vec2::new(0.0, 3.0),
            color: Color::srgba(0.0, 0.0, 0.0, 0.6),
        },
        ChildOf(b),
    ));
    b
}
