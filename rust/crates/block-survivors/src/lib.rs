//! Block Survivors : assemblage de l'application Bevy, commun au PC et au navigateur.
//! Prototype de l'étape 0 bis (docs/PLAN-RUST.md) : l'arène, le joueur, 300 zombies et une
//! arme (bs-regles pour la simulation, bs-rendu pour l'affichage).

use bevy::prelude::*;

/// Lance le jeu (ne rend pas la main avant la fermeture de la fenêtre).
pub fn lancer() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("Block Survivors {}", bs_commun::VERSION),
                // Navigateur : le jeu remplit le canevas de la page
                canvas: Some("#jeu".into()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(bs_rendu::PluginPrototype { graine: 1 })
        .run();
}
