// Vent du feuillage et des plantes (src/monde/vegetation.ts), partagé par le rendu des
// blocs et leur passe d'ombre : des vagues traversent la prairie, portées par des
// rafales plus lentes. `vent` : centre x, pied y, centre z du bloc, mode (0 immobile,
// 1 feuillage, 2 plante).

#define_import_path bs_rendu::vent

// src/config.ts, `shaders.vent` : direction, vitesse des vagues, amplitudes
const VENT_DIRECTION: vec2<f32> = vec2(0.93, 0.37);
const VENT_VITESSE: f32 = 1.8;
const VENT_PLANTES: f32 = 0.1;
const VENT_FEUILLAGE: f32 = 0.035;

// Décalage d'un sommet (en coordonnées du monde) sous l'effet du vent
fn souffle(p: vec3<f32>, vent: vec4<f32>, t: f32) -> vec3<f32> {
    let mode = vent.w;
    if mode < 0.5 {
        return vec3(0.0);
    }
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

