//! Armes de corps à corps : balayage en arc, objets en orbite, heurt de bouclier et
//! tourbillon de lames (port de balayages.ts, orbites.ts, heurts.ts, tourbillons.ts ;
//! sans les images, purement visuelles).

use std::f32::consts::TAU;

use bs_commun::maths::{atan2, cos, ecart_angle, sin};
use bs_donnees::armes::Mecanique;

use super::arme::ArmeEquipee;
use super::tours;
use crate::ennemis::Nature;
use crate::partie::Partie;

/// Un ennemi plus proche que ça (blocs) est touché quel que soit l'angle.
const COLLE: f32 = 0.8;
/// Heurt : marge de recherche de cible et portée en plus du renvoi des projectiles (blocs).
const HEURT_MARGE: f32 = 1.2;
const HEURT_PORTEE_RENVOI: f32 = 1.5;
/// Tourbillon : rotation (rad/s), déclenché si un ennemi est à zone × `DECLENCHEMENT`.
const TOURBILLON_ROTATION: f32 = 12.0;
const TOURBILLON_DECLENCHEMENT: f32 = 1.3;

/// Tourbillon en cours (un seul à la fois).
#[derive(Debug, Clone, Copy, Default)]
pub struct Tourbillon {
    /// Position de l'arme dans l'arsenal (ses statistiques sont lues à chaque coup).
    pub arme: Option<usize>,
    temps: f32,
    prochain_coup: f32,
    coups_restants: u32,
}

impl Partie {
    /// Vrai si l'ennemi `e` est dans le cône `angle` ± `demi` devant le joueur, ou collé à lui.
    fn dans_cone(&self, e: usize, angle: f32, demi: f32) -> bool {
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let (dx, dz) = (self.ennemis.pool[e].x - jx, self.ennemis.pool[e].z - jz);
        dx * dx + dz * dz < COLLE * COLLE || ecart_angle(angle, atan2(dx, dz)).abs() <= demi
    }

    pub(super) fn frapper_balayage(&mut self, arme: &ArmeEquipee, angle_arc: f32) -> bool {
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let s = arme.stats;
        let mut tampon = [0u32; 256];
        let n = self.ennemis.chercher(jx, jz, s.zone, &mut tampon);
        let mut cible = None;
        let mut meilleure = f32::INFINITY;
        for &k in &tampon[..n] {
            let e = &self.ennemis.pool[k as usize];
            let d2 = (e.x - jx) * (e.x - jx) + (e.z - jz) * (e.z - jz);
            if d2 < meilleure {
                meilleure = d2;
                cible = Some(k as usize);
            }
        }
        let Some(c) = cible else {
            return false;
        };
        let angle = atan2(self.ennemis.pool[c].x - jx, self.ennemis.pool[c].z - jz);
        for &k in &tampon[..n] {
            let e = k as usize;
            if self.dans_cone(e, angle, angle_arc / 2.0) {
                self.blesser(e, s.degats, jx, jz, s.recul, Some(arme.index), Nature::Arme);
            }
        }
        self.etat.joueur.angle = angle;
        self.ev.geste(None);
        let images =
            matches!(&arme.def.mecanique, Mecanique::Balayage { effet, .. } if !effet.is_empty());
        if !images {
            let avant = s.zone * 0.45;
            self.ev.gerbe(
                "coup",
                jx + sin(angle) * avant,
                1.0,
                jz + cos(angle) * avant,
            );
        }
        true
    }

    /// Fait tourner les objets de l'arme autour du joueur ; chacun blesse ce qu'il traverse,
    /// au plus une fois toutes les `cadence` s par ennemi.
    pub(super) fn maj_orbite(&mut self, arme: &ArmeEquipee, dt: f32) {
        let Some(rayon) = arme
            .def
            .mecanique
            .projectile()
            .and_then(|p| self.d.projectiles.get(p))
            .map(|p| p.rayon)
        else {
            return;
        };
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let s = arme.stats;
        let angle = (self.armes.angles_orbite[arme.index] + s.vitesse * dt) % TAU;
        self.armes.angles_orbite[arme.index] = angle;
        let mut tampon = [0u32; 64];
        for k in 0..tours(s.nombre) {
            let a = angle + k as f32 * TAU / s.nombre;
            let (x, z) = (jx + sin(a) * s.zone, jz + cos(a) * s.zone);
            let n = self.ennemis.chercher(x, z, rayon, &mut tampon);
            for &e in &tampon[..n] {
                self.blesser_avec_delai(
                    e as usize, arme.index, s.cadence, s.degats, jx, jz, s.recul,
                );
            }
        }
    }

    pub(super) fn frapper_heurt(
        &mut self,
        arme: &ArmeEquipee,
        angle_cone: f32,
        renvoi: f32,
    ) -> bool {
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let s = arme.stats;
        let Some(c) = self.ennemis.plus_proche(jx, jz, s.zone + HEURT_MARGE) else {
            return false;
        };
        let angle = atan2(self.ennemis.pool[c].x - jx, self.ennemis.pool[c].z - jz);
        let mut tampon = [0u32; 128];
        let n = self.ennemis.chercher(jx, jz, s.zone, &mut tampon);
        for &k in &tampon[..n] {
            let e = k as usize;
            if !self.dans_cone(e, angle, angle_cone / 2.0) {
                continue;
            }
            self.toucher(e, arme.index, &s, jx, jz, s.recul, 1.0);
            self.interrompre(e);
        }
        if arme.rang as f32 >= renvoi {
            let portee = s.zone + HEURT_PORTEE_RENVOI;
            self.renvoyer_projectiles(
                jx,
                jz,
                angle,
                angle_cone / 2.0,
                portee,
                s.degats,
                Some(arme.index),
            );
        }
        self.etat.joueur.angle = angle;
        self.ev.geste(None);
        let devant = s.zone * 0.5;
        self.ev.gerbe(
            "heurt",
            jx + sin(angle) * devant,
            0.9,
            jz + cos(angle) * devant,
        );
        self.ev.secousse(0.12);
        true
    }

    /// Lance le tourbillon de l'arme `i` de l'arsenal s'il y a un ennemi assez proche.
    pub(super) fn frapper_tourbillon(&mut self, i: usize, intervalle: f32) -> bool {
        let s = self.arsenal.armes[i].stats;
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let mut tampon = [0u32; 128];
        if self
            .ennemis
            .chercher(jx, jz, s.zone * TOURBILLON_DECLENCHEMENT, &mut tampon)
            == 0
        {
            return false;
        }
        self.armes.tourbillon = Tourbillon {
            arme: Some(i),
            temps: 0.0,
            prochain_coup: 0.0,
            coups_restants: 1 + libm::floorf(s.duree / intervalle + 1e-6) as u32,
        };
        true
    }

    pub fn maj_tourbillon(&mut self, dt: f32) {
        let mut t = self.armes.tourbillon;
        let Some(i) = t.arme else {
            return;
        };
        let arme = self.arsenal.armes[i];
        let Mecanique::Tourbillon { intervalle, .. } = arme.def.mecanique else {
            return;
        };
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        t.temps += dt;
        t.prochain_coup -= dt;
        let mut tampon = [0u32; 128];
        while t.prochain_coup <= 0.0 && t.coups_restants > 0 {
            t.prochain_coup += intervalle;
            t.coups_restants -= 1;
            let n = self.ennemis.chercher(jx, jz, arme.stats.zone, &mut tampon);
            for &e in &tampon[..n] {
                self.toucher(
                    e as usize,
                    arme.index,
                    &arme.stats,
                    jx,
                    jz,
                    arme.stats.recul,
                    1.0,
                );
            }
            self.ev.son_hauteur("balayage", 1.1);
            self.etat.joueur.angle = t.temps * TOURBILLON_ROTATION;
            self.ev.geste(None);
        }
        if t.coups_restants == 0 && t.prochain_coup <= 0.0 {
            t.arme = None;
        }
        self.armes.tourbillon = t;
    }
}
