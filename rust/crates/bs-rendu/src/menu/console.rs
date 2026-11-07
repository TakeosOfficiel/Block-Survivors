//! Mise en page « menu de console » des pages du menu (Multijoueur, Collection) : fond de
//! nuit en dégradé, en-tête « BLOCK SURVIVORS • PAGE », à gauche une liste de grands
//! intitulés en police du logo (l'ouvert énorme et doré, précédé d'un losange), un filet
//! doré, à droite la scène de l'intitulé ouvert, et un pied de page avec « Retour ».
//! Aucun cadre : du texte, des pastilles rondes, des pilules.

use bevy::prelude::*;

use super::air;
use super::vue::Lien;
use super::{ActionMenu, Icones};
use crate::ecrans::charte::{AIDE, IVOIRE, OR_VIF};

/// Intitulés non choisis, libellés éteints.
pub const TERNE: Color = Color::srgba(0.95, 0.92, 0.83, 0.38);
const OMBRE_OR: Color = Color::srgb_u8(0x8a, 0x5f, 0x0b);

/// Police d'un texte de la page.
#[derive(Clone, Copy)]
pub enum Police {
    Normale,
    Grasse,
    Logo,
}

/// Texte dans la police, la taille (px) et la couleur données.
pub fn ecrit(
    c: &mut Commands,
    ic: &Icones,
    parent: Entity,
    t: &str,
    (police, taille, couleur): (Police, f32, Color),
) -> Entity {
    let p = ic.polices;
    let font = match police {
        Police::Normale => &p.normale,
        Police::Grasse => &p.grasse,
        Police::Logo => &p.logo,
    };
    let e = c
        .spawn((
            Text::new(t),
            TextFont {
                font: font.clone(),
                font_size: taille,
                ..default()
            },
            TextColor(couleur),
            ChildOf(parent),
        ))
        .id();
    if let Police::Logo = police {
        c.entity(e).insert(TextShadow {
            offset: Vec2::new(0.0, taille / 16.0),
            color: Color::srgba(0.0, 0.0, 0.0, 0.5),
        });
    }
    e
}

/// Petit surtitre doré en capitales.
pub fn surtitre(c: &mut Commands, ic: &Icones, parent: Entity, t: &str) {
    let or = OR_VIF.with_alpha(0.85);
    ecrit(c, ic, parent, t, (Police::Grasse, 13.0, or));
}

/// Phrase d'accroche de la scène.
pub fn accroche(c: &mut Commands, ic: &Icones, parent: Entity, t: &str) {
    let e = ecrit(c, ic, parent, t, (Police::Grasse, 30.0, IVOIRE));
    c.entity(e).insert(Node {
        margin: UiRect::top(Val::Px(6.0)),
        ..default()
    });
}

/// Texte d'aide gris.
pub fn aide(c: &mut Commands, ic: &Icones, parent: Entity, t: &str) -> Entity {
    ecrit(c, ic, parent, t, (Police::Normale, 15.0, AIDE))
}

/// Un intitulé de la liste de gauche, grand titre et ligne d'aide : ouvert (doré,
/// `grand` px) sans action, sinon éteint et cliquable.
pub fn intitule(
    c: &mut Commands,
    ic: &Icones,
    parent: Entity,
    (t, aide): (&str, &str),
    (grand, action): (f32, Option<ActionMenu>),
) {
    let ouvert = action.is_none();
    let l = air::rangee(c, parent, 22.0);
    // Losange doré devant l'intitulé ouvert, place gardée sinon
    c.spawn((
        Node {
            width: Val::Px(16.0),
            height: Val::Px(16.0),
            flex_shrink: 0.0,
            margin: UiRect::bottom(Val::Px(if ouvert { 26.0 } else { 0.0 })),
            ..default()
        },
        UiTransform::from_rotation(Rot2::degrees(45.0)),
        BackgroundColor(if ouvert { OR_VIF } else { Color::NONE }),
        ChildOf(l),
    ));
    let b = air::colonne(c, l, if ouvert { 10.0 } else { 4.0 });
    if let Some(a) = action {
        c.entity(b).insert((Button, Lien(TERNE), a));
    }
    let (taille, couleur) = if ouvert {
        (grand, OR_VIF)
    } else {
        (grand * 0.62, TERNE)
    };
    let e = ecrit(c, ic, b, t, (Police::Logo, taille, couleur));
    if ouvert {
        c.entity(e).insert(TextShadow {
            offset: Vec2::new(0.0, grand / 17.0),
            color: OMBRE_OR,
        });
    }
    let (taille, couleur) = if ouvert {
        (16.0, IVOIRE.with_alpha(0.75))
    } else {
        (14.0, AIDE.with_alpha(0.6))
    };
    ecrit(c, ic, b, aide, (Police::Normale, taille, couleur));
}

/// Les parties d'une page construite par `page`.
pub struct Page {
    pub racine: Entity,
    /// Liste des intitulés.
    pub gauche: Entity,
    /// Scène de l'intitulé ouvert.
    pub scene: Entity,
}

/// Racine plein écran : fond de nuit en dégradé et en-tête « BLOCK SURVIVORS • titre ».
pub fn racine(c: &mut Commands, ic: &Icones, (nom, titre): (&str, &str)) -> Entity {
    let nuit = |r, g, b, p| ColorStop::new(Color::srgb_u8(r, g, b), Val::Percent(p));
    let racine = c
        .spawn((
            Name::new(nom.to_owned()),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(Val::Px(96.0), Val::Px(56.0)),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(0x02, 0x03, 0x08)),
            BackgroundGradient::from(LinearGradient::to_bottom(vec![
                nuit(0x0b, 0x14, 0x30, 0.0),
                nuit(0x07, 0x0c, 0x1d, 45.0),
                nuit(0x02, 0x03, 0x08, 100.0),
            ])),
        ))
        .id();
    // En-tête : nom du jeu · page
    let h = air::rangee(c, racine, 22.0);
    let pale = AIDE.with_alpha(0.8);
    ecrit(c, ic, h, "BLOCK SURVIVORS", (Police::Logo, 18.0, pale));
    air::point(c, h, 6.0, OR_VIF);
    let t = IVOIRE.with_alpha(0.9);
    ecrit(c, ic, h, titre, (Police::Logo, 30.0, t));

    racine
}

/// Fond, en-tête, liste de gauche (`gauche` px), filet doré et scène (au plus `scene` px).
pub fn page(
    c: &mut Commands,
    ic: &Icones,
    (nom, titre): (&str, &str),
    (gauche, scene): (f32, f32),
) -> Page {
    let racine = racine(c, ic, (nom, titre));
    let milieu = air::rangee(c, racine, 0.0);
    c.entity(milieu).entry::<Node>().and_modify(|mut n| {
        n.flex_grow = 1.0;
        n.min_height = Val::ZERO;
    });
    let g = air::colonne(c, milieu, 26.0);
    c.entity(g).entry::<Node>().and_modify(move |mut n| {
        n.width = Val::Px(gauche);
        n.flex_shrink = 1.0;
    });
    // Filet doré, fondu aux deux bouts
    c.spawn((
        Node {
            width: Val::Px(1.0),
            height: Val::Px(500.0),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::new(OR_VIF.with_alpha(0.0), Val::Percent(0.0)),
            ColorStop::new(OR_VIF.with_alpha(0.35), Val::Percent(50.0)),
            ColorStop::new(OR_VIF.with_alpha(0.0), Val::Percent(100.0)),
        ])),
        ChildOf(milieu),
    ));
    let d = air::colonne(c, milieu, 14.0);
    c.entity(d).entry::<Node>().and_modify(move |mut n| {
        n.max_width = Val::Px(scene);
        n.flex_grow = 1.0;
        n.flex_shrink = 1.0;
        n.margin = UiRect::left(Val::Px(64.0));
    });
    Page {
        racine,
        gauche: g,
        scene: d,
    }
}

/// Pied de page : « Retour » à gauche (ferme la page) ; renvoie la rangée, la suite à
/// droite.
pub fn pied(c: &mut Commands, ic: &Icones, racine: Entity) -> Entity {
    let p = air::rangee(c, racine, 40.0);
    c.entity(p)
        .entry::<Node>()
        .and_modify(|mut n| n.width = Val::Percent(100.0));
    let r = air::rangee(c, p, 12.0);
    c.entity(r).insert((Button, Lien(AIDE), ActionMenu::Fermer));
    let k = air::point(c, r, 30.0, Color::NONE);
    c.entity(k)
        .insert(BorderColor::all(AIDE))
        .entry::<Node>()
        .and_modify(|mut n| {
            n.border = UiRect::all(Val::Px(1.5));
            n.justify_content = JustifyContent::Center;
            n.align_items = AlignItems::Center;
        });
    ecrit(c, ic, k, "‹", (Police::Grasse, 16.0, AIDE));
    ecrit(c, ic, r, "Retour", (Police::Grasse, 16.0, AIDE));
    let vide = air::rangee(c, p, 0.0);
    c.entity(vide)
        .entry::<Node>()
        .and_modify(|mut n| n.flex_grow = 1.0);
    p
}

/// Pilule arrondie à bord fin : or si choisie, pâle sinon ; éteinte sans action.
pub fn pilule(
    c: &mut Commands,
    ic: &Icones,
    parent: Entity,
    t: &str,
    (choisie, action): (bool, Option<ActionMenu>),
) -> Entity {
    let (bord, couleur) = match (choisie, &action) {
        (true, _) => (OR_VIF, OR_VIF),
        (false, Some(_)) => (Color::srgba(1.0, 1.0, 1.0, 0.22), IVOIRE),
        (false, None) => (Color::srgba(1.0, 1.0, 1.0, 0.08), TERNE),
    };
    let b = c
        .spawn((
            Node {
                height: Val::Px(38.0),
                padding: UiRect::horizontal(Val::Px(20.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(1.5)),
                border_radius: BorderRadius::MAX,
                flex_shrink: 0.0,
                ..default()
            },
            BorderColor::all(bord),
            BackgroundColor(if choisie {
                OR_VIF.with_alpha(0.1)
            } else {
                Color::NONE
            }),
            ChildOf(parent),
        ))
        .id();
    if let Some(a) = action {
        c.entity(b).insert((Button, Lien(couleur), a));
    }
    ecrit(c, ic, b, t, (Police::Grasse, 15.0, couleur));
    b
}
