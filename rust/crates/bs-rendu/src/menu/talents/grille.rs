//! Un arbre de talents en grille, à la manière des jeux de rôle en ligne : un panneau
//! tapissé de la texture de l'arbre (`fond`), son nom et les points qui y sont mis en
//! tête, puis une case carrée par talent (4 colonnes, une rangée par ligne). Une rangée
//! s'ouvre avec les points dépensés au-dessus d'elle dans l'arbre ; seuls les vrais
//! prérequis (amélioration d'une capacité) ont une flèche. Bord vert : apprenable ;
//! doré : complet ; gris et icône éteinte : fermé. Le rang s'écrit au coin.

use bevy::prelude::*;
use bs_donnees::progression::Noeud;

use super::{ActionTalent, Contexte, ecrire, rangee};
use crate::ecrans::charte::{AIDE, IVOIRE, OR_VIF, VERT_GAIN};
use crate::hud::police::TextePixel;
use crate::menu::{ActionMenu, Icones};

/// Côté d'une case, pas entre deux cases, marge intérieure (px).
const CASE: f32 = 46.0;
const PAS_X: f32 = 64.0;
const PAS_Y: f32 = 58.0;
const MARGE: f32 = 20.0;
const COLONNES: f32 = 4.0;
const LIGNES: f32 = 7.0;
const FERME: Color = Color::srgb_u8(0x5a, 0x5e, 0x62);

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

fn coin(n: &Noeud) -> Vec2 {
    Vec2::new(n.colonne as f32 * PAS_X, n.ligne as f32 * PAS_Y)
}

/// Panneau de l'arbre `a` de la classe consultée.
pub fn construire(
    commandes: &mut Commands,
    c: &Contexte,
    ic: &mut Icones,
    parent: Entity,
    a: usize,
) {
    let prog = &c.sauvegarde.progression;
    let p = c.id();
    let def = &c.dj.d.talents.arbres[p][a];
    let panneau = commandes
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(3.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BorderColor::all(Color::srgb_u8(0x8b, 0x84, 0x74)),
            BackgroundColor(Color::BLACK),
            BoxShadow::new(
                Color::srgba(0.0, 0.0, 0.0, 0.7),
                Val::ZERO,
                Val::Px(6.0),
                Val::ZERO,
                Val::Px(18.0),
            ),
            ChildOf(parent),
        ))
        .id();
    // Tapis de l'arbre, assombri
    commandes.spawn((
        ImageNode {
            image: ic.icone(&def.fond),
            color: Color::srgb(0.33, 0.33, 0.33),
            image_mode: NodeImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 3.0,
            },
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        ChildOf(panneau),
    ));
    // Tête : icône, nom, points mis dans l'arbre
    let tete = rangee(commandes, panneau, 10.0);
    commandes.entity(tete).insert((
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.12)),
    ));
    commandes.entity(tete).entry::<Node>().and_modify(|mut n| {
        n.padding = UiRect::axes(Val::Px(14.0), Val::Px(8.0));
        n.border = UiRect::bottom(Val::Px(1.0));
    });
    commandes.spawn((
        ImageNode::new(ic.icone(&def.icone)),
        Node {
            width: Val::Px(26.0),
            height: Val::Px(26.0),
            ..default()
        },
        ChildOf(tete),
    ));
    ecrire(commandes, ic, tete, &def.nom, (true, 22.0, IVOIRE));
    let mis = prog.points(p, a, None);
    let e = ecrire(commandes, ic, tete, &mis.to_string(), (true, 22.0, OR_VIF));
    commandes.entity(e).insert(Node {
        margin: UiRect::left(Val::Auto),
        ..default()
    });
    let largeur = (COLONNES - 1.0) * PAS_X + CASE;
    let hauteur = (LIGNES - 1.0) * PAS_Y + CASE;
    let grille = commandes
        .spawn((
            Node {
                width: Val::Px(largeur),
                height: Val::Px(hauteur),
                margin: UiRect::all(Val::Px(MARGE)),
                ..default()
            },
            ChildOf(panneau),
        ))
        .id();
    // Flèches des vrais prérequis (même colonne), sous les cases
    for n in &def.noeuds {
        for id in &n.prerequis {
            if let Some(parent) = def.noeuds.iter().find(|x| &x.id == id) {
                fleche(commandes, grille, parent, n, prog.complet(p, a, parent));
            }
        }
    }
    for (k, n) in def.noeuds.iter().enumerate() {
        case(commandes, c, ic, grille, a, k, n);
    }
}

/// Flèche verticale d'un talent vers celui qu'il ouvre.
fn fleche(commandes: &mut Commands, grille: Entity, de: &Noeud, vers: &Noeud, allumee: bool) {
    let couleur = if allumee {
        OR_VIF
    } else {
        Color::srgb_u8(0x8a, 0x8a, 0x8a)
    };
    let (a, b) = (coin(de), coin(vers));
    let x = a.x + CASE / 2.0;
    let haut = a.y + CASE;
    let bas = b.y - 2.0;
    commandes.spawn((
        absolu(x - 3.0, haut, 6.0, (bas - haut - 4.0).max(0.0)),
        BackgroundColor(couleur),
        BorderColor::all(Color::BLACK),
        ChildOf(grille),
    ));
    // Pointe : un carré tourné, à moitié caché sous la case
    commandes.spawn((
        absolu(x - 6.0, bas - 10.0, 12.0, 12.0),
        UiTransform::from_rotation(Rot2::degrees(45.0)),
        BackgroundColor(couleur),
        ChildOf(grille),
    ));
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
    grille: Entity,
    a: usize,
    k: usize,
    n: &Noeud,
) {
    let prog = &c.sauvegarde.progression;
    let p = c.id();
    let r = prog.rang(p, a, n);
    let disponible = prog.refus_achat(p, a, n).is_none();
    let complet = r >= n.rang_max;
    let ferme = r == 0 && !disponible;
    let inactif = r > 0 && !prog.actif(p, a, n);
    let bord = if complet {
        OR_VIF
    } else if r > 0 || disponible {
        VERT_GAIN
    } else {
        FERME
    };
    let choisie = a == c.etat.arbre && k == c.etat.noeud;
    let o = coin(n);
    let b = commandes
        .spawn((
            Button,
            ActionMenu::Talent(ActionTalent::Noeud(a, k)),
            Node {
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                padding: UiRect::all(Val::Px(2.0)),
                ..absolu(o.x, o.y, CASE, CASE)
            },
            BorderColor::all(bord),
            BackgroundColor(Color::BLACK),
            Survol(bord),
            ChildOf(grille),
        ))
        .id();
    if choisie {
        commandes.entity(b).insert(BoxShadow::new(
            Color::WHITE.with_alpha(0.8),
            Val::ZERO,
            Val::ZERO,
            Val::Px(2.0),
            Val::Px(6.0),
        ));
    } else if disponible {
        commandes.entity(b).insert(BoxShadow::new(
            VERT_GAIN.with_alpha(0.45),
            Val::ZERO,
            Val::ZERO,
            Val::ZERO,
            Val::Px(8.0),
        ));
    }
    let teinte = if ferme {
        Color::srgb(0.3, 0.3, 0.3)
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
        BackgroundColor(Color::srgb_u8(0x1c, 0x1e, 0x22)),
        ChildOf(b),
    ));
    // Rang au coin bas droit, sur fond noir
    let couleur = if complet {
        OR_VIF
    } else if r > 0 || disponible {
        VERT_GAIN
    } else {
        AIDE
    };
    let pastille = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(-8.0),
                bottom: Val::Px(-8.0),
                padding: UiRect::axes(Val::Px(4.0), Val::ZERO),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(0x08, 0x08, 0x0a)),
            BorderColor::all(Color::srgb_u8(0x55, 0x55, 0x55)),
            ChildOf(b),
        ))
        .id();
    let t = format!("{r}/{}", n.rang_max);
    ecrire(commandes, ic, pastille, &t, (true, 12.0, couleur));
    if n.gains.arme_depart.is_some() {
        let e = ecrire(commandes, ic, b, "★", (true, 15.0, OR_VIF));
        commandes.entity(e).insert((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(-9.0),
                left: Val::Px(-7.0),
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
