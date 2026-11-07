//! Mécaniques des talents du Guerrier (src/combat/talents/guerrier.ts).
//! Gladiateur : enchaînements, Frénésie, critiques de mêlée, exécutions.
//! Gardien : réactions aux coups reçus (épines, posture, bastion), défense, Onde de choc
//! (Répliques et Séisme, lus par les fissures).

use bs_donnees::Donnees;
use bs_donnees::armes::FamilleArme;

use super::valeur;
use crate::ennemis::Nature;
use crate::partie::Partie;
use valeurs::*;
pub use valeurs::{DIRECTIONS_SEISME, REPLIQUE_DELAI, REPLIQUES, Rangs};

mod valeurs;

#[derive(Debug, Clone)]
pub struct Guerrier {
    pub r: Rangs,
    epee: Option<usize>,
    onde: Option<usize>,
    heurt: Option<usize>,
    /// Frénésie : temps de jeu des dernières éliminations (anneau, en f32 comme en TS).
    eliminations: [f32; 16],
    prochaine: usize,
    frenesie: f32,
    fureur: f32,
    /// Coups d'épée : touches du même coup (même instant).
    instant_epee: f64,
    touches_epee: u32,
    enchainement_pret: bool,
    elan_pret: bool,
    elan_applique: bool,
    delai_elan: f32,
    /// Fureur du gladiateur : attaques de mêlée qui touchent.
    instant_melee: f64,
    arme_melee: Option<usize>,
    attaques_melee: u32,
    delai_fureur: f32,
    delai_execution: f32,
    delai_posture: f32,
    delai_epines: f32,
    minuterie_rempart: f32,
    charge_bastion: f32,
    delai_bastion: f32,
    dernier_rempart_utilise: bool,
    ondes: u32,
}

impl Guerrier {
    pub fn nouveau(d: &Donnees) -> Self {
        Self {
            r: Rangs::default(),
            epee: d.armes.get_index_of("epee"),
            onde: d.armes.get_index_of("onde_choc"),
            heurt: d.armes.get_index_of("heurt_bouclier"),
            eliminations: [0.0; 16],
            prochaine: 0,
            frenesie: 0.0,
            fureur: 0.0,
            instant_epee: -1.0,
            touches_epee: 0,
            enchainement_pret: false,
            elan_pret: false,
            elan_applique: false,
            delai_elan: 0.0,
            instant_melee: -1.0,
            arme_melee: None,
            attaques_melee: 0,
            delai_fureur: 0.0,
            delai_execution: 0.0,
            delai_posture: 0.0,
            delai_epines: 0.0,
            minuterie_rempart: REMPART_PERIODE,
            charge_bastion: 0.0,
            delai_bastion: 0.0,
            dernier_rempart_utilise: false,
            ondes: 0,
        }
    }
}

macro_rules! guerrier {
    ($p:expr) => {
        match $p.talents.classes.guerrier.as_mut() {
            Some(g) => g,
            None => return Default::default(),
        }
    };
}

mod gardien;

impl Partie {
    pub(crate) fn guerrier_maj(&mut self, dt: f32) {
        let g = guerrier!(self);
        if g.frenesie > 0.0 {
            g.frenesie -= dt;
            if g.frenesie <= 0.0 && g.r.fureur > 0 {
                g.fureur = FUREUR_DUREE;
            }
        }
        if g.fureur > 0.0 {
            g.fureur -= dt;
        }
        for delai in [
            &mut g.delai_elan,
            &mut g.delai_fureur,
            &mut g.delai_execution,
            &mut g.delai_posture,
            &mut g.delai_epines,
            &mut g.delai_bastion,
        ] {
            *delai -= dt;
        }
        // Rempart : un bouclier toutes les 12 s
        if g.r.rempart == 0 {
            return;
        }
        g.minuterie_rempart -= dt;
        if g.minuterie_rempart <= 0.0 {
            g.minuterie_rempart = REMPART_PERIODE;
            self.ajouter_bouclier(REMPART_BOUCLIER);
            let j = &self.etat.joueur;
            self.ev.gerbe("bouclier", j.x, 1.0, j.z);
            self.ev.son("bouclier");
        }
    }

    /// Juste avant que l'arme `i` de l'arsenal se déclenche (Élan critique, Séisme).
    pub(crate) fn guerrier_avant_declenchement(&mut self, i: usize) {
        let index = Some(self.arsenal.armes[i].index);
        let g = guerrier!(self);
        if index == g.epee && g.elan_pret {
            g.elan_pret = false;
            g.elan_applique = true;
            let s = &mut self.arsenal.armes[i].stats;
            s.zone *= ELAN_ZONE;
            s.degats *= ELAN_DEGATS;
        }
        if index == g.onde && g.r.seisme > 0 && g.ondes >= SEISME_ONDES {
            self.talents.classes.onde_circulaire = true;
        }
    }

    pub(crate) fn guerrier_apres_declenchement(&mut self, i: usize, declenchee: bool) {
        let index = Some(self.arsenal.armes[i].index);
        let circulaire = std::mem::take(&mut self.talents.classes.onde_circulaire);
        let g = guerrier!(self);
        if g.elan_applique && index == g.epee {
            g.elan_applique = false;
            let s = &mut self.arsenal.armes[i].stats;
            s.zone /= ELAN_ZONE;
            s.degats /= ELAN_DEGATS;
        }
        if index == g.onde && declenchee && g.r.seisme > 0 {
            g.ondes = if circulaire { 0 } else { g.ondes + 1 };
        }
    }

    /// Multiplicateur du délai de l'arme d'index `arme` (Enchaînement, Frénésie).
    pub(crate) fn guerrier_delai(&mut self, arme: usize) -> f32 {
        let melee = self.d.armes[arme].famille == Some(FamilleArme::Melee);
        let Some(g) = self.talents.classes.guerrier.as_mut() else {
            return 1.0;
        };
        let mut delai = 1.0;
        if Some(arme) == g.epee && g.enchainement_pret {
            g.enchainement_pret = false;
            delai *= ENCHAINEMENT_DELAI;
        }
        if g.frenesie > 0.0 && melee {
            let v = valeur(g.r.frenesie, &FRENESIE_DELAI);
            delai *= if v == 0.0 { 1.0 } else { v };
        }
        delai
    }

    /// Fureur, Exécution contre un boss presque vaincu.
    pub(crate) fn guerrier_multiplicateur(&self, i: usize, arme: Option<usize>) -> f32 {
        let Some(g) = &self.talents.classes.guerrier else {
            return 1.0;
        };
        let e = &self.ennemis.pool[i];
        let mut m = 1.0;
        if g.fureur > 0.0 {
            m += valeur(g.r.fureur, &FUREUR_DEGATS);
        }
        if arme == g.epee
            && e.def.boss.is_some()
            && e.pv < EXECUTION_SEUIL * e.pv_max
            && g.r.execution > 0
        {
            m += EXECUTION_BOSS;
        }
        m
    }

    /// Œil du bourreau : critiques plus forts contre un ennemi affaibli.
    pub(crate) fn guerrier_facteur_critique(&self, i: usize, base: f32) -> f32 {
        let Some(g) = &self.talents.classes.guerrier else {
            return base;
        };
        let e = &self.ennemis.pool[i];
        if e.pv < BOURREAU_SEUIL * e.pv_max {
            return base * (1.0 + valeur(g.r.bourreau, &BOURREAU_DEGATS));
        }
        base
    }

    pub(crate) fn guerrier_apres_coup(
        &mut self,
        i: usize,
        arme: Option<usize>,
        critique: bool,
        nature: Nature,
    ) {
        if nature != Nature::Arme {
            return;
        }
        let instant = self.etat.temps_jeu;
        let melee = self.famille(arme) == Some(FamilleArme::Melee);
        let g = guerrier!(self);
        // Enchaînement : un coup d'épée qui touche 3 ennemis
        if arme == g.epee && g.r.enchainement > 0 {
            if instant != g.instant_epee {
                g.instant_epee = instant;
                g.touches_epee = 0;
            }
            g.touches_epee += 1;
            if g.touches_epee == ENCHAINEMENT_TOUCHES {
                g.enchainement_pret = true;
            }
        }
        let saignee = critique && melee && g.r.saignee > 0;
        if critique && melee && g.r.elan > 0 && g.delai_elan <= 0.0 {
            g.elan_pret = true;
            g.delai_elan = ELAN_DELAI;
        }
        // Fureur du gladiateur : une attaque de mêlée qui touche (une fois par arme et par instant)
        let mut anneau = false;
        if melee
            && g.r.fureur_gladiateur > 0
            && (instant != g.instant_melee || arme != g.arme_melee)
        {
            g.instant_melee = instant;
            g.arme_melee = arme;
            g.attaques_melee += 1;
            if g.attaques_melee >= FUREUR_ATTAQUES && g.delai_fureur <= 0.0 {
                g.attaques_melee = 0;
                g.delai_fureur = FUREUR_DELAI;
                anneau = true;
            }
        }
        if saignee {
            let (cumuls, dps, duree) = SAIGNEE;
            self.saigner(i, cumuls, dps, duree, arme);
        }
        if anneau {
            self.anneau_fureur();
        }
        // Exécution : l'épée achève un ennemi normal affaibli
        let g = guerrier!(self);
        let e = &self.ennemis.pool[i];
        if arme == g.epee
            && e.vivant()
            && e.def.boss.is_none()
            && e.pv < EXECUTION_SEUIL * e.pv_max
            && g.delai_execution <= 0.0
            && g.r.execution > 0
        {
            g.delai_execution = EXECUTION_DELAI;
            let (ex, ez, pv) = (e.x, e.z, e.pv);
            self.ev.gerbe("impactLame", ex, 1.0, ez);
            let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
            self.blesser_reaction(i, pv, jx, jz, 0.0, arme, Nature::Arme);
        }
    }

    /// Soif de combat, Frénésie (assez d'éliminations dans la fenêtre).
    pub(crate) fn guerrier_sur_mort(&mut self, arme: Option<usize>) {
        let melee = self.famille(arme) == Some(FamilleArme::Melee);
        let g = guerrier!(self);
        let r = g.r;
        if melee && r.soif > 0 {
            self.soigner_plafonne(valeur(r.soif, &SOIF));
        }
        if r.frenesie == 0 {
            return;
        }
        let maintenant = self.etat.temps_jeu;
        let g = guerrier!(self);
        let champion = r.champion > 0;
        let seuil = if champion {
            CHAMPION_ELIMINATIONS
        } else {
            FRENESIE_ELIMINATIONS
        };
        g.eliminations[g.prochaine] = maintenant as f32;
        g.prochaine = (g.prochaine + 1) % g.eliminations.len();
        if g.frenesie > 0.0 {
            return;
        }
        let recentes = g
            .eliminations
            .iter()
            .filter(|&&t| t > 0.0 && maintenant - f64::from(t) <= FRENESIE_FENETRE)
            .count() as u32;
        if recentes < seuil {
            return;
        }
        g.eliminations = [0.0; 16];
        g.frenesie = if champion {
            CHAMPION_DUREE
        } else {
            FRENESIE_DUREE
        };
        g.fureur = 0.0;
        self.soigner(FRENESIE_SOIN);
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        self.ev.gerbe("critique", jx, 1.2, jz);
        self.cercle(jx, jz, 1.2, "anneau", 12);
    }

    /// Lame lourde : un ennemi projeté contre un mur prend des dégâts.
    pub(crate) fn guerrier_sur_impact_mur(&mut self, i: usize) {
        let g = guerrier!(self);
        let degats = valeur(g.r.lame_lourde, &LAME_LOURDE);
        let epee = g.epee;
        if degats <= 0.0 {
            return;
        }
        let (ex, ez) = (self.ennemis.pool[i].x, self.ennemis.pool[i].z);
        self.ev.gerbe("heurt", ex, 0.8, ez);
        self.blesser_reaction(i, degats, ex, ez, 0.0, epee, Nature::Arme);
    }

    /// Fureur du gladiateur : anneau autour du Guerrier, 60 % des dégâts de l'épée.
    fn anneau_fureur(&mut self) {
        let epee = guerrier!(self).epee;
        let degats_epee = self
            .arsenal
            .armes
            .iter()
            .rev()
            .find(|a| Some(a.index) == epee)
            .map(|a| a.stats.degats)
            .unwrap_or(0.0);
        if degats_epee <= 0.0 {
            return;
        }
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let r = FUREUR_RAYON;
        self.blesser_zone_reaction((jx, jz, r), degats_epee * FUREUR_PART, epee, 4.0);
        self.cercle(jx, jz, r, "anneau", 32);
        self.cercle(jx, jz, r * 0.6, "anneau", 20);
        self.ev.son("balayage");
    }
}
