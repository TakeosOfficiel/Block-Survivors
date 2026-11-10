//! Éclairs : un tracé brisé qui tombe du ciel avec deux ramifications, un éclair de
//! lumière au sol, une onde qui s'étend jusqu'au bord de la zone touchée et une
//! étincelle. La couleur vient de l'arme (`Mecanique::Foudre { couleur }`).

use bevy::prelude::*;
use bs_commun::Alea;

use super::lueur::{Forme, MaillageLueur};

pub const CAPACITE: usize = 48;
/// Points du tracé principal, des ramifications.
const POINTS: usize = 14;
const POINTS_BRANCHE: usize = 5;
const BRANCHES: usize = 2;
/// Hauteur du départ (blocs), décalage horizontal du départ, écart du zigzag.
const HAUT: f32 = 18.0;
const DEPART: f32 = 3.0;
const ZIGZAG: f32 = 0.9;
/// Durée de vie (s), largeur du tracé et des ramifications (blocs).
const DUREE: f32 = 0.38;
const LARGEUR: f32 = 0.32;
const LARGEUR_BRANCHE: f32 = 0.14;
/// Hauteur des lueurs au sol.
const SOL: f32 = 0.06;

#[derive(Clone, Copy)]
pub struct Eclair {
    centre: Vec3,
    zone: f32,
    rgb: [f32; 3],
    pub age: f32,
    points: [Vec3; POINTS],
    branches: [[Vec3; POINTS_BRANCHE]; BRANCHES],
}

impl Eclair {
    pub fn nouveau(a: &mut Alea, x: f32, z: f32, zone: f32, rgb: [f32; 3]) -> Self {
        let bas = Vec3::new(x, 0.0, z);
        let haut = bas
            + Vec3::new(
                (a.unite() - 0.5) * 2.0 * DEPART,
                HAUT,
                (a.unite() - 0.5) * 2.0 * DEPART,
            );
        let mut points = [Vec3::ZERO; POINTS];
        for (k, p) in points.iter_mut().enumerate() {
            let t = k as f32 / (POINTS - 1) as f32;
            // Écart nul aux deux bouts, plus fort au milieu
            let ecart = ZIGZAG * (t * (1.0 - t) * 4.0).sqrt();
            *p = haut.lerp(bas, t)
                + Vec3::new(
                    (a.unite() - 0.5) * 2.0 * ecart,
                    0.0,
                    (a.unite() - 0.5) * 2.0 * ecart,
                );
        }
        let mut branches = [[Vec3::ZERO; POINTS_BRANCHE]; BRANCHES];
        for b in &mut branches {
            let depart = 3 + (a.unite() * (POINTS as f32 - 7.0)) as usize;
            let dir = Vec3::new(a.unite() - 0.5, -0.6 - a.unite() * 0.4, a.unite() - 0.5)
                .normalize()
                * (0.7 + a.unite() * 0.6);
            let mut p = points[depart];
            for q in b.iter_mut() {
                *q = p;
                p += dir + Vec3::new(a.unite() - 0.5, 0.0, a.unite() - 0.5) * 0.6;
            }
        }
        Self {
            centre: bas,
            zone,
            rgb,
            age: 0.0,
            points,
            branches,
        }
    }
}

impl Eclair {
    /// Déplace tout l'éclair (vitrine : il suit le joueur).
    pub fn deplacer(&mut self, d: Vec3) {
        self.centre += d;
        for p in &mut self.points {
            *p += d;
        }
        for b in &mut self.branches {
            for p in b.iter_mut() {
                *p += d;
            }
        }
    }
}

pub fn avancer(eclairs: &mut Vec<Eclair>, dt: f32) {
    eclairs.retain_mut(|e| {
        e.age += dt;
        e.age < DUREE
    });
}

pub fn dessiner(m: &mut MaillageLueur, eclairs: &[Eclair], oeil: Vec3, droite: Vec3, haut: Vec3) {
    for e in eclairs {
        let t = e.age / DUREE;
        // Plein éclat au début, deux rappels, puis extinction
        let scintille = if t < 0.15 {
            1.0
        } else {
            0.55 + 0.45 * (t * 38.0).cos().max(0.0)
        };
        let force = 2.2 * (1.0 - t).powf(0.7) * scintille;
        let c = [e.rgb[0], e.rgb[1], e.rgb[2], force];
        // Halo large et pâle autour du trait, cœur blanc au centre
        let halo = [e.rgb[0], e.rgb[1], e.rgb[2], force * 0.22];
        let blanc = [1.0, 1.0, 1.0, force * 0.9];
        for w in e.points.windows(2) {
            m.faisceau(oeil, w[0], w[1], LARGEUR * 5.0, LARGEUR * 5.0, halo, halo);
            m.faisceau(oeil, w[0], w[1], LARGEUR, LARGEUR, c, c);
            m.faisceau(
                oeil,
                w[0],
                w[1],
                LARGEUR * 0.35,
                LARGEUR * 0.35,
                blanc,
                blanc,
            );
        }
        let cb = [e.rgb[0], e.rgb[1], e.rgb[2], force * 0.7];
        for b in &e.branches {
            for (k, w) in b.windows(2).enumerate() {
                let f = 1.0 - k as f32 / (POINTS_BRANCHE - 1) as f32;
                let l = LARGEUR_BRANCHE * f;
                m.faisceau(oeil, w[0], w[1], l, l * 0.7, cb, cb);
            }
        }
        // Au sol : lumière vive, onde jusqu'au bord de la zone, étincelle
        let sol = e.centre + Vec3::Y * SOL;
        let (x, z) = (Vec3::X, Vec3::Z);
        let flash = (1.0 - t * 3.0).max(0.0);
        let r = e.zone * 1.3;
        m.carre(
            sol,
            x * r,
            z * r,
            [e.rgb[0], e.rgb[1], e.rgb[2], 1.6 * flash],
            Forme::Disque,
            0.0,
        );
        let onde = e.zone * (0.3 + 0.7 * (1.0 - (1.0 - t).powi(3)));
        let ca = [e.rgb[0], e.rgb[1], e.rgb[2], 1.4 * (1.0 - t)];
        m.carre(sol, x * onde, z * onde, ca, Forme::Anneau, 0.12);
        // Sol roussi qui s'éteint lentement
        let roussi = e.zone * 0.8;
        let cr = [e.rgb[0], e.rgb[1], e.rgb[2], 0.5 * (1.0 - t)];
        m.carre(sol, x * roussi, z * roussi, cr, Forme::Disque, 0.0);
        let s = 1.8 * (1.0 - t).powi(2);
        let haut_sol = e.centre + Vec3::Y * 0.4;
        m.carre(
            haut_sol,
            droite * s,
            haut * s,
            [e.rgb[0], e.rgb[1], e.rgb[2], 1.5 * flash],
            Forme::Etincelle,
            0.0,
        );
    }
}
