//! Construction du panneau des options, à la manière des pages « console » du menu : un
//! voile de nuit plein écran, l'en-tête « BLOCK SURVIVORS • OPTIONS », à gauche les sons
//! (un rail fin doré et sa pastille par volume, clic ou glissé), à droite l'affichage
//! (une ligne par réglage, sa valeur en or quand il est actif), « Retour » en bas. Aucun
//! cadre : du texte, des rails, des pastilles rondes.

use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};

use super::panneau::{ActionOptions, Barre, Curseur, Libelle, Reglage, nom, valeur, volume};
use super::{CATEGORIES, Options};
use crate::ecrans::charte::{AIDE, IVOIRE, OR_VIF};
use crate::polices::PolicesMenu;

/// Valeurs éteintes (réglage coupé).
pub(super) const TERNE: Color = Color::srgba(0.95, 0.92, 0.83, 0.38);
const RAIL: f32 = 300.0;
/// Réglages de la colonne « Affichage », dans l'ordre.
const AFFICHAGE: [Reglage; 8] = [
    Reglage::Fps,
    Reglage::Secousses,
    Reglage::Ombres,
    Reglage::Chiffres,
    Reglage::Compteur,
    Reglage::Vue,
    Reglage::Allies,
    Reglage::Beta,
];

/// Survol d'une ligne cliquable : un halo très léger, arrondi.
#[derive(Component)]
pub(super) struct Survol;

pub(super) const HALO: Color = Color::srgba(1.0, 1.0, 1.0, 0.05);

fn ecrit(
    c: &mut Commands,
    parent: Entity,
    t: &str,
    (police, taille, couleur): (&Handle<Font>, f32, Color),
) -> Entity {
    c.spawn((
        Text::new(t),
        TextFont {
            font: police.clone(),
            font_size: taille,
            ..default()
        },
        TextColor(couleur),
        ChildOf(parent),
    ))
    .id()
}

fn noeud(c: &mut Commands, parent: Entity, n: Node) -> Entity {
    c.spawn((n, ChildOf(parent))).id()
}

fn colonne(c: &mut Commands, parent: Entity, ecart: f32) -> Entity {
    let n = Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(ecart),
        ..default()
    };
    noeud(c, parent, n)
}

fn rangee(c: &mut Commands, parent: Entity, ecart: f32) -> Entity {
    let n = Node {
        align_items: AlignItems::Center,
        column_gap: Val::Px(ecart),
        ..default()
    };
    noeud(c, parent, n)
}

fn rond(c: &mut Commands, parent: Entity, cote: f32, couleur: Color) -> Entity {
    let n = Node {
        width: Val::Px(cote),
        height: Val::Px(cote),
        border_radius: BorderRadius::MAX,
        flex_shrink: 0.0,
        ..default()
    };
    let e = noeud(c, parent, n);
    c.entity(e).insert(BackgroundColor(couleur));
    e
}

fn surtitre(c: &mut Commands, p: &PolicesMenu, parent: Entity, t: &str) {
    let or = OR_VIF.with_alpha(0.85);
    let e = ecrit(c, parent, t, (&p.grasse, 13.0, or));
    c.entity(e).insert(Node {
        margin: UiRect::bottom(Val::Px(10.0)),
        ..default()
    });
}

/// Libellé d'une valeur, suivi par `rafraichir`.
fn valeur_suivie(c: &mut Commands, p: &PolicesMenu, parent: Entity, o: &Options, r: Reglage) {
    let (t, actif) = valeur(o, r);
    let couleur = if actif { OR_VIF } else { TERNE };
    let e = ecrit(c, parent, &t, (&p.grasse, 16.0, couleur));
    c.entity(e).insert((Libelle(r), FocusPolicy::Pass));
}

/// Volume : nom, rail cliquable (remplissage doré, pastille au bout), valeur.
fn curseur(c: &mut Commands, p: &PolicesMenu, parent: Entity, o: &Options, k: usize) {
    let l = rangee(c, parent, 20.0);
    let e = ecrit(c, l, nom(Reglage::Volume(k)), (&p.grasse, 17.0, IVOIRE));
    c.entity(e).insert(Node {
        width: Val::Px(150.0),
        ..default()
    });
    let zone = c
        .spawn((
            Button,
            Curseur(k),
            RelativeCursorPosition::default(),
            Node {
                width: Val::Px(RAIL),
                height: Val::Px(34.0),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(l),
        ))
        .id();
    let rail = c
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(6.0),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12)),
            FocusPolicy::Pass,
            ChildOf(zone),
        ))
        .id();
    let plein = c
        .spawn((
            Barre(k),
            Node {
                width: Val::Percent(volume(o, k) * 100.0),
                height: Val::Percent(100.0),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(OR_VIF),
            FocusPolicy::Pass,
            ChildOf(rail),
        ))
        .id();
    let bout = rond(c, plein, 18.0, IVOIRE);
    c.entity(bout).insert(FocusPolicy::Pass);
    c.entity(bout).entry::<Node>().and_modify(|mut n| {
        n.position_type = PositionType::Absolute;
        n.right = Val::Px(-9.0);
        n.top = Val::Px(-6.0);
    });
    valeur_suivie(c, p, l, o, Reglage::Volume(k));
}

/// Bascule : toute la ligne est cliquable, nom à gauche, valeur à droite.
fn bascule(c: &mut Commands, p: &PolicesMenu, parent: Entity, o: &Options, r: Reglage) {
    let b = c
        .spawn((
            Button,
            Survol,
            ActionOptions::Basculer(r),
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(44.0),
                padding: UiRect::horizontal(Val::Px(18.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(Color::NONE),
            ChildOf(parent),
        ))
        .id();
    let e = ecrit(c, b, nom(r), (&p.normale, 17.0, IVOIRE));
    c.entity(e).insert(FocusPolicy::Pass);
    valeur_suivie(c, p, b, o, r);
}

/// Construit le panneau ; renvoie sa racine.
pub(super) fn construire(c: &mut Commands, p: &PolicesMenu, o: &Options) -> Entity {
    let nuit = |r, g, b, a, pc| ColorStop::new(Color::srgba_u8(r, g, b, a), Val::Percent(pc));
    let racine = c
        .spawn((
            Name::new("options"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(Val::Px(96.0), Val::Px(56.0)),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(0x02, 0x03, 0x08, 235)),
            BackgroundGradient::from(LinearGradient::to_bottom(vec![
                nuit(0x0b, 0x14, 0x30, 245, 0.0),
                nuit(0x07, 0x0c, 0x1d, 248, 45.0),
                nuit(0x02, 0x03, 0x08, 252, 100.0),
            ])),
            FocusPolicy::Block,
            GlobalZIndex(30),
        ))
        .id();
    let h = rangee(c, racine, 22.0);
    ecrit(
        c,
        h,
        "BLOCK SURVIVORS",
        (&p.logo, 18.0, AIDE.with_alpha(0.8)),
    );
    rond(c, h, 6.0, OR_VIF);
    ecrit(c, h, "OPTIONS", (&p.logo, 30.0, IVOIRE.with_alpha(0.9)));

    let milieu = noeud(
        c,
        racine,
        Node {
            flex_grow: 1.0,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(72.0),
            ..default()
        },
    );
    let g = colonne(c, milieu, 14.0);
    surtitre(c, p, g, "MUSIQUE ET SONS");
    for k in 0..=CATEGORIES.len() {
        curseur(c, p, g, o, k);
    }
    // Filet doré fondu aux deux bouts
    c.spawn((
        Node {
            width: Val::Px(1.0),
            height: Val::Px(440.0),
            ..default()
        },
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::new(OR_VIF.with_alpha(0.0), Val::Percent(0.0)),
            ColorStop::new(OR_VIF.with_alpha(0.35), Val::Percent(50.0)),
            ColorStop::new(OR_VIF.with_alpha(0.0), Val::Percent(100.0)),
        ])),
        ChildOf(milieu),
    ));
    let d = colonne(c, milieu, 2.0);
    c.entity(d)
        .entry::<Node>()
        .and_modify(|mut n| n.width = Val::Px(480.0));
    surtitre(c, p, d, "AFFICHAGE");
    for r in AFFICHAGE {
        bascule(c, p, d, o, r);
    }

    let pied = rangee(c, racine, 40.0);
    c.entity(pied).entry::<Node>().and_modify(|mut n| {
        n.width = Val::Percent(100.0);
        n.justify_content = JustifyContent::SpaceBetween;
    });
    let r = rangee(c, pied, 12.0);
    c.entity(r).insert((Button, Survol, ActionOptions::Fermer));
    c.entity(r).entry::<Node>().and_modify(|mut n| {
        n.padding = UiRect::new(Val::Px(4.0), Val::Px(18.0), Val::Px(4.0), Val::Px(4.0));
        n.border_radius = BorderRadius::MAX;
    });
    let k = rond(c, r, 30.0, Color::NONE);
    c.entity(k)
        .insert(BorderColor::all(AIDE))
        .entry::<Node>()
        .and_modify(|mut n| {
            n.border = UiRect::all(Val::Px(1.5));
            n.justify_content = JustifyContent::Center;
            n.align_items = AlignItems::Center;
        });
    ecrit(c, k, "‹", (&p.grasse, 16.0, AIDE));
    ecrit(c, r, "Retour", (&p.grasse, 16.0, AIDE));
    let t = "Chaque réglage est enregistré aussitôt · Échap : fermer";
    ecrit(c, pied, t, (&p.normale, 15.0, AIDE));
    racine
}
