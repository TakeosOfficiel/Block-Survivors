//! Schéma d'un arbre de talents (src/ui/ecranTalents.ts `dessinerSchema`) : liens de
//! redstone allumés quand le talent du dessus est complet, paliers (rangs à acheter
//! au-dessus), cases des talents par type (bonus, capacité, à choix, final), rang acheté,
//! pied à la couleur de la branche.

use bevy::prelude::*;
use bs_donnees::progression::{Branche, Noeud, TypeNoeud};

use super::{ActionTalent, Contexte};
use crate::hud::police::TextePixel;
use crate::menu::ActionMenu;
use crate::menu::Icones;

/// Placement des talents (px).
const ECART_X: f32 = 66.0;
const ECART_Y: f32 = 68.0;
const MARGE: f32 = 18.0;
const TAILLE: f32 = 46.0;
pub const LARGEUR: f32 = 540.0;
pub const HAUTEUR: f32 = 486.0;

const LIEN: Color = Color::srgb_u8(0x5a, 0x17, 0x12);
const LIEN_ALLUME: Color = Color::srgb_u8(0xff, 0x3b, 0x2a);
const VERT: Color = Color::srgb_u8(0x55, 0xff, 0x55);
const ROUGE: Color = Color::srgb_u8(0xff, 0x8a, 0x8a);
const JAUNE: Color = Color::srgb_u8(0xff, 0xff, 0x55);

pub fn couleur_branche(b: Branche) -> Color {
    match b {
        Branche::Tronc => Color::srgb_u8(0xb3, 0xa3, 0x97),
        Branche::Gauche => Color::srgb_u8(0x6f, 0xb2, 0xff),
        Branche::Centre => Color::srgb_u8(0xff, 0xa2, 0x4a),
        Branche::Droite => Color::srgb_u8(0x67, 0xd9, 0x93),
        Branche::Libre => Color::srgb_u8(0xc9, 0x9b, 0xff),
    }
}

fn coin(n: &Noeud) -> Vec2 {
    Vec2::new(
        MARGE + n.colonne as f32 * ECART_X,
        MARGE + n.ligne as f32 * ECART_Y,
    )
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

pub fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, parent: Entity) {
    let schema = commandes
        .spawn((
            Node {
                width: Val::Px(LARGEUR),
                height: Val::Px(HAUTEUR),
                flex_shrink: 0.0,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            BorderColor::all(Color::srgb_u8(0x1a, 0x1a, 0x1a)),
            ChildOf(parent),
        ))
        .id();
    let def = c.arbre();
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    // Liens : un trait tourné entre les centres
    for n in &def.noeuds {
        for id in &n.prerequis {
            let Some(parent) = def.noeuds.iter().find(|x| &x.id == id) else {
                continue;
            };
            let (de, vers) = (coin(parent), coin(n));
            let d = vers - de;
            let longueur = d.length();
            let milieu = (de + vers) / 2.0 + Vec2::splat(TAILLE / 2.0);
            let couleur = if prog.complet(p, a, parent) {
                LIEN_ALLUME
            } else {
                LIEN
            };
            commandes.spawn((
                absolu(
                    milieu - Vec2::new(longueur / 2.0, 2.0),
                    Vec2::new(longueur, 4.0),
                ),
                UiTransform::from_rotation(Rot2::radians(d.y.atan2(d.x))),
                BackgroundColor(couleur),
                ChildOf(schema),
            ));
        }
    }
    // Paliers
    for palier in &c.dj.d.talents.paliers {
        let atteint = prog.palier_atteint(p, a, palier.ligne);
        let haut = MARGE + palier.ligne as f32 * ECART_Y - 15.0;
        let couleur = if atteint { VERT } else { ROUGE };
        let porte = commandes
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(8.0),
                    right: Val::Px(8.0),
                    top: Val::Px(haut),
                    height: Val::Px(2.0),
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
                },
                BackgroundColor(couleur.with_alpha(0.5)),
                ChildOf(schema),
            ))
            .id();
        let achetes = palier.rangs.min(prog.points(p, a, Some(palier.ligne)));
        let t = if atteint {
            format!("{} rangs : ouvert", palier.rangs)
        } else {
            format!("{achetes} / {} rangs", palier.rangs)
        };
        commandes.spawn((
            TextePixel::nouveau(&t, couleur, 1.5),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(-8.0),
                padding: UiRect::horizontal(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(0x14, 0x14, 0x14)),
            ChildOf(porte),
        ));
    }
    for (k, n) in def.noeuds.iter().enumerate() {
        noeud(commandes, c, ic, schema, k, n);
    }
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
    let rang = prog.rang(p, a, n);
    let ferme = rang == 0 && !prog.ouvert(p, a, n);
    let inactif = rang > 0 && !prog.actif(p, a, n);
    let (fond, bord) = match n.genre {
        TypeNoeud::Capacite => (
            Color::srgb_u8(0x6b, 0x4a, 0x22),
            Color::srgb_u8(0xf2, 0xd1, 0x4b),
        ),
        TypeNoeud::Final => (
            Color::srgb_u8(0x3a, 0x1f, 0x5c),
            Color::srgb_u8(0xb4, 0x8c, 0xff),
        ),
        _ => (
            Color::srgb_u8(0x37, 0x37, 0x37),
            Color::srgb_u8(0x8b, 0x8b, 0x8b),
        ),
    };
    let fond = if rang > 0 {
        Color::srgb_u8(0x5f, 0x8f, 0x4f)
    } else {
        fond
    };
    let choisi = k == c.etat.noeud;
    let contour = if choisi {
        Outline::new(Val::Px(3.0), Val::Px(2.0), JAUNE)
    } else if rang >= n.rang_max {
        Outline::new(Val::Px(2.0), Val::ZERO, Color::srgba(1.0, 0.86, 0.24, 0.75))
    } else {
        Outline::new(Val::Px(2.0), Val::ZERO, Color::BLACK)
    };
    let mut boite = absolu(coin(n), Vec2::splat(TAILLE));
    boite.border = UiRect::all(Val::Px(3.0));
    boite.justify_content = JustifyContent::Center;
    boite.align_items = AlignItems::Center;
    let b = commandes
        .spawn((
            Button,
            ActionMenu::Talent(ActionTalent::Noeud(k)),
            boite,
            BackgroundColor(fond),
            BorderColor::all(bord),
            contour,
            ChildOf(schema),
        ))
        .id();
    // Icône (grisée si fermée ou inactive) ; talent à choix : deux moitiés
    let teinte = if ferme {
        Color::srgb(0.45, 0.45, 0.45)
    } else if inactif {
        Color::srgb(0.6, 0.6, 0.6)
    } else {
        Color::WHITE
    };
    match &n.options {
        Some(options) => {
            let choisie = prog.option(p, a, n);
            for (i, o) in options.iter().enumerate() {
                let eteinte = rang > 0 && i != choisie;
                let t = if eteinte {
                    Color::srgb(0.35, 0.35, 0.35)
                } else {
                    teinte
                };
                let place = if i == 0 {
                    Vec2::splat(0.0)
                } else {
                    Vec2::splat(TAILLE - 6.0 - 24.0)
                };
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
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    ..default()
                },
                ChildOf(b),
            ));
        }
    }
    if n.rang_max > 1 || rang > 0 {
        let couleur = if rang >= n.rang_max {
            JAUNE
        } else {
            Color::WHITE
        };
        commandes.spawn((
            TextePixel::nouveau(&format!("{rang}/{}", n.rang_max), couleur, 1.5),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(-4.0),
                bottom: Val::Px(-8.0),
                ..default()
            },
            ChildOf(b),
        ));
    }
    if n.gains.arme_depart.is_some() {
        commandes.spawn((
            TextePixel::nouveau("+", JAUNE, 2.0),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(-2.0),
                top: Val::Px(-6.0),
                ..default()
            },
            ChildOf(b),
        ));
    }
    if inactif {
        commandes.spawn((
            TextePixel::nouveau("zZ", Color::srgb(0.87, 0.87, 0.87), 1.0),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            ChildOf(b),
        ));
    }
    commandes.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(5.0),
            right: Val::Px(5.0),
            bottom: Val::Px(-10.0),
            height: Val::Px(3.0),
            ..default()
        },
        BackgroundColor(couleur_branche(n.branche)),
        ChildOf(b),
    ));
}
