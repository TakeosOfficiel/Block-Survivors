//! Points d'accroche des talents dans la simulation (port de src/combat/talents/talents.ts
//! et objets.ts) : état infligé par l'éveil de chaque arme, effets propres de l'équipement
//! (cloche, couronne, bastion, filon, spores, magma) et coup qui enflamme.
//!
//! Les parties du robot partent d'une sauvegarde vierge : les arbres de talents de classe
//! sont au rang 0, où leurs mécaniques sont neutres. Seule exception : le Venin de
//! l'Archère tire dans l'aléa à chaque coup de flèche, même sans le talent ; on garde ce
//! tirage. Les dégâts des talents et des objets sont des « réactions » : ils ne relancent
//! ni les talents ni les objets.

use bs_donnees::armes::{EtatEveil, Eveil, FamilleArme, TypeEtat};
use bs_donnees::equipement::EffetObjet;

use crate::ennemis::Nature;
use crate::partie::Partie;

/// Brûlure d'un coup qui enflamme : dégâts par seconde (× dégâts des armes) et durée.
const BRULURE_OBJET_DPS: f32 = 3.0;
const BRULURE_OBJET_DUREE: f32 = 3.0;
/// Recul des explosions des objets.
const RECUL_EXPLOSION: f32 = 3.0;
/// Les spores d'un ennemi tué ne partent jamais plus d'une fois sur deux.
const SPORES_CHANCE_MAX: f32 = 0.5;
const EFFETS_MAX: usize = 8;

#[derive(Debug, Clone)]
pub struct Talents {
    /// État infligé par l'éveil de chaque arme (par index d'arme).
    etats_eveils: Vec<Option<&'static EtatEveil>>,
    /// Dégâts causés par un talent ou un objet en cours : ils ne relancent rien.
    en_reaction: bool,
    /// Classe jouée : l'Archère (tirage du Venin).
    archere: bool,
    /// Effets propres des objets portés et facteur de leur niveau (rempli par l'équipement).
    pub effets: Vec<(&'static EffetObjet, f32)>,
    /// s depuis la dernière cloche, la dernière couronne ; s avant le prochain Cœur de magma.
    cloche: f32,
    couronne: f32,
    recharge_magma: f32,
}

impl Talents {
    pub fn nouveaux(armes: usize, archere: bool) -> Self {
        Self {
            etats_eveils: vec![None; armes],
            en_reaction: false,
            archere,
            effets: Vec::with_capacity(EFFETS_MAX),
            cloche: 0.0,
            couronne: 0.0,
            recharge_magma: 0.0,
        }
    }

    /// L'arme d'index `arme` vient d'être éveillée.
    pub fn definir_eveil(&mut self, arme: usize, eveil: &'static Eveil) {
        self.etats_eveils[arme] = eveil.etat.as_ref();
    }
}

impl Partie {
    /// Multiplicateur de dégâts contre l'ennemi `i` (Amulette du bastion).
    pub fn talent_multiplicateur(&self, _i: usize, _arme: Option<usize>, _nature: Nature) -> f32 {
        let j = &self.etat.joueur;
        let mut m = 1.0;
        for &(effet, facteur) in &self.talents.effets {
            if let EffetObjet::Bastion { seuil, bonus } = *effet
                && j.pv < seuil * j.pv_max
            {
                m *= 1.0 + bonus * facteur;
            }
        }
        m
    }
    pub fn talent_bonus_critique(&self, _i: usize, _arme: usize, _nature: Nature) -> f32 {
        0.0
    }
    pub fn talent_facteur_critique(
        &self,
        _i: usize,
        _arme: Option<usize>,
        _nature: Nature,
        base: f32,
    ) -> f32 {
        base
    }

    /// Coup porté par une arme ou un allié : état de l'éveil, Venin, coup qui enflamme.
    pub fn talent_apres_coup(
        &mut self,
        i: usize,
        _degats: f32,
        arme: Option<usize>,
        _critique: bool,
        nature: Nature,
    ) {
        if self.talents.en_reaction {
            return;
        }
        let Some(a) = arme else {
            return;
        };
        // État de l'éveil de l'arme (Aura de feu, Haches givrées…)
        if matches!(nature, Nature::Arme | Nature::Allie)
            && self.ennemis.pool[i].vivant()
            && let Some(etat) = self.talents.etats_eveils[a]
        {
            match etat.genre {
                TypeEtat::Brulure => self.bruler(i, 1.0, etat.valeur, etat.duree, arme),
                TypeEtat::Poison => self.empoisonner(i, 1.0, etat.valeur, etat.duree, arme),
                TypeEtat::Ralenti => self.ralentir(i, etat.valeur, etat.duree),
                TypeEtat::Etourdi => self.etourdir(i, etat.valeur),
            }
        }
        if nature != Nature::Arme || !self.ennemis.pool[i].vivant() {
            return;
        }
        // Venin (rang 0 : le tirage a lieu, jamais le poison)
        if self.talents.archere && self.d.armes[a].famille == Some(FamilleArme::Projectile) {
            self.alea.tirer();
        }
        // Coup qui enflamme (Bague de magma, garnitures du Nether)
        let chance = self.etat.modificateurs.chance_brulure;
        if chance <= 0.0 || self.alea.tirer() >= f64::from(chance) {
            return;
        }
        let dps = BRULURE_OBJET_DPS * self.etat.modificateurs.degats;
        self.bruler(i, 1.0, dps, BRULURE_OBJET_DUREE, arme);
    }

    /// Ennemi tué : éclat de diamant (Cœur du filon), spores (Amulette de spores).
    pub fn talent_sur_mort(&mut self, i: usize, _arme: Option<usize>, _nature: Nature) {
        if self.talents.en_reaction {
            return;
        }
        let (x, z) = (self.ennemis.pool[i].x, self.ennemis.pool[i].z);
        let degats_armes = self.etat.modificateurs.degats;
        for k in 0..self.talents.effets.len() {
            let (effet, facteur) = self.talents.effets[k];
            match *effet {
                EffetObjet::Filon {
                    chance,
                    rayon,
                    degats,
                } => {
                    if self.alea.tirer() < f64::from(chance) {
                        self.exploser_objet(x, z, rayon, degats * degats_armes, "critique");
                        self.ev.gerbe("eboulis", x, 0.8, z);
                        self.ev.son("explosion");
                    }
                }
                EffetObjet::Spores {
                    chance,
                    rayon,
                    poison,
                    duree,
                } => {
                    let chance = SPORES_CHANCE_MAX.min(chance * facteur);
                    if self.alea.tirer() < f64::from(chance) {
                        self.empoisonner_zone(x, z, rayon, poison * degats_armes, duree);
                        self.ev.gerbe("nuage", x, 0.6, z);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn talent_sur_impact_mur(&mut self, _i: usize) {}
    pub fn talent_sur_sortie_piege(&mut self, _i: usize) {}
    /// Multiplicateur des dégâts de l'arme qui se déclenche (talents « au déclenchement »).
    pub fn talent_facteur_declenchement(&self) -> f32 {
        1.0
    }
    /// Multiplicateur du délai entre deux déclenchements de l'arme `_arme`.
    pub fn talent_delai(&self, _arme: usize) -> f32 {
        self.d.regles.plafonds.delai_min.max(1.0)
    }
    pub fn talent_vitesse(&self) -> f32 {
        1.0
    }
    pub fn talent_degats_recus(&mut self, degats: f32, _boss: bool) -> f32 {
        degats
    }
    pub fn talent_armure(&self) -> f32 {
        0.0
    }
    pub fn talent_eviter_mort(&mut self) -> bool {
        false
    }

    /// Gros coup reçu : nova de feu et protection (Cœur de magma).
    pub fn talent_apres_coup_recu(&mut self, subis: f32, _evites: f32) {
        if self.talents.recharge_magma > 0.0 {
            return;
        }
        for k in 0..self.talents.effets.len() {
            let EffetObjet::Magma {
                seuil,
                rayon,
                degats,
                protection,
                recharge,
            } = *self.talents.effets[k].0
            else {
                continue;
            };
            let j = &mut self.etat.joueur;
            if subis < seuil * j.pv_max {
                continue;
            }
            self.talents.recharge_magma = recharge;
            j.protection = j.protection.max(protection);
            let (x, z) = (j.x, j.z);
            let degats_armes = self.etat.modificateurs.degats;
            self.exploser_objet(x, z, rayon, degats * degats_armes, "brulure");
            let mut tampon = [0u32; 256];
            let n = self.ennemis.chercher(x, z, rayon, &mut tampon);
            let dps = BRULURE_OBJET_DPS * degats_armes;
            for &e in &tampon[..n] {
                self.bruler(e as usize, 1.0, dps, BRULURE_OBJET_DUREE, None);
            }
            self.ev.son("nova");
            return;
        }
    }

    /// Effets périodiques des objets (cloche, couronne), juste après l'arsenal.
    pub fn maj_talents(&mut self, dt: f32) {
        if self.etat.joueur.mort || self.etat.issue.is_some() {
            return;
        }
        if self.talents.recharge_magma > 0.0 {
            self.talents.recharge_magma -= dt;
        }
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        for k in 0..self.talents.effets.len() {
            let (effet, facteur) = self.talents.effets[k];
            match *effet {
                EffetObjet::Cloche {
                    intervalle,
                    rayon,
                    force,
                    etourdi,
                } => {
                    self.talents.cloche += dt;
                    if self.talents.cloche < intervalle / facteur {
                        continue;
                    }
                    self.talents.cloche = 0.0;
                    self.repousser_ennemis(jx, jz, rayon, force);
                    let mut tampon = [0u32; 256];
                    let n = self.ennemis.chercher(jx, jz, rayon, &mut tampon);
                    for &e in &tampon[..n] {
                        self.etourdir(e as usize, etourdi);
                    }
                    self.ev.son("bouclier");
                }
                EffetObjet::Couronne {
                    intervalle,
                    rayon,
                    degats,
                    poison,
                    duree,
                } => {
                    self.talents.couronne += dt;
                    if self.talents.couronne < intervalle {
                        continue;
                    }
                    self.talents.couronne = 0.0;
                    let degats = degats * self.etat.modificateurs.degats;
                    self.exploser_objet(jx, jz, rayon, degats, "nuage");
                    self.empoisonner_zone(jx, jz, rayon, poison, duree);
                    self.ev.son("explosion");
                    self.ev.secousse(0.3);
                }
                _ => {}
            }
        }
    }

    /// Explosion d'un objet : dégâts de zone (réaction, sans arme).
    fn exploser_objet(&mut self, x: f32, z: f32, rayon: f32, degats: f32, gerbe: &'static str) {
        let mut tampon = [0u32; 256];
        let n = self.ennemis.chercher(x, z, rayon, &mut tampon);
        let avant = self.talents.en_reaction;
        self.talents.en_reaction = true;
        for &e in &tampon[..n] {
            self.blesser(
                e as usize,
                degats,
                x,
                z,
                RECUL_EXPLOSION,
                None,
                Nature::Arme,
            );
        }
        self.talents.en_reaction = avant;
        self.ev.gerbe(gerbe, x, 0.8, z);
    }

    fn empoisonner_zone(&mut self, x: f32, z: f32, rayon: f32, dps: f32, duree: f32) {
        let mut tampon = [0u32; 256];
        let n = self.ennemis.chercher(x, z, rayon, &mut tampon);
        for &e in &tampon[..n] {
            self.empoisonner(e as usize, 1.0, dps, duree, None);
        }
    }
}
