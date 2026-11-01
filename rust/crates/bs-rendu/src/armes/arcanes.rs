//! Orbes arcaniques et invocations. Les orbes : vrai bloc d'améthyste qui tournoie dans
//! un halo, cœur scintillant, sillage qui suit leur trajet récent. Les alliés invoqués
//! arrivent dans une colonne de lumière, sur un cercle runique qui s'ouvre, et partent
//! dans une bouffée de lumière.

use std::f32::consts::TAU;

use bevy::prelude::*;
use bs_regles::combat::traque::Orbe;

use super::lueur::{Forme, MaillageLueur};
use super::outils::{au_sol, hachage, rgba, trait_au_sol};
use super::solides::{Piece, Solides};
use crate::modeles::rendu::Composants;

/// Sillages : capacité, points par sillage, écart entre deux points (s).
pub const SILLAGES: usize = 48;
const POINTS: usize = 9;
const ECHANTILLON: f32 = 0.025;
/// Hauteur de vol des orbes, côté de leur bloc.
const HAUTEUR: f32 = 1.0;
const COTE: f32 = 0.42;
/// Apparitions : capacité, durées d'une arrivée et d'un départ (s).
pub const APPARITIONS: usize = 32;
const ARRIVEE: f32 = 0.75;
const DEPART: f32 = 0.45;
/// Alliés suivis (au-delà, pas d'apparition).
const CONNUS: usize = 64;

const AMETHYSTE: [f32; 3] = [0.7, 0.35, 1.0];
const COEUR: [f32; 3] = [1.0, 0.85, 1.0];
const ESPRIT: [f32; 3] = [0.55, 0.85, 1.0];
const RUNES: [f32; 3] = [0.75, 0.9, 1.0];

/// Trajet récent d'une orbe (le premier point est sa position actuelle).
struct Sillage {
    cle: u32,
    points: [Vec3; POINTS],
    n: usize,
    depuis: f32,
    vu: bool,
}

/// Arrivée ou départ d'un allié.
#[derive(Clone, Copy)]
pub struct Apparition {
    pub x: f32,
    pub z: f32,
    pub age: f32,
    pub depart: bool,
}

#[derive(Default)]
pub struct Arcanes {
    sillages: Vec<Sillage>,
    connus: Vec<(u32, Vec2, bool)>,
    pub apparitions: Vec<Apparition>,
}

/// Identifiant d'une orbe d'une image à l'autre (son angle de départ est unique).
fn cle(o: &Orbe) -> u32 {
    o.angle.to_bits() ^ (o.arme as u32).wrapping_mul(2_654_435_761)
}

/// Position affichée d'une orbe (elle flotte).
fn position(o: &Orbe, temps: f32) -> Vec3 {
    let flotte = (temps * 3.0 + o.angle * 5.0).sin() * 0.12;
    Vec3::new(o.x, HAUTEUR + flotte, o.z)
}

impl Arcanes {
    pub fn nouvelles() -> Self {
        Self {
            sillages: Vec::with_capacity(SILLAGES),
            connus: Vec::with_capacity(CONNUS),
            apparitions: Vec::with_capacity(APPARITIONS),
        }
    }

    /// Une image de plus : sillages des orbes, arrivées et départs des alliés.
    pub fn avancer<'a>(
        &mut self,
        orbes: impl Iterator<Item = &'a Orbe>,
        allies: impl Iterator<Item = (u32, Vec2)>,
        temps: f32,
        dt: f32,
    ) {
        for s in &mut self.sillages {
            s.vu = false;
            s.depuis += dt;
        }
        for o in orbes {
            let (c, p) = (cle(o), position(o, temps));
            let s = match self.sillages.iter().position(|s| s.cle == c) {
                Some(i) => &mut self.sillages[i],
                None if self.sillages.len() < SILLAGES => {
                    self.sillages.push(Sillage {
                        cle: c,
                        points: [p; POINTS],
                        n: 1,
                        depuis: 0.0,
                        vu: false,
                    });
                    let n = self.sillages.len();
                    &mut self.sillages[n - 1]
                }
                None => continue,
            };
            if s.depuis >= ECHANTILLON {
                s.depuis = 0.0;
                s.points.copy_within(0..POINTS - 1, 1);
                s.n = (s.n + 1).min(POINTS);
            }
            s.points[0] = p;
            s.vu = true;
        }
        self.sillages.retain(|s| s.vu);

        for a in &mut self.apparitions {
            a.age += dt;
        }
        self.apparitions
            .retain(|a| a.age < if a.depart { DEPART } else { ARRIVEE });
        for c in &mut self.connus {
            c.2 = false;
        }
        for (id, p) in allies {
            if let Some(c) = self.connus.iter_mut().find(|c| c.0 == id) {
                c.1 = p;
                c.2 = true;
            } else if self.connus.len() < CONNUS {
                self.connus.push((id, p, true));
                self.apparaitre(p, false);
            }
        }
        let mut i = self.connus.len();
        while i > 0 {
            i -= 1;
            if !self.connus[i].2 {
                let p = self.connus[i].1;
                self.connus.swap_remove(i);
                self.apparaitre(p, true);
            }
        }
    }

    pub fn apparaitre(&mut self, p: Vec2, depart: bool) {
        if self.apparitions.len() < APPARITIONS {
            self.apparitions.push(Apparition {
                x: p.x,
                z: p.y,
                age: 0.0,
                depart,
            });
        }
    }

    /// Lueurs des orbes (halo, cœur, sillage).
    #[allow(clippy::too_many_arguments)]
    pub fn dessiner_orbes<'a>(
        &self,
        m: &mut MaillageLueur,
        orbes: impl Iterator<Item = &'a Orbe>,
        temps: f32,
        oeil: Vec3,
        droite: Vec3,
        haut: Vec3,
    ) {
        for o in orbes {
            let p = position(o, temps);
            let vif = if o.lance { 1.4 } else { 1.0 };
            let pulse = 0.85 + 0.15 * (temps * 8.0 + o.angle * 3.0).sin();
            m.carre(
                p,
                droite * 1.0,
                haut * 1.0,
                rgba(AMETHYSTE, 1.7 * vif * pulse),
                Forme::Disque,
                0.0,
            );
            m.carre(
                p,
                droite * 0.4,
                haut * 0.4,
                rgba(COEUR, 0.8 * vif),
                Forme::Etincelle,
                0.0,
            );
            au_sol(
                m,
                o.x,
                o.z,
                0.6,
                rgba(AMETHYSTE, 0.6 * vif),
                Forme::Disque,
                0.0,
            );
            let c = cle(o);
            let Some(s) = self.sillages.iter().find(|s| s.cle == c) else {
                continue;
            };
            for i in 1..s.n {
                let k0 = (i - 1) as f32 / (POINTS - 1) as f32;
                let k1 = i as f32 / (POINTS - 1) as f32;
                let (a, b) = (s.points[i - 1], s.points[i]);
                let (ca, cb) = (
                    rgba(AMETHYSTE, vif * (1.0 - k0)),
                    rgba(AMETHYSTE, vif * (1.0 - k1)),
                );
                m.faisceau(oeil, a, b, 0.32 * (1.0 - k0), 0.32 * (1.0 - k1), ca, cb);
            }
        }
    }

    /// Lueurs des arrivées et départs d'alliés.
    pub fn dessiner_apparitions(
        &self,
        m: &mut MaillageLueur,
        figees: &[Apparition],
        oeil: Vec3,
        droite: Vec3,
        haut: Vec3,
    ) {
        for a in self.apparitions.iter().chain(figees) {
            if a.depart {
                let k = a.age / DEPART;
                let p = Vec3::new(a.x, 0.9, a.z);
                let r = 0.8 + 0.8 * k;
                m.carre(
                    p,
                    droite * r,
                    haut * r,
                    rgba(ESPRIT, 0.7 * (1.0 - k)),
                    Forme::Disque,
                    0.0,
                );
                au_sol(
                    m,
                    a.x,
                    a.z,
                    0.6 + 1.2 * k,
                    rgba(ESPRIT, 1.0 - k),
                    Forme::Anneau,
                    0.1,
                );
                continue;
            }
            let k = a.age / ARRIVEE;
            let e = 1.0 - (1.0 - k).powi(3);
            let pied = Vec3::new(a.x, 0.0, a.z);
            // Colonne de lumière
            let h = 4.5 * (0.5 + 0.5 * e);
            let colonne = 0.75 * (1.0 - k);
            let (bas, fin) = (rgba(ESPRIT, 1.5 * (1.0 - k)), rgba(ESPRIT, 0.0));
            m.faisceau(
                oeil,
                pied,
                pied + Vec3::Y * h,
                colonne,
                colonne * 0.4,
                bas,
                fin,
            );
            // Cercle runique : anneau et étoile à six branches qui tourne
            let r = 0.5 + 1.1 * e;
            let f = 1.0 - k * k;
            au_sol(m, a.x, a.z, r, rgba(RUNES, 1.3 * f), Forme::Anneau, 0.06);
            au_sol(
                m,
                a.x,
                a.z,
                r * 0.7,
                rgba(RUNES, 0.8 * f),
                Forme::Anneau,
                0.08,
            );
            let centre = Vec2::new(a.x, a.z);
            let tour = a.age * 2.0;
            let sommet = |i: u32| {
                let t = tour + i as f32 * TAU / 6.0;
                centre + Vec2::new(t.sin(), t.cos()) * r * 0.92
            };
            for i in 0..6 {
                trait_au_sol(
                    m,
                    sommet(i),
                    sommet(i + 2),
                    0.05,
                    0.05,
                    rgba(RUNES, 0.9 * f),
                );
            }
            // Étincelles qui montent en spirale
            for i in 0..8u32 {
                let t = i as f32 * TAU / 8.0 + a.age * 3.0;
                let d = 0.6 + 0.4 * hachage(a.x, a.z, i);
                let y = (k * 3.0 + hachage(a.z, a.x, i)) % 3.0;
                let p = Vec3::new(a.x + t.sin() * d, y, a.z + t.cos() * d);
                m.carre(
                    p,
                    droite * 0.25,
                    haut * 0.25,
                    rgba(COEUR, 1.4 * (1.0 - k)),
                    Forme::Etincelle,
                    0.0,
                );
            }
        }
    }
}

/// Blocs d'améthyste des orbes.
pub fn placer_orbes<'a>(
    s: &mut Solides,
    commandes: &mut Commands,
    q: &mut Composants,
    orbes: impl Iterator<Item = (&'a Orbe, &'a str)>,
    temps: f32,
) {
    for (o, texture) in orbes {
        let tour = temps * (2.0 + hachage(o.angle, 1.0, 0) * 2.0);
        let rotation = Quat::from_rotation_y(tour) * Quat::from_rotation_x(0.6);
        let tf = Transform::from_translation(position(o, temps))
            .with_rotation(rotation)
            .with_scale(Vec3::splat(COTE));
        s.poser(commandes, q, texture, Piece::Cube, tf);
    }
}
