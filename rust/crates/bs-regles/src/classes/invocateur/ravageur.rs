//! Évocateur : Évocation supérieure (un ravageur charge la horde) et Crocs de l'évocateur
//! (une rangée de crocs jaillit vers le plus gros groupe). Dégâts crédités aux loups.

use bs_commun::maths::{atan2, cos, longueur, sin};

use super::invocateur;
use crate::partie::Partie;

const EVOCATION_DUREE: f32 = 8.0;
const EVOCATION_DEGATS: f32 = 40.0;
const EVOCATION_RECUL: f32 = 18.0;
const EVOCATION_VITESSE: f32 = 8.0;
const EVOCATION_RAYON: f32 = 1.8;
const EVOCATION_IMMUNITE: f32 = 0.8;
const EVOCATION_CAP: f32 = 1.2;
/// Cap après un mur, recul d'apparition derrière le joueur, période des pas (s).
const CAP_MUR: f32 = 0.3;
const APPARITION: f32 = 2.0;
const PAS: f32 = 0.35;
/// Recherche du groupe visé : portée, rayon, distance minimale.
const GROUPE_PORTEE: f32 = 14.0;
const GROUPE_RAYON: f32 = 3.0;
const GROUPE_MIN: f32 = 0.5;
pub(super) const CROCS_PERIODE: f32 = 6.0;
const CROCS_NOMBRE: u32 = 8;
pub(super) const CROCS_CAPACITE: usize = CROCS_NOMBRE as usize * 3;
const CROCS_DEPART: f32 = 1.5;
const CROCS_ECART: f32 = 1.2;
const CROCS_DECALAGE: f32 = 0.06;
const CROCS_JITTER: f32 = 0.6;
/// Durée de vie d'un croc et instant où il claque (s).
pub const CROCS_VIE: f32 = 1.0;
pub const CROCS_CLAQUE: f32 = 0.4;
const CROCS_DEGATS: f32 = 20.0;
const CROCS_RAYON: f32 = 1.0;
const CROCS_RECUL: f32 = 2.0;
const CROCS_PORTEE: f32 = 12.0;

#[derive(Debug, Clone, Copy, Default)]
pub struct Ravageur {
    pub actif: bool,
    pub x: f32,
    pub z: f32,
    pub dir_x: f32,
    pub dir_z: f32,
    pub temps: f32,
    cap: f32,
    pub vitesse_sol: f32,
    pas: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Croc {
    pub x: f32,
    pub z: f32,
    pub angle: f32,
    /// s depuis l'apparition (négatif : attend son tour).
    pub t: f32,
    frappe: bool,
}

impl Partie {
    pub(super) fn invoquer_ravageur(&mut self) {
        let j = &self.etat.joueur;
        let (x, z) = (
            j.x - sin(j.angle) * APPARITION,
            j.z - cos(j.angle) * APPARITION,
        );
        let v = invocateur!(self);
        let r = &mut v.ravageur;
        r.actif = true;
        r.temps = 0.0;
        r.cap = 0.0;
        r.x = x;
        r.z = z;
        self.viser_groupe();
        self.ev.gerbe("invocation", x, 1.0, z);
        self.ev.son("ravageur");
    }

    fn viser_groupe(&mut self) {
        let (x, z) = invocateur!(self).ravageur.pos();
        let Some((gx, gz)) = self.plus_gros_groupe(x, z, GROUPE_PORTEE, GROUPE_RAYON) else {
            return;
        };
        let d = longueur(gx - x, gz - z);
        if d < GROUPE_MIN {
            return;
        }
        let r = &mut invocateur!(self).ravageur;
        r.dir_x = (gx - x) / d;
        r.dir_z = (gz - z) / d;
    }

    pub(super) fn maj_ravageur(&mut self, dt: f32) {
        let r = &mut invocateur!(self).ravageur;
        if !r.actif {
            return;
        }
        r.temps += dt;
        if r.temps >= EVOCATION_DUREE {
            r.actif = false;
            let (x, z) = r.pos();
            self.ev.gerbe("invocation", x, 1.0, z);
            return;
        }
        // Charge en ligne droite, change de cap vers le plus gros groupe de temps en temps ou
        // contre un mur
        r.cap -= dt;
        if r.cap <= 0.0 {
            r.cap = EVOCATION_CAP;
            self.viser_groupe();
        }
        let r = invocateur!(self).ravageur;
        let nx = r.x + r.dir_x * EVOCATION_VITESSE * dt;
        let nz = r.z + r.dir_z * EVOCATION_VITESSE * dt;
        let bloque = self.carte.bloque_marche(nx, nz);
        let r = &mut invocateur!(self).ravageur;
        if bloque {
            r.dir_x = -r.dir_x;
            r.dir_z = -r.dir_z;
            r.cap = CAP_MUR;
        } else {
            r.x = nx;
            r.z = nz;
        }
        r.vitesse_sol = EVOCATION_VITESSE;
        r.pas -= dt;
        let pas = r.pas <= 0.0;
        if pas {
            r.pas = PAS;
        }
        let r = *r;
        if pas {
            self.ev.son("ravageur_pas");
            self.ev.gerbe("repousse", r.x, 0.2, r.z);
        }
        // Renverse tout sur son passage (une fois par ennemi et par instant)
        let Some(credit) = invocateur!(self).loups else {
            return;
        };
        let mut tampon = [0u32; 128];
        let n = self
            .ennemis
            .chercher(r.x, r.z, EVOCATION_RAYON, &mut tampon);
        for &e in &tampon[..n] {
            let e = e as usize;
            if self.immunise(e, credit, EVOCATION_IMMUNITE) {
                continue;
            }
            // Projeté sur le côté de la trajectoire
            let (ex, ez) = (self.ennemis.pool[e].x, self.ennemis.pool[e].z);
            let cote = if (ex - r.x) * r.dir_z - (ez - r.z) * r.dir_x >= 0.0 {
                1.0
            } else {
                -1.0
            };
            let sx = ex - r.dir_z * cote - r.dir_x;
            let sz = ez + r.dir_x * cote - r.dir_z;
            let (degats, recul) = (EVOCATION_DEGATS, EVOCATION_RECUL);
            let nature = crate::ennemis::Nature::Arme;
            self.blesser_reaction(e, degats, sx, sz, recul, Some(credit), nature);
        }
    }

    pub(super) fn maj_crocs(&mut self, dt: f32) {
        let v = invocateur!(self);
        if v.r.crocs == 0 {
            return;
        }
        let credit = v.loups;
        v.minuterie_crocs -= dt;
        if v.minuterie_crocs <= 0.0 {
            v.minuterie_crocs = CROCS_PERIODE;
            // Une rangée vers le groupe le plus proche
            let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
            if let Some((gx, gz)) = self.plus_gros_groupe(jx, jz, CROCS_PORTEE, GROUPE_RAYON) {
                let angle = atan2(gx - jx, gz - jz);
                for k in 0..CROCS_NOMBRE {
                    if invocateur!(self).crocs.len() >= CROCS_CAPACITE {
                        break;
                    }
                    let d = CROCS_DEPART + k as f32 * CROCS_ECART;
                    let jitter = (self.alea.tirer() as f32 - 0.5) * CROCS_JITTER;
                    invocateur!(self).crocs.push(Croc {
                        x: jx + sin(angle) * d,
                        z: jz + cos(angle) * d,
                        angle: angle + jitter,
                        t: -(k as f32) * CROCS_DECALAGE,
                        frappe: false,
                    });
                }
                self.ev.son("incantation");
            }
        }
        let mut i = invocateur!(self).crocs.len();
        while i > 0 {
            i -= 1;
            let c = &mut invocateur!(self).crocs[i];
            c.t += dt;
            if c.t < 0.0 {
                continue;
            }
            let claque = !c.frappe && c.t >= CROCS_CLAQUE;
            if claque {
                c.frappe = true;
            }
            let c = *c;
            if claque {
                self.blesser_zone_reaction(
                    (c.x, c.z, CROCS_RAYON),
                    CROCS_DEGATS,
                    credit,
                    CROCS_RECUL,
                );
                self.ev.gerbe("critique", c.x, 0.4, c.z);
                self.ev.son("crocs");
            }
            if c.t >= CROCS_VIE {
                invocateur!(self).crocs.swap_remove(i);
            }
        }
    }
}

impl Ravageur {
    fn pos(&self) -> (f32, f32) {
        (self.x, self.z)
    }
}
