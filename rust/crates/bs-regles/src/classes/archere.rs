//! Mécaniques des talents de l'Archère (src/combat/talents/archer.ts).
//! Élite : Marque du chasseur, Premier sang, Point faible, Puissance (flèche géante), Œil
//! de l'aigle (ricochets critiques), Exécutrice. Visée stable, Perforation nette, Tir
//! groupé et Tir rebondissant sont lus par les systèmes d'armes (projectiles, traque).
//! Rôdeuse : mouvement (Pas du chasseur, Course chargée, Concentration), Venin, pièges
//! (Piège à loups, Ronces venimeuses, lues aussi par pose.rs), Contagion, Nuage toxique.

use bs_commun::maths::{cos, sin};
use bs_donnees::Donnees;
use bs_donnees::armes::{FamilleArme, Mecanique};

use super::valeur;
use crate::ennemis::Nature;
use crate::partie::Partie;
use crate::talents::Talents;

const PREMIER_SANG_SEUIL: f32 = 0.9;
const PREMIER_SANG: [f32; 2] = [0.2, 0.35];
const MARQUE_DUREE: f32 = 4.0;
const MARQUE: [f32; 2] = [0.1, 0.2];
const POINT_FAIBLE: [f32; 2] = [0.1, 0.15];
const CONCENTRATION_IMMOBILE: f32 = 1.0;
const CONCENTRATION: [f32; 2] = [0.92, 0.85];
const PUISSANCE_TIRS: u32 = 12;
const PUISSANCE_MULTIPLICATEUR: f32 = 3.0;
const PUISSANCE_AJOUT: f32 = 20.0;
const PUISSANCE_PORTEE: f32 = 30.0;
const PUISSANCE_DELAI: f32 = 4.0;
const OEIL_PORTEE: f32 = 14.0;
const OEIL_PART: f32 = 1.0;
const EXECUTRICE_SEUIL: f32 = 0.1;
const EXECUTRICE_PAR_CRITIQUE: f32 = 0.02;
const EXECUTRICE_CUMULS: f32 = 10.0;
const EXECUTRICE_DUREE: f32 = 6.0;
const PAS_CHASSEUR_MOUVEMENT: f32 = 2.0;
const PAS_CHASSEUR_VITESSE: f32 = 1.1;
const VENIN_CHANCE: [f32; 2] = [0.2, 0.35];
const VENIN_DPS: f32 = 2.0;
const VENIN_DUREE: f32 = 4.0;
const CONCENTRE: (f32, f32) = (1.5, 3.0);
const VIRULENT: (f32, f32) = (0.8, 8.0);
const CONTAGION: [f32; 2] = [1.0, 2.0];
const CONTAGION_RAYON: f32 = 3.0;
const COURSE_DISTANCE: f32 = 1.5;
const COURSE_CHARGES: u32 = 10;
const COURSE_CHARGES_SECOND_VENT: u32 = 7;
const COURSE_FLECHES: f32 = 3.0;
const COURSE_DEGATS: [f32; 2] = [0.15, 0.3];
const PIEGE_LOUPS_RALENTI: f32 = 0.3;
const PIEGE_LOUPS: [f32; 2] = [1.0, 2.0];
pub const RONCES_VENIMEUSES: f32 = 0.3;
const NUAGE_DUREE: f32 = 2.0;
const NUAGE_RAYON: f32 = 2.0;
const NUAGE_PERIODE: f32 = 0.5;
const NUAGE_DELAI: f32 = 1.0;
const NUAGE_MAX: usize = 3;

/// Rangs des talents de l'Archère, lus une fois au début de la partie.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rangs {
    pub premier_sang: u32,
    pub marque: u32,
    pub point_faible: u32,
    pub concentration: u32,
    pub puissance: u32,
    pub oeil_aigle: u32,
    pub executrice: u32,
    pub pas_chasseur: u32,
    pub venin: u32,
    pub concentre: u32,
    pub virulent: u32,
    pub contagion: u32,
    pub course: u32,
    pub second_vent: u32,
    pub vent_arriere: u32,
    pub piege_loups: u32,
    pub ronces_venimeuses: u32,
    pub nuage: u32,
}

impl Rangs {
    pub fn lire(t: &Talents) -> Self {
        Self {
            premier_sang: t.rang("premier_sang"),
            marque: t.rang("marque_chasseur"),
            point_faible: t.rang("point_faible"),
            concentration: t.rang("concentration"),
            puissance: t.rang("puissance"),
            oeil_aigle: t.rang("oeil_aigle"),
            executrice: t.rang("executrice"),
            pas_chasseur: t.rang("pas_chasseur"),
            venin: t.rang("venin"),
            concentre: t.rang("venin_concentre"),
            virulent: t.rang("venin_virulent"),
            contagion: t.rang("contagion"),
            course: t.rang("course_chargee"),
            second_vent: t.rang("second_vent"),
            vent_arriere: t.rang("vent_arriere"),
            piege_loups: t.rang("piege_a_loups"),
            ronces_venimeuses: t.rang("ronces_venimeuses"),
            nuage: t.rang("nuage_toxique"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Nuage {
    x: f32,
    z: f32,
    temps: f32,
    tic: f32,
    arme: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Archere {
    pub r: Rangs,
    arc: Option<usize>,
    tirs: u32,
    delai_puissance: f32,
    puissance_appliquee: bool,
    charges: u32,
    dernier_parcouru: f32,
    course_appliquee: bool,
    nuages: Vec<Nuage>,
    delai_nuage: f32,
}

impl Archere {
    pub fn nouvelle(d: &Donnees) -> Self {
        Self {
            r: Rangs::default(),
            arc: d.armes.get_index_of("arc"),
            tirs: 0,
            delai_puissance: 0.0,
            puissance_appliquee: false,
            charges: 0,
            dernier_parcouru: 0.0,
            course_appliquee: false,
            nuages: Vec::with_capacity(NUAGE_MAX),
            delai_nuage: 0.0,
        }
    }

    fn charges_max(&self) -> u32 {
        if self.r.second_vent > 0 {
            COURSE_CHARGES_SECOND_VENT
        } else {
            COURSE_CHARGES
        }
    }

    /// Cumuls de poison au maximum (Venin concentré ou virulent).
    fn cumuls_poison(&self, defaut: f32) -> f32 {
        if self.r.concentre > 0 {
            CONCENTRE.1
        } else if self.r.virulent > 0 {
            VIRULENT.1
        } else {
            defaut
        }
    }
}

macro_rules! archere {
    ($p:expr) => {
        match $p.talents.classes.archere.as_mut() {
            Some(a) => a,
            None => return Default::default(),
        }
    };
}

mod coups;

impl Partie {
    pub(crate) fn archere_maj(&mut self, dt: f32) {
        let parcouru = self.talents.classes.parcouru;
        let a = archere!(self);
        a.delai_puissance -= dt;
        a.delai_nuage -= dt;
        // Course chargée : une charge tous les 1,5 bloc parcourus
        if a.r.course > 0 {
            while parcouru - a.dernier_parcouru >= COURSE_DISTANCE {
                a.dernier_parcouru += COURSE_DISTANCE;
                if a.charges < a.charges_max() {
                    a.charges += 1;
                }
            }
        } else {
            a.dernier_parcouru = parcouru;
        }
        // Nuages toxiques : un cumul de poison aux ennemis dedans toutes les 0,5 s
        let mut k = a.nuages.len();
        while k > 0 {
            k -= 1;
            let a = archere!(self);
            let n = &mut a.nuages[k];
            n.temps -= dt;
            if n.temps <= 0.0 {
                a.nuages.swap_remove(k);
                continue;
            }
            n.tic -= dt;
            if n.tic > 0.0 {
                continue;
            }
            n.tic += NUAGE_PERIODE;
            let n = *n;
            let mut tampon = [0u32; 128];
            let touches = self.ennemis.chercher(n.x, n.z, NUAGE_RAYON, &mut tampon);
            for &e in &tampon[..touches] {
                self.archere_empoisonner(e as usize, 1.0, n.arme);
            }
            for _ in 0..6 {
                let angle = (self.alea.tirer() * std::f64::consts::TAU) as f32;
                let r = (self.alea.tirer().sqrt() * f64::from(NUAGE_RAYON)) as f32;
                self.ev
                    .gerbe("nuage", n.x + sin(angle) * r, 0.5, n.z + cos(angle) * r);
            }
        }
    }

    pub(crate) fn archere_avant_declenchement(&mut self, i: usize) {
        let arme = self.arsenal.armes[i];
        let a = archere!(self);
        // Puissance : après 12 tirs, la prochaine flèche de l'Arc devient géante
        if Some(arme.index) == a.arc
            && a.r.puissance > 0
            && a.tirs >= PUISSANCE_TIRS
            && a.delai_puissance <= 0.0
        {
            a.puissance_appliquee = true;
            let c = &mut self.talents.classes;
            c.fleche_geante = arme.stats.degats * PUISSANCE_MULTIPLICATEUR + PUISSANCE_AJOUT;
            c.portee_fleche_geante = PUISSANCE_PORTEE;
            return;
        }
        // Course chargée : la volée pleine tire 3 flèches de plus, plus fortes
        if matches!(arme.def.mecanique, Mecanique::Tir { .. })
            && a.r.course > 0
            && a.charges >= a.charges_max()
        {
            a.course_appliquee = true;
            let (course, vent_arriere) = (a.r.course, a.r.vent_arriere);
            let s = &mut self.arsenal.armes[i].stats;
            s.nombre += COURSE_FLECHES;
            s.degats *= 1.0 + valeur(course, &COURSE_DEGATS);
            self.talents.classes.volee_arriere = vent_arriere > 0;
        }
    }

    pub(crate) fn archere_apres_declenchement(&mut self, i: usize, declenchee: bool) {
        let index = self.arsenal.armes[i].index;
        let projectile = self.famille(Some(index)) == Some(FamilleArme::Projectile);
        let a = archere!(self);
        if a.puissance_appliquee {
            a.puissance_appliquee = false;
            if declenchee {
                a.tirs = 0;
                a.delai_puissance = PUISSANCE_DELAI;
                self.ev.son("fusee");
            }
            self.talents.classes.fleche_geante = 0.0;
        } else if declenchee && projectile {
            a.tirs += 1;
        }
        let a = archere!(self);
        if a.course_appliquee {
            a.course_appliquee = false;
            if declenchee {
                a.charges = 0;
            }
            let course = a.r.course;
            let s = &mut self.arsenal.armes[i].stats;
            s.nombre -= COURSE_FLECHES;
            s.degats /= 1.0 + valeur(course, &COURSE_DEGATS);
            self.talents.classes.volee_arriere = false;
        }
    }

    /// Concentration : les armes de tir vont plus vite à l'arrêt.
    pub(crate) fn archere_delai(&self, arme: usize) -> f32 {
        let Some(a) = &self.talents.classes.archere else {
            return 1.0;
        };
        if self.famille(Some(arme)) == Some(FamilleArme::Projectile)
            && self.talents.classes.temps_immobile >= CONCENTRATION_IMMOBILE
        {
            let v = valeur(a.r.concentration, &CONCENTRATION);
            return if v == 0.0 { 1.0 } else { v };
        }
        1.0
    }

    /// Pas du chasseur : plus vite après 2 s de marche.
    pub(crate) fn archere_vitesse(&self) -> f32 {
        match &self.talents.classes.archere {
            Some(a)
                if a.r.pas_chasseur > 0
                    && self.talents.classes.temps_mouvement >= PAS_CHASSEUR_MOUVEMENT =>
            {
                PAS_CHASSEUR_VITESSE
            }
            _ => 1.0,
        }
    }

    /// Cumuls de poison au maximum (Venin concentré ou virulent).
    pub fn talent_cumuls_poison(&self) -> f32 {
        let defaut = self.c.etats.poison.cumuls_max.unwrap_or(1) as f32;
        match &self.talents.classes.archere {
            Some(a) => a.cumuls_poison(defaut),
            None => defaut,
        }
    }

    /// Poison du Venin (options comprises).
    fn archere_empoisonner(&mut self, i: usize, cumuls: f32, arme: Option<usize>) {
        let Some(a) = &self.talents.classes.archere else {
            return;
        };
        let mut dps = VENIN_DPS;
        if a.r.concentre > 0 {
            dps *= CONCENTRE.0;
        }
        if a.r.virulent > 0 {
            dps *= VIRULENT.0;
        }
        let max = self.talent_cumuls_poison();
        self.empoisonner_max(i, cumuls, dps, VENIN_DUREE, arme, max);
    }
}
