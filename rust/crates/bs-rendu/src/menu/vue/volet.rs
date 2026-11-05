//! Volet de l'accueil, ouvert par la classe ou l'arène du résumé : un panneau à deux
//! colonnes posé à droite de la colonne du menu (la liste à gauche, la fiche de la
//! choisie à droite), sur un voile qui le referme au clic.

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use super::bas::BAS;
use super::colonne;
use super::{Pinceau, arenes, classes, place};
use crate::ecrans::charte::{self, BORD};
use crate::lancement::{ChoixPartie, DonneesJeu};
use crate::menu::{ActionMenu, Volet};
use crate::sauvegarde::Sauvegarde;

const LARGEUR: f32 = 720.0;
/// Bord gauche du volet : après la colonne du menu.
const GAUCHE: f32 = charte::MARGE + colonne::LARGEUR + 32.0;
/// Haut du volet : sous le profil.
const HAUT: f32 = 84.0;

pub(super) fn construire(
    p: &mut Pinceau,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    (choix, ouvert, message): (ChoixPartie, Volet, Option<&str>),
    racine: Entity,
) {
    if ouvert == Volet::Aucun {
        return;
    }
    // Voile cliquable à droite de la colonne (qui reste cliquable) : referme le volet
    let tout = Some(Val::ZERO);
    let gauche = Some(Val::Px(GAUCHE - 16.0));
    p.c.spawn((
        Button,
        ActionMenu::Volet(Volet::Aucun),
        place(gauche, tout, tout, tout),
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.45)),
        ChildOf(racine),
    ));
    let zone = p.noeud(
        racine,
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            ..place(
                Some(Val::Px(GAUCHE)),
                Some(Val::Px(charte::MARGE)),
                Some(Val::Px(HAUT)),
                Some(Val::Px(BAS + 40.0)),
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
        Volet::Arenes => arenes::construire(p, dj, sauvegarde, (choix, message), colonnes),
        Volet::Aucun => {}
    }
}
