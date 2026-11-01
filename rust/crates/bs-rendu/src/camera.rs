//! Caméra de jeu, vue de dessus (src/moteur/camera.ts) : inclinée, elle suit le joueur
//! avec un lissage et se règle à la molette. La vue à la 3e personne (F5) et les
//! secousses viendront avec les effets (étape 2d).

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bs_commun::maths::facteur_lissage;

/// Réglages de src/config.ts (`camera`).
const INCLINAISON_DEG: f32 = 55.0;
const CHAMP_DEG: f32 = 45.0;
const HAUTEUR_VISEE: f32 = 1.0;
const DISTANCE: f32 = 24.0;
const DISTANCE_MIN: f32 = 10.0;
const DISTANCE_MAX: f32 = 40.0;
/// Facteur exponentiel par pixel de molette.
const SENSIBILITE_ZOOM: f32 = 0.0012;
/// Raideurs du suivi du joueur et du zoom (1/s).
const SUIVI: f32 = 10.0;
const LISSAGE_ZOOM: f32 = 12.0;
/// Pixels par cran de molette (les souris à crans comptent en lignes).
const PIXELS_PAR_LIGNE: f32 = 100.0;

#[derive(Component)]
pub struct CameraJeu {
    /// Point suivi (lissé).
    x: f32,
    z: f32,
    distance: f32,
    distance_cible: f32,
}

/// Point que la caméra doit suivre (le joueur, interpolé).
#[derive(Resource, Default)]
pub struct Cible(pub Vec2);

pub fn creer_camera(commandes: &mut Commands, depart: Vec2, ciel: Color) {
    commandes.spawn((
        CameraJeu {
            x: depart.x,
            z: depart.y,
            distance: DISTANCE,
            distance_cible: DISTANCE,
        },
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(ciel),
            ..default()
        },
        // Couleurs telles quelles, comme le rendu three.js (pas de tonemapping)
        Tonemapping::None,
        Projection::Perspective(PerspectiveProjection {
            fov: CHAMP_DEG.to_radians(),
            near: 0.1,
            far: 500.0,
            ..default()
        }),
        Transform::default(),
    ));
}

pub fn suivre(
    temps: Res<Time>,
    cible: Res<Cible>,
    mut molette: MessageReader<MouseWheel>,
    mut cameras: Query<(&mut CameraJeu, &mut Transform)>,
) {
    let dt = temps.delta_secs();
    let mut crans = 0.0;
    for m in molette.read() {
        crans += match m.unit {
            bevy::input::mouse::MouseScrollUnit::Line => m.y * PIXELS_PAR_LIGNE,
            bevy::input::mouse::MouseScrollUnit::Pixel => m.y,
        };
    }
    let (sin_i, cos_i) = INCLINAISON_DEG.to_radians().sin_cos();
    for (mut c, mut t) in &mut cameras {
        // Molette vers soi (y < 0) : on s'éloigne
        c.distance_cible = (c.distance_cible * (-crans * SENSIBILITE_ZOOM).exp())
            .clamp(DISTANCE_MIN, DISTANCE_MAX);
        let k = facteur_lissage(SUIVI, dt);
        c.x += (cible.0.x - c.x) * k;
        c.z += (cible.0.y - c.z) * k;
        let kz = facteur_lissage(LISSAGE_ZOOM, dt);
        c.distance += (c.distance_cible - c.distance) * kz;
        t.translation = Vec3::new(
            c.x,
            HAUTEUR_VISEE + c.distance * sin_i,
            c.z + c.distance * cos_i,
        );
        t.rotation = Quat::from_rotation_x(-sin_i.asin());
    }
}
