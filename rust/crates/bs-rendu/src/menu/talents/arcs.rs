//! L'arbre choisi en arcs, posé sur la scène de l'accueil : le talent de base en bas,
//! au centre ; chaque ligne de l'arbre est un arc plus grand, qui s'ouvre avec les
//! points mis plus bas (le seuil est écrit là où l'arc croise l'horizontale du talent de
//! base, cadenas tant qu'il manque).
//! Une case carrée par talent, à la charte de l'accueil : bord vert si apprenable ou
//! commencé, or si complet, creux et icône éteinte si fermé ; le rang dessous.

use bevy::prelude::*;
use bs_donnees::progression::Noeud;

use super::{ActionTalent, Contexte, ecrire};
use crate::ecrans::charte::{
    self, AIDE, BORD, CHOISI, CREUX, OR_SOURD, OR_VIF, SURFACE, VERT_GAIN,
};
use crate::hud::police::TextePixel;
use crate::menu::{ActionMenu, Icones};

/// Zone des arcs, centre des arcs (le talent de base), rayons (px).
const LARGEUR: f32 = 800.0;
const HAUTEUR: f32 = 600.0;
const CENTRE: Vec2 = Vec2::new(LARGEUR / 2.0, 548.0);
const RAYON_1: f32 = 88.0;
const PAS_RAYON: f32 = 58.0;
const CASE: f32 = 50.0;

fn absolu(x: f32, y: f32, l: f32, h: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(x),
        top: Val::Px(y),
        width: Val::Px(l),
        height: Val::Px(h),
        ..default()
    }
}

fn rayon(ligne: u32) -> f32 {
    if ligne == 0 {
        0.0
    } else {
        RAYON_1 + (ligne - 1) as f32 * PAS_RAYON
    }
}

/// Ouverture de l'arc d'une ligne de `n` talents (radians).
fn ouverture(ligne: u32, n: usize) -> f32 {
    let degres = if ligne < 3 { (30 * n).min(100) } else { 110 };
    (degres as f32).to_radians()
}

/// Centre de la case `i` (sur `n`) de la ligne `ligne`.
fn place(ligne: u32, i: usize, n: usize) -> Vec2 {
    if ligne == 0 {
        return CENTRE;
    }
    let pas = if n > 1 {
        ouverture(ligne, n) / (n - 1) as f32
    } else {
        0.0
    };
    let angle = -std::f32::consts::FRAC_PI_2 + (i as f32 - (n - 1) as f32 / 2.0) * pas;
    CENTRE + rayon(ligne) * Vec2::new(angle.cos(), angle.sin())
}

pub fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, parent: Entity) {
    let def = c.arbre();
    let zone = commandes
        .spawn((
            Node {
                width: Val::Px(LARGEUR),
                height: Val::Px(HAUTEUR),
                flex_shrink: 0.0,
                overflow: Overflow::clip(),
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    let lignes = def.noeuds.iter().map(|n| n.ligne).max().unwrap_or(0);
    for l in 1..=lignes {
        arc(commandes, c, ic, zone, l);
    }
    for l in 0..=lignes {
        let mut ligne: Vec<(usize, &Noeud)> = def
            .noeuds
            .iter()
            .enumerate()
            .filter(|(_, n)| n.ligne == l)
            .collect();
        ligne.sort_by_key(|(_, n)| n.colonne);
        let total = ligne.len();
        for (i, (k, n)) in ligne.into_iter().enumerate() {
            case(commandes, c, ic, zone, (k, n), place(l, i, total));
        }
    }
}

/// Arc de la ligne `l` et, à son bout gauche, le seuil de points qui l'ouvre.
fn arc(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, zone: Entity, l: u32) {
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let ouverte = prog.palier_atteint(p, a, l);
    let r = rayon(l);
    let alpha = if ouverte { 0.22 } else { 0.08 };
    commandes.spawn((
        Node {
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..absolu(CENTRE.x - r, CENTRE.y - r, 2.0 * r, 2.0 * r)
        },
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, alpha)),
        ChildOf(zone),
    ));
    let palier = c.dj.d.talents.paliers.iter().find(|x| x.ligne == l);
    let Some(palier) = palier.filter(|x| x.rangs > 0) else {
        return;
    };
    let o = CENTRE - Vec2::new(r, 0.0);
    let marque = commandes
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(4.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..absolu(o.x - 20.0, o.y - 11.0, 40.0, 22.0)
            },
            BackgroundColor(charte::PANNEAU),
            BorderColor::all(if ouverte { BORD } else { OR_SOURD }),
            ChildOf(zone),
        ))
        .id();
    let couleur = if ouverte { AIDE } else { OR_VIF };
    if !ouverte {
        charte::cadenas(commandes, marque, 9.0, couleur);
    }
    let t = palier.rangs.to_string();
    ecrire(commandes, ic, marque, &t, (true, 12.0, couleur));
}

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

fn case(
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
                ..absolu(o.x, o.y, CASE, CASE)
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
