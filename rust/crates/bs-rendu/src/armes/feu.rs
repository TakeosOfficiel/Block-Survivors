//! Armes de feu : tornade (brins de feu en spirale qui s'évasent vers le haut, flammes
//! à la base), météore (vrai bloc qui tombe en tournoyant, halo et traînée, cible
//! marquée au sol, onde à l'impact), feu au sol (flammes animées par le shader) et
//! braises laissées par les tornades.

use std::f32::consts::TAU;

use bevy::prelude::*;
use bs_regles::combat::feu::{Feu, METEORE_DUREE, Meteore, Tornade};

use super::lueur::{Forme, MaillageLueur};

/// Braises : capacité, durée de vie (s), rayon (blocs).
pub const BRAISES: usize = 160;
const BRAISE_DUREE: f32 = 2.0;
const BRAISE_RAYON: f32 = 0.3;
/// Anneaux qui montent le long d'une tornade.
const ANNEAUX: usize = 4;
/// Tornade : brins, segments par brin, hauteur (blocs), rotation (rad/s).
const BRINS: usize = 7;
const SEGMENTS: usize = 14;
const TORNADE_HAUTEUR: f32 = 3.6;
const TORNADE_ROTATION: f32 = 7.0;
/// Fondu d'apparition et de disparition (s).
const APPARITION: f32 = 0.3;
const DISPARITION: f32 = 0.5;
/// Météore : départ de la chute (décalage depuis la cible, blocs).
const METEORE_DEPART: Vec3 = Vec3::new(3.0, 13.0, -2.0);
/// Impact : durée de l'onde (s).
const IMPACT: f32 = 0.45;

/// Couleurs (linéaires, intensité comprise).
const BRAISE: [f32; 3] = [1.0, 0.28, 0.04];
const FEU_BAS: [f32; 3] = [1.0, 0.22, 0.03];
const FEU_HAUT: [f32; 3] = [1.0, 0.75, 0.25];
const FLAMME: [f32; 3] = [1.0, 0.45, 0.1];
const CIBLE: [f32; 3] = [1.0, 0.18, 0.05];

#[derive(Clone, Copy)]
pub struct Braise {
    pub x: f32,
    pub z: f32,
    pub age: f32,
}

pub fn avancer(braises: &mut Vec<Braise>, dt: f32) {
    braises.retain_mut(|b| {
        b.age += dt;
        b.age < BRAISE_DUREE
    });
}

fn rgba(c: [f32; 3], a: f32) -> [f32; 4] {
    [c[0], c[1], c[2], a]
}

fn melange(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t)
}

/// Hasard fixe tiré d'une position (les flammes d'un feu restent à leur place).
fn hachage(x: f32, z: f32, k: u32) -> f32 {
    let h = (x * 12.9898 + z * 78.233 + k as f32 * 37.719).sin() * 43_758.547;
    h - h.floor()
}

/// Fondu d'entrée et de sortie d'un effet de durée `duree`.
fn fondu(temps: f32, duree: f32) -> f32 {
    (temps / APPARITION).min(1.0) * ((duree - temps) / DISPARITION).clamp(0.0, 1.0)
}

/// Disque à plat sur le sol.
fn au_sol(m: &mut MaillageLueur, x: f32, z: f32, r: f32, c: [f32; 4], forme: Forme, reglage: f32) {
    let centre = Vec3::new(x, 0.05, z);
    m.carre(centre, Vec3::X * r, Vec3::Z * r, c, forme, reglage);
}

/// Flamme posée en (x, z), tournée vers l'écran.
#[allow(clippy::too_many_arguments)]
fn flamme(
    m: &mut MaillageLueur,
    x: f32,
    z: f32,
    h: f32,
    droite: Vec3,
    haut: Vec3,
    a: f32,
    graine: f32,
) {
    let base = Vec3::new(x, 0.0, z);
    m.carre(
        base + haut * (h / 2.0),
        droite * (h * 0.4),
        haut * (h / 2.0),
        rgba(FLAMME, a),
        Forme::Flamme,
        graine,
    );
}

/// Position du météore à l'instant `t` de sa chute (cible en `x`, `z`).
pub fn position_meteore(x: f32, z: f32, t: f32, taille: f32) -> Vec3 {
    let k = (t / METEORE_DUREE).clamp(0.0, 1.0);
    let cible = Vec3::new(x, 0.5 * taille, z);
    // Accélère en tombant
    cible + METEORE_DEPART * (1.0 - k * k)
}

/// Ce que le feu montre à l'image (les objets de la simulation, ou ceux de la vitrine).
pub struct Scene<'a> {
    pub tornades: &'a [Tornade],
    pub meteores: &'a [Meteore],
    pub feux: &'a [Feu],
    pub braises: &'a [Braise],
    /// Temps écoulé depuis le dernier pas (mouvement fluide des météores).
    pub avance: f32,
}

pub fn dessiner(
    m: &mut MaillageLueur,
    scene: &Scene,
    zone: impl Fn(usize) -> f32,
    temps: f32,
    oeil: Vec3,
    droite: Vec3,
    haut: Vec3,
) {
    let braises = scene.braises;

    for b in braises {
        let t = b.age / BRAISE_DUREE;
        let vacille = 0.8 + 0.2 * (temps * 9.0 + b.x * 3.0).sin();
        let a = (1.0 - t) * vacille;
        au_sol(
            m,
            b.x,
            b.z,
            BRAISE_RAYON * (1.0 - t * 0.4),
            rgba(BRAISE, 1.3 * a),
            Forme::Disque,
            0.0,
        );
        // Escarbille qui s'élève en dérivant
        let derive = hachage(b.x, b.z, 0) - 0.5;
        let p = Vec3::new(b.x + derive * b.age, 0.2 + b.age * 0.9, b.z);
        m.carre(
            p,
            droite * 0.07,
            haut * 0.07,
            rgba(FEU_HAUT, 2.0 * a),
            Forme::Disque,
            0.0,
        );
    }

    for t in scene.tornades {
        let r = zone(t.arme) * t.zone;
        let f = fondu(t.temps, t.duree);
        let h = TORNADE_HAUTEUR * (0.6 + 0.4 * r.min(2.5) / 2.5);
        au_sol(
            m,
            t.x,
            t.z,
            r * 1.4,
            rgba(FEU_BAS, 0.8 * f),
            Forme::Disque,
            0.0,
        );
        // Colonne de chaleur au centre
        let pied = Vec3::new(t.x, 0.0, t.z);
        m.faisceau(
            oeil,
            pied,
            pied + Vec3::Y * h,
            r * 0.3,
            r * 0.9,
            rgba(FEU_BAS, 0.35 * f),
            rgba(FEU_BAS, 0.0),
        );
        // Brins en spirale
        for k in 0..BRINS {
            let phase = k as f32 * TAU / BRINS as f32;
            let mut prec: Option<(Vec3, f32, [f32; 4])> = None;
            for s in 0..=SEGMENTS {
                let y = s as f32 / SEGMENTS as f32;
                let rayon = r * (0.2 + 0.7 * y);
                let a = phase + y * 3.0 - temps * TORNADE_ROTATION;
                let p = Vec3::new(t.x + a.sin() * rayon, y * h, t.z + a.cos() * rayon);
                let largeur = 0.1 + 0.18 * y;
                let bout = (y * (1.0 - y) * 4.0).sqrt();
                let c = rgba(melange(FEU_BAS, FLAMME, y), 0.5 * f * bout);
                if let Some((p0, l0, c0)) = prec {
                    m.faisceau(oeil, p0, p, l0, largeur, c0, c);
                }
                prec = Some((p, largeur, c));
            }
        }
        // Anneaux qui montent en s'élargissant
        for k in 0..ANNEAUX {
            let y = (k as f32 / ANNEAUX as f32 + temps * 0.7 + t.x * 0.1).fract();
            let rayon = r * (0.25 + 0.75 * y);
            let a = 0.6 * f * (y * (1.0 - y) * 4.0);
            let c = Vec3::new(t.x, y * h, t.z);
            m.carre(
                c,
                Vec3::X * rayon,
                Vec3::Z * rayon,
                rgba(FLAMME, a),
                Forme::Anneau,
                0.18,
            );
        }
        for k in 0..5 {
            let a = k as f32 * TAU / 5.0 - temps * TORNADE_ROTATION * 0.5;
            let (x, z) = (t.x + a.sin() * r * 0.4, t.z + a.cos() * r * 0.4);
            flamme(m, x, z, 1.3, droite, haut, 0.9 * f, k as f32 + t.x);
        }
    }

    for me in scene.meteores {
        let r = zone(me.arme) * me.taille;
        // Cible au sol : plus vive à l'approche de l'impact
        let t = me.t + scene.avance;
        let k = (t / METEORE_DUREE).clamp(0.0, 1.0);
        au_sol(
            m,
            me.x,
            me.z,
            r,
            rgba(CIBLE, 0.4 + 0.9 * k),
            Forme::Anneau,
            0.08,
        );
        au_sol(m, me.x, me.z, r, rgba(CIBLE, 0.25 * k), Forme::Disque, 0.0);
        if t < 0.0 {
            continue;
        }
        let pos = position_meteore(me.x, me.z, t, me.taille);
        let s = 1.9 * me.taille;
        m.carre(
            pos,
            droite * s,
            haut * s,
            rgba(FLAMME, 1.3),
            Forme::Disque,
            0.0,
        );
        let arriere = pos + METEORE_DEPART.normalize() * 4.0 * me.taille;
        m.faisceau(
            oeil,
            pos,
            arriere,
            0.55 * me.taille,
            0.05,
            rgba(FEU_HAUT, 1.6),
            rgba(FEU_BAS, 0.0),
        );
    }

    for fe in scene.feux {
        let r = zone(fe.arme) * fe.zone;
        let f = fondu(fe.temps, fe.duree);
        let vacille = 0.85 + 0.15 * (temps * 11.0 + fe.x).sin();
        au_sol(
            m,
            fe.x,
            fe.z,
            r * 1.25,
            rgba(FEU_BAS, 0.8 * f * vacille),
            Forme::Disque,
            0.0,
        );
        let n = ((r * r * 3.0) as u32).clamp(5, 14);
        for k in 0..n {
            let a = hachage(fe.x, fe.z, k) * TAU;
            let d = hachage(fe.z, fe.x, k).sqrt() * r * 0.85;
            let h = 0.9 + 0.8 * hachage(fe.x + 1.0, fe.z, k);
            let (x, z) = (fe.x + a.sin() * d, fe.z + a.cos() * d);
            flamme(m, x, z, h, droite, haut, 0.9 * f, k as f32 * 1.7 + fe.z);
        }
        // Impact : éclair de lumière et onde
        if fe.temps < IMPACT {
            let t = fe.temps / IMPACT;
            let onde = r * (0.3 + 1.3 * (1.0 - (1.0 - t).powi(3)));
            au_sol(
                m,
                fe.x,
                fe.z,
                onde,
                rgba(FEU_HAUT, 2.0 * (1.0 - t)),
                Forme::Anneau,
                0.1,
            );
            let flash = (1.0 - t * 2.5).max(0.0);
            au_sol(
                m,
                fe.x,
                fe.z,
                r * 1.8,
                rgba(FEU_HAUT, 1.5 * flash),
                Forme::Disque,
                0.0,
            );
        }
    }
}
