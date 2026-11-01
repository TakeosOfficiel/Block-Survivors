//! Block Survivors : assemblage de l'application Bevy, commun au PC et au navigateur.
//! La simulation vient de bs-regles, l'affichage de bs-rendu (docs/PLAN-RUST.md).

use bevy::prelude::*;

/// Lance le jeu (ne rend pas la main avant la fermeture de la fenêtre).
pub fn lancer() {
    App::new()
        // Source « mc:// » des textures vanilla : avant l'AssetPlugin de DefaultPlugins
        .add_plugins(bs_rendu::PluginSourceTextures)
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
        .add_plugins(bs_rendu::PluginJeu {
            graine: 1,
            // Provisoire (jusqu'au menu, étape 3) : BS_ARENE=map_mushroom,
            // BS_PERSONNAGE=1 par exemple
            arene: std::env::var("BS_ARENE").ok(),
            personnage: std::env::var("BS_PERSONNAGE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
        })
        .run();
}
