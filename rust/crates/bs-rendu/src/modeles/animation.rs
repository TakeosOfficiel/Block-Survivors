//! Animation des modèles cubiques (src/modeles/animation.ts) : balancement des membres
//! à la marche, bras au repos, tournoiement, geste d'attaque, flottement, chute à la
//! mort. Un petit état par entité ; les rotations sont écrites dans un tampon.

use std::f32::consts::{PI, TAU};

use bevy::math::Vec3;
use bs_commun::maths::facteur_lissage;

use super::compilation::ModeleCompile;

#[derive(Debug, Clone, Copy, Default)]
pub struct EtatAnimation {
    /// Cycle de marche (rad).
    pub phase: f32,
    /// 0 : immobile, 1 : pleine allure.
    pub intensite: f32,
    /// Horloge du balancement au repos (s).
    pub temps: f32,
    /// s restantes du geste d'attaque (0 : aucun).
    pub attaque: f32,
    /// Dernier pas où l'entité existait (tri des états d'entités disparues).
    pub vu: u32,
}

impl EtatAnimation {
    /// État neuf ; `decalage` (0 à 1) désynchronise le balancement des entités.
    pub fn nouveau(decalage: f32) -> Self {
        Self {
            temps: decalage * 10.0,
            ..Self::default()
        }
    }

    /// Lance le geste d'attaque du modèle (sans effet s'il n'en a pas).
    pub fn declencher_attaque(&mut self, m: &ModeleCompile) {
        self.attaque = m.def.attaque.as_ref().map_or(0.0, |a| a.duree);
    }

    /// Avance l'animation : `dt` en temps de jeu (s), `vitesse_sol` en blocs/s.
    pub fn avancer(&mut self, m: &ModeleCompile, dt: f32, vitesse_sol: f32) {
        self.temps += dt;
        if self.attaque > 0.0 {
            self.attaque = (self.attaque - dt).max(0.0);
        }
        let Some(marche) = &m.def.marche else {
            return;
        };
        self.phase = (self.phase + vitesse_sol * dt * marche.frequence) % TAU;
        let cible = (vitesse_sol / marche.vitesse_reference).min(1.0);
        self.intensite += (cible - self.intensite) * facteur_lissage(marche.lissage, dt);
    }
}

/// Écrit dans `sortie` les rotations (x, y, z) de chaque partie : base + animation.
pub fn calculer_rotations(m: &ModeleCompile, e: &EtatAnimation, sortie: &mut [Vec3]) {
    sortie.copy_from_slice(&m.pose_base);
    for b in &m.marche {
        let angle = e.phase * b.frequence + b.dephasage;
        let onde = if b.absolu {
            angle.sin().abs()
        } else {
            angle.cos()
        };
        sortie[b.partie][b.axe] += onde * b.amplitude * e.intensite;
    }
    if let Some(r) = &m.def.repos {
        let ecart = (e.temps * r.frequence_z).cos() * r.amplitude + r.amplitude;
        let avant = (e.temps * r.frequence_x).sin() * r.amplitude;
        for &(p, signe) in &m.repos {
            sortie[p].x += signe * avant;
            // Écarte le bras du corps
            sortie[p].z -= signe * ecart;
        }
    }
    for &(p, vitesse) in &m.tournoiement {
        sortie[p].y += (e.temps * vitesse) % TAU;
    }
    if let Some(a) = &m.def.attaque
        && e.attaque > 0.0
    {
        let levee = ((1.0 - e.attaque / a.duree) * PI).sin();
        for &(p, amplitude) in &m.attaque {
            sortie[p].x -= levee * amplitude;
        }
    }
}

/// Hauteur de flottement ou de bond (blocs, avant mise à l'échelle).
pub fn hauteur_flottement(m: &ModeleCompile, e: &EtatAnimation) -> f32 {
    let Some(f) = &m.def.flottement else {
        return 0.0;
    };
    let onde = (e.temps * f.frequence).sin();
    let onde = if f.bonds { onde.abs() } else { onde };
    f.hauteur + onde * f.amplitude
}

/// Angle de chute sur le côté après la mort (rapide puis amortie, comme en jeu).
pub fn angle_chute(temps: f32, duree: f32) -> f32 {
    ((temps / duree) * 1.6).sqrt().min(1.0) * (PI / 2.0)
}
