//! Mise en page des écrans en jeu (src/ui/ecranPause.ts, ecranFin.ts) : bandeau de pause
//! en travers de l'écran, panneau orné des statistiques, écran de fin sur fond rouge
//! (défaite) ou doré (victoire). Le bilan défile à la molette (`Defilant`).

use bevy::prelude::*;
use bs_regles::etat::Issue;
use bs_regles::partie::Partie;

use super::Action;
use super::bilan::{self, Contexte, texte};
use super::style::{self, GRIS, JOYAU, OR_CLAIR, Teinte, plein};

/// Au-dessus du HUD et des cartes.
const PLAN: i32 = 20;
const NOIR: (u8, u8, u8) = (6, 8, 14);
const ROUGE: Color = Color::srgb_u8(0xff, 0x6a, 0x6a);
const SOUS_TITRE: Color = Color::srgb_u8(0xc8, 0xcc, 0xd6);

/// Zone qui défile à la molette.
#[derive(Component)]
pub struct Defilant;

fn noir(alpha: f32) -> Color {
    Color::srgba_u8(NOIR.0, NOIR.1, NOIR.2, (alpha * 255.0) as u8)
}

/// Racine plein écran, contenu centré en colonne.
fn racine(commandes: &mut Commands, fond: impl Bundle) -> Entity {
    commandes
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                padding: UiRect::all(Val::Px(24.0)),
                ..plein()
            },
            GlobalZIndex(PLAN),
            fond,
        ))
        .id()
}

fn rangee(commandes: &mut Commands, parent: Entity) -> Entity {
    let n = Node {
        align_items: AlignItems::Center,
        column_gap: Val::Px(12.0),
        ..default()
    };
    commandes.spawn((n, ChildOf(parent))).id()
}

/// Panneau orné qui contient le bilan défilant.
fn panneau(commandes: &mut Commands, c: &Contexte, parent: Entity, p: &Partie) -> Entity {
    let cadre = commandes
        .spawn((
            Node {
                width: Val::Px(800.0),
                max_width: Val::Percent(96.0),
                min_height: Val::ZERO,
                flex_shrink: 1.0,
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(Val::Px(20.0), Val::Px(16.0)),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(10, 12, 18, 230)),
            ChildOf(parent),
        ))
        .id();
    style::orner(commandes, c.dessins, cadre);
    let zone = commandes
        .spawn((
            Defilant,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                min_height: Val::ZERO,
                padding: UiRect::right(Val::Px(6.0)),
                overflow: Overflow {
                    x: OverflowAxis::Clip,
                    y: OverflowAxis::Scroll,
                },
                ..default()
            },
            ScrollPosition::default(),
            ChildOf(cadre),
        ))
        .id();
    bilan::bilan(commandes, c, zone, p);
    cadre
}

/// Pause : bandeau sombre aux traits d'or, résumé, Reprendre / Recommencer / Statistiques.
pub fn pause(commandes: &mut Commands, c: &Contexte, p: &Partie) -> Entity {
    let r = racine(commandes, BackgroundColor(noir(0.45)));
    let stops = vec![
        ColorStop::new(noir(0.3), Val::Percent(0.0)),
        ColorStop::new(noir(0.92), Val::Percent(25.0)),
        ColorStop::new(noir(0.92), Val::Percent(75.0)),
        ColorStop::new(noir(0.3), Val::Percent(100.0)),
    ];
    let bande = commandes
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                padding: UiRect {
                    top: Val::Px(32.0),
                    bottom: Val::Px(28.0),
                    left: Val::Px(16.0),
                    right: Val::Px(16.0),
                },
                ..default()
            },
            BackgroundGradient::from(LinearGradient::to_right(stops)),
            ChildOf(r),
        ))
        .id();
    for haut in [true, false] {
        let (trait_, joyau) = if haut {
            (
                Node {
                    top: Val::Px(-3.0),
                    ..default()
                },
                Node {
                    top: Val::Px(-18.0),
                    ..default()
                },
            )
        } else {
            (
                Node {
                    bottom: Val::Px(-3.0),
                    ..default()
                },
                Node {
                    bottom: Val::Px(-18.0),
                    ..default()
                },
            )
        };
        style::trait_estompe(commandes, bande, trait_);
        let joyau = Node {
            left: Val::Percent(50.0),
            margin: UiRect::left(Val::Px(-JOYAU / 2.0)),
            ..joyau
        };
        style::joyau(commandes, c.dessins, bande, joyau);
    }
    style::titre(commandes, bande, "Pause", OR_CLAIR, 6.0);
    bilan::pastilles(commandes, c, bande, p);
    let boutons = rangee(commandes, bande);
    style::bouton(
        commandes,
        boutons,
        "Recommencer",
        180.0,
        false,
        Action::Recommencer,
    );
    style::bouton(
        commandes,
        boutons,
        "Reprendre",
        240.0,
        true,
        Action::Reprendre,
    );
    style::bouton(
        commandes,
        boutons,
        "Statistiques",
        180.0,
        false,
        Action::Statistiques,
    );
    texte(commandes, bande, "Échap : reprendre", GRIS, 1.5);
    r
}

/// Statistiques de la pause : panneau orné, bilan, Retour.
pub fn statistiques(commandes: &mut Commands, c: &Contexte, p: &Partie) -> Entity {
    let r = racine(commandes, BackgroundColor(noir(0.74)));
    style::titre(commandes, r, "Statistiques", OR_CLAIR, 5.0);
    panneau(commandes, c, r, p);
    style::bouton(commandes, r, "Retour", 200.0, false, Action::Retour);
    r
}

/// Fin de partie : titre, personnage et arène, tuiles, bilan, Rejouer (et « Continuer
/// sans fin » après une victoire hors mode sans fin).
pub fn fin(commandes: &mut Commands, c: &Contexte, p: &Partie) -> Entity {
    let e = &p.etat;
    let victoire = e.issue == Some(Issue::Victoire);
    let (haut, bas) = if victoire {
        (
            Color::srgba_u8(90, 70, 0, 107),
            Color::srgba_u8(14, 10, 4, 230),
        )
    } else {
        (
            Color::srgba_u8(70, 0, 0, 115),
            Color::srgba_u8(10, 6, 10, 230),
        )
    };
    let stops = vec![
        ColorStop::new(haut, Val::Percent(0.0)),
        ColorStop::new(bas, Val::Percent(100.0)),
    ];
    let r = racine(
        commandes,
        BackgroundGradient::from(LinearGradient::to_bottom(stops)),
    );
    let (titre, couleur) = if victoire {
        ("Victoire !", OR_CLAIR)
    } else {
        ("Vous êtes mort !", ROUGE)
    };
    style::titre(commandes, r, titre, couleur, 6.0);
    let lieu = format!("{} · {}", e.personnage.nom, e.arene.nom);
    let lieu = if e.sans_fin {
        format!("Mode sans fin · {lieu}")
    } else {
        lieu
    };
    texte(commandes, r, &lieu, SOUS_TITRE, 2.0);
    bilan::tuiles(commandes, c, r, p);
    panneau(commandes, c, r, p);
    let boutons = rangee(commandes, r);
    style::bouton(commandes, boutons, "Rejouer", 240.0, true, Action::Rejouer);
    if victoire && !e.sans_fin {
        let teinte = Teinte {
            bord: Color::srgb_u8(0xa0, 0x60, 0xe0),
            fond: Color::srgba_u8(40, 16, 60, 191),
        };
        let b = style::bouton(
            commandes,
            boutons,
            "Continuer sans fin",
            240.0,
            false,
            (Action::Continuer, teinte),
        );
        commandes
            .entity(b)
            .insert((BorderColor::all(teinte.bord), BackgroundColor(teinte.fond)));
    }
    texte(commandes, r, "Entrée : rejouer", GRIS, 1.5);
    r
}
