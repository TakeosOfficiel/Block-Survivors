//! Impacts : chaque coup porté par un projectile, une orbe, une flèche ou une arme qui
//! tombe laisse un éclat à la teinte de l'arme. Petit coup : étincelle, éclair rond et
//! rayons courts. Explosion : flash au sol, onde de choc, gerbe de rayons, braise qui
//! s'attarde. Pool de taille fixe : au-delà, les nouveaux remplacent les plus vieux.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::lueur::{Forme, MaillageLueur};
use super::outils::{au_sol, hachage, rgba};

/// Impacts affichés à la fois au plus.
pub const CAPACITE: usize = 160;
/// Durées de vie (s) d'un petit coup et d'une explosion.
const COUP: f32 = 0.22;
const EXPLOSION: f32 = 0.6;
/// Rayons de la gerbe d'une explosion.
const RAYONS: u32 = 12;

#[derive(Clone, Copy)]
pub struct Impact {
    pub x: f32,
    pub z: f32,
    /// Rayon de l'explosion (0 : petit coup).
    pub zone: f32,
    pub rgb: [f32; 3],
    pub age: f32,
    pub graine: f32,
}

impl Impact {
    fn duree(&self) -> f32 {
        if self.zone > 0.0 { EXPLOSION } else { COUP }
    }
}

/// Ajoute un impact ; pool plein : remplace le plus vieux.
pub fn ajouter(pool: &mut Vec<Impact>, i: Impact) {
    if pool.len() < CAPACITE {
        pool.push(i);
    } else if let Some(vieux) = pool
        .iter_mut()
        .max_by(|a, b| (a.age / a.duree()).total_cmp(&(b.age / b.duree())))
    {
        *vieux = i;
    }
}

pub fn avancer(pool: &mut Vec<Impact>, dt: f32) {
    pool.retain_mut(|i| {
        i.age += dt;
        i.age < i.duree()
    });
}

/// Blanchit une couleur (cœur des éclats).
fn blanchi(c: [f32; 3], k: f32) -> [f32; 3] {
    c.map(|v| v + (1.0 - v) * k)
}

pub fn dessiner(m: &mut MaillageLueur, pool: &[Impact], oeil: Vec3, droite: Vec3, haut: Vec3) {
    for i in pool {
        let t = i.age / i.duree();
        let c = i.rgb;
        let coeur = blanchi(c, 0.6);
        if i.zone <= 0.0 {
            // Petit coup : étincelle en croix, éclair rond, quatre rayons courts
            let p = Vec3::new(i.x, 1.0, i.z);
            let s = 0.9 * (1.0 - t).powf(0.5);
            m.carre(
                p,
                droite * s,
                haut * s,
                rgba(coeur, 2.2 * (1.0 - t)),
                Forme::Etincelle,
                0.0,
            );
            let d = 0.6 * (0.4 + t);
            m.carre(
                p,
                droite * d,
                haut * d,
                rgba(c, 1.1 * (1.0 - t)),
                Forme::Disque,
                0.0,
            );
            for k in 0..4u32 {
                let a = (k as f32 + hachage(i.x, i.graine, k) * 0.8) * TAU / 4.0;
                let dir = droite * a.cos() + haut * a.sin();
                let a0 = p + dir * (0.15 + 0.5 * t);
                let a1 = p + dir * (0.45 + 0.9 * t);
                m.faisceau(
                    oeil,
                    a0,
                    a1,
                    0.07,
                    0.0,
                    rgba(coeur, 1.6 * (1.0 - t)),
                    rgba(c, 0.0),
                );
            }
            continue;
        }
        let z = i.zone;
        // Flash au sol et boule de lumière au-dessus
        let flash = (1.0 - t / 0.25).max(0.0);
        au_sol(
            m,
            i.x,
            i.z,
            z * 1.5,
            rgba(coeur, 2.4 * flash),
            Forme::Disque,
            0.0,
        );
        let b = z * (0.7 + 0.6 * t);
        let haut_boule = Vec3::new(i.x, 0.6 + z * 0.3, i.z);
        m.carre(
            haut_boule,
            droite * b,
            haut * b,
            rgba(c, 1.8 * flash),
            Forme::Disque,
            0.0,
        );
        m.carre(
            haut_boule,
            droite * z * 1.6,
            haut * z * 1.6,
            rgba(coeur, 2.0 * flash),
            Forme::Etincelle,
            0.0,
        );
        // Onde de choc qui court jusqu'au-delà de la zone
        let o = 1.0 - (1.0 - t).powi(3);
        let r = z * (0.3 + 1.3 * o);
        au_sol(
            m,
            i.x,
            i.z,
            r,
            rgba(coeur, 2.2 * (1.0 - t)),
            Forme::Anneau,
            0.08,
        );
        au_sol(
            m,
            i.x,
            i.z,
            r * 0.85,
            rgba(c, 0.9 * (1.0 - t)),
            Forme::Anneau,
            0.3,
        );
        // Gerbe de rayons qui jaillissent puis retombent
        for k in 0..RAYONS {
            let a = (k as f32 + hachage(i.x, i.z + i.graine, k) * 0.7) * TAU / RAYONS as f32;
            let v = 0.7 + 0.6 * hachage(i.z, i.graine, k);
            let d = z * v * (0.2 + 1.1 * o);
            let h = (0.3 + 1.6 * v) * z * 0.6 * (t * 4.0).min(1.0) * (1.0 - t * t);
            let tete = Vec3::new(i.x + a.sin() * d, 0.3 + h, i.z + a.cos() * d);
            let recul = Vec3::new(a.sin(), 0.5, a.cos()) * (0.6 * z * (1.0 - t));
            m.faisceau(
                oeil,
                tete,
                tete - recul,
                0.12,
                0.0,
                rgba(coeur, 2.0 * (1.0 - t)),
                rgba(c, 0.0),
            );
        }
        // Braise qui s'attarde au sol
        let reste = (1.0 - t).powi(2);
        au_sol(
            m,
            i.x,
            i.z,
            z * 0.9,
            rgba(c, 0.6 * reste),
            Forme::Disque,
            0.0,
        );
    }
}
