//! Éléments des écrans en jeu au style des menus (pages « console ») : voile de nuit
//! sur la partie, titres en police du logo, textes en Inter, pilule verte pour l'action
//! principale, liens texte (or au survol) pour le reste. Aucun cadre ni panneau.

use bevy::prelude::*;

use super::charte::{self, AIDE, IVOIRE, OR_VIF, Relief};
use super::style::plein;
use crate::menu::Lien;
use crate::polices::PolicesMenu;

/// Au-dessus du HUD et des cartes.
pub const PLAN: i32 = 20;
/// Libellés éteints.
pub const TERNE: Color = Color::srgba(0.95, 0.92, 0.83, 0.38);
const OMBRE_OR: Color = Color::srgb_u8(0x8a, 0x5f, 0x0b);

/// Police d'un texte.
#[derive(Clone, Copy)]
pub enum Style {
    Normale,
    Grasse,
    Logo,
}

/// Texte dans la police, la taille (px) et la couleur données.
pub fn ecrit(
    c: &mut Commands,
    p: &PolicesMenu,
    parent: Entity,
    t: &str,
    (style, taille, couleur): (Style, f32, Color),
) -> Entity {
    let font = match style {
        Style::Normale => &p.normale,
        Style::Grasse => &p.grasse,
        Style::Logo => &p.logo,
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
    if let Style::Logo = style {
        c.entity(e).insert(TextShadow {
            offset: Vec2::new(0.0, taille / 16.0),
            color: Color::srgba(0.0, 0.0, 0.0, 0.5),
        });
    }
    e
}

/// Grand titre en police du logo, ombre portée dorée (ou sombre si pas doré).
pub fn titre(c: &mut Commands, p: &PolicesMenu, parent: Entity, t: &str, couleur: Color) {
    let e = ecrit(c, p, parent, t, (Style::Logo, 76.0, couleur));
    let ombre = if couleur == OR_VIF {
        OMBRE_OR
    } else {
        Color::srgba(0.0, 0.0, 0.0, 0.6)
    };
    c.entity(e).insert(TextShadow {
        offset: Vec2::new(0.0, 5.0),
        color: ombre,
    });
}

/// Petit surtitre doré en capitales.
pub fn surtitre(c: &mut Commands, p: &PolicesMenu, parent: Entity, t: &str) -> Entity {
    ecrit(
        c,
        p,
        parent,
        t,
        (Style::Grasse, 13.0, OR_VIF.with_alpha(0.85)),
    )
}

/// Texte d'aide gris.
pub fn aide(c: &mut Commands, p: &PolicesMenu, parent: Entity, t: &str) -> Entity {
    ecrit(c, p, parent, t, (Style::Normale, 15.0, AIDE))
}

/// Voile plein écran sur la partie : nuit transparente en haut (`haut`), plus dense en
/// bas ; contenu centré en colonne.
pub fn voile(c: &mut Commands, (haut, bas): (Color, Color)) -> Entity {
    c.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(18.0),
            padding: UiRect::axes(Val::Px(48.0), Val::Px(40.0)),
            ..plein()
        },
        GlobalZIndex(PLAN),
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::new(haut, Val::Percent(0.0)),
            ColorStop::new(bas, Val::Percent(100.0)),
        ])),
    ))
    .id()
}

/// Nuit d'alpha `a`.
pub fn nuit(a: f32) -> Color {
    Color::srgba_u8(0x05, 0x08, 0x14, (a * 255.0) as u8)
}

pub fn rangee(c: &mut Commands, parent: Entity, ecart: f32) -> Entity {
    let n = Node {
        align_items: AlignItems::Center,
        column_gap: Val::Px(ecart),
        flex_shrink: 0.0,
        ..default()
    };
    c.spawn((n, ChildOf(parent))).id()
}

pub fn colonne(c: &mut Commands, parent: Entity, ecart: f32) -> Entity {
    let n = Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(ecart),
        flex_shrink: 0.0,
        ..default()
    };
    c.spawn((n, ChildOf(parent))).id()
}

/// Pastille ronde pleine.
pub fn point(c: &mut Commands, parent: Entity, cote: f32, couleur: Color) -> Entity {
    c.spawn((
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

/// Pilule en relief (verte par défaut), libellé en police du logo ; `focus` : liseré
/// d'or (l'action d'Entrée).
pub fn pilule(
    c: &mut Commands,
    p: &PolicesMenu,
    parent: Entity,
    t: &str,
    (relief, focus): (Relief, bool),
    marque: impl Bundle,
) -> Entity {
    let b = charte::bouton_relief(c, parent, relief, 64.0, marque);
    c.entity(b).entry::<Node>().and_modify(|mut n| {
        n.border_radius = BorderRadius::MAX;
        n.padding = UiRect::horizontal(Val::Px(48.0));
        n.min_width = Val::Px(240.0);
        n.flex_shrink = 0.0;
    });
    if focus {
        c.entity(b)
            .insert(Outline::new(Val::Px(2.0), Val::Px(3.0), OR_VIF));
    }
    let e = ecrit(c, p, b, t, (Style::Logo, 28.0, IVOIRE));
    c.entity(e).insert(TextShadow {
        offset: Vec2::new(0.0, 3.0),
        color: Color::srgba(0.0, 0.0, 0.0, 0.6),
    });
    b
}

/// Relief rouge (confirmation d'une perte).
pub const ROUGE: Relief = Relief {
    fond: Color::srgb_u8(0x8a, 0x2e, 0x2e),
    bord: Color::srgb_u8(0x5a, 0x1a, 0x1a),
    fond_survol: Color::srgb_u8(0xa3, 0x3a, 0x3a),
    bord_survol: IVOIRE,
    tranche: Color::srgb_u8(0x5a, 0x1a, 0x1a),
};

/// Relief violet (mode sans fin).
pub const VIOLET: Relief = Relief {
    fond: Color::srgb_u8(0x5b, 0x3a, 0x8c),
    bord: Color::srgb_u8(0x36, 0x20, 0x58),
    fond_survol: Color::srgb_u8(0x6e, 0x48, 0xa8),
    bord_survol: IVOIRE,
    tranche: Color::srgb_u8(0x36, 0x20, 0x58),
};

/// Lien texte (or au survol).
pub fn lien(
    c: &mut Commands,
    p: &PolicesMenu,
    parent: Entity,
    t: &str,
    (taille, couleur): (f32, Color),
    marque: impl Bundle,
) -> Entity {
    let b = c
        .spawn((
            Button,
            Lien(couleur),
            marque,
            Node {
                padding: UiRect::axes(Val::Px(6.0), Val::Px(6.0)),
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    ecrit(c, p, b, t, (Style::Grasse, taille, couleur));
    b
}

/// Liens séparés de petites pastilles : (libellé, marque).
pub fn liens<M: Bundle>(
    c: &mut Commands,
    p: &PolicesMenu,
    parent: Entity,
    liste: impl IntoIterator<Item = (&'static str, M)>,
) -> Entity {
    let r = rangee(c, parent, 18.0);
    for (k, (t, m)) in liste.into_iter().enumerate() {
        if k > 0 {
            point(c, r, 5.0, TERNE);
        }
        lien(c, p, r, t, (19.0, IVOIRE), m);
    }
    r
}
