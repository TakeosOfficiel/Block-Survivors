//! Case d'un talent : bord vert si apprenable ou commencé, or si complet, creux et
//! icône éteinte si fermé ; halo de la case choisie ; le rang en pastille dessous,
//! ★ pour une arme de départ, « zZ » pour un talent mis de côté.

use bevy::prelude::*;
use bs_donnees::progression::Noeud;

use super::{ActionTalent, Contexte, ecrire};
use crate::ecrans::charte::{self, AIDE, BORD, CHOISI, CREUX, OR_VIF, SURFACE, VERT_GAIN};
use crate::hud::police::TextePixel;
use crate::menu::{ActionMenu, Icones};

/// Côté d'une case (px).
pub(super) const CASE: f32 = 54.0;

/// Case survolée : son bord s'éclaire (couleur de repos gardée).
#[derive(Component, Clone, Copy)]
pub struct Survol(Color);

pub fn survoler_cases(
    mut q: Query<(&Interaction, &Survol, &mut BorderColor), Changed<Interaction>>,
) {
    for (i, s, mut bord) in &mut q {
        *bord = BorderColor::all(if *i == Interaction::None {
            s.0
        } else {
            s.0.lighter(0.25)
        });
    }
}

pub(super) fn construire(
    commandes: &mut Commands,
    c: &Contexte,
    ic: &mut Icones,
    zone: Entity,
    (k, n): (usize, &Noeud),
    centre: Vec2,
) {
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let r = prog.rang(p, a, n);
    let disponible = prog.refus_achat(p, a, n).is_none();
    let complet = r >= n.rang_max;
    let ferme = r == 0 && !disponible;
    let inactif = r > 0 && !prog.actif(p, a, n);
    let (bord, fond) = if complet {
        (OR_VIF, CHOISI)
    } else if r > 0 || disponible {
        (VERT_GAIN, SURFACE)
    } else {
        (BORD, CREUX)
    };
    let o = centre - Vec2::splat(CASE / 2.0);
    let b = commandes
        .spawn((
            Button,
            ActionMenu::Talent(ActionTalent::Noeud(a, k)),
            Node {
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                padding: UiRect::all(Val::Px(7.0)),
                position_type: PositionType::Absolute,
                left: Val::Px(o.x),
                top: Val::Px(o.y),
                width: Val::Px(CASE),
                height: Val::Px(CASE),
                ..default()
            },
            BorderColor::all(bord),
            BackgroundColor(fond),
            Survol(bord),
            ChildOf(zone),
        ))
        .id();
    let halo = if k == c.etat.noeud {
        Some((Color::WHITE.with_alpha(0.85), 3.0, 8.0))
    } else if complet {
        Some((OR_VIF.with_alpha(0.4), 0.0, 10.0))
    } else if disponible {
        Some((VERT_GAIN.with_alpha(0.45), 0.0, 10.0))
    } else {
        None
    };
    if let Some((couleur, etendue, flou)) = halo {
        commandes.entity(b).insert(BoxShadow::new(
            couleur,
            Val::ZERO,
            Val::ZERO,
            Val::Px(etendue),
            Val::Px(flou),
        ));
    }
    let teinte = if ferme {
        Color::srgb(0.32, 0.32, 0.32)
    } else if inactif {
        Color::srgb(0.55, 0.55, 0.55)
    } else {
        Color::WHITE
    };
    // Icône ; talent à choix : l'option retenue
    let icone = n
        .options
        .as_ref()
        .map_or(&n.icone, |o| &o[prog.option(p, a, n)].icone);
    commandes.spawn((
        ImageNode::new(ic.icone(icone)).with_color(teinte),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        ChildOf(b),
    ));
    // Rang, centré sous la case
    let couleur = if complet {
        OR_VIF
    } else if r > 0 || disponible {
        VERT_GAIN
    } else {
        AIDE
    };
    let pied = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                right: Val::ZERO,
                bottom: Val::Px(-12.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ChildOf(b),
        ))
        .id();
    let pastille = commandes
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(5.0), Val::ZERO),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(charte::PANNEAU),
            BorderColor::all(BORD),
            ChildOf(pied),
        ))
        .id();
    let t = format!("{r}/{}", n.rang_max);
    ecrire(commandes, ic, pastille, &t, (true, 12.0, couleur));
    if n.gains.arme_depart.is_some() {
        let e = ecrire(commandes, ic, b, "★", (true, 15.0, OR_VIF));
        commandes.entity(e).insert((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(-10.0),
                left: Val::Px(-8.0),
                ..default()
            },
            TextShadow::default(),
        ));
    }
    if inactif {
        commandes.spawn((
            TextePixel::nouveau("zZ", Color::srgb(0.87, 0.87, 0.87), 1.0),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(3.0),
                top: Val::Px(2.0),
                ..default()
            },
            ChildOf(b),
        ));
    }
}
