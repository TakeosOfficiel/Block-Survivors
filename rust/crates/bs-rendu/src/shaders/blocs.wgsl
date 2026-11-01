// Blocs de l'arène : éclairage façon Minecraft de la version TypeScript (lumière
// ambiante + soleil sur une surface mate, sans reflets), texture animée en bande
// (eau, lave), découpe nette (feuilles, fleurs), blocs lumineux et brouillard linéaire
// lissé comme THREE.Fog.

#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct ParametresBloc {
    // rgb × intensité
    ambiante: vec4<f32>,
    soleil: vec4<f32>,
    // vers le soleil (normalisée)
    direction: vec4<f32>,
    ciel: vec4<f32>,
    // début, fin du brouillard, seuil de découpe (0 : aucune), lumineux (1)
    brouillard: vec4<f32>,
    // images de la bande, image affichée, texture répétée (1 : uv du monde)
    animation: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> p: ParametresBloc;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var texture_bloc: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var echantillonneur: sampler;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) devant: bool) -> @location(0) vec4<f32> {
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
    return vec4(mix(c.rgb * lumiere, p.ciel.rgb, f), 1.0);
}
