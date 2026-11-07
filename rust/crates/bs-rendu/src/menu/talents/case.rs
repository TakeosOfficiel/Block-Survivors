//! Pastille ronde d'un talent : anneau vert si apprenable ou commencé, or si complet,
//! à la couleur de sa branche (pâle) et icône éteinte si fermé ; la choisie est plus
//! grande, avec un halo blanc. Le rang écrit dessous, ★ pour une arme de départ,
//! « zZ » pour un talent mis de côté.

use bevy::prelude::*;
use bs_donnees::progression::Noeud;

use super::{ActionTalent, Contexte, ecrire};
use crate::ecrans::charte::{AIDE, OR_VIF, VERT_GAIN};
use crate::hud::police::TextePixel;
use crate::menu::{ActionMenu, Icones};

/// Diamètre d'une pastille, et de la pastille choisie (px).
pub(super) const CASE: f32 = 48.0;
const CASE_CHOISIE: f32 = 62.0;
/// Fond d'une pastille : nuit, or sombre si complète.
const NUIT: Color = Color::srgb(0.07, 0.10, 0.14);
const NUIT_OR: Color = Color::srgb(0.20, 0.16, 0.07);

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
    (centre, branche): (Vec2, Color),
) {
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let r = prog.rang(p, a, n);
    let disponible = prog.refus_achat(p, a, n).is_none();
    let complet = r >= n.rang_max;
    let ferme = r == 0 && !disponible;
    let inactif = r > 0 && !prog.actif(p, a, n);
    let (bord, fond) = if complet {
        (OR_VIF, NUIT_OR)
    } else if r > 0 || disponible {
        (VERT_GAIN, NUIT)
    } else {
        (branche.with_alpha(0.45), NUIT.with_alpha(0.8))
    };
    let choisie = k == c.etat.noeud;
    let cote = if choisie { CASE_CHOISIE } else { CASE };
    let o = centre - Vec2::splat(cote / 2.0);
    let b = commandes
        .spawn((
            Button,
            ActionMenu::Talent(ActionTalent::Noeud(a, k)),
            Node {
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                padding: UiRect::all(Val::Px(cote * 0.2)),
                position_type: PositionType::Absolute,
                left: Val::Px(o.x),
                top: Val::Px(o.y),
                width: Val::Px(cote),
                height: Val::Px(cote),
                ..default()
            },
            BorderColor::all(bord),
            BackgroundColor(fond),
            Survol(bord),
            ChildOf(zone),
        ))
        .id();
    let halo = if choisie {
        Some((Color::WHITE.with_alpha(0.6), 2.0, 12.0))
    } else if complet {
        Some((OR_VIF.with_alpha(0.4), 0.0, 10.0))
    } else if disponible {
        Some((VERT_GAIN.with_alpha(0.3), 0.0, 10.0))
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
                left: Val::Px(-20.0),
                right: Val::Px(-20.0),
                bottom: Val::Px(-17.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ChildOf(b),
        ))
        .id();
    let t = format!("{r}/{}", n.rang_max);
    let taille = if choisie { 14.0 } else { 12.0 };
    let e = ecrire(commandes, ic, pied, &t, (true, taille, couleur));
    commandes.entity(e).insert(TextShadow::default());
    if n.gains.arme_depart.is_some() {
        let e = ecrire(commandes, ic, b, "★", (true, 15.0, OR_VIF));
        commandes.entity(e).insert((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(-8.0),
                left: Val::Px(-6.0),
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
                right: Val::Px(-6.0),
                top: Val::Px(-4.0),
                ..default()
            },
            ChildOf(b),
        ));
    }
}
