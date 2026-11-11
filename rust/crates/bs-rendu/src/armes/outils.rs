//! Petits outils de dessin des lueurs, partagés par les armes : couleur, hasard fixe
//! tiré d'une position, disque et trait couchés au sol.

use bevy::prelude::*;

use super::lueur::{Forme, MaillageLueur};

pub fn rgba(c: [f32; 3], a: f32) -> [f32; 4] {
    [c[0], c[1], c[2], a]
}

/// `c` éclairci vers le blanc de la fraction `k`.
pub fn blanchi(c: [f32; 3], k: f32) -> [f32; 3] {
    c.map(|v| v + (1.0 - v) * k)
}

pub fn hachage(x: f32, z: f32, k: u32) -> f32 {
    let h = (x * 12.9898 + z * 78.233 + k as f32 * 37.719).sin() * 43_758.547;
    h - h.floor()
}

pub fn au_sol(
    m: &mut MaillageLueur,
    x: f32,
    z: f32,
    r: f32,
    c: [f32; 4],
    forme: Forme,
    reglage: f32,
) {
    m.carre(
        Vec3::new(x, 0.05, z),
        Vec3::X * r,
        Vec3::Z * r,
        c,
        forme,
        reglage,
    );
}

/// Trait lumineux couché au sol de `a` à `b`.
pub fn trait_au_sol(m: &mut MaillageLueur, a: Vec2, b: Vec2, la: f32, lb: f32, c: [f32; 4]) {
    let n = (b - a).perp().normalize_or_zero();
    let p = |v: Vec2, d: f32| Vec3::new(v.x + n.x * d, 0.06, v.y + n.y * d);
    let fin = [c[0], c[1], c[2], 0.0];
    m.quad(
        [p(a, -la), p(b, -lb), p(b, lb), p(a, la)],
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        [c, fin, fin, c],
        Forme::Faisceau,
        0.0,
    );
}
