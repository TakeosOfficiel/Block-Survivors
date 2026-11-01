// Entités (joueur, ennemis, alliés, objets tenus) : même éclairage et même brouillard
// que les blocs, découpe nette des pixels transparents, faces visibles des deux côtés
// (rendu « entity cutout no cull » du jeu) et surcouche de couleur par instance
// (rouge de blessure, blanc du creeper) lue dans le MeshTag de l'entité.
// Effets (projectiles, orbes, ombres) : non éclairés (`brouillard.w`), teinte
// multipliée au lieu de la surcouche (`animation.y`), translucides avec une opacité
// (`animation.w`, `animation.z`), toujours devant le décor (`animation.x` : chiffres
// de dégâts, barres de vie).

#import bevy_pbr::mesh_functions
#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::view_transformations::position_world_to_clip
#import bs_rendu::lumiere::lumiere_blocs
#import bs_rendu::soleil::ombre_soleil

struct ParametresBloc {
    ambiante: vec4<f32>,
    soleil: vec4<f32>,
    direction: vec4<f32>,
    ciel: vec4<f32>,
    // début, fin du brouillard, seuil de découpe, lumineux
    brouillard: vec4<f32>,
    // devant le décor (1), teinte multipliée (1), opacité, translucide (1)
    animation: vec4<f32>,
    // lumière des blocs : coin de la boîte, active (1) ; taille de la boîte
    volume: vec4<f32>,
    taille: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> p: ParametresBloc;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var texture_skin: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var echantillonneur: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var niveaux: texture_3d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var lissage: sampler;

struct Sommet {
    @builtin(instance_index) instance: u32,
    @location(0) position: vec3<f32>,
    @location(1) normale: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(5) couleur: vec4<f32>,
}

struct Sortie {
    @builtin(position) position: vec4<f32>,
    @location(0) monde: vec4<f32>,
    @location(1) normale: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) couleur: vec4<f32>,
    @location(4) surcouche: vec4<f32>,
}

// Quatre octets (r, g, b, force), sans unpack4x8unorm (absent de WebGL2)
fn deballer(t: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(t & 255u),
        f32((t >> 8u) & 255u),
        f32((t >> 16u) & 255u),
        f32((t >> 24u) & 255u),
    ) / 255.0;
}

@vertex
fn vertex(s: Sommet) -> Sortie {
    var o: Sortie;
    let monde_depuis_local = mesh_functions::get_world_from_local(s.instance);
    o.monde = mesh_functions::mesh_position_local_to_world(monde_depuis_local, vec4(s.position, 1.0));
    o.position = position_world_to_clip(o.monde.xyz);
    if p.animation.x > 0.5 {
        // Profondeur inversée : z = w, sur le plan proche, devant tout
        o.position.z = o.position.w;
    }
    o.normale = mesh_functions::mesh_normal_local_to_world(s.normale, s.instance);
    o.uv = s.uv;
    o.couleur = s.couleur;
    o.surcouche = deballer(mesh_functions::get_tag(s.instance));
    return o;
}

@fragment
fn fragment(in: Sortie, @builtin(front_facing) devant: bool) -> @location(0) vec4<f32> {
    let c = textureSample(texture_skin, echantillonneur, in.uv) * in.couleur;
    if c.a < p.brouillard.z {
        discard;
    }
    var n = normalize(in.normale);
    if !devant {
        n = -n;
    }
    let ombre = ombre_soleil(in.monde, n);
    var lumiere = p.ambiante.rgb + p.soleil.rgb * max(dot(n, p.direction.xyz), 0.0) * ombre
        + lumiere_blocs(niveaux, lissage, p.volume, p.taille, in.monde.xyz, n);
    if p.brouillard.w > 0.5 {
        lumiere = vec3(1.0);
    }
    var couleur = mix(c.rgb * lumiere, in.surcouche.rgb, in.surcouche.a);
    if p.animation.y > 0.5 {
        couleur = c.rgb * lumiere * in.surcouche.rgb;
    }
    let profondeur = -(view.view_from_world * in.monde).z;
    let f = smoothstep(p.brouillard.x, p.brouillard.y, profondeur);
    var alpha = 1.0;
    if p.animation.w > 0.5 {
        alpha = c.a * p.animation.z;
    }
    return vec4(mix(couleur, p.ciel.rgb, f), alpha);
}
