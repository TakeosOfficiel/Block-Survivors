//! Style commun des écrans en jeu (styles.css `.page`, `.orne`, `.bouton-mc`) : couleurs
//! d'or, voile plein écran, cadre au liseré d'or bordé de sombre avec ses losanges,
//! traits d'or estompés aux bouts, boutons arrondis qui s'éclairent au survol.

use bevy::prelude::*;

use crate::dessins::{COTE_JOYAU, Dessins};
use crate::hud::police::TextePixel;

pub const OR: Color = Color::srgb_u8(0xe0, 0xb0, 0x40);
pub const OR_CLAIR: Color = Color::srgb_u8(0xff, 0xd7, 0x5e);
pub const SOMBRE: Color = Color::srgb_u8(0x1a, 0x12, 0x06);
pub const GRIS: Color = Color::srgb_u8(0x9a, 0xa0, 0xad);
pub const TEXTE: Color = Color::srgb_u8(0xe8, 0xe8, 0xe8);
/// Losange affiché à 3 pixels d'écran par pixel de dessin.
pub const JOYAU: f32 = COTE_JOYAU as f32 * 3.0;
const FOND_BOUTON: Color = Color::srgba_u8(12, 14, 20, 166);
const FOND_BOUTON_SURVOL: Color = Color::srgba_u8(60, 45, 10, 153);
const BORD_BOUTON: Color = Color::srgba(1.0, 1.0, 1.0, 0.22);

/// Nœud absolu qui couvre son parent.
pub fn plein() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::ZERO,
        right: Val::ZERO,
        top: Val::ZERO,
        bottom: Val::ZERO,
        ..default()
    }
}

/// Losange posé en absolu (`node` donne sa place).
pub fn joyau(commandes: &mut Commands, dessins: &Dessins, parent: Entity, node: Node) {
    commandes.spawn((
        ImageNode::new(dessins.joyau.clone()),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(JOYAU),
            height: Val::Px(JOYAU),
            ..node
        },
        ChildOf(parent),
    ));
}

/// Cadre orné : liseré d'or de 3 px bordé de sombre dedans et dehors, un losange à
/// chaque coin (posé à −21 px : sa rangée du milieu tombe sur le trait d'or).
pub fn orner(commandes: &mut Commands, dessins: &Dessins, cadre: Entity) {
    commandes.entity(cadre).insert((
        BorderColor::all(OR),
        Outline::new(Val::Px(3.0), Val::ZERO, SOMBRE),
    ));
    commandes.spawn((
        Node {
            border: UiRect::all(Val::Px(3.0)),
            ..plein()
        },
        BorderColor::all(SOMBRE),
        ChildOf(cadre),
    ));
    let coin = Val::Px(-21.0);
    for (gauche, haut) in [(true, true), (false, true), (true, false), (false, false)] {
        let mut n = Node::default();
        if gauche {
            n.left = coin;
        } else {
            n.right = coin;
        }
        if haut {
            n.top = coin;
        } else {
            n.bottom = coin;
        }
        joyau(commandes, dessins, cadre, n);
    }
}

/// Trait d'or horizontal (sombre, or, sombre : 9 px), estompé aux deux bouts ; posé en
/// absolu à `node`.
pub fn trait_estompe(commandes: &mut Commands, parent: Entity, node: Node) {
    let ligne = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                right: Val::ZERO,
                height: Val::Px(9.0),
                flex_direction: FlexDirection::Column,
                ..node
            },
            ChildOf(parent),
        ))
        .id();
    for couleur in [SOMBRE, OR, SOMBRE] {
        let stops = vec![
            ColorStop::new(couleur.with_alpha(0.0), Val::Percent(0.0)),
            ColorStop::new(couleur, Val::Percent(18.0)),
            ColorStop::new(couleur, Val::Percent(82.0)),
            ColorStop::new(couleur.with_alpha(0.0), Val::Percent(100.0)),
        ];
        commandes.spawn((
            Node {
                height: Val::Px(3.0),
                ..default()
            },
            BackgroundGradient::from(LinearGradient::to_right(stops)),
            ChildOf(ligne),
        ));
    }
}

/// Bouton de page : texte centré, `large` pour l'action principale (240 × 52).
pub fn bouton(
    commandes: &mut Commands,
    parent: Entity,
    texte: &str,
    largeur: f32,
    large: bool,
    marque: impl Bundle,
) -> Entity {
    let b = commandes
        .spawn((
            Button,
            BoutonPage,
            marque,
            Node {
                width: Val::Px(largeur),
                height: Val::Px(if large { 52.0 } else { 40.0 }),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BorderColor::all(BORD_BOUTON),
            BackgroundColor(FOND_BOUTON),
            ChildOf(parent),
        ))
        .id();
    let taille = if large { 2.5 } else { 2.0 };
    commandes.spawn((TextePixel::nouveau(texte, TEXTE, taille), ChildOf(b)));
    b
}

/// Marque des boutons de page (couleurs de survol).
#[derive(Component)]
pub struct BoutonPage;

/// Bouton teinté (« Continuer sans fin ») : bord et fond hors survol.
#[derive(Component, Clone, Copy)]
pub struct Teinte {
    pub bord: Color,
    pub fond: Color,
}

/// Survol des boutons de page : bord d'or, fond doré.
pub fn survoler(
    mut boutons: Query<
        (
            &Interaction,
            &mut BorderColor,
            &mut BackgroundColor,
            Option<&Teinte>,
        ),
        (Changed<Interaction>, With<BoutonPage>),
    >,
) {
    for (i, mut bord, mut fond, teinte) in &mut boutons {
        let repos = teinte.copied().unwrap_or(Teinte {
            bord: BORD_BOUTON,
            fond: FOND_BOUTON,
        });
        let survol = *i != Interaction::None;
        *bord = BorderColor::all(if survol { OR } else { repos.bord });
        fond.0 = if survol {
            FOND_BOUTON_SURVOL
        } else {
            repos.fond
        };
    }
}

/// Titre d'écran (police du jeu, or clair).
pub fn titre(commandes: &mut Commands, parent: Entity, texte: &str, couleur: Color, taille: f32) {
    commandes.spawn((TextePixel::nouveau(texte, couleur, taille), ChildOf(parent)));
}
