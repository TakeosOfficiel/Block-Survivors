//! Crochets des talents de l'Invocateur dans le système des invocations : quota (Lune de
//! sang), création (Chef de meute, Alpha, Gardien alpha), cible prioritaire (Traque,
//! Coordination), dégâts, cadence et vitesse des alliés, morsures (Morsure marquante,
//! Encerclement) et mort d'un allié (Gardien alpha, Lien spirituel, Réactivité, Essaim).
//! Neutres pour les autres classes.

use bs_donnees::entites::Allie as DefinitionAllie;

use super::*;

impl Partie {
    /// Places en plus pour ce type d'allié (Lune de sang : 12 loups).
    pub(crate) fn talent_quota_allie(&self, def: &DefinitionAllie) -> f32 {
        let Some(v) = self.talents.classes.invocateur.as_ref() else {
            return 0.0;
        };
        let loup = v.type_loup.and_then(|t| self.d.allies.allies.get_index(t));
        if v.lune > 0.0 && loup.is_some_and(|(_, l)| std::ptr::eq(l, def)) {
            LUNE_QUOTA - def.quota
        } else {
            0.0
        }
    }

    /// L'allié `k` vient d'être créé.
    pub(crate) fn talent_sur_creation_allie(&mut self, k: usize) {
        let n_presents = {
            let a = &self.armes.invocations.pool[k];
            self.presents(a.type_allie)
        };
        let v = invocateur!(self);
        let a = &mut self.armes.invocations.pool[k];
        if Some(a.type_allie) != v.type_loup {
            return;
        }
        a.premiere_morsure = v.r.chef_meute > 0;
        // Pendant la Lune, les loups au-delà du quota normal repartent à la fin
        if v.lune > 0.0 && n_presents as f32 > v.quota_loup {
            a.temporaire = true;
            a.vie = a.vie.min(v.lune);
        }
        // Alpha : le premier loup (s'il n'y en a pas d'autre en vie)
        if v.r.alpha == 0 || a.temporaire {
            return;
        }
        let gardien = v.r.gardien_alpha > 0;
        let pool = &mut self.armes.invocations.pool;
        let autre = pool
            .iter()
            .enumerate()
            .any(|(o, a)| o != k && a.alpha && a.arme.is_some() && a.pv > 0.0);
        if autre {
            return;
        }
        let a = &mut pool[k];
        a.alpha = true;
        a.pv_max *= ALPHA_PV;
        a.pv = a.pv_max;
        if gardien {
            a.echelle = GARDIEN_ECHELLE;
        }
    }

    /// Traque (loups) et Coordination (vex, blaze) : l'ennemi marqué le plus proche.
    pub(crate) fn talent_cible_prioritaire(&mut self, i: usize) -> Option<usize> {
        let v = self.talents.classes.invocateur.as_ref()?;
        let a = self.armes.invocations.pool[i];
        let t = Some(a.type_allie);
        let loup = t == v.type_loup;
        let tireur = t == v.type_vex || t == v.type_blaze;
        if !(loup && v.r.traque > 0) && !(tireur && v.r.coordination > 0) {
            return None;
        }
        let mut tampon = [0u32; 128];
        let n = self
            .ennemis
            .chercher(a.x, a.z, a.def.detection, &mut tampon);
        let mut cible = None;
        let mut meilleure = f32::INFINITY;
        for &e in &tampon[..n] {
            let e = e as usize;
            let en = &self.ennemis.pool[e];
            if en.marque_temps <= 0.0 {
                continue;
            }
            let d2 = (en.x - a.x) * (en.x - a.x) + (en.z - a.z) * (en.z - a.z);
            if d2 < meilleure {
                meilleure = d2;
                cible = Some(e);
            }
        }
        cible
    }

    /// Dégâts de l'allié `i` : première morsure, alpha, ruée.
    pub(crate) fn talent_multiplicateur_allie(&self, i: usize) -> f32 {
        let Some(v) = self.talents.classes.invocateur.as_ref() else {
            return 1.0;
        };
        let a = &self.armes.invocations.pool[i];
        let mut m = 1.0;
        if a.premiere_morsure {
            m *= CHEF_DEGATS;
        }
        if a.alpha {
            m *= 1.0 + valeur(v.r.alpha, &ALPHA_DEGATS);
        }
        if v.ruee > 0.0 && Some(a.type_allie) == v.type_loup {
            m *= 1.0 + RUEE_DEGATS;
        }
        m
    }

    /// Délai entre deux attaques des alliés (Hurlement).
    pub(crate) fn talent_cadence_allie(&self) -> f32 {
        match self.talents.classes.invocateur.as_ref() {
            Some(v) if v.hurlement > 0.0 => 1.0 - valeur(v.r.hurlement, &HURLEMENT_CADENCE),
            _ => 1.0,
        }
    }

    /// Vitesse de l'allié `i` (Ruée de la meute).
    pub(crate) fn talent_vitesse_allie(&self, i: usize) -> f32 {
        match self.talents.classes.invocateur.as_ref() {
            Some(v)
                if v.ruee > 0.0
                    && Some(self.armes.invocations.pool[i].type_allie) == v.type_loup =>
            {
                RUEE_VITESSE
            }
            _ => 1.0,
        }
    }

    /// L'allié `i` vient de mordre l'ennemi `e`.
    pub(crate) fn talent_sur_morsure(&mut self, i: usize, e: usize) {
        let a = self.armes.invocations.pool[i];
        let Some(v) = self.talents.classes.invocateur.as_ref() else {
            return;
        };
        if Some(a.type_allie) != v.type_loup || !self.ennemis.pool[e].vivant() {
            return;
        }
        let r = v.r;
        // Morsure marquante (Traque : plus longue)
        let marque = valeur(r.morsure, &MORSURE_MARQUE);
        if marque > 0.0 {
            let traque = if r.traque > 0 { MORSURE_TRAQUE } else { 0.0 };
            self.marquer(e, marque, MORSURE_DUREE + traque);
        }
        // Encerclement : 3 loups différents en moins de 2 s
        let en = &mut self.ennemis.pool[e];
        if r.encerclement > 0 && a.id != en.morsure_a && a.id != en.morsure_b {
            let maintenant = self.etat.temps_jeu;
            if en.temps_morsure_a >= 0.0
                && en.temps_morsure_b >= 0.0
                && maintenant - en.temps_morsure_a.min(en.temps_morsure_b) <= ENCERCLEMENT_FENETRE
            {
                en.encercle = ENCERCLEMENT_DUREE;
            }
            en.morsure_b = en.morsure_a;
            en.temps_morsure_b = en.temps_morsure_a;
            en.morsure_a = a.id;
            en.temps_morsure_a = maintenant;
        }
    }

    /// L'allié `k` vient de tomber ; vrai si un talent le ramène.
    pub(crate) fn talent_sur_mort_allie(&mut self, k: usize) -> bool {
        let v = invocateur!(self);
        let a = &mut self.armes.invocations.pool[k];
        // Gardien alpha : l'alpha revient une fois par invocation
        if a.alpha && !a.revenu && v.r.gardien_alpha > 0 {
            a.revenu = true;
            a.pv = a.pv_max;
            let (x, z) = (a.x, a.z);
            self.ev.gerbe("totem", x, 1.0, z);
            self.ev.son("totem");
            return true;
        }
        let a = *a;
        // Lien spirituel : un allié tué donne du bouclier
        if v.r.lien > 0 && v.delai_lien <= 0.0 {
            v.delai_lien = LIEN_DELAI_INTERNE;
            self.ajouter_bouclier(LIEN_BOUCLIER);
            let j = &self.etat.joueur;
            self.ev.gerbe("bouclier", j.x, 1.4, j.z);
        }
        let v = invocateur!(self);
        let (r, vex, type_vex) = (v.r, v.vex, v.type_vex);
        // Réactivité : son arme revient plus tôt
        let avance = valeur(r.reactivite, &REACTIVITE_AVANCE);
        if avance > 0.0
            && let Some(arme) = a.arme
        {
            self.arsenal.armes[arme].minuterie *= 1.0 - avance;
        }
        // Essaim : un vex tué explose
        let essaim = valeur(r.essaim, &ESSAIM_DEGATS);
        if essaim > 0.0 && Some(a.type_allie) == type_vex && a.arme.is_some() {
            self.blesser_zone_reaction((a.x, a.z, ESSAIM_RAYON), essaim, vex, ESSAIM_RECUL);
            self.ev.gerbe("explosionArme", a.x, 1.0, a.z);
        }
        false
    }
}
