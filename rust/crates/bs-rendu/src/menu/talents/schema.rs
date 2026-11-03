//! Arbre de talents, centré sur le fond sans panneau, défilant à la molette. Chaque
//! palier s'ouvre sur un filet discret et son libellé à gauche (« PALIER II · 5/8
//! rangs »). Les liaisons s'allument d'or quand le talent du dessus est complet. Les
//! cases (56) se lisent par leur bord : or et halo si apprises, vert et halo si
//! achetables, éteintes et creusées si fermées ; le rang est une pastille au coin, la
//! case consultée est cerclée d'ivoire.

use bevy::prelude::*;
use bs_donnees::progression::{Branche, Noeud, TypeNoeud};

use super::{ActionTalent, Contexte, ecrire};
use crate::ecrans::charte::{
    AIDE, IVOIRE, OR_SOURD, OR_VIF, ROUGE, Relief, VERT_GAIN, bord_relief, cadenas,
};
use crate::ecrans::vues::Defilant;
use crate::hud::police::TextePixel;
use crate::menu::ActionMenu;
use crate::menu::Icones;

/// Placement des talents (px).
const ECART_X: f32 = 108.0;
const ECART_Y: f32 = 64.0;
/// Hauteur ajoutée au-dessus de chaque palier.
const BANDE: f32 = 26.0;
const BORD_X: f32 = 120.0;
const TAILLE: f32 = 52.0;
const COLONNES: f32 = 7.0;
const LIGNES: u32 = 7;
const APPRIS: Color = Color::srgb_u8(0x3a, 0x30, 0x1c);
const APPRIS_SURVOL: Color = Color::srgb_u8(0x4a, 0x3e, 0x24);
const OUVERT: Color = Color::srgb_u8(0x1d, 0x23, 0x2a);
const OUVERT_SURVOL: Color = Color::srgb_u8(0x29, 0x31, 0x3a);
const FERME: Color = Color::srgb_u8(0x10, 0x13, 0x17);
const BORD_FERME: Color = Color::srgb_u8(0x2a, 0x30, 0x36);
const BORD_OUVERT: Color = Color::srgb_u8(0x5a, 0x63, 0x6b);
const LIAISON: Color = Color::srgb_u8(0x26, 0x2b, 0x33);
const FILET: Color = Color::srgba(1.0, 1.0, 1.0, 0.07);
const FINAL: Color = Color::srgb_u8(0x8a, 0x6c, 0xcf);
const CHIFFRES: [&str; 4] = ["I", "II", "III", "IV"];

pub fn couleur_branche(b: Branche) -> Color {
    match b {
        Branche::Tronc => Color::srgb_u8(0xb3, 0xa3, 0x97),
        Branche::Gauche => Color::srgb_u8(0x6f, 0xb2, 0xff),
        Branche::Centre => Color::srgb_u8(0xff, 0xa2, 0x4a),
        Branche::Droite => Color::srgb_u8(0x67, 0xd9, 0x93),
        Branche::Libre => Color::srgb_u8(0xc9, 0x9b, 0xff),
    }
}

/// Haut de la ligne `ligne` : chaque palier au-dessus la décale d'une bande.
fn haut(c: &Contexte, ligne: u32) -> f32 {
    let bandes = 1 + c
        .dj
        .d
        .talents
        .paliers
        .iter()
        .filter(|p| p.ligne <= ligne)
        .count();
    bandes as f32 * BANDE + ligne as f32 * ECART_Y
}

fn coin(c: &Contexte, n: &Noeud) -> Vec2 {
    Vec2::new(BORD_X + n.colonne as f32 * ECART_X, haut(c, n.ligne))
}

fn absolu(position: Vec2, taille: Vec2) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(position.x),
        top: Val::Px(position.y),
        width: Val::Px(taille.x),
        height: Val::Px(taille.y),
        ..default()
    }
}

fn halo(couleur: Color, flou: f32) -> BoxShadow {
    BoxShadow::new(couleur, Val::ZERO, Val::ZERO, Val::ZERO, Val::Px(flou))
}

pub fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, parent: Entity) {
    let zone = commandes
        .spawn((
            Defilant,
            Node {
                flex_grow: 1.0,
                min_height: Val::ZERO,
                align_self: AlignSelf::Stretch,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition::default(),
            ChildOf(parent),
        ))
        .id();
    let largeur = 2.0 * BORD_X + (COLONNES - 1.0) * ECART_X + TAILLE;
    let hauteur = haut(c, LIGNES - 1) + TAILLE + 24.0;
    let schema = commandes
        .spawn((
            Node {
                width: Val::Px(largeur),
                height: Val::Px(hauteur),
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(zone),
        ))
        .id();
    paliers(commandes, c, ic, schema);
    let def = c.arbre();
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    // Liaisons : un trait tourné entre les centres
    for n in &def.noeuds {
        for id in &n.prerequis {
            let Some(parent) = def.noeuds.iter().find(|x| &x.id == id) else {
                continue;
            };
            let (de, vers) = (coin(c, parent), coin(c, n));
            let d = vers - de;
            let longueur = d.length();
            let milieu = (de + vers) / 2.0 + Vec2::splat(TAILLE / 2.0);
            let allumee = prog.complet(p, a, parent);
            let e = commandes
                .spawn((
                    absolu(
                        milieu - Vec2::new(longueur / 2.0, 2.0),
                        Vec2::new(longueur, 4.0),
                    ),
                    UiTransform::from_rotation(Rot2::radians(d.y.atan2(d.x))),
                    BackgroundColor(if allumee { OR_VIF } else { LIAISON }),
                    ChildOf(schema),
                ))
                .id();
            if allumee {
                commandes
                    .entity(e)
                    .insert(halo(OR_VIF.with_alpha(0.45), 8.0));
            }
        }
    }
    for (k, n) in def.noeuds.iter().enumerate() {
        noeud(commandes, c, ic, schema, k, n);
    }
}

/// Filet et libellé de chaque palier : nom, rangs requis (ou ouvert), cadenas.
fn paliers(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, schema: Entity) {
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let tous =
        std::iter::once((0, 0)).chain(c.dj.d.talents.paliers.iter().map(|x| (x.ligne, x.rangs)));
    for (i, (ligne, rangs)) in tous.enumerate() {
        let atteint = rangs == 0 || prog.palier_atteint(p, a, ligne);
        let y = haut(c, ligne) - BANDE + 2.0;
        commandes.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(BORD_X - 16.0),
                right: Val::ZERO,
                top: Val::Px(y + 9.0),
                height: Val::Px(1.0),
                ..default()
            },
            BackgroundColor(FILET),
            ChildOf(schema),
        ));
        let b = commandes
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::ZERO,
                    width: Val::Px(BORD_X - 24.0),
                    top: Val::Px(y),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                ChildOf(schema),
            ))
            .id();
        let nom = format!("PALIER {}", CHIFFRES.get(i).copied().unwrap_or("?"));
        let couleur = if atteint { AIDE } else { ROUGE };
        ecrire(commandes, ic, b, &nom, (true, 13.0, couleur));
        if !atteint {
            let achetes = rangs.min(prog.points(p, a, Some(ligne)));
            let l = super::rangee(commandes, b, 5.0);
            cadenas(commandes, l, 9.0, ROUGE);
            let t = format!("{achetes}/{rangs} rangs");
            ecrire(commandes, ic, l, &t, (false, 13.0, ROUGE));
        }
    }
}

/// Anneau ivoire autour de la case consultée.
fn anneau(commandes: &mut Commands, b: Entity) {
    commandes.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(-8.0),
            right: Val::Px(-8.0),
            top: Val::Px(-8.0),
            bottom: Val::Px(-10.0),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(10.0)),
            ..default()
        },
        BorderColor::all(IVOIRE),
        ChildOf(b),
    ));
}

/// Petite marque en coin haut gauche.
fn marque(commandes: &mut Commands, ic: &Icones, b: Entity, t: &str, couleur: Color) {
    let n = Node {
        position_type: PositionType::Absolute,
        top: Val::Px(-10.0),
        left: Val::Px(-8.0),
        ..default()
    };
    let e = ecrire(commandes, ic, b, t, (true, 16.0, couleur));
    commandes.entity(e).insert((n, TextShadow::default()));
}

/// Pastille de rang au coin bas droit.
fn rang(commandes: &mut Commands, ic: &Icones, b: Entity, t: &str, couleur: Color) {
    let e = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(-10.0),
                bottom: Val::Px(-12.0),
                padding: UiRect::axes(Val::Px(5.0), Val::Px(1.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(0x0b, 0x0d, 0x10)),
            BorderColor::all(couleur.with_alpha(0.6)),
            ChildOf(b),
        ))
        .id();
    ecrire(commandes, ic, e, t, (true, 12.0, couleur));
}

fn noeud(
    commandes: &mut Commands,
    c: &Contexte,
    ic: &mut Icones,
    schema: Entity,
    k: usize,
    n: &Noeud,
) {
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let r = prog.rang(p, a, n);
    let ferme = r == 0 && !prog.ouvert(p, a, n);
    // Cadenas seulement derrière un palier fermé (le reste se lit aux liaisons)
    let palier =
        c.dj.d
            .talents
            .paliers
            .iter()
            .rev()
            .find(|x| n.ligne >= x.ligne);
    let derriere = palier.is_some_and(|x| !prog.palier_atteint(p, a, x.ligne));
    let inactif = r > 0 && !prog.actif(p, a, n);
    let disponible = prog.refus_achat(p, a, n).is_none();
    let complet = r >= n.rang_max;
    let bord = match n.genre {
        _ if r > 0 => OR_VIF,
        _ if disponible => VERT_GAIN,
        _ if ferme => BORD_FERME,
        TypeNoeud::Capacite => OR_SOURD,
        TypeNoeud::Final => FINAL,
        _ => BORD_OUVERT,
    };
    let (fond, fond_survol) = if r > 0 {
        (APPRIS, APPRIS_SURVOL)
    } else if ferme {
        (FERME, OUVERT)
    } else {
        (OUVERT, OUVERT_SURVOL)
    };
    let relief = Relief {
        fond,
        bord,
        fond_survol,
        bord_survol: IVOIRE,
        tranche: bord.darker(0.15),
    };
    let mut boite = absolu(coin(c, n), Vec2::splat(TAILLE));
    boite.border = bord_relief();
    boite.border_radius = BorderRadius::all(Val::Px(6.0));
    boite.justify_content = JustifyContent::Center;
    boite.align_items = AlignItems::Center;
    let b = commandes
        .spawn((
            Button,
            ActionMenu::Talent(ActionTalent::Noeud(k)),
            boite,
            relief.composants(),
            ChildOf(schema),
        ))
        .id();
    if r > 0 && !inactif {
        commandes
            .entity(b)
            .insert(halo(OR_VIF.with_alpha(0.4), 14.0));
    } else if disponible {
        commandes
            .entity(b)
            .insert(halo(VERT_GAIN.with_alpha(0.35), 12.0));
    }
    // Icône (éteinte si fermée ou inactive) ; talent à choix : deux moitiés
    let teinte = if ferme {
        Color::srgb(0.32, 0.32, 0.34)
    } else if inactif {
        Color::srgb(0.6, 0.6, 0.6)
    } else {
        Color::WHITE
    };
    match &n.options {
        Some(options) => {
            let choisie = prog.option(p, a, n);
            for (i, o) in options.iter().enumerate() {
                let eteinte = r > 0 && i != choisie;
                let t = if eteinte {
                    Color::srgb(0.35, 0.35, 0.35)
                } else {
                    teinte
                };
                let place = Vec2::splat(if i == 0 { 2.0 } else { TAILLE - 6.0 - 26.0 });
                commandes.spawn((
                    ImageNode::new(ic.icone(&o.icone)).with_color(t),
                    absolu(place, Vec2::splat(24.0)),
                    ChildOf(b),
                ));
            }
        }
        None => {
            commandes.spawn((
                ImageNode::new(ic.icone(&n.icone)).with_color(teinte),
                Node {
                    width: Val::Px(36.0),
                    height: Val::Px(36.0),
                    ..default()
                },
                ChildOf(b),
            ));
        }
    }
    let couleur = if complet {
        OR_VIF
    } else if r > 0 {
        IVOIRE
    } else if disponible {
        VERT_GAIN
    } else {
        AIDE
    };
    if !ferme || r > 0 {
        rang(commandes, ic, b, &format!("{r}/{}", n.rang_max), couleur);
    } else if derriere {
        let cad = cadenas(commandes, b, 10.0, AIDE);
        commandes.entity(cad).entry::<Node>().and_modify(|mut n| {
            n.position_type = PositionType::Absolute;
            n.right = Val::Px(-5.0);
            n.bottom = Val::Px(-6.0);
        });
    }
    if n.gains.arme_depart.is_some() {
        marque(commandes, ic, b, "★", OR_VIF);
    }
    if inactif {
        commandes.spawn((
            TextePixel::nouveau("zZ", Color::srgb(0.87, 0.87, 0.87), 1.0),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(2.0),
                top: Val::Px(0.0),
                ..default()
            },
            ChildOf(b),
        ));
    }
    if k == c.etat.noeud {
        anneau(commandes, b);
    }
}
