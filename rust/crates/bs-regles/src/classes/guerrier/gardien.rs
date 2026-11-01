//! Mécaniques défensives du Gardien (Guerrier) : Inébranlable, Tenir la ligne, Peau de
//! fer, réactions aux coups reçus (Posture défensive, Épines, Ronces d'acier, Contre-attaque
//! du bastion) et Dernier rempart.

use super::*;

impl Partie {
    /// Inébranlable (boss), Tenir la ligne (immobile).
    pub(crate) fn guerrier_degats_recus(&self, mut degats: f32, boss: bool) -> f32 {
        let Some(g) = &self.talents.classes.guerrier else {
            return degats;
        };
        if boss && g.r.inebranlable > 0 {
            degats *= INEBRANLABLE;
        }
        if self.talents.classes.temps_immobile >= TENIR_IMMOBILE {
            degats *= 1.0 - valeur(g.r.tenir, &TENIR_REDUCTION);
        }
        degats
    }

    /// Peau de fer : armure en plus sous la moitié des PV.
    pub(crate) fn guerrier_armure(&self) -> f32 {
        let Some(g) = &self.talents.classes.guerrier else {
            return 0.0;
        };
        let j = &self.etat.joueur;
        if j.pv < PEAU_SEUIL * j.pv_max {
            valeur(g.r.peau_de_fer, &PEAU_ARMURE)
        } else {
            0.0
        }
    }

    /// Posture défensive, Épines (et Ronces d'acier), Contre-attaque du bastion, Dernier
    /// rempart.
    pub(crate) fn guerrier_apres_coup_recu(&mut self, evites: f32) {
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let g = guerrier!(self);
        let (r, heurt) = (g.r, g.heurt);
        if r.posture > 0 && g.delai_posture <= 0.0 {
            let (rayon, force, delai) = POSTURE;
            g.delai_posture = delai;
            self.repousser_ennemis(jx, jz, rayon, force);
            self.cercle(jx, jz, rayon * 0.6, "repousse", 12);
        }
        let g = guerrier!(self);
        let epines = valeur(r.epines, &EPINES);
        if epines > 0.0 && g.delai_epines <= 0.0 {
            g.delai_epines = EPINES_DELAI;
            let rayon = EPINES_RAYON + valeur(r.ronces_acier, &RONCES_RAYON);
            let mut tampon = [0u32; 128];
            let n = self.ennemis.chercher(jx, jz, rayon, &mut tampon);
            for &e in &tampon[..n] {
                let e = e as usize;
                self.blesser_reaction(e, epines, jx, jz, 2.0, heurt, Nature::Arme);
                if r.ronces_acier > 0 {
                    self.ralentir(e, RONCES_RALENTI, RONCES_DUREE);
                }
            }
            self.cercle(jx, jz, rayon * 0.7, "epines", 16);
        }
        let g = guerrier!(self);
        if r.bastion > 0 {
            g.charge_bastion += evites;
            if g.charge_bastion >= BASTION_CHARGE && g.delai_bastion <= 0.0 {
                let degats = BASTION_MAX.min(BASTION_DEGATS + g.charge_bastion);
                g.charge_bastion = 0.0;
                g.delai_bastion = BASTION_DELAI;
                self.blesser_zone_reaction((jx, jz, BASTION_RAYON), degats, heurt, 6.0);
                self.ev.gerbe("impactLame", jx, 0.3, jz);
                self.cercle(jx, jz, BASTION_RAYON, "repousse", 20);
                self.ev.son("lame_colossale");
                self.ev.secousse(0.4);
            }
        }
        let j = &self.etat.joueur;
        if j.pv < REMPART_FINAL_SEUIL * j.pv_max {
            self.dernier_rempart();
        }
    }

    /// Le Dernier rempart sauve aussi d'un coup mortel, s'il n'a pas encore servi.
    pub(crate) fn guerrier_eviter_mort(&mut self) -> bool {
        let g = guerrier!(self);
        if g.r.dernier_rempart == 0 || g.dernier_rempart_utilise {
            return false;
        }
        self.etat.joueur.pv = 1.0;
        self.dernier_rempart();
        true
    }

    fn dernier_rempart(&mut self) {
        let g = guerrier!(self);
        if g.r.dernier_rempart == 0 || g.dernier_rempart_utilise {
            return;
        }
        g.dernier_rempart_utilise = true;
        let j = &mut self.etat.joueur;
        j.protection = j.protection.max(REMPART_FINAL_PROTECTION);
        let (jx, jz) = (j.x, j.z);
        self.repousser_ennemis(jx, jz, REMPART_FINAL_RAYON, REMPART_FINAL_FORCE);
        self.ev.gerbe("totem", jx, 1.0, jz);
        self.cercle(jx, jz, REMPART_FINAL_RAYON * 0.6, "repousse", 24);
        self.ev.son("totem");
        self.ev.secousse(0.5);
    }
}
