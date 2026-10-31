//! Block Survivors : assemblage de l'application Bevy, commun au PC et au navigateur.
//! Étape 0 de la migration (docs/PLAN-RUST.md) : une fenêtre, une caméra, une lumière et
//! un bloc qui tourne, pour valider le rendu sur chaque plateforme.

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
        .insert_resource(ClearColor(Color::srgb(0.47, 0.65, 1.0)))
        .add_systems(Startup, installer_scene)
        .add_systems(Update, tourner)
        .run();
}

/// Marque le bloc de démonstration.
#[derive(Component)]
struct Bloc;

fn installer_scene(
    mut commandes: Commands,
    mut maillages: ResMut<Assets<Mesh>>,
    mut materiaux: ResMut<Assets<StandardMaterial>>,
) {
    commandes.spawn((
        Bloc,
        Mesh3d(maillages.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materiaux.add(Color::srgb(0.36, 0.62, 0.25))),
    ));
    commandes.spawn((
        DirectionalLight {
            illuminance: 8000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(3.0, 6.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commandes.spawn((
        Camera3d::default(),
        Transform::from_xyz(2.5, 2.0, 3.5).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn tourner(temps: Res<Time>, mut blocs: Query<&mut Transform, With<Bloc>>) {
    for mut t in &mut blocs {
        t.rotate_y(temps.delta_secs() * 0.8);
    }
}
