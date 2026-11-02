// Blocs de l'arène : éclairage façon Minecraft de la version TypeScript (lumière
// ambiante + soleil sur une surface mate, sans reflets), texture animée en bande
// (eau, lave), découpe nette (feuilles, fleurs), blocs lumineux et brouillard linéaire
// lissé comme THREE.Fog. Feuillage qui frémit et plantes dont le sommet se balance
// (src/monde/vegetation.ts) : des vagues de vent traversent la prairie, portées par
// des rafales plus lentes.

#import bevy_pbr::mesh_functions
#import bevy_pbr::mesh_view_bindings::{globals, view}
#import bevy_pbr::view_transformations::position_world_to_clip
#import bs_rendu::lumiere::lumiere_blocs
#import bs_rendu::soleil::ombre_soleil
#import bs_rendu::vent::souffle

struct ParametresBloc {
    // rgb × intensité
    ambiante: vec4<f32>,
    soleil: vec4<f32>,
    // vers le soleil (normalisée)
    direction: vec4<f32>,
    ciel: vec4<f32>,
    // début, fin du brouillard, seuil de découpe (0 : aucune), lumineux (1)
    brouillard: vec4<f32>,
    // images de la bande, image affichée, texture répétée (1 : uv du monde),
    // alpha transmis (1 : translucide)
    animation: vec4<f32>,
    // lumière des blocs : coin de la boîte, active (1) ; taille de la boîte
    volume: vec4<f32>,
    taille: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> p: ParametresBloc;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var texture_bloc: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var echantillonneur: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var niveaux: texture_3d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var lissage: sampler;

struct Sommet {
    @builtin(instance_index) instance: u32,
    @location(0) position: vec3<f32>,
    @location(1) normale: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(5) couleur: vec4<f32>,
    // centre x, pied y, centre z du bloc, mode (0 immobile, 1 feuillage, 2 plante)
    @location(6) vent: vec4<f32>,
}

struct Sortie {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
}

@vertex
fn vertex(s: Sommet) -> Sortie {
    var o: Sortie;
    let monde_depuis_local = mesh_functions::get_world_from_local(s.instance);
    var monde = mesh_functions::mesh_position_local_to_world(monde_depuis_local, vec4(s.position, 1.0));
    monde = vec4(monde.xyz + souffle(monde.xyz, s.vent, globals.time) * (1.0 - p.direction.w), 1.0);
    o.world_position = monde;
    o.position = position_world_to_clip(monde.xyz);
    o.world_normal = mesh_functions::mesh_normal_local_to_world(s.normale, s.instance);
    o.uv = s.uv;
    o.color = s.couleur;
    return o;
}

@fragment
fn fragment(in: Sortie, @builtin(front_facing) devant: bool) -> @location(0) vec4<f32> {
    var uv = in.uv;
    if p.animation.z > 0.5 {
        uv = fract(uv);
    }
    let images = max(p.animation.x, 1.0);
    uv.y = (uv.y + p.animation.y) / images;
    let c = textureSample(texture_bloc, echantillonneur, uv) * in.color;
    if p.brouillard.z > 0.0 && c.a < p.brouillard.z {
        discard;
    }
    var n = normalize(in.world_normal);
    if !devant {
        n = -n;
    }
    let ombre = ombre_soleil(in.world_position, n);
    var lumiere = p.ambiante.rgb + p.soleil.rgb * max(dot(n, p.direction.xyz), 0.0) * ombre
        + lumiere_blocs(niveaux, lissage, p.volume, p.taille, in.world_position.xyz, n);
    if p.brouillard.w > 0.5 {
        lumiere = vec3(1.0);
    }
    let profondeur = -(view.view_from_world * in.world_position).z;
    let f = smoothstep(p.brouillard.x, p.brouillard.y, profondeur);
    var alpha = 1.0;
    if p.animation.w > 0.5 {
        alpha = c.a;
    }
    return vec4(mix(c.rgb * lumiere, p.ciel.rgb, f), alpha);
}
