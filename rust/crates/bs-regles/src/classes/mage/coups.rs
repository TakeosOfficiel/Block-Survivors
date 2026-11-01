//! Mage : coups portés (Résonance, Fournaise, Brasier, Cœur de blaze, Surcharge), ennemis
//! tués (Cendres protectrices, Propagation) et Phénix.

use super::*;
use crate::ennemis::Nature;

impl Partie {
    /// Résonance : +5 % par sort différent lancé dans les 3 s.
    pub(crate) fn mage_multiplicateur(&self, arme: Option<usize>, nature: Nature) -> f32 {
        let Some(m) = &self.talents.classes.mage else {
            return 1.0;
        };
        if arme.is_none() || nature != Nature::Arme || m.r.resonance == 0 {
            return 1.0;
        }
        let temps = self.etat.temps_jeu;
        let sorts = m
            .derniers_sorts
            .iter()
            .zip(m.sorts)
            .filter(|&(&t, s)| s.is_some() && temps - f64::from(t) <= f64::from(RESONANCE_FENETRE))
            .count();
        1.0 + RESONANCE_MAX.min(sorts as f32 * RESONANCE_PAR_SORT)
    }

    /// Fournaise : plus de critiques contre les ennemis en feu.
    pub(crate) fn mage_bonus_critique(&self, i: usize) -> f32 {
        match &self.talents.classes.mage {
            Some(m) if m.r.fournaise > 0 && self.ennemis.pool[i].brulure.cumuls > 0.0 => FOURNAISE,
            _ => 0.0,
        }
    }

    /// Brasier : critiques plus forts contre les ennemis en feu.
    pub(crate) fn mage_facteur_critique(&self, i: usize, base: f32) -> f32 {
        match &self.talents.classes.mage {
            Some(m) if m.r.brasier > 0 && self.ennemis.pool[i].brulure.cumuls > 0.0 => {
                base + BRASIER
            }
            _ => base,
        }
    }

    pub(crate) fn mage_apres_coup(
        &mut self,
        i: usize,
        arme: Option<usize>,
        critique: bool,
        nature: Nature,
    ) {
        let Some(m) = &self.talents.classes.mage else {
            return;
        };
        if nature != Nature::Arme || !self.ennemis.pool[i].vivant() {
            return;
        }
        let (boules, eclairs, r) = (m.boules, m.eclairs, m.r);
        // Cœur de blaze : les Boules de feu enflamment
        if arme.is_some() && arme == boules && r.coeur_blaze > 0 {
            let (cumuls, dps, duree) = COEUR_BLAZE;
            self.bruler(i, cumuls, dps, duree, arme);
        }
        // Surcharge : un éclair critique étourdit
        if arme.is_some() && arme == eclairs && critique {
            let duree = valeur(r.surcharge, &SURCHARGE);
            if duree > 0.0 {
                self.etourdir(i, duree);
            }
        }
    }

    pub(crate) fn mage_sur_mort(&mut self, i: usize) {
        let brulure = self.ennemis.pool[i].brulure;
        if brulure.cumuls == 0.0 {
            return;
        }
        let m = mage!(self);
        // Cendres protectrices : un ennemi en feu tué donne du bouclier
        if m.r.cendres > 0 && m.delai_cendres <= 0.0 {
            m.delai_cendres = CENDRES_DELAI;
            self.ajouter_bouclier(CENDRES_BOUCLIER);
        }
        // Propagation : ses brûlures passent à ses voisins (moitié des cumuls)
        let rang = mage!(self).r.propagation;
        let voisins = valeur(rang, &PROPAGATION_VOISINS) as usize;
        if voisins == 0 {
            return;
        }
        let part = valeur(rang, &PROPAGATION_PART);
        let cumuls = libm::floorf(brulure.cumuls * part * 0.5 + 0.5).max(1.0);
        let modif = &self.etat.modificateurs;
        let dps = brulure.degats_par_seconde / (1.0 + modif.degats_brulure);
        let duree = brulure.temps / (1.0 + modif.duree_brulure);
        let (x, z) = (self.ennemis.pool[i].x, self.ennemis.pool[i].z);
        let mut tampon = [0u32; 256];
        let n = self.ennemis.chercher(x, z, PROPAGATION_RAYON, &mut tampon);
        let mut faits = 0;
        for &o in &tampon[..n] {
            let o = o as usize;
            if faits >= voisins {
                break;
            }
            if o == i || !self.ennemis.pool[o].vivant() {
                continue;
            }
            self.bruler(o, cumuls, dps, duree.max(1.0), brulure.arme);
            let (ox, oz) = (self.ennemis.pool[o].x, self.ennemis.pool[o].z);
            self.ev.gerbe("brulure", ox, 0.8, oz);
            faits += 1;
        }
    }

    /// Phénix : une fois par partie, explosion de feu et retour à 40 % des PV.
    pub(crate) fn mage_eviter_mort(&mut self) -> bool {
        let m = mage!(self);
        if m.r.phenix == 0 || m.phenix_utilise {
            return false;
        }
        m.phenix_utilise = true;
        let boules = m.boules;
        let j = &mut self.etat.joueur;
        j.pv = libm::ceilf(j.pv_max * PHENIX_PV);
        j.protection = j.protection.max(PHENIX_PROTECTION);
        let (x, z) = (j.x, j.z);
        self.blesser_zone_reaction((x, z, PHENIX_RAYON), PHENIX_DEGATS, boules, 0.0);
        let mut tampon = [0u32; 256];
        let n = self.ennemis.chercher(x, z, PHENIX_RAYON, &mut tampon);
        let (cumuls, dps, duree) = PHENIX_BRULURE;
        for &e in &tampon[..n] {
            self.bruler(e as usize, cumuls, dps, duree, boules);
        }
        self.repousser_ennemis(x, z, PHENIX_RAYON, PHENIX_FORCE);
        self.ev.gerbe("explosion", x, 1.0, z);
        self.ev.gerbe("totem", x, 1.0, z);
        self.cercle(x, z, PHENIX_RAYON * 0.7, "brulure", 24);
        self.ev.son("totem");
        self.ev.son("explosion");
        self.ev.secousse(0.6);
        true
    }
}
