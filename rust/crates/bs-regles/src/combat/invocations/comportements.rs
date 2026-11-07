//! Comportements des alliés invoqués : combat au contact, tir, soutien du joueur, aura
//! des totems, déplacement et séparation (port de src/combat/invocations.ts).

use bs_commun::maths::{atan2, cos, ecart_angle, facteur_lissage, longueur, sin};
use bs_donnees::armes::Mecanique;

use super::super::projectiles::{ModeProjectile, Origine};
use super::super::tours;
use crate::ennemis::Nature;
use crate::partie::Partie;

const DISTANCE_SUIVI: f32 = 3.0;
const DELAI_SOIN: f64 = 8.0;
const DEGATS_BRULURE: f32 = 3.0;
const SEPARATION_MARGE: f32 = 0.1;
const SEPARATION_RAYON_JOUEUR: f32 = 0.45;

impl Partie {
    pub(super) fn separer_allies(&mut self) {
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let pool = &mut self.armes.invocations.pool;
        let n = pool.len();
        for i in 0..n {
            let mut a = pool[i];
            if a.arme.is_none() || a.pv <= 0.0 {
                continue;
            }
            let vol_a = a.def.vol.is_some();
            let rayon_a = a.def.rayon * a.echelle;
            for (k, b) in pool[i + 1..].iter_mut().enumerate() {
                let q = i + 1 + k;
                if b.arme.is_none() || b.pv <= 0.0 || b.def.vol.is_some() != vol_a {
                    continue;
                }
                let (mobile_a, mobile_b) = (a.vitesse > 0.0, b.vitesse > 0.0);
                if !mobile_a && !mobile_b {
                    continue;
                }
                let ecart = rayon_a + b.def.rayon * b.echelle + SEPARATION_MARGE;
                let (mut dx, mut dz) = (b.x - a.x, b.z - a.z);
                let d = longueur(dx, dz);
                if d >= ecart {
                    continue;
                }
                if d < 1e-4 {
                    dx = cos(q as f32 * 2.4);
                    dz = sin(q as f32 * 2.4);
                } else {
                    dx /= d;
                    dz /= d;
                }
                let poussee = ecart - d;
                let part_a = match (mobile_a, mobile_b) {
                    (false, _) => 0.0,
                    (true, true) => 0.5,
                    (true, false) => 1.0,
                };
                let part_b = 1.0 - part_a;
                a.x -= dx * poussee * part_a;
                a.z -= dz * poussee * part_a;
                b.x += dx * poussee * part_b;
                b.z += dz * poussee * part_b;
            }
            if !vol_a && a.vitesse > 0.0 {
                let ecart = rayon_a + SEPARATION_RAYON_JOUEUR;
                let (dx, dz) = (a.x - jx, a.z - jz);
                let d = longueur(dx, dz);
                if d < ecart && d > 1e-4 {
                    a.x += dx / d * (ecart - d);
                    a.z += dz / d * (ecart - d);
                }
            }
            pool[i] = a;
        }
        for a in pool.iter_mut() {
            if a.arme.is_none() || a.vitesse <= 0.0 || a.def.vol.is_some() {
                continue;
            }
            let h = self.carte.atteinte(a.x, a.z);
            (a.x, a.z) = self.carte.resoudre_collision(a.x, a.z, a.def.rayon, h);
        }
    }

    pub(super) fn combattre(&mut self, i: usize, arme: usize, dt: f32) {
        let a = self.armes.invocations.pool[i];
        let s = self.arsenal.armes[arme].stats;
        let index_arme = self.arsenal.armes[arme].index;
        let cible = match self.talent_cible_prioritaire(i) {
            Some(c) => Some(c),
            None => self.ennemis.plus_proche(a.x, a.z, a.def.detection),
        };
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let (vx, vz, arret) = match cible {
            Some(c) => {
                let e = &self.ennemis.pool[c];
                (e.x, e.z, a.def.rayon + e.def.rayon + s.zone)
            }
            None => (jx, jz, DISTANCE_SUIVI),
        };
        let distance = self.aller_vers(i, vx, vz, arret, dt);
        if let Some(c) = cible
            && distance <= arret
            && a.minuterie <= 0.0
        {
            let degats = s.degats * self.talent_multiplicateur_allie(i);
            self.armes.invocations.pool[i].premiere_morsure = false;
            let a = self.armes.invocations.pool[i];
            let source = Some(index_arme);
            self.blesser(c, degats, a.x, a.z, s.recul, source, Nature::Allie);
            self.talent_sur_morsure(i, c);
            self.ev.son(&a.def.son_coup);
            self.ev.geste(Some(a.id));
            let cadence = a.def.cadence_coups * self.talent_cadence_allie();
            self.armes.invocations.pool[i].minuterie = cadence;
        }
    }

    pub(super) fn tirer_allie(&mut self, i: usize, arme: usize, dt: f32) {
        let a = self.armes.invocations.pool[i];
        let Some(tir) = &a.def.tir else {
            return;
        };
        let equipee = self.arsenal.armes[arme];
        let s = equipee.stats;
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let cible = match self.talent_cible_prioritaire(i) {
            Some(c) => Some(c),
            None => self
                .ennemis
                .plus_proche(a.x, a.z, a.def.detection.min(s.portee)),
        };
        let Some(c) = cible else {
            self.aller_vers(i, jx, jz, DISTANCE_SUIVI, dt);
            return;
        };
        let (cx, cz) = (self.ennemis.pool[c].x, self.ennemis.pool[c].z);
        let distance = self.aller_vers(i, cx, cz, tir.distance, dt);
        if distance > s.portee || a.minuterie > 0.0 {
            return;
        }
        let a = self.armes.invocations.pool[i];
        // Brûlure au dernier rang, ou dès le rang 1 avec Braises de blaze (dégâts doublés
        // au rang 2)
        let braises = self.talents.rang("braises_blaze");
        let brulure = match &equipee.def.mecanique {
            _ if braises > 0 => DEGATS_BRULURE * braises as f32,
            Mecanique::Invocation {
                brulure_au_rang: Some(r),
                ..
            } if equipee.rang as f32 >= *r => DEGATS_BRULURE,
            _ => 0.0,
        };
        let base = atan2(cx - a.x, cz - a.z);
        let origine = Origine {
            x: a.x,
            z: a.z,
            brulure,
            nature: Nature::Allie,
        };
        for k in 0..tours(tir.salve) {
            let angle = base + (k as f32 - (tir.salve - 1.0) / 2.0) * tir.ecart;
            self.lancer_depuis(
                &equipee,
                &tir.projectile,
                ModeProjectile::Ligne,
                sin(angle),
                cos(angle),
                false,
                origine,
            );
        }
        self.ev.son_hauteur(&a.def.son_coup, 1.3);
        let cadence = a.def.cadence_coups * self.talent_cadence_allie();
        self.armes.invocations.pool[i].minuterie = cadence;
    }

    pub(super) fn soutenir(&mut self, i: usize, arme: usize, dt: f32) {
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        self.aller_vers(i, jx, jz, DISTANCE_SUIVI * 0.6, dt);
        if self.armes.invocations.pool[i].minuterie > 0.0 {
            return;
        }
        let s = self.arsenal.armes[arme].stats;
        self.armes.invocations.pool[i].minuterie = s.cadence;
        let j = &self.etat.joueur;
        if j.bouclier < self.d.regles.plafonds.bouclier * j.pv_max - 0.05 {
            self.ajouter_bouclier(s.degats);
            self.ev.gerbe("bouclier", jx, 1.4, jz);
            self.ev.son(&self.armes.invocations.pool[i].def.son_coup);
            return;
        }
        let temps = self.etat.temps_jeu;
        let pool = &mut self.armes.invocations.pool;
        let mut blesse: Option<usize> = None;
        for (k, b) in pool.iter().enumerate() {
            if b.arme.is_none()
                || b.pv <= 0.0
                || b.pv >= b.pv_max
                || temps - b.dernier_soin < DELAI_SOIN
            {
                continue;
            }
            if blesse.is_none_or(|c| b.pv / b.pv_max < pool[c].pv / pool[c].pv_max) {
                blesse = Some(k);
            }
        }
        if let Some(k) = blesse {
            let b = &mut pool[k];
            b.pv = b.pv_max.min(b.pv + b.pv_max * s.zone);
            b.dernier_soin = temps;
            let (x, z) = (b.x, b.z);
            self.ev.gerbe("niveau", x, 1.0, z);
        }
    }

    pub(super) fn pulser_aura(&mut self, i: usize, arme: usize) {
        let a = self.armes.invocations.pool[i];
        let Some(aura) = &a.def.aura else {
            return;
        };
        if a.minuterie > 0.0 {
            return;
        }
        self.armes.invocations.pool[i].minuterie = aura.intervalle;
        let s = self.arsenal.armes[arme].stats;
        let index_arme = self.arsenal.armes[arme].index;
        let temps = self.etat.temps_jeu;
        let mut tampon = [0u32; 128];
        let n = self.ennemis.chercher(a.x, a.z, s.zone, &mut tampon);
        for &e in &tampon[..n] {
            let e = e as usize;
            self.ralentir(e, s.ralenti, aura.intervalle * 1.5);
            if s.degats > 0.0 {
                self.blesser(e, s.degats, a.x, a.z, 0.0, Some(index_arme), Nature::Allie);
            }
            if f64::from(self.ennemis.pool[e].immunites[index_arme]) <= temps {
                self.enraciner(e, aura.enracinement);
                self.ennemis.pool[e].immunites[index_arme] = (temps + f64::from(aura.delai)) as f32;
            }
        }
    }

    /// Avance l'allié vers (x, z) jusqu'à `arret` blocs ; renvoie la distance de départ.
    pub(super) fn aller_vers(&mut self, i: usize, x: f32, z: f32, arret: f32, dt: f32) -> f32 {
        let mut a = self.armes.invocations.pool[i];
        let (mut vers_x, mut vers_z) = (x - a.x, z - a.z);
        let distance = longueur(vers_x, vers_z);
        let distance = if distance == 0.0 { 1e-6 } else { distance };
        vers_x /= distance;
        vers_z /= distance;
        a.vitesse_sol = 0.0;
        let vitesse = a.vitesse * self.talent_vitesse_allie(i);
        if distance > arret && vitesse > 0.0 {
            let nx = a.x + vers_x * vitesse * dt;
            let nz = a.z + vers_z * vitesse * dt;
            (a.x, a.z) = if a.def.vol.is_some() {
                (nx, nz)
            } else {
                let h = self.carte.atteinte(a.x, a.z);
                self.carte.resoudre_collision(nx, nz, a.def.rayon, h)
            };
            a.vitesse_sol = vitesse;
        }
        a.angle += ecart_angle(a.angle, atan2(vers_x, vers_z)) * facteur_lissage(10.0, dt);
        self.armes.invocations.pool[i] = a;
        distance
    }
}
