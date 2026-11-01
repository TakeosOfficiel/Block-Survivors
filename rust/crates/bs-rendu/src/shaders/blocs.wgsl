// Blocs de l'arène : éclairage façon Minecraft de la version TypeScript (lumière
// ambiante + soleil sur une surface mate, sans reflets), texture animée en bande
// (eau, lave), découpe nette (feuilles, fleurs), blocs lumineux et brouillard linéaire
// lissé comme THREE.Fog. Feuillage qui frémit et plantes dont le sommet se balance
// (src/monde/vegetation.ts) : des vagues de vent traversent la prairie, portées par
// des rafales plus lentes.

#import bevy_pbr::mesh_functions
#import bevy_pbr::mesh_view_bindings::{globals, view}
#import bevy_pbr::view_transformations::position_world_to_clip

// Vent (src/config.ts, `shaders.vent`) : direction, vitesse des vagues, amplitudes
const VENT_DIRECTION: vec2<f32> = vec2(0.93, 0.37);
const VENT_VITESSE: f32 = 1.8;
const VENT_PLANTES: f32 = 0.1;
const VENT_FEUILLAGE: f32 = 0.035;

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
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> p: ParametresBloc;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var texture_bloc: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var echantillonneur: sampler;

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

// Décalage d'un sommet (en coordonnées du monde) sous l'effet du vent
fn souffle(p: vec3<f32>, vent: vec4<f32>) -> vec3<f32> {
    let mode = vent.w;
    if mode < 0.5 {
        return vec3(0.0);
    }
    let t = globals.time;
    let d = VENT_DIRECTION;
    let phase = dot(vent.xz, d) * 0.6 - t * VENT_VITESSE;
    // Rafales : de larges bandes plus fortes qui balaient l'arène
    let rafale = 0.65 + 0.45 * sin(dot(vent.xz, d) * 0.11 - t * 0.7);
    if mode < 1.5 {
        // Feuillage : tout le bloc frémit, chaque sommet un peu différemment
        let l = p - vec3(vent.x, vent.y + 0.5, vent.z);
        return vec3(
            sin(phase + l.y * 4.0),
            sin(phase * 1.3 + l.x * 4.0) * 0.5,
            cos(phase + l.z * 4.0),
        ) * VENT_FEUILLAGE * rafale;
    }
    // Plante : pied fixe, sommet qui se balance (plus haut, plus loin)
    let hauteur = max(p.y - vent.y, 0.0);
    let balancement = sin(phase) * VENT_PLANTES * rafale
        + sin(t * 4.3 + vent.x * 3.1) * VENT_PLANTES * 0.25;
    // Le sommet s'abaisse un peu quand il penche (longueur à peu près conservée)
    let b = balancement * hauteur;
    return vec3(d.x * b, -abs(b) * 0.3, d.y * b);
}

@vertex
fn vertex(s: Sommet) -> Sortie {
    var o: Sortie;
    let monde_depuis_local = mesh_functions::get_world_from_local(s.instance);
    var monde = mesh_functions::mesh_position_local_to_world(monde_depuis_local, vec4(s.position, 1.0));
    monde = vec4(monde.xyz + souffle(monde.xyz, s.vent), 1.0);
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
    var lumiere = p.ambiante.rgb + p.soleil.rgb * max(dot(n, p.direction.xyz), 0.0);
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
