//! Coups de l'Archère : bonus de dégâts (Premier sang, Marque, Point faible, Exécutrice,
//! Ronces venimeuses), effets après un coup (Marque, Exécutrice, Œil de l'aigle, Venin),
//! ennemis tués (Contagion, Nuage toxique) et sortie d'un piège (Piège à loups).

use super::*;

impl Partie {
    pub(crate) fn archere_multiplicateur(
        &self,
        i: usize,
        arme: Option<usize>,
        nature: Nature,
    ) -> f32 {
        let Some(a) = &self.talents.classes.archere else {
            return 1.0;
        };
        if arme.is_none() || nature != Nature::Arme {
            return 1.0;
        }
        let e = &self.ennemis.pool[i];
        let boss = e.boss();
        let mut m = 1.0;
        if e.pv > PREMIER_SANG_SEUIL * e.pv_max {
            m += valeur(a.r.premier_sang, &PREMIER_SANG);
        }
        if e.marque_chasseur > 0.0 {
            m += valeur(a.r.marque, &MARQUE);
            if boss {
                m += valeur(a.r.point_faible, &POINT_FAIBLE);
            }
        }
        if boss && e.cumuls_talent > 0.0 {
            m += e.cumuls_talent * EXECUTRICE_PAR_CRITIQUE;
        }
        if e.dans_piege
            && self.famille(arme) == Some(FamilleArme::Projectile)
            && a.r.ronces_venimeuses > 0
        {
            m += RONCES_VENIMEUSES;
        }
        m
    }

    pub(crate) fn archere_apres_coup(
        &mut self,
        i: usize,
        degats: f32,
        arme: Option<usize>,
        critique: bool,
        nature: Nature,
    ) {
        let Some(index) = arme else {
            return;
        };
        if nature != Nature::Arme {
            return;
        }
        let r = archere!(self).r;
        let fleche = self.famille(arme) == Some(FamilleArme::Projectile);
        if critique {
            let e = &mut self.ennemis.pool[i];
            if r.marque > 0 {
                e.marque_chasseur = MARQUE_DUREE;
            }
            // Exécutrice : chaque critique sur un boss ajoute un cumul
            if e.boss() && r.executrice > 0 {
                e.cumuls_talent = EXECUTRICE_CUMULS.min(e.cumuls_talent + 1.0);
                e.temps_cumuls = EXECUTRICE_DUREE;
            }
            // Œil de l'aigle : le critique ricoche vers l'ennemi le plus éloigné
            if r.oeil_aigle > 0 {
                self.oeil_aigle(i, degats, index);
            }
        }
        // Venin : une part des flèches empoisonne (le tirage a lieu même sans le talent)
        if fleche
            && self.ennemis.pool[i].vivant()
            && self.alea.tirer() < f64::from(valeur(r.venin, &VENIN_CHANCE))
        {
            self.archere_empoisonner(i, 1.0, arme);
        }
        // Exécutrice : les flèches achèvent un ennemi normal presque vaincu
        let e = &self.ennemis.pool[i];
        if fleche
            && e.vivant()
            && !e.boss()
            && e.pv < EXECUTRICE_SEUIL * e.pv_max
            && r.executrice > 0
        {
            let (pv, x, z) = (e.pv, e.x, e.z);
            self.blesser_reaction(i, pv, x, z, 0.0, arme, Nature::Arme);
        }
    }

    pub(crate) fn archere_sur_mort(&mut self, i: usize) {
        let poison = self.ennemis.pool[i].poison;
        if poison.cumuls == 0.0 {
            return;
        }
        let r = archere!(self).r;
        let (ex, ez, id) = {
            let e = &self.ennemis.pool[i];
            (e.x, e.z, e.id)
        };
        // Contagion : le voisin le plus proche reçoit des cumuls
        let contagion = valeur(r.contagion, &CONTAGION);
        if contagion > 0.0 {
            let mut tampon = [0u32; 128];
            let n = self.ennemis.chercher(ex, ez, CONTAGION_RAYON, &mut tampon);
            let mut voisin = None;
            let mut meilleur = f32::INFINITY;
            for &k in &tampon[..n] {
                let o = &self.ennemis.pool[k as usize];
                let d2 = (o.x - ex) * (o.x - ex) + (o.z - ez) * (o.z - ez);
                if o.id != id && o.vivant() && d2 < meilleur {
                    meilleur = d2;
                    voisin = Some(k as usize);
                }
            }
            if let Some(v) = voisin {
                self.archere_empoisonner(v, contagion, poison.arme);
            }
        }
        // Nuage toxique : mort avec le maximum de cumuls
        let max = self.talent_cumuls_poison();
        let a = archere!(self);
        if r.nuage > 0 && poison.cumuls >= max && a.delai_nuage <= 0.0 && a.nuages.len() < NUAGE_MAX
        {
            a.delai_nuage = NUAGE_DELAI;
            a.nuages.push(Nuage {
                x: ex,
                z: ez,
                temps: NUAGE_DUREE,
                tic: 0.0,
                arme: poison.arme,
            });
        }
    }

    pub(crate) fn archere_sur_sortie_piege(&mut self, i: usize) {
        let duree = valeur(archere!(self).r.piege_loups, &PIEGE_LOUPS);
        if duree > 0.0 {
            self.ralentir(i, PIEGE_LOUPS_RALENTI, duree);
        }
    }

    /// Œil de l'aigle : flèche spectrale vers l'ennemi le plus éloigné.
    fn oeil_aigle(&mut self, source: usize, degats: f32, arme: usize) {
        let (sx, sz, id) = {
            let e = &self.ennemis.pool[source];
            (e.x, e.z, e.id)
        };
        let mut tampon = [0u32; 256];
        let n = self.ennemis.chercher(sx, sz, OEIL_PORTEE, &mut tampon);
        let mut cible = None;
        let mut meilleur = -1.0;
        for &k in &tampon[..n] {
            let o = &self.ennemis.pool[k as usize];
            let d2 = (o.x - sx) * (o.x - sx) + (o.z - sz) * (o.z - sz);
            if o.id != id && o.vivant() && d2 > meilleur {
                meilleur = d2;
                cible = Some(k as usize);
            }
        }
        let Some(cible) = cible else {
            return;
        };
        if let Some(position) = self.arsenal.armes.iter().position(|a| a.index == arme) {
            self.ricocher(position, sx, sz, cible, degats * OEIL_PART);
        }
    }
}
