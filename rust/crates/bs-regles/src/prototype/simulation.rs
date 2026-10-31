//! La simulation complète du prototype : le joueur, la foule de zombies et l'arme.
//! `avancer` fait un pas fixe à partir de la commande du joueur ; `empreinte` résume tout
//! l'état en un nombre (tests de déterminisme, contrôle de la coop).

use bs_commun::PAS_SIMULATION;

use super::armes::{ParametresArme, Projectiles};
use super::ennemis::{Ennemis, ParametresEnnemis};

/// Ce que veut le joueur pendant un pas : une direction (longueur ≤ 1).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Commande {
    pub dx: f32,
    pub dz: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Joueur {
    pub x: f32,
    pub z: f32,
    pub x_prec: f32,
    pub z_prec: f32,
    pub angle: f32,
    pub pv: f32,
}

/// Réglages d'une partie du prototype.
#[derive(Clone, Debug)]
pub struct Parametres {
    /// Taille de l'arène (blocs), centrée sur l'origine.
    pub largeur: f32,
    pub profondeur: f32,
    pub vitesse_joueur: f32,
    pub pv_joueur: f32,
    /// Prototype : le joueur ne meurt pas (mesure de performance sur la durée).
    pub invincible: bool,
    pub ennemis: ParametresEnnemis,
    pub arme: ParametresArme,
}

impl Default for Parametres {
    fn default() -> Self {
        Self {
            largeur: 80.0,
            profondeur: 80.0,
            vitesse_joueur: 4.5,
            pv_joueur: 100.0,
            invincible: true,
            ennemis: ParametresEnnemis::default(),
            arme: ParametresArme::default(),
        }
    }
}

pub struct Simulation {
    pub joueur: Joueur,
    pub ennemis: Ennemis,
    pub projectiles: Projectiles,
    pub pas: u64,
    pub tues: u32,
    pub degats_subis: f32,
    demi_l: f32,
    demi_p: f32,
    vitesse_joueur: f32,
    invincible: bool,
}

impl Simulation {
    pub fn nouvelle(graine: u64, p: Parametres) -> Self {
        let (demi_l, demi_p) = (p.largeur / 2.0, p.profondeur / 2.0);
        let mut ennemis = Ennemis::nouveaux(p.ennemis, p.largeur, p.profondeur, graine);
        for i in 0..ennemis.x.len() {
            ennemis.apparaitre(i, 0.0, 0.0, demi_l, demi_p);
        }
        ennemis
            .grille
            .reconstruire(&ennemis.x, &ennemis.z, &ennemis.vivant);
        Self {
            joueur: Joueur {
                pv: p.pv_joueur,
                ..Joueur::default()
            },
            ennemis,
            projectiles: Projectiles::nouveaux(p.arme),
            pas: 0,
            tues: 0,
            degats_subis: 0.0,
            demi_l,
            demi_p,
            vitesse_joueur: p.vitesse_joueur,
            invincible: p.invincible,
        }
    }

    /// Taille de l'arène (blocs).
    pub fn largeur(&self) -> f32 {
        self.demi_l * 2.0
    }

    pub fn profondeur(&self) -> f32 {
        self.demi_p * 2.0
    }

    /// Un pas fixe de simulation.
    pub fn avancer(&mut self, commande: Commande) {
        let dt = PAS_SIMULATION;
        let j = &mut self.joueur;
        j.x_prec = j.x;
        j.z_prec = j.z;
        let n = (commande.dx * commande.dx + commande.dz * commande.dz).sqrt();
        if n > 1e-3 {
            let k = n.max(1.0);
            j.x =
                (j.x + commande.dx / k * self.vitesse_joueur * dt).clamp(-self.demi_l, self.demi_l);
            j.z =
                (j.z + commande.dz / k * self.vitesse_joueur * dt).clamp(-self.demi_p, self.demi_p);
            j.angle = libm::atan2f(commande.dx, commande.dz);
        }
        let (jx, jz) = (j.x, j.z);
        let degats = self.ennemis.avancer(dt, jx, jz, self.demi_l, self.demi_p);
        self.degats_subis += degats;
        if !self.invincible {
            self.joueur.pv -= degats;
        }
        self.tues += self.projectiles.avancer(dt, jx, jz, &mut self.ennemis);
        self.pas += 1;
    }

    /// Empreinte FNV-1a de tout l'état (positions au bit près).
    pub fn empreinte(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut melanger = |v: u32| {
            for o in v.to_le_bytes() {
                h ^= o as u64;
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
        };
        let e = &self.ennemis;
        for v in [
            self.joueur.x,
            self.joueur.z,
            self.joueur.pv,
            self.degats_subis,
        ] {
            melanger(v.to_bits());
        }
        for i in 0..e.x.len() {
            melanger(e.x[i].to_bits());
            melanger(e.z[i].to_bits());
            melanger(e.pv[i].to_bits());
            melanger(e.vivant[i] as u32);
        }
        let p = &self.projectiles;
        for k in 0..p.nombre {
            melanger(p.x[k].to_bits());
            melanger(p.z[k].to_bits());
        }
        melanger(self.tues);
        melanger(p.nombre as u32);
        h
    }
}
