//! Style commun des écrans en jeu (styles.css `.page`, `.orne`, `.bouton-mc`) : couleurs
//! d'or, voile plein écran, cadre au liseré d'or bordé de sombre avec ses losanges,
//! boutons de page en relief aux couleurs de la charte (`charte`).

use bevy::prelude::*;

use super::charte;
use crate::hud::police::TextePixel;

pub const OR_CLAIR: Color = Color::srgb_u8(0xff, 0xd7, 0x5e);
pub const GRIS: Color = Color::srgb_u8(0x9a, 0xa0, 0xad);
pub const TEXTE: Color = Color::srgb_u8(0xe8, 0xe8, 0xe8);
const FOND_BOUTON: Color = charte::SURFACE;
const FOND_BOUTON_SURVOL: Color = charte::SURFACE_SURVOL;
const BORD_BOUTON: Color = charte::BORD;

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

/// Bouton de page aux couleurs de la charte (relief, tranche basse) : texte centré,
/// `large` pour l'action principale (vert, 52 de haut).
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
                height: Val::Px(if large { 52.0 } else { 44.0 }),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: charte::bord_relief(),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            bord(repos(large), false),
            BackgroundColor(repos(large).fond),
            ChildOf(parent),
        ))
        .id();
    if large {
        commandes.entity(b).insert(Teinte {
            bord: charte::VERT_SURVOL,
            fond: charte::VERT,
        });
    }
    let taille = if large { 2.5 } else { 2.0 };
    commandes.spawn((TextePixel::nouveau(texte, TEXTE, taille), ChildOf(b)));
    b
}

/// Couleurs de repos d'un bouton de page.
fn repos(large: bool) -> Teinte {
    if large {
        Teinte {
            bord: charte::VERT_SURVOL,
            fond: charte::VERT,
        }
    } else {
        Teinte {
            bord: BORD_BOUTON,
            fond: FOND_BOUTON,
        }
    }
}

/// Bord d'un bouton de page : tranche basse sombre au repos, bord uni au survol.
fn bord(t: Teinte, survol: bool) -> BorderColor {
    let c = if survol { charte::IVOIRE } else { t.bord };
    BorderColor {
        top: c,
        left: c,
        right: c,
        bottom: if survol { c } else { charte::TRANCHE },
    }
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

/// Survol des boutons de page : bord ivoire, fond éclairci.
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
        *bord = self::bord(repos, survol);
        fond.0 = match (survol, repos.fond == charte::VERT) {
            (false, _) => repos.fond,
            (true, true) => charte::VERT_SURVOL,
            (true, false) => FOND_BOUTON_SURVOL,
        };
    }
}
