//! Volet de l'accueil, ouvert par la tuile de la classe ou la vignette de l'arène : un
//! panneau à deux colonnes posé au-dessus du choix de partie (la liste à gauche, la
//! fiche de la choisie à droite), sur un voile qui le referme au clic.

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use super::bas::HAUTEUR_CHOIX;
use super::{Pinceau, arenes, classes, place};
use crate::ecrans::charte::{self, BORD};
use crate::lancement::{ChoixPartie, DonneesJeu};
use crate::menu::{ActionMenu, Volet};
use crate::sauvegarde::Sauvegarde;

const LARGEUR: f32 = 720.0;
/// Écart entre le volet et le haut des tuiles.
const ECART: f32 = 12.0;

pub(super) fn construire(
    p: &mut Pinceau,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    (choix, ouvert): (ChoixPartie, Volet),
    racine: Entity,
) {
    if ouvert == Volet::Aucun {
        return;
    }
    // Voile cliquable : referme le volet (sous les tuiles, qui restent cliquables)
    let tout = Some(Val::ZERO);
    p.c.spawn((
        Button,
        ActionMenu::Volet(Volet::Aucun),
        place(tout, tout, tout, Some(Val::Px(HAUTEUR_CHOIX + 4.0))),
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.45)),
        ChildOf(racine),
    ));
    let zone = p.noeud(
        racine,
        Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::Center,
            ..place(
                tout,
                tout,
                Some(Val::Px(24.0)),
                Some(Val::Px(HAUTEUR_CHOIX + ECART)),
            )
        },
    );
    let cadre = charte::panneau(
        p.c,
        zone,
        Node {
            width: Val::Px(LARGEUR),
            max_width: Val::Percent(96.0),
            max_height: Val::Percent(100.0),
            overflow: Overflow::scroll_y(),
            ..default()
        },
    );
    p.c.entity(cadre)
        .insert((FocusPolicy::Block, ScrollPosition::default()));
    let rangee = p.noeud(
        cadre,
        Node {
            column_gap: Val::Px(20.0),
            align_items: AlignItems::FlexStart,
            ..default()
        },
    );
    let gauche = p.noeud(
        rangee,
        Node {
            width: Val::Px(300.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            ..default()
        },
    );
    let droite = p.noeud(
        rangee,
        Node {
            flex_grow: 1.0,
            flex_basis: Val::ZERO,
            flex_direction: FlexDirection::Column,
            padding: UiRect::left(Val::Px(20.0)),
            border: UiRect::left(Val::Px(2.0)),
            ..default()
        },
    );
    p.c.entity(droite).insert(BorderColor::all(BORD));
    let colonnes = (gauche, droite);
    match ouvert {
        Volet::Classes => classes::construire(p, dj, sauvegarde, choix, colonnes),
        Volet::Arenes => arenes::construire(p, dj, sauvegarde, choix, colonnes),
        Volet::Aucun => {}
    }
}
