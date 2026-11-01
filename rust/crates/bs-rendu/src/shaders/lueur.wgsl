// Lueurs des armes : formes lumineuses dessinées au calcul (pas d'image), ajoutées à
// la scène (mélange additif). Chaque sommet porte sa forme et un réglage (UV_1), sa
// couleur (déjà multipliée par l'intensité) et ses coordonnées dans la forme (UV_0).
//   0 disque doux          1 faisceau (cœur blanc, halo coloré ; u le long, v en travers)
//   2 anneau (réglage : épaisseur)      3 flamme (réglage : graine)
//   4 traînée d'arc (u : de la queue à la tête, v : de l'intérieur au bord)
//   5 étincelle en croix
// Les lueurs s'éteignent dans le brouillard de l'arène.

#import bevy_pbr::mesh_functions
#import bevy_pbr::mesh_view_bindings::{view, globals}
#import bevy_pbr::view_transformations::position_world_to_clip

struct ParametresLueur {
    // début, fin du brouillard, intensité générale, (libre)
    brouillard: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> p: ParametresLueur;

struct Sommet {
    @builtin(instance_index) instance: u32,
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) forme: vec2<f32>,
    @location(5) couleur: vec4<f32>,
}

struct Sortie {
    @builtin(position) position: vec4<f32>,
    @location(0) monde: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) forme: vec2<f32>,
    @location(3) couleur: vec4<f32>,
}

@vertex
fn vertex(s: Sommet) -> Sortie {
    var o: Sortie;
    let m = mesh_functions::get_world_from_local(s.instance);
    o.monde = mesh_functions::mesh_position_local_to_world(m, vec4(s.position, 1.0));
    o.position = position_world_to_clip(o.monde.xyz);
    o.uv = s.uv;
    o.forme = s.forme;
    o.couleur = s.couleur;
    return o;
}

fn hachage(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

// Bruit de valeur lissé
fn bruit(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = hachage(i);
    let b = hachage(i + vec2(1.0, 0.0));
    let c = hachage(i + vec2(0.0, 1.0));
    let d = hachage(i + vec2(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

@fragment
fn fragment(in: Sortie) -> @location(0) vec4<f32> {
    let forme = i32(in.forme.x + 0.5);
    let q = in.uv * 2.0 - 1.0;
    let r = length(q);
    var a = 0.0;
    // Part de blanc ajoutée au cœur (lumière qui sature)
    var blanc = 0.0;
    if forme == 0 {
        let k = max(1.0 - r, 0.0);
        a = k * k;
        blanc = pow(k, 6.0);
    } else if forme == 1 {
        let t = abs(q.y);
        a = exp(-t * t * 5.0) * (1.0 - t);
        blanc = exp(-t * t * 40.0);
    } else if forme == 2 {
        let e = max(in.forme.y, 0.02);
        let d = (r - (1.0 - e)) / e;
        a = exp(-d * d * 3.0) * step(r, 1.0) + max(1.0 - r, 0.0) * 0.12;
        blanc = exp(-d * d * 18.0) * 0.6;
    } else if forme == 3 {
        // Flamme : goutte qui ondule, rongée par un bruit qui monte ; cœur doré
        let h = in.uv.y;
        let t = globals.time * 2.4 + in.forme.y * 17.0;
        let n = bruit(vec2(in.uv.x * 3.0, h * 3.0 - t)) * 0.65
            + bruit(vec2(in.uv.x * 7.0, h * 6.0 - t * 1.7)) * 0.35;
        let x = q.x + (bruit(vec2(h * 2.0 - t * 0.6, in.forme.y)) - 0.5) * 0.5 * h;
        let largeur = mix(0.8, 0.04, pow(h, 0.75));
        let bord = abs(x) / largeur;
        a = clamp((1.0 - bord) * 1.4 + (n - 0.5) * 0.9 - h * 0.35, 0.0, 1.0)
            * smoothstep(0.0, 0.12, h);
        blanc = smoothstep(0.7, 1.0, a) * (1.0 - h) * 0.35;
    } else if forme == 4 {
        // Traînée : plus vive vers la tête et vers le bord extérieur, stries le long
        let tete = in.uv.x;
        let bord = in.uv.y;
        // Corps coloré qui monte vers le bord, fil blanc du tranchant tout au bord
        let profil = smoothstep(0.0, 0.9, bord) * smoothstep(1.0, 0.93, bord);
        let stries = 0.7 + 0.3 * bruit(vec2(bord * 26.0, tete * 4.0));
        let fil = exp(-pow((bord - 0.94) * 28.0, 2.0));
        a = profil * pow(tete, 1.6) * stries;
        blanc = fil * pow(tete, 3.0) * 0.9;
    } else {
        let x = abs(q.x);
        let y = abs(q.y);
        let croix = max(max(1.0 - x * 9.0 - y, 0.0), max(1.0 - y * 9.0 - x, 0.0));
        let k = max(1.0 - r, 0.0);
        a = croix + k * k * 0.5;
        blanc = croix * croix;
    }
    let profondeur = -(view.view_from_world * in.monde).z;
    let f = 1.0 - smoothstep(p.brouillard.x, p.brouillard.y, profondeur);
    let force = a * in.couleur.a * f * p.brouillard.z;
    let rgb = (in.couleur.rgb + vec3(blanc)) * force;
    return vec4(rgb, 0.0);
}
