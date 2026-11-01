//! Armes de feu à effet de zone : tornades qui avancent vers les ennemis et météores qui
//! tombent sur un groupe puis laissent du feu au sol (port de tornades.ts, meteores.ts).
//! Les traces de brûlure des tornades sont visuelles mais tirent dans l'aléa : on garde
//! leur compte.

use bs_commun::maths::{atan2, cos, longueur, sin};
use bs_donnees::armes::Mecanique;

use super::tours;
use crate::ennemis::Nature;
use crate::evenements::EffetArme;
use crate::partie::Partie;

const TORNADE_CAPACITE: usize = 6;
const TORNADE_PORTEE: f32 = 10.0;
const TORNADE_RECHERCHE: f32 = 8.0;
/// Écart angulaire entre deux tornades lancées ensemble (rad).
const TORNADE_ECART: f32 = 0.8;
const TRACE_CAPACITE: usize = 160;
const TRACE_ESPACEMENT: f32 = 0.5;
const TRACE_DUREE: f32 = 2.0;

const METEORE_CAPACITE: usize = 18;
/// Durée de la chute d'un météore (s).
pub const METEORE_DUREE: f32 = 0.6;
const METEORE_TIC: f32 = 0.5;
const METEORE_DECALAGE: f32 = 0.3;

#[derive(Debug, Clone, Copy)]
pub struct Tornade {
    /// Position de l'arme dans l'arsenal.
    pub arme: usize,
    pub x: f32,
    pub z: f32,
    pub dir_x: f32,
    pub dir_z: f32,
    pub temps: f32,
    pub prochain_coup: f32,
    pub facteur: f32,
    pub zone: f32,
    pub duree: f32,
    pub depuis_trace: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Meteore {
    pub arme: usize,
    pub x: f32,
    pub z: f32,
    pub t: f32,
    pub facteur: f32,
    pub taille: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Feu {
    pub arme: usize,
    pub x: f32,
    pub z: f32,
    pub temps: f32,
    pub prochain_tic: f32,
    pub facteur: f32,
    pub zone: f32,
    pub duree: f32,
}

#[derive(Debug, Clone)]
pub struct ArmesFeu {
    pub tornades: Vec<Tornade>,
    /// Âge des traces de brûlure (visuelles).
    pub traces: Vec<f32>,
    pub meteores: Vec<Meteore>,
    pub feux: Vec<Feu>,
}

impl Default for ArmesFeu {
    fn default() -> Self {
        Self {
            tornades: Vec::with_capacity(TORNADE_CAPACITE),
            traces: Vec::with_capacity(TRACE_CAPACITE),
            meteores: Vec::with_capacity(METEORE_CAPACITE),
            feux: Vec::with_capacity(METEORE_CAPACITE),
        }
    }
}

impl Partie {
    pub(super) fn frapper_tornade(&mut self, i: usize) -> bool {
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let Some(c) = self.ennemis.plus_proche(jx, jz, TORNADE_PORTEE) else {
            return false;
        };
        let angle = atan2(self.ennemis.pool[c].x - jx, self.ennemis.pool[c].z - jz);
        let s = self.arsenal.armes[i].stats;
        for k in 0..tours(s.nombre) {
            if self.armes.feu.tornades.len() >= TORNADE_CAPACITE {
                break;
            }
            let a = angle + (k as f32 - (s.nombre - 1.0) / 2.0) * TORNADE_ECART;
            let (dir_x, dir_z) = (sin(a), cos(a));
            // Graine de la forme (visuelle)
            self.alea.tirer();
            self.armes.feu.tornades.push(Tornade {
                arme: i,
                x: jx + dir_x,
                z: jz + dir_z,
                dir_x,
                dir_z,
                temps: 0.0,
                prochain_coup: 0.0,
                facteur: self.talent_facteur_declenchement(),
                zone: 1.0,
                duree: s.duree,
                // Première brûlure dès l'apparition
                depuis_trace: f32::INFINITY,
            });
        }
        true
    }

    pub fn maj_tornades(&mut self, dt: f32) {
        let mut tampon = [0u32; 128];
        let mut i = self.armes.feu.tornades.len();
        while i > 0 {
            i -= 1;
            let mut t = self.armes.feu.tornades[i];
            let arme = self.arsenal.armes[t.arme];
            t.temps += dt;
            let Mecanique::Tornade {
                vitesse,
                intervalle,
                ..
            } = arme.def.mecanique
            else {
                self.armes.feu.tornades.swap_remove(i);
                continue;
            };
            if t.temps >= t.duree {
                self.ev.gerbe("finTornade", t.x, 1.0, t.z);
                self.armes.feu.tornades.swap_remove(i);
                continue;
            }
            if let Some(c) = self.ennemis.plus_proche(t.x, t.z, TORNADE_RECHERCHE) {
                let (dx, dz) = (self.ennemis.pool[c].x - t.x, self.ennemis.pool[c].z - t.z);
                let d = longueur(dx, dz);
                if d > 0.3 {
                    t.dir_x = dx / d;
                    t.dir_z = dz / d;
                }
            }
            let nx = t.x + t.dir_x * vitesse * dt;
            let nz = t.z + t.dir_z * vitesse * dt;
            if !self.carte.bloque_marche(nx, nz) {
                t.depuis_trace += longueur(nx - t.x, nz - t.z);
                t.x = nx;
                t.z = nz;
            }
            if t.depuis_trace >= TRACE_ESPACEMENT {
                t.depuis_trace = 0.0;
                if self.armes.feu.traces.len() < TRACE_CAPACITE {
                    self.armes.feu.traces.push(0.0);
                    self.ev.effet(t.arme, EffetArme::Brulure, t.x, t.z);
                    // Position, taille et rotation de la trace
                    for _ in 0..4 {
                        self.alea.tirer();
                    }
                }
            }
            t.prochain_coup -= dt;
            if t.prochain_coup <= 0.0 {
                t.prochain_coup += intervalle;
                let s = arme.stats;
                let n = self
                    .ennemis
                    .chercher(t.x, t.z, s.zone * t.zone, &mut tampon);
                for &e in &tampon[..n] {
                    self.toucher(e as usize, arme.index, &s, t.x, t.z, s.recul, t.facteur);
                }
            }
            self.armes.feu.tornades[i] = t;
        }
        let traces = &mut self.armes.feu.traces;
        let mut i = traces.len();
        while i > 0 {
            i -= 1;
            traces[i] += dt;
            if traces[i] >= TRACE_DUREE {
                traces.swap_remove(i);
            }
        }
    }

    pub(super) fn frapper_meteore(&mut self, i: usize) -> bool {
        let s = self.arsenal.armes[i].stats;
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let facteur = self.talent_facteur_declenchement();
        let mut lance = false;
        for k in 0..tours(s.nombre) {
            let Some((x, z)) = self.plus_gros_groupe(jx, jz, s.portee, s.zone) else {
                break;
            };
            if self.armes.feu.meteores.len() < METEORE_CAPACITE {
                self.armes.feu.meteores.push(Meteore {
                    arme: i,
                    x,
                    z,
                    t: -(k as f32) * METEORE_DECALAGE,
                    facteur,
                    taille: 1.0,
                });
                lance = true;
            }
        }
        lance
    }

    pub fn maj_meteores(&mut self, dt: f32) {
        let mut tampon = [0u32; 256];
        let mut i = self.armes.feu.meteores.len();
        while i > 0 {
            i -= 1;
            let mut m = self.armes.feu.meteores[i];
            m.t += dt;
            if m.t < METEORE_DUREE {
                self.armes.feu.meteores[i] = m;
                continue;
            }
            // Impact
            let arme = self.arsenal.armes[m.arme];
            let s = arme.stats;
            let n = self
                .ennemis
                .chercher(m.x, m.z, s.zone * m.taille, &mut tampon);
            for &e in &tampon[..n] {
                self.toucher(e as usize, arme.index, &s, m.x, m.z, s.recul, m.facteur);
            }
            self.ev.gerbe("impactMeteore", m.x, 0.4, m.z);
            let texture = match &arme.def.mecanique {
                Mecanique::Meteore { texture, .. } => texture.as_str(),
                _ => "block/magma",
            };
            self.ev.gerbe_icone("eboulis", texture, m.x, 0.4, m.z);
            self.ev.son_hauteur("explosion", 0.8);
            self.ev.secousse(0.3);
            if self.armes.feu.feux.len() < METEORE_CAPACITE {
                // Graine des flammes (visuelle)
                self.alea.tirer();
                self.armes.feu.feux.push(Feu {
                    arme: m.arme,
                    x: m.x,
                    z: m.z,
                    temps: 0.0,
                    prochain_tic: METEORE_TIC,
                    facteur: m.facteur,
                    zone: m.taille,
                    duree: s.duree,
                });
            }
            self.armes.feu.meteores.swap_remove(i);
        }

        let mut i = self.armes.feu.feux.len();
        while i > 0 {
            i -= 1;
            let mut f = self.armes.feu.feux[i];
            let arme = self.arsenal.armes[f.arme];
            f.temps += dt;
            let Mecanique::Meteore { degats_sol, .. } = arme.def.mecanique else {
                self.armes.feu.feux.swap_remove(i);
                continue;
            };
            if f.temps >= f.duree {
                self.armes.feu.feux.swap_remove(i);
                continue;
            }
            f.prochain_tic -= dt;
            if f.prochain_tic <= 0.0 {
                f.prochain_tic += METEORE_TIC;
                let s = arme.stats;
                let degats = s.degats * degats_sol * f.facteur;
                let n = self
                    .ennemis
                    .chercher(f.x, f.z, s.zone * f.zone, &mut tampon);
                for &e in &tampon[..n] {
                    let e = e as usize;
                    self.blesser(e, degats, f.x, f.z, 0.0, Some(arme.index), Nature::Brulure);
                }
            }
            self.armes.feu.feux[i] = f;
        }
    }
}
