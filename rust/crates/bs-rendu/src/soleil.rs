//! Ombres du soleil (option « shaders » de la version TS, src/moteur/rendu.ts) : une
//! lumière directionnelle Bevy, utilisée seulement pour sa carte d'ombres. Les shaders
//! des blocs et des entités gardent leur éclairage façon Minecraft et retirent le soleil
//! là où il est caché (`shaders/soleil.wgsl`). La carte suit la caméra (une cascade
//! nette près du joueur, une plus large au loin) : tout le décor visible a ses ombres,
//! même hors de l'arène. `BS_OMBRES=0` les coupe (les ombres rondes reviennent), comme
//! l'option « Ombres : NON » (appliquée aussitôt, même en partie).

use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::prelude::*;

use crate::materiau::DIRECTION_SOLEIL;

/// Résolution de la carte d'ombres (config.ts `tailleOmbres.complets`).
const TAILLE_CARTE: usize = 4096;
/// Distances couvertes par les cascades (blocs depuis la caméra).
const PREMIERE_CASCADE: f32 = 34.0;
const PORTEE: f32 = 90.0;

/// Vraies ombres actives (sinon, ombres rondes sous les entités).
#[derive(Resource, Clone, Copy)]
pub struct OmbresSoleil(pub bool);

impl Default for OmbresSoleil {
    fn default() -> Self {
        Self(std::env::var("BS_OMBRES").map_or(true, |v| v != "0"))
    }
}

pub struct PluginSoleil;

impl Plugin for PluginSoleil {
    fn build(&self, app: &mut App) {
        app.init_resource::<OmbresSoleil>()
            .insert_resource(DirectionalLightShadowMap { size: TAILLE_CARTE });
    }
}

/// Marque de la lumière du soleil.
#[derive(Component)]
pub struct Soleil;

/// La lumière du soleil, au lancement de la partie.
pub fn creer(mut commandes: Commands, actives: Res<OmbresSoleil>) {
    if actives.0 {
        allumer(&mut commandes);
    }
}

/// Les options allument ou éteignent les ombres en cours de partie.
pub fn suivre(
    mut commandes: Commands,
    actives: Res<OmbresSoleil>,
    soleils: Query<Entity, With<Soleil>>,
) {
    if !actives.is_changed() || actives.is_added() {
        return;
    }
    for s in &soleils {
        commandes.entity(s).despawn();
    }
    if actives.0 {
        allumer(&mut commandes);
    }
}

fn allumer(commandes: &mut Commands) {
    commandes.spawn((
        Soleil,
        Name::new("soleil"),
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_translation(DIRECTION_SOLEIL).looking_at(Vec3::ZERO, Vec3::Y),
        CascadeShadowConfigBuilder {
            num_cascades: 2,
            minimum_distance: 0.1,
            first_cascade_far_bound: PREMIERE_CASCADE,
            maximum_distance: PORTEE,
            overlap_proportion: 0.2,
        }
        .build(),
    ));
}
