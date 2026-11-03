//! Arbre de talents (src/ui/ecranTalents.ts `dessinerSchema`), dans un panneau qui
//! défile à la molette : un bandeau par palier (« Palier II · 3/8 rangs requis »),
//! liaisons de 3 (ardoise fermées, vertes quand le talent du dessus est complet), cases
//! de 48 par état (disponible : bord clair et « + » ; acquis : fond vert et coche ;
//! fermé : creusé, icône éteinte, cadenas ; consulté : quatre coins d'or), rang sous la
//! case, tranche basse à la couleur de la branche.

use bevy::prelude::*;
use bs_donnees::progression::{Branche, Noeud, TypeNoeud};

use super::{ActionTalent, Contexte, ecrire};
use crate::ecrans::charte::{
    self, AIDE, BORD, CREUX, IVOIRE, OR_SOURD, OR_VIF, ROUGE, Relief, SURFACE, SURFACE_SURVOL,
    T_AIDE, VERT_GAIN,
};
use crate::ecrans::vues::Defilant;
use crate::hud::police::TextePixel;
use crate::menu::ActionMenu;
use crate::menu::Icones;

/// Placement des talents (px).
const ECART_X: f32 = 96.0;
const ECART_Y: f32 = 62.0;
const BANDE: f32 = 24.0;
const BORD_X: f32 = 24.0;
const TAILLE: f32 = 48.0;
const COLONNES: f32 = 7.0;
const LIGNES: u32 = 7;
const ACQUIS: Color = Color::srgb_u8(0x2c, 0x46, 0x32);
const ACQUIS_SURVOL: Color = Color::srgb_u8(0x36, 0x55, 0x3d);
const FINAL: Color = Color::srgb_u8(0x8a, 0x6c, 0xcf);
const FOND_BANDE: Color = Color::srgb_u8(0x1d, 0x23, 0x27);
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

/// Haut de la ligne `ligne` : les bandeaux de palier au-dessus la décalent.
fn haut(c: &Contexte, ligne: u32) -> f32 {
    let bandes = 1 + c
        .dj
        .d
        .talents
        .paliers
        .iter()
        .filter(|p| p.ligne <= ligne)
        .count();
    bandes as f32 * BANDE + ligne as f32 * ECART_Y + 6.0
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

pub fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, parent: Entity) {
    let panneau = charte::panneau(
        commandes,
        parent,
        Node {
            flex_grow: 65.0,
            flex_basis: Val::ZERO,
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
            overflow: Overflow::scroll_y(),
            ..default()
        },
    );
    commandes.entity(panneau).insert(Defilant);
    let largeur = 2.0 * BORD_X + (COLONNES - 1.0) * ECART_X + TAILLE;
    let hauteur = haut(c, LIGNES - 1) + TAILLE + 20.0;
    let schema = commandes
        .spawn((
            Node {
                width: Val::Px(largeur),
                height: Val::Px(hauteur),
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(panneau),
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
            let couleur = if prog.complet(p, a, parent) {
                VERT_GAIN
            } else {
                BORD
            };
            commandes.spawn((
                absolu(
                    milieu - Vec2::new(longueur / 2.0, 1.5),
                    Vec2::new(longueur, 3.0),
                ),
                UiTransform::from_rotation(Rot2::radians(d.y.atan2(d.x))),
                BackgroundColor(couleur),
                ChildOf(schema),
            ));
        }
    }
    for (k, n) in def.noeuds.iter().enumerate() {
        noeud(commandes, c, ic, schema, k, n);
    }
}

/// Bandeau de chaque palier : nom, rangs requis (ou ouvert), cadenas s'il est fermé.
fn paliers(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, schema: Entity) {
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let tous =
        std::iter::once((0, 0)).chain(c.dj.d.talents.paliers.iter().map(|x| (x.ligne, x.rangs)));
    for (i, (ligne, rangs)) in tous.enumerate() {
        let atteint = rangs == 0 || prog.palier_atteint(p, a, ligne);
        let b = commandes
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::ZERO,
                    right: Val::ZERO,
                    top: Val::Px(haut(c, ligne) - 25.0),
                    height: Val::Px(22.0),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    padding: UiRect::horizontal(Val::Px(8.0)),
                    border_radius: BorderRadius::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(FOND_BANDE),
                ChildOf(schema),
            ))
            .id();
        let nom = format!("Palier {}", CHIFFRES.get(i).copied().unwrap_or("?"));
        ecrire(commandes, ic, b, &nom, (true, T_AIDE, IVOIRE));
        let achetes = rangs.min(prog.points(p, a, Some(ligne)));
        let (t, couleur) = if atteint {
            ("ouvert".to_owned(), VERT_GAIN)
        } else {
            (format!("{achetes}/{rangs} rangs requis"), ROUGE)
        };
        ecrire(
            commandes,
            ic,
            b,
            &format!("· {t}"),
            (false, T_AIDE, couleur),
        );
        if !atteint {
            charte::cadenas(commandes, b, 10.0, ROUGE);
        }
    }
}

/// Coins d'or autour de la case consultée.
fn coins(commandes: &mut Commands, b: Entity) {
    let (e, l, d) = (Val::Px(3.0), Val::Px(12.0), Val::Px(-8.0));
    for (gauche, dessus) in [(true, true), (false, true), (true, false), (false, false)] {
        let mut n = Node {
            position_type: PositionType::Absolute,
            width: l,
            height: l,
            ..default()
        };
        if gauche {
            n.left = d;
            n.border.left = e;
        } else {
            n.right = d;
            n.border.right = e;
        }
        if dessus {
            n.top = d;
            n.border.top = e;
        } else {
            n.bottom = Val::Px(-10.0);
            n.border.bottom = e;
        }
        commandes.spawn((n, BorderColor::all(OR_VIF), ChildOf(b)));
    }
}

/// Petite marque en coin (texte Inter).
fn marque(commandes: &mut Commands, ic: &Icones, b: Entity, t: &str, couleur: Color, gauche: bool) {
    let n = Node {
        position_type: PositionType::Absolute,
        top: Val::Px(-9.0),
        left: if gauche { Val::Px(-6.0) } else { Val::Auto },
        right: if gauche { Val::Auto } else { Val::Px(-6.0) },
        ..default()
    };
    let e = ecrire(commandes, ic, b, t, (true, 16.0, couleur));
    commandes.entity(e).insert((n, TextShadow::default()));
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
    // Cadenas seulement derrière un palier fermé (le reste se lit aux liaisons)
    let palier =
        c.dj.d
            .talents
            .paliers
            .iter()
            .rev()
            .find(|x| n.ligne >= x.ligne);
    let derriere = palier.is_some_and(|x| !prog.palier_atteint(p, a, x.ligne));
    let inactif = rang > 0 && !prog.actif(p, a, n);
    let disponible = prog.refus_achat(p, a, n).is_none();
    let bord = match n.genre {
        _ if disponible => AIDE,
        TypeNoeud::Capacite => OR_SOURD,
        TypeNoeud::Final => FINAL,
        _ => BORD,
    };
    let (fond, fond_survol) = if rang > 0 {
        (ACQUIS, ACQUIS_SURVOL)
    } else if ferme {
        (CREUX, SURFACE)
    } else {
        (SURFACE, SURFACE_SURVOL)
    };
    let relief = Relief {
        fond,
        bord,
        fond_survol,
        bord_survol: IVOIRE,
        tranche: couleur_branche(n.branche),
    };
    let mut boite = absolu(coin(c, n), Vec2::splat(TAILLE));
    boite.border = charte::bord_relief();
    boite.border_radius = BorderRadius::all(Val::Px(2.0));
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
    // Icône (éteinte si fermée ou inactive) ; talent à choix : deux moitiés
    let teinte = if ferme {
        Color::srgb(0.4, 0.4, 0.4)
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
                let place = Vec2::splat(if i == 0 { 1.0 } else { TAILLE - 6.0 - 23.0 });
                commandes.spawn((
                    ImageNode::new(ic.icone(&o.icone)).with_color(t),
                    absolu(place, Vec2::splat(22.0)),
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
    // Rang sous la case
    let ligne = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(-16.0),
                right: Val::Px(-16.0),
                top: Val::Px(TAILLE + 1.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ChildOf(b),
        ))
        .id();
    let sous = commandes
        .spawn((
            Node {
                padding: UiRect::horizontal(Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(charte::FOND),
            ChildOf(ligne),
        ))
        .id();
    let couleur = if rang >= n.rang_max {
        VERT_GAIN
    } else if rang > 0 {
        IVOIRE
    } else {
        AIDE
    };
    ecrire(
        commandes,
        ic,
        sous,
        &format!("{rang}/{}", n.rang_max),
        (true, 13.0, couleur),
    );
    if rang >= n.rang_max {
        marque(commandes, ic, b, "✓", VERT_GAIN, false);
    } else if disponible {
        marque(commandes, ic, b, "+", VERT_GAIN, false);
    } else if ferme && derriere {
        let cad = charte::cadenas(commandes, b, 10.0, AIDE);
        commandes.entity(cad).entry::<Node>().and_modify(|mut n| {
            n.position_type = PositionType::Absolute;
            n.right = Val::Px(-5.0);
            n.top = Val::Px(-6.0);
        });
    }
    if n.gains.arme_depart.is_some() {
        marque(commandes, ic, b, "★", OR_VIF, true);
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
    if k == c.etat.noeud {
        coins(commandes, b);
    }
}
