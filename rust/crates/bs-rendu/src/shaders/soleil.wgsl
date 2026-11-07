// Ombre du soleil (carte d'ombres de la lumière directionnelle de `soleil.rs`) :
// 1 en pleine lumière, 0 à l'ombre. Seule la passe principale y a accès.

#define_import_path bs_rendu::soleil

#import bevy_pbr::mesh_view_bindings::{lights, view}
#import bevy_pbr::mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT
#import bevy_pbr::shadows::fetch_directional_shadow

fn ombre_soleil(monde: vec4<f32>, normale: vec3<f32>) -> f32 {
    if lights.n_directional_lights == 0u {
        return 1.0;
    }
    if (lights.directional_lights[0].flags & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) == 0u {
        return 1.0;
    }
    let profondeur_vue = (view.view_from_world * monde).z;
    return fetch_directional_shadow(0u, monde, normale, profondeur_vue);
}
