//! Mécaniques des talents de l'Invocateur (src/combat/talents/invocateur.ts).
//! Maître-chien : Chef de meute, marques (Morsure marquante, Traque, Curée, Coordination),
//! Meute soudée, Hurlement, Encerclement, Alpha et Gardien alpha, Ruée de la meute, Lune de
//! sang. Braises de blaze et le Bosquet sont lus par le système des invocations.
//! Évocateur : Lien spirituel, Garde rapprochée, Allay chanteur, Essaim, Évocation
//! supérieure et Crocs de l'évocateur (ravageur.rs). Crochets des alliés : allies.rs.

use bs_donnees::Donnees;
use bs_donnees::armes::Mecanique;

use super::valeur;
use crate::ennemis::Nature;
use crate::partie::Partie;
use crate::talents::Talents;

mod allies;
mod ravageur;

pub use ravageur::{CROCS_CLAQUE, CROCS_VIE, Croc, Ravageur};

const CHEF_DEGATS: f32 = 1.5;
const MORSURE_MARQUE: [f32; 2] = [0.15, 0.25];
const MORSURE_DUREE: f32 = 3.0;
const MORSURE_TRAQUE: f32 = 2.0;
const REACTIVITE_AVANCE: [f32; 2] = [0.25, 0.4];
const MEUTE_PAR_ALLIE: [f32; 2] = [0.03, 0.04];
const MEUTE_MAXIMUM: [f32; 2] = [0.24, 0.32];
const CUREE_SOIN: f32 = 0.1;
const CUREE_RAYON: f32 = 6.0;
const CUREE_DELAI: f64 = 8.0;
const HURLEMENT_PERIODE: f32 = 10.0;
const HURLEMENT_DUREE: f32 = 3.0;
const HURLEMENT_CADENCE: [f32; 2] = [0.1, 0.2];
const ENCERCLEMENT_FENETRE: f64 = 2.0;
const ENCERCLEMENT_DUREE: f32 = 3.0;
const ENCERCLEMENT_DEGATS: [f32; 2] = [0.15, 0.3];
const COORDINATION_DEGATS: [f32; 2] = [0.1, 0.2];
const ALPHA_PV: f32 = 2.0;
const ALPHA_DEGATS: [f32; 2] = [0.3, 0.5];
const RUEE_COUPS: u32 = 15;
const RUEE_DUREE: f32 = 3.0;
const RUEE_VITESSE: f32 = 2.0;
const RUEE_DEGATS: f32 = 0.5;
const RUEE_FANTOMES: f32 = 2.0;
const RUEE_DELAI_INTERNE: f32 = 12.0;
const LUNE_PERIODE: f32 = 60.0;
const LUNE_DUREE: f32 = 8.0;
const LUNE_QUOTA: f32 = 12.0;
const LUNE_PART_PV: f32 = 0.5;
const GARDIEN_ECHELLE: f32 = 1.25;
const LIEN_BOUCLIER: f32 = 2.0;
const LIEN_DELAI_INTERNE: f32 = 5.0;
const GARDE_RAYON: f32 = 4.0;
const GARDE_ARMURE: [f32; 2] = [1.0, 2.0];
const ALLAY_RAYON: f32 = 3.5;
const ALLAY_PERIODE: f32 = 0.25;
const ESSAIM_DEGATS: [f32; 2] = [6.0, 10.0];
const ESSAIM_RAYON: f32 = 1.5;
const ESSAIM_RECUL: f32 = 3.0;
const EVOCATION_DECLENCHEMENTS: u32 = 8;
const EVOCATION_DELAI_INTERNE: f32 = 30.0;

/// Rangs des talents de l'Invocateur, lus une fois au début de la partie.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rangs {
    pub chef_meute: u32,
    pub morsure: u32,
    pub reactivite: u32,
    pub meute: u32,
    pub traque: u32,
    pub curee: u32,
    pub hurlement: u32,
    pub encerclement: u32,
    pub coordination: u32,
    pub alpha: u32,
    pub ruee: u32,
    pub lune: u32,
    pub gardien_alpha: u32,
    pub lien: u32,
    pub garde: u32,
    pub allay: u32,
    pub essaim: u32,
    pub evocation: u32,
    pub crocs: u32,
}

impl Rangs {
    pub fn lire(t: &Talents) -> Self {
        Self {
            chef_meute: t.rang("chef_meute"),
            morsure: t.rang("morsure_marquante"),
            reactivite: t.rang("reactivite"),
            meute: t.rang("meute_soudee"),
            traque: t.rang("traque"),
            curee: t.rang("curee"),
            hurlement: t.rang("hurlement"),
            encerclement: t.rang("encerclement"),
            coordination: t.rang("coordination"),
            alpha: t.rang("alpha"),
            ruee: t.rang("ruee_meute"),
            lune: t.rang("lune_de_sang"),
            gardien_alpha: t.rang("gardien_alpha"),
            lien: t.rang("lien_spirituel"),
            garde: t.rang("garde_rapprochee"),
            allay: t.rang("allay_chanteur"),
            essaim: t.rang("essaim"),
            evocation: t.rang("evocation_superieure"),
            crocs: t.rang("crocs_evocateur"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Invocateur {
    pub r: Rangs,
    /// Index des armes dans le catalogue (les loups créditent le ravageur et les crocs).
    loups: Option<usize>,
    vex: Option<usize>,
    blaze: Option<usize>,
    /// Index des types d'alliés.
    type_loup: Option<usize>,
    type_vex: Option<usize>,
    type_blaze: Option<usize>,
    quota_loup: f32,
    vivants: u32,
    minuterie_comptage: f32,
    minuterie_hurlement: f32,
    hurlement: f32,
    minuterie_lune: f32,
    lune: f32,
    coups_marques: u32,
    ruee: f32,
    delai_ruee: f32,
    delai_lien: f32,
    declenchements: u32,
    delai_evocation: f32,
    pub ravageur: Ravageur,
    minuterie_crocs: f32,
    pub crocs: Vec<Croc>,
}

impl Invocateur {
    pub fn nouveau(d: &Donnees) -> Self {
        let allies = &d.allies.allies;
        Self {
            r: Rangs::default(),
            loups: d.armes.get_index_of("loups"),
            vex: d.armes.get_index_of("vex"),
            blaze: d.armes.get_index_of("esprit_blaze"),
            type_loup: allies.get_index_of("loup"),
            type_vex: allies.get_index_of("vex"),
            type_blaze: allies.get_index_of("esprit_blaze"),
            quota_loup: allies.get("loup").map_or(0.0, |a| a.quota),
            vivants: 0,
            minuterie_comptage: 0.0,
            minuterie_hurlement: HURLEMENT_PERIODE,
            hurlement: 0.0,
            minuterie_lune: LUNE_PERIODE,
            lune: 0.0,
            coups_marques: 0,
            ruee: 0.0,
            delai_ruee: 0.0,
            delai_lien: 0.0,
            declenchements: 0,
            delai_evocation: 0.0,
            ravageur: Ravageur::default(),
            minuterie_crocs: ravageur::CROCS_PERIODE,
            crocs: Vec::with_capacity(ravageur::CROCS_CAPACITE),
        }
    }
}

macro_rules! invocateur {
    ($p:expr) => {
        match $p.talents.classes.invocateur.as_mut() {
            Some(m) => m,
            None => return Default::default(),
        }
    };
}
pub(crate) use invocateur;

impl Partie {
    pub(crate) fn invocateur_maj(&mut self, dt: f32) {
        let v = invocateur!(self);
        v.delai_ruee -= dt;
        v.delai_lien -= dt;
        v.delai_evocation -= dt;
        if v.ruee > 0.0 {
            v.ruee -= dt;
        }
        // Alliés en vie (Meute soudée), recomptés quatre fois par seconde ; l'allay
        // ramasse l'XP
        v.minuterie_comptage -= dt;
        if v.minuterie_comptage <= 0.0 {
            v.minuterie_comptage = ALLAY_PERIODE;
            let allay = v.r.allay > 0;
            let mut n = 0;
            for k in 0..self.armes.invocations.pool.len() {
                let a = self.armes.invocations.pool[k];
                if a.arme.is_none() || a.pv <= 0.0 {
                    continue;
                }
                n += 1;
                if a.def.soutien && allay {
                    self.aimanter_orbes(a.x, a.z, ALLAY_RAYON);
                }
            }
            invocateur!(self).vivants = n;
        }
        // Hurlement : toutes les 10 s, les alliés attaquent plus vite
        let v = invocateur!(self);
        if v.r.hurlement > 0 {
            if v.hurlement > 0.0 {
                v.hurlement -= dt;
            }
            v.minuterie_hurlement -= dt;
            if v.minuterie_hurlement <= 0.0 {
                v.minuterie_hurlement = HURLEMENT_PERIODE;
                v.hurlement = HURLEMENT_DUREE;
                self.hurler();
            }
        }
        // Lune de sang : toutes les 60 s, le quota de loups double pendant 8 s
        let v = invocateur!(self);
        if v.r.lune > 0 {
            if v.lune > 0.0 {
                v.lune -= dt;
            }
            v.minuterie_lune -= dt;
            if v.minuterie_lune <= 0.0 {
                v.minuterie_lune = LUNE_PERIODE;
                v.lune = LUNE_DUREE;
                let (type_loup, loups) = (v.type_loup, v.loups);
                if let Some(k) = self.position_arme(loups)
                    && let Some(t) = type_loup
                {
                    let manquants = LUNE_QUOTA - self.presents(t) as f32;
                    self.invoquer_temporaire(k, manquants, LUNE_DUREE, LUNE_PART_PV);
                }
                self.ev.son("loup_hurlement");
                let j = &self.etat.joueur;
                self.ev.gerbe("critique", j.x, 2.0, j.z);
            }
        }
        self.maj_ravageur(dt);
        self.maj_crocs(dt);
    }

    /// Multiplicateur (additif) contre l'ennemi `i` : Encerclement, Meute soudée,
    /// Coordination.
    pub(crate) fn invocateur_multiplicateur(
        &self,
        i: usize,
        arme: Option<usize>,
        nature: Nature,
    ) -> f32 {
        let Some(v) = self.talents.classes.invocateur.as_ref() else {
            return 1.0;
        };
        let e = &self.ennemis.pool[i];
        let mut m = 1.0;
        if e.encercle > 0.0 {
            m += valeur(v.r.encerclement, &ENCERCLEMENT_DEGATS);
        }
        if nature != Nature::Allie {
            return m;
        }
        let meute = valeur(v.r.meute, &MEUTE_PAR_ALLIE);
        if meute > 0.0 {
            m += valeur(v.r.meute, &MEUTE_MAXIMUM).min(meute * v.vivants as f32);
        }
        if arme.is_some() && (arme == v.vex || arme == v.blaze) && e.marque_temps > 0.0 {
            m += valeur(v.r.coordination, &COORDINATION_DEGATS);
        }
        m
    }

    /// Ruée de la meute : 15 coups d'allié sur des cibles marquées.
    pub(crate) fn invocateur_apres_coup(&mut self, i: usize, nature: Nature) {
        let marque = self.ennemis.pool[i].marque_temps > 0.0;
        let v = invocateur!(self);
        if nature != Nature::Allie || !marque || v.r.ruee == 0 || v.delai_ruee > 0.0 {
            return;
        }
        v.coups_marques += 1;
        if v.coups_marques < RUEE_COUPS {
            return;
        }
        v.coups_marques = 0;
        v.delai_ruee = RUEE_DELAI_INTERNE;
        v.ruee = RUEE_DUREE;
        let loups = v.loups;
        self.hurler();
        if let Some(k) = self.position_arme(loups) {
            self.invoquer_temporaire(k, RUEE_FANTOMES, RUEE_DUREE, 1.0);
        }
    }

    /// Curée : tuer une cible marquée soigne les alliés proches.
    pub(crate) fn invocateur_sur_mort(&mut self, i: usize) {
        let e = &self.ennemis.pool[i];
        let (ex, ez) = (e.x, e.z);
        if e.marque_temps <= 0.0 || invocateur!(self).r.curee == 0 {
            return;
        }
        let maintenant = self.etat.temps_jeu;
        for k in 0..self.armes.invocations.pool.len() {
            let a = &mut self.armes.invocations.pool[k];
            if a.arme.is_none() || a.pv <= 0.0 || maintenant - a.dernier_soin < CUREE_DELAI {
                continue;
            }
            if (a.x - ex) * (a.x - ex) + (a.z - ez) * (a.z - ez) > CUREE_RAYON * CUREE_RAYON {
                continue;
            }
            a.pv = a.pv_max.min(a.pv + a.pv_max * CUREE_SOIN);
            a.dernier_soin = maintenant;
            let (x, z) = (a.x, a.z);
            self.ev.gerbe("niveau", x, 1.0, z);
        }
    }

    /// Garde rapprochée : armure avec un allié près du joueur.
    pub(crate) fn invocateur_armure(&self) -> f32 {
        let Some(v) = self.talents.classes.invocateur.as_ref() else {
            return 0.0;
        };
        let j = &self.etat.joueur;
        if v.r.garde == 0 || !self.allie_proche(j.x, j.z, GARDE_RAYON) {
            return 0.0;
        }
        valeur(v.r.garde, &GARDE_ARMURE)
    }

    /// Évocation supérieure : chaque déclenchement d'une invocation compte (pas l'allay).
    pub(crate) fn invocateur_apres_declenchement(&mut self, i: usize, declenchee: bool) {
        let soutien = match &self.arsenal.armes[i].def.mecanique {
            Mecanique::Invocation { allie, .. } => {
                self.d.allies.allies.get(allie).map(|a| a.soutien)
            }
            _ => None,
        };
        let v = invocateur!(self);
        if !declenchee || soutien != Some(false) || v.r.evocation == 0 {
            return;
        }
        if v.declenchements < EVOCATION_DECLENCHEMENTS {
            v.declenchements += 1;
        }
        if v.declenchements >= EVOCATION_DECLENCHEMENTS
            && !v.ravageur.actif
            && v.delai_evocation <= 0.0
        {
            v.declenchements = 0;
            v.delai_evocation = EVOCATION_DELAI_INTERNE;
            self.invoquer_ravageur();
        }
    }

    /// Les loups en vie hurlent.
    fn hurler(&mut self) {
        let Some(loup) = self
            .talents
            .classes
            .invocateur
            .as_ref()
            .and_then(|v| v.type_loup)
        else {
            return;
        };
        for k in 0..self.armes.invocations.pool.len() {
            let a = self.armes.invocations.pool[k];
            if a.arme.is_some() && a.pv > 0.0 && a.type_allie == loup {
                self.ev.gerbe("critique", a.x, 1.2, a.z);
            }
        }
        self.ev.son("loup_hurlement");
    }

    /// Position dans l'arsenal de l'arme d'index `index` du catalogue.
    fn position_arme(&self, index: Option<usize>) -> Option<usize> {
        let index = index?;
        self.arsenal.armes.iter().position(|a| a.index == index)
    }
}
