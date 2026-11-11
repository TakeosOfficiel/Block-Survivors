//! Coups de mêlée : traînée en croissant du balayage, onde en arc du coup de bouclier,
//! cercle de lames du tourbillon. Dessinés en lueurs autour du joueur (ils le suivent
//! pendant leur courte vie).

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::lueur::{Forme, MaillageLueur};
use super::outils::{au_sol, blanchi, rgba};

/// Coups affichés à la fois au plus.
pub const CAPACITE: usize = 32;
/// Hauteur des traînées (au niveau de l'arme tenue) et de l'onde du bouclier.
const HAUTEUR_LAME: f32 = 0.85;
const HAUTEUR_ONDE: f32 = 0.45;
/// Balayage : temps de l'aller, puis du fondu (s) ; part de l'arc couverte par la
/// traînée ; rayon intérieur (part de la portée).
const BALAYAGE_ALLER: f32 = 0.13;
const BALAYAGE_FONDU: f32 = 0.16;
const BALAYAGE_TRAINEE: f32 = 0.85;
const BALAYAGE_INTERIEUR: f32 = 0.3;
/// Heurt : durée de l'onde, épaisseur (blocs), départ (part de la portée).
const HEURT_DUREE: f32 = 0.3;
const HEURT_EPAISSEUR: f32 = 0.9;
const HEURT_DEPART: f32 = 0.25;
/// Tourbillon : durée d'un tour de lame, arc de la traînée (rad).
const TOURBILLON_DUREE: f32 = 0.24;
const TOURBILLON_TRAINEE: f32 = 4.4;
/// Segments d'un arc.
const SEGMENTS: usize = 20;
/// Couleurs (linéaires, intensité comprise).
pub const ACIER: [f32; 3] = [0.3, 0.55, 1.0];
pub const OR: [f32; 3] = [1.0, 0.62, 0.22];

#[derive(Clone, Copy, PartialEq)]
pub enum Genre {
    Balayage,
    Heurt,
    Tourbillon,
}

#[derive(Clone, Copy)]
pub struct Coup {
    pub genre: Genre,
    pub angle: f32,
    pub portee: f32,
    pub ouverture: f32,
    /// Sens du geste (±1), alterné d'un coup à l'autre.
    pub sens: f32,
    /// Teinte de l'arme (linéaire).
    pub rgb: [f32; 3],
    pub age: f32,
    /// Écart au joueur (vitrine seulement).
    pub decalage: Vec2,
}

impl Coup {
    fn duree(&self) -> f32 {
        match self.genre {
            Genre::Balayage => BALAYAGE_ALLER + BALAYAGE_FONDU,
            Genre::Heurt => HEURT_DUREE,
            Genre::Tourbillon => TOURBILLON_DUREE,
        }
    }
}

/// Vieillit les coups et retire les finis.
pub fn avancer(coups: &mut Vec<Coup>, dt: f32) {
    coups.retain_mut(|c| {
        c.age += dt;
        c.age < c.duree()
    });
}

/// Point de l'arc d'angle `a` (convention `atan2(dx, dz)`) à la distance `r`.
fn point(centre: Vec3, a: f32, r: f32) -> Vec3 {
    centre + Vec3::new(a.sin() * r, 0.0, a.cos() * r)
}

fn couleur(rgb: [f32; 3], a: f32) -> [f32; 4] {
    [rgb[0], rgb[1], rgb[2], a]
}

/// Blanchit une couleur (fil de la lame).
/// Traînée posée à plat : de l'angle `queue` à `tete`, entre les rayons `r0` et `r1`.
#[allow(clippy::too_many_arguments)]
fn trainee(
    m: &mut MaillageLueur,
    centre: Vec3,
    queue: f32,
    tete: f32,
    r0: f32,
    r1: f32,
    rgb: [f32; 3],
    force: f32,
) {
    for k in 0..SEGMENTS {
        let (t0, t1) = (k as f32 / SEGMENTS as f32, (k + 1) as f32 / SEGMENTS as f32);
        let (a0, a1) = (queue + (tete - queue) * t0, queue + (tete - queue) * t1);
        let c = couleur(rgb, force);
        m.quad(
            [
                point(centre, a0, r0),
                point(centre, a1, r0),
                point(centre, a1, r1),
                point(centre, a0, r1),
            ],
            [[t0, 0.0], [t1, 0.0], [t1, 1.0], [t0, 1.0]],
            [c; 4],
            Forme::Trainee,
            0.0,
        );
    }
}

pub fn dessiner(m: &mut MaillageLueur, coups: &[Coup], joueur: Vec2, droite: Vec3, haut: Vec3) {
    for c in coups {
        let fil = blanchi(c.rgb, 0.65);
        let joueur = joueur + c.decalage;
        match c.genre {
            Genre::Balayage => {
                let centre = Vec3::new(joueur.x, HAUTEUR_LAME, joueur.y);
                let aller = (c.age / BALAYAGE_ALLER).min(1.0);
                // Tête qui accélère puis freine, d'un bord de l'arc à l'autre
                let avance = 1.0 - (1.0 - aller) * (1.0 - aller);
                let debut = c.angle - c.sens * c.ouverture / 2.0;
                let tete = debut + c.sens * c.ouverture * avance;
                let queue = tete - c.sens * c.ouverture * BALAYAGE_TRAINEE * avance;
                let fondu = 1.0 - ((c.age - BALAYAGE_ALLER) / BALAYAGE_FONDU).clamp(0.0, 1.0);
                let r0 = c.portee * BALAYAGE_INTERIEUR;
                trainee(m, centre, queue, tete, r0, c.portee, c.rgb, 1.1 * fondu);
                // Fil de la lame, reflet au sol, étincelle au bout
                let r1 = c.portee * 0.86;
                trainee(m, centre, queue, tete, r1, c.portee, fil, 1.5 * fondu);
                let sol = Vec3::new(joueur.x, 0.06, joueur.y);
                trainee(m, sol, queue, tete, r0, c.portee * 1.1, c.rgb, 0.35 * fondu);
                let bout = point(centre, tete, c.portee);
                let e = 0.9 * fondu;
                m.carre(
                    bout,
                    droite * e,
                    haut * e,
                    rgba(fil, 1.8 * fondu),
                    Forme::Etincelle,
                    0.0,
                );
            }
            Genre::Tourbillon => {
                let centre = Vec3::new(joueur.x, HAUTEUR_LAME, joueur.y);
                let t = c.age / TOURBILLON_DUREE;
                let tete = c.angle + c.sens * TAU * t;
                let queue = tete - c.sens * TOURBILLON_TRAINEE;
                let fondu = 1.0 - t * t;
                let r0 = c.portee * 0.45;
                // Deux lames opposées, fil vif, cercle tranchant au sol
                for d in [0.0, std::f32::consts::PI] {
                    let (q, h) = (queue + d, tete + d);
                    trainee(m, centre, q, h, r0, c.portee, c.rgb, 1.0 * fondu);
                    trainee(m, centre, q, h, c.portee * 0.88, c.portee, fil, 1.3 * fondu);
                    let bout = point(centre, h, c.portee);
                    let e = 0.7 * fondu;
                    m.carre(
                        bout,
                        droite * e,
                        haut * e,
                        rgba(fil, 1.5 * fondu),
                        Forme::Etincelle,
                        0.0,
                    );
                }
                au_sol(
                    m,
                    joueur.x,
                    joueur.y,
                    c.portee,
                    rgba(c.rgb, 0.9 * fondu),
                    Forme::Anneau,
                    0.06,
                );
                au_sol(
                    m,
                    joueur.x,
                    joueur.y,
                    c.portee,
                    rgba(c.rgb, 0.15 * fondu),
                    Forme::Disque,
                    0.0,
                );
            }
            Genre::Heurt => {
                let centre = Vec3::new(joueur.x, HAUTEUR_ONDE, joueur.y);
                let t = c.age / HEURT_DUREE;
                let r =
                    c.portee * (HEURT_DEPART + (1.0 - HEURT_DEPART) * (1.0 - (1.0 - t).powi(3)));
                let e = HEURT_EPAISSEUR * (1.0 - t * 0.5);
                let force = 2.4 * (1.0 - t).powf(1.5);
                let demi = c.ouverture / 2.0;
                // Éventail lumineux derrière l'onde, plus vif près d'elle
                let cf = couleur(c.rgb, force * 0.5);
                // Choc : éclair au départ, poussière qui s'étale au sol
                let flash = (1.0 - t * 4.0).max(0.0);
                let devant = Vec2::new(c.angle.sin(), c.angle.cos()) * c.portee * 0.35;
                let f = joueur + devant;
                au_sol(
                    m,
                    f.x,
                    f.y,
                    c.portee * 0.6,
                    rgba(fil, 2.0 * flash),
                    Forme::Disque,
                    0.0,
                );
                au_sol(
                    m,
                    f.x,
                    f.y,
                    r * 0.8,
                    rgba(c.rgb, 0.8 * (1.0 - t)),
                    Forme::Anneau,
                    0.3,
                );
                for k in 0..SEGMENTS {
                    let (t0, t1) = (k as f32 / SEGMENTS as f32, (k + 1) as f32 / SEGMENTS as f32);
                    let (a0, a1) = (
                        c.angle - demi + c.ouverture * t0,
                        c.angle - demi + c.ouverture * t1,
                    );
                    let r0 = (r - e).max(0.0);
                    m.quad(
                        [
                            point(centre, a0, 0.2),
                            point(centre, a1, 0.2),
                            point(centre, a1, r0),
                            point(centre, a0, r0),
                        ],
                        [[0.6, 0.0], [0.6, 0.0], [0.6, 0.9], [0.6, 0.9]],
                        [cf; 4],
                        Forme::Trainee,
                        0.0,
                    );
                }
                for k in 0..SEGMENTS {
                    let (t0, t1) = (k as f32 / SEGMENTS as f32, (k + 1) as f32 / SEGMENTS as f32);
                    let (a0, a1) = (
                        c.angle - demi + c.ouverture * t0,
                        c.angle - demi + c.ouverture * t1,
                    );
                    // Bords de l'arc éteints
                    let bord = |t: f32| (1.0 - (t * 2.0 - 1.0).abs()).sqrt();
                    let (c0, c1) = (
                        couleur(fil, force * bord(t0)),
                        couleur(fil, force * bord(t1)),
                    );
                    m.quad(
                        [
                            point(centre, a0, r - e),
                            point(centre, a1, r - e),
                            point(centre, a1, r + e),
                            point(centre, a0, r + e),
                        ],
                        [[t0, 0.0], [t1, 0.0], [t1, 1.0], [t0, 1.0]],
                        [c0, c1, c1, c0],
                        Forme::Faisceau,
                        0.0,
                    );
                }
            }
        }
    }
}
