//! Mécaniques des talents du Mage (src/combat/talents/mage.ts).
//! Pyromancien : Cœur de blaze, brûlures (Propagation, Brasier, Fournaise, Cendres
//! protectrices), Embrasement, Phénix. Lave en fusion, Tempête de feu et Pluie de feu sont
//! lues par le système des météores (combat/feu.rs).
//! Arcaniste : Résonance, Surcharge, Accélération, Tempête d'orage, Écho arcanique, Zéro
//! absolu. Haute tension, Conduction, Orbes givrées et Flux arcanique sont lus par les
//! systèmes des éclairs et des orbes (combat/foudre.rs, combat/traque.rs).

use bs_donnees::Donnees;

use super::valeur;
use crate::combat::arme::ArmeEquipee;
use crate::partie::Partie;
use crate::talents::Talents;

const COEUR_BLAZE: (f32, f32, f32) = (1.0, 3.0, 3.0);
const CENDRES_BOUCLIER: f32 = 1.0;
const CENDRES_DELAI: f32 = 1.0;
const PROPAGATION_VOISINS: [f32; 2] = [1.0, 2.0];
const PROPAGATION_PART: [f32; 2] = [0.5, 1.0];
const PROPAGATION_RAYON: f32 = 3.0;
const BRASIER: f32 = 0.3;
const FOURNAISE: f32 = 0.08;
const EMBRASEMENT_BRULANTS: usize = 15;
const EMBRASEMENT_MULTIPLICATEUR: f32 = 1.5;
const EMBRASEMENT_RAYON: f32 = 2.0;
const EMBRASEMENT_DELAI: f32 = 10.0;
const EMBRASEMENT_PERIODE: f32 = 0.25;
const PHENIX_PV: f32 = 0.4;
const PHENIX_RAYON: f32 = 5.0;
const PHENIX_DEGATS: f32 = 40.0;
const PHENIX_PROTECTION: f32 = 2.0;
const PHENIX_FORCE: f32 = 14.0;
/// Brûlure du Phénix : cumuls, dégâts par seconde, durée (s).
const PHENIX_BRULURE: (f32, f32, f32) = (2.0, 3.0, 4.0);
const RESONANCE_FENETRE: f32 = 3.0;
const RESONANCE_PAR_SORT: f32 = 0.05;
const RESONANCE_MAX: f32 = 0.15;
const SURCHARGE: [f32; 2] = [0.3, 0.5];
const ACCELERATION_SORTS: u32 = 10;
const ACCELERATION_DELAI: f32 = 0.8;
const ACCELERATION_DUREE: [f32; 2] = [2.0, 3.0];
const ORAGE_PERIODE: f32 = 20.0;
const ORAGE_ECLAIRS: u32 = 8;
const ORAGE_DUREE: f32 = 2.0;
const ORAGE_PORTEE: f32 = 10.0;
const ECHO_SORTS: u32 = 6;
const ECHO_DELAI: f32 = 0.5;
/// Part des dégâts du sort répété par l'Écho arcanique.
pub const ECHO_PART: f32 = 0.6;
const ECHO_DELAI_INTERNE: f32 = 3.0;
const ZERO_ABSOLU_NOVAS: u32 = 4;
/// Sorts du Mage.
const SORTS: [&str; 6] = [
    "boules_feu",
    "eclairs",
    "tornade_feu",
    "nova_glace",
    "meteore",
    "orbes_arcaniques",
];

/// Rangs des talents du Mage, lus une fois au début de la partie.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rangs {
    pub coeur_blaze: u32,
    pub cendres: u32,
    pub propagation: u32,
    pub brasier: u32,
    pub fournaise: u32,
    pub embrasement: u32,
    pub phenix: u32,
    pub resonance: u32,
    pub surcharge: u32,
    pub acceleration: u32,
    pub orage: u32,
    pub echo: u32,
    pub zero_absolu: u32,
}

impl Rangs {
    pub fn lire(t: &Talents) -> Self {
        Self {
            coeur_blaze: t.rang("coeur_blaze"),
            cendres: t.rang("cendres_protectrices"),
            propagation: t.rang("propagation"),
            brasier: t.rang("brasier"),
            fournaise: t.rang("fournaise"),
            embrasement: t.rang("embrasement"),
            phenix: t.rang("phenix"),
            resonance: t.rang("resonance"),
            surcharge: t.rang("surcharge"),
            acceleration: t.rang("acceleration"),
            orage: t.rang("tempete_orage"),
            echo: t.rang("echo_arcanique"),
            zero_absolu: t.rang("zero_absolu"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mage {
    pub r: Rangs,
    boules: Option<usize>,
    eclairs: Option<usize>,
    nova: Option<usize>,
    /// Index des sorts dans le catalogue des armes.
    sorts: [Option<usize>; 6],
    /// Temps de jeu du dernier lancer de chaque sort (par index de `sorts`).
    derniers_sorts: [f32; 6],
    delai_cendres: f32,
    minuterie_embrasement: f32,
    delai_embrasement: f32,
    phenix_utilise: bool,
    sorts_acceleration: u32,
    acceleration: f32,
    minuterie_orage: f32,
    eclairs_orage: u32,
    prochain_eclair: f32,
    sorts_echo: u32,
    echo_pret: bool,
    delai_echo: f32,
    /// Sort à répéter (index dans le catalogue) et s avant la répétition.
    echo_arme: Option<usize>,
    echo_minuterie: f32,
    novas: u32,
    gel_applique: bool,
}

impl Mage {
    pub fn nouveau(d: &Donnees) -> Self {
        Self {
            r: Rangs::default(),
            boules: d.armes.get_index_of("boules_feu"),
            eclairs: d.armes.get_index_of("eclairs"),
            nova: d.armes.get_index_of("nova_glace"),
            sorts: SORTS.map(|id| d.armes.get_index_of(id)),
            derniers_sorts: [-1e9; 6],
            delai_cendres: 0.0,
            minuterie_embrasement: 0.0,
            delai_embrasement: 0.0,
            phenix_utilise: false,
            sorts_acceleration: 0,
            acceleration: 0.0,
            minuterie_orage: ORAGE_PERIODE,
            eclairs_orage: 0,
            prochain_eclair: 0.0,
            sorts_echo: 0,
            echo_pret: false,
            delai_echo: 0.0,
            echo_arme: None,
            echo_minuterie: 0.0,
            novas: 0,
            gel_applique: false,
        }
    }

    /// Place de l'arme `arme` parmi les sorts du Mage.
    fn sort(&self, arme: usize) -> Option<usize> {
        self.sorts.iter().position(|&s| s == Some(arme))
    }
}

macro_rules! mage {
    ($p:expr) => {
        match $p.talents.classes.mage.as_mut() {
            Some(m) => m,
            None => return Default::default(),
        }
    };
}

mod coups;

impl Partie {
    pub(crate) fn mage_maj(&mut self, dt: f32) {
        let m = mage!(self);
        m.delai_cendres -= dt;
        m.delai_embrasement -= dt;
        m.delai_echo -= dt;
        if m.acceleration > 0.0 {
            m.acceleration -= dt;
        }
        // Embrasement : 15 ennemis en feu explosent ensemble
        if m.r.embrasement > 0 {
            m.minuterie_embrasement -= dt;
            if m.minuterie_embrasement <= 0.0 {
                m.minuterie_embrasement = EMBRASEMENT_PERIODE;
                if m.delai_embrasement <= 0.0 {
                    self.embrasement();
                }
            }
        }
        // Tempête d'orage : 8 éclairs en 2 s toutes les 20 s
        let m = mage!(self);
        if m.r.orage > 0 {
            m.minuterie_orage -= dt;
            if m.minuterie_orage <= 0.0 {
                m.minuterie_orage = ORAGE_PERIODE;
                m.eclairs_orage = ORAGE_ECLAIRS;
                m.prochain_eclair = 0.0;
            }
            if m.eclairs_orage > 0 {
                m.prochain_eclair -= dt;
                if m.prochain_eclair <= 0.0 {
                    m.prochain_eclair = ORAGE_DUREE / ORAGE_ECLAIRS as f32;
                    m.eclairs_orage -= 1;
                    self.eclair_orage();
                }
            }
        }
        // Écho arcanique : le sort se répète, plus faible
        let m = mage!(self);
        if let Some(arme) = m.echo_arme {
            m.echo_minuterie -= dt;
            if m.echo_minuterie <= 0.0 {
                m.echo_arme = None;
                if let Some(i) = self.arsenal.armes.iter().position(|a| a.index == arme) {
                    self.talents.classes.echo = true;
                    self.declencher(i);
                    self.talents.classes.echo = false;
                }
            }
        }
    }

    pub(crate) fn mage_avant_declenchement(&mut self, i: usize) {
        let index = self.arsenal.armes[i].index;
        let m = mage!(self);
        if Some(index) == m.nova && m.r.zero_absolu > 0 && m.novas >= ZERO_ABSOLU_NOVAS {
            m.gel_applique = true;
            self.talents.classes.nova_gel = true;
        }
    }

    pub(crate) fn mage_apres_declenchement(&mut self, i: usize, declenchee: bool) {
        let index = self.arsenal.armes[i].index;
        let temps = self.etat.temps_jeu;
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let m = mage!(self);
        if Some(index) == m.nova && m.r.zero_absolu > 0 {
            if declenchee {
                m.novas = if m.gel_applique { 0 } else { m.novas + 1 };
            }
            m.gel_applique = false;
            self.talents.classes.nova_gel = false;
        }
        let m = mage!(self);
        let Some(sort) = m.sort(index).filter(|_| declenchee) else {
            return;
        };
        // Comme le Float32Array du jeu TS : temps arrondi en f32
        m.derniers_sorts[sort] = temps as f32;
        // Accélération : tous les 10 sorts
        if m.r.acceleration > 0 {
            m.sorts_acceleration += 1;
            if m.sorts_acceleration >= ACCELERATION_SORTS {
                m.sorts_acceleration = 0;
                m.acceleration = valeur(m.r.acceleration, &ACCELERATION_DUREE);
            }
        }
        // Écho arcanique : après 6 sorts, le suivant se répète 0,5 s plus tard
        if m.r.echo > 0 {
            if m.echo_pret && m.delai_echo <= 0.0 && m.echo_arme.is_none() {
                m.echo_pret = false;
                m.delai_echo = ECHO_DELAI_INTERNE;
                m.echo_arme = Some(index);
                m.echo_minuterie = ECHO_DELAI;
                self.ev.gerbe("impactArcane", jx, 1.2, jz);
            } else if !m.echo_pret {
                m.sorts_echo += 1;
                if m.sorts_echo >= ECHO_SORTS {
                    m.sorts_echo = 0;
                    m.echo_pret = true;
                }
            }
        }
    }

    /// Accélération : les sorts reviennent plus vite.
    pub(crate) fn mage_delai(&self, arme: usize) -> f32 {
        match &self.talents.classes.mage {
            Some(m) if m.acceleration > 0.0 && m.sort(arme).is_some() => ACCELERATION_DELAI,
            _ => 1.0,
        }
    }

    /// Embrasement : chaque ennemi en feu explose (1,5 × ce qui reste de sa brûlure).
    fn embrasement(&mut self) {
        let mut brulants = [0u32; 256];
        let mut n = 0;
        for (k, e) in self.ennemis.actifs().iter().enumerate() {
            if n < brulants.len() && e.vivant() && e.brulure.cumuls > 0.0 {
                brulants[n] = k as u32;
                n += 1;
            }
        }
        if n < EMBRASEMENT_BRULANTS {
            return;
        }
        mage!(self).delai_embrasement = EMBRASEMENT_DELAI;
        for &k in &brulants[..n] {
            let e = &mut self.ennemis.pool[k as usize];
            let b = &mut e.brulure;
            if b.cumuls == 0.0 {
                continue;
            }
            let restant =
                b.degats_par_seconde * b.cumuls * b.temps.max(0.0) * EMBRASEMENT_MULTIPLICATEUR;
            let arme = b.arme;
            b.cumuls = 0.0;
            let (x, z) = (e.x, e.z);
            self.ev.gerbe("explosionArme", x, 0.8, z);
            self.blesser_zone_reaction((x, z, EMBRASEMENT_RAYON), restant, arme, 3.0);
        }
        self.ev.son("explosion");
        self.ev.secousse(0.5);
    }

    /// Un éclair de la Tempête d'orage : les dégâts de l'arme Éclairs (rang actuel, ou
    /// rang 1).
    fn eclair_orage(&mut self) {
        let Some(eclairs) = mage!(self).eclairs else {
            return;
        };
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let Some(c) = self.ennemi_au_hasard(jx, jz, ORAGE_PORTEE) else {
            return;
        };
        let stats = match self.arsenal.armes.iter().find(|a| a.index == eclairs) {
            Some(a) => a.stats,
            None => {
                let m = self.etat.modificateurs;
                ArmeEquipee::nouvelle(self.d, eclairs, &m, self.talents.armes[eclairs]).stats
            }
        };
        let (x, z) = (self.ennemis.pool[c].x, self.ennemis.pool[c].z);
        self.eclair_talent(eclairs, (x, z), stats.degats, stats.zone);
    }
}
