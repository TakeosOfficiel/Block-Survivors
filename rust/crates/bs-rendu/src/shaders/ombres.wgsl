// Passe d'ombre des blocs (`BLOC` : vent, bande d'images) et des entités : position
// des sommets, découpe des pixels transparents (feuilles, fleurs, skins), comme leur
// rendu. Les translucides et les effets non éclairés ne projettent pas d'ombre.

#import bevy_pbr::mesh_functions
#import bevy_pbr::view_transformations::position_world_to_clip
#import bevy_render::globals::Globals
#ifdef BLOC
#import bs_rendu::vent::souffle
#endif

// Dans la passe d'ombre, le temps est rangé juste après la vue
@group(0) @binding(1) var<uniform> globals: Globals;

struct ParametresBloc {
    ambiante: vec4<f32>,
    soleil: vec4<f32>,
    direction: vec4<f32>,
    ciel: vec4<f32>,
    brouillard: vec4<f32>,
    animation: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> p: ParametresBloc;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var texture_bloc: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var echantillonneur: sampler;

struct Sommet {
    @builtin(instance_index) instance: u32,
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,
#ifdef BLOC
    @location(6) vent: vec4<f32>,
#endif
}

struct Sortie {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
    @location(1) profondeur: f32,
#endif
}

@vertex
fn vertex(s: Sommet) -> Sortie {
    var o: Sortie;
    let monde_depuis_local = mesh_functions::get_world_from_local(s.instance);
    var monde = mesh_functions::mesh_position_local_to_world(monde_depuis_local, vec4(s.position, 1.0));
    var exclu = p.animation.w > 0.5;
#ifdef BLOC
    monde = vec4(monde.xyz + souffle(monde.xyz, s.vent, globals.time), 1.0);
#else
    exclu = exclu || p.brouillard.w > 0.5;
#endif
    o.position = position_world_to_clip(monde.xyz);
    if exclu {
        // Triangle réduit à un point : rien n'est dessiné
        o.position = vec4(0.0, 0.0, 0.5, 1.0);
    }
#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
    o.profondeur = o.position.z;
    o.position.z = min(o.position.z, 1.0);
#endif
    o.uv = s.uv;
    return o;
}

fn decouper(uv_sommet: vec2<f32>) {
    var uv = uv_sommet;
#ifdef BLOC
    if p.animation.z > 0.5 {
        uv = fract(uv);
    }
    uv.y = (uv.y + p.animation.y) / max(p.animation.x, 1.0);
#endif
    let a = textureSampleLevel(texture_bloc, echantillonneur, uv, 0.0).a;
    if p.brouillard.z > 0.0 && a < p.brouillard.z {
        discard;
    }
}

#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
struct SortieFragment {
    @builtin(frag_depth) profondeur: f32,
}

@fragment
fn fragment(in: Sortie) -> SortieFragment {
    decouper(in.uv);
    var f: SortieFragment;
    f.profondeur = in.profondeur;
    return f;
}
#else
@fragment
fn fragment(in: Sortie) {
    decouper(in.uv);
}
#endif
