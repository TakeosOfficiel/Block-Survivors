//! Alliés invoqués par les armes : ils combattent au contact, tirent, soutiennent le
//! joueur ou posent une aura (port de src/combat/invocations.ts, sans les talents de
//! l'Invocateur ni l'affichage). Les ennemis les visent par (index, identifiant).

use bs_commun::maths::{cos, longueur, sin};
use bs_donnees::armes::Mecanique;
use bs_donnees::entites::Allie as DefinitionAllie;

mod comportements;

use super::arme::ArmeEquipee;
use super::tours;
use crate::joueur::degats_apres_defense;
use crate::partie::Partie;

const CAPACITE: usize = 32;
const PORTEE_TOTEM: f32 = 9.0;
/// Un allié tué revient après `RAPPEL` recharges de son arme ; places de rappel par type.
const RAPPEL: f32 = 1.5;
const RAPPELS_MAX: usize = 16;
const SEPARATION_PASSES: u32 = 3;
/// s de surcouche rouge après une blessure (visuel).
const DUREE_BLESSURE: f32 = 0.3;

#[derive(Debug, Clone, Copy)]
pub struct Allie {
    pub id: u32,
    /// Index dans la table des alliés.
    pub type_allie: usize,
    pub def: &'static DefinitionAllie,
    /// Position dans l'arsenal de l'arme qui l'a invoqué (None : il disparaît).
    pub arme: Option<usize>,
    pub x: f32,
    pub z: f32,
    pub angle: f32,
    /// s restantes.
    pub vie: f32,
    pub pv: f32,
    pub pv_max: f32,
    pub vitesse: f32,
    pub blessure: f32,
    pub minuterie: f32,
    pub vitesse_sol: f32,
    pub dernier_soin: f64,
}

#[derive(Debug, Clone)]
pub struct Invocations {
    pub pool: Vec<Allie>,
    prochain_id: u32,
    /// Par type d'allié : temps de jeu où chaque place en rappel se libère.
    rappels: Vec<[f64; RAPPELS_MAX]>,
    /// Centre du dernier groupe visé par un totem (partagé, comme dans le jeu TS).
    groupe: (f32, f32),
}

impl Invocations {
    pub fn nouvelles(types: usize) -> Self {
        Self {
            pool: Vec::with_capacity(CAPACITE),
            prochain_id: 1,
            rappels: vec![[0.0; RAPPELS_MAX]; types],
            groupe: (0.0, 0.0),
        }
    }

    /// Index actuel de l'allié `id` (dernier index connu `a`) s'il vit encore.
    fn retrouver(&self, a: usize, id: u32) -> Option<usize> {
        let a = if self.pool.get(a).is_some_and(|x| x.id == id) {
            a
        } else {
            self.pool.iter().position(|x| x.id == id)?
        };
        let x = &self.pool[a];
        (x.arme.is_some() && x.pv > 0.0).then_some(a)
    }
}

impl Partie {
    /// Invoque les alliés de l'arme `i` de l'arsenal ; renvoie le nombre de nouveaux.
    pub(super) fn invoquer(&mut self, i: usize) -> u32 {
        let arme = self.arsenal.armes[i];
        let Mecanique::Invocation { allie, .. } = &arme.def.mecanique else {
            return 0;
        };
        let Some((type_allie, _, def)) = self.d.allies.allies.get_full(allie) else {
            return 0;
        };
        if def.aura.is_some() {
            let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
            let Some(g) = self.plus_gros_groupe(jx, jz, PORTEE_TOTEM, arme.stats.zone) else {
                return 0;
            };
            self.armes.invocations.groupe = g;
        }
        let mut nouveaux = 0;
        for _ in 0..tours(arme.stats.nombre) {
            if ((self.presents(type_allie) + self.en_rappel(type_allie)) as f32) < def.quota
                && self.creer(i, type_allie, def)
            {
                nouveaux += 1;
            } else if def.remplacement {
                self.replanter(type_allie, arme.stats.duree);
            } else {
                self.renouveler(type_allie, arme.stats.duree);
            }
        }
        nouveaux
    }

    /// La vague d'alliés de cette arme est encore là (hors alliés de soutien).
    pub(super) fn vague_en_place(&self, i: usize, arme: &ArmeEquipee) -> bool {
        let Mecanique::Invocation { allie, .. } = &arme.def.mecanique else {
            return false;
        };
        if self.d.allies.allies.get(allie).is_none_or(|d| d.soutien) {
            return false;
        }
        self.armes
            .invocations
            .pool
            .iter()
            .any(|a| a.arme == Some(i) && a.pv > 0.0)
    }

    fn presents(&self, type_allie: usize) -> usize {
        let pool = &self.armes.invocations.pool;
        pool.iter()
            .filter(|a| a.type_allie == type_allie && a.arme.is_some() && a.pv > 0.0)
            .count()
    }

    fn en_rappel(&self, type_allie: usize) -> usize {
        let maintenant = self.etat.temps_jeu;
        self.armes.invocations.rappels[type_allie]
            .iter()
            .filter(|&&r| r > maintenant)
            .count()
    }

    fn mettre_en_rappel(&mut self, a: usize) {
        let allie = self.armes.invocations.pool[a];
        let Some(arme) = allie.arme else {
            return;
        };
        let cadence = self.arsenal.armes[arme].stats.cadence;
        let maintenant = self.etat.temps_jeu;
        let rappels = &mut self.armes.invocations.rappels[allie.type_allie];
        if let Some(r) = rappels.iter_mut().find(|r| **r <= maintenant) {
            *r = maintenant + f64::from(cadence * RAPPEL);
        }
    }

    fn plus_ancien(&self, type_allie: usize) -> Option<usize> {
        let mut choisi: Option<usize> = None;
        let pool = &self.armes.invocations.pool;
        for (k, a) in pool.iter().enumerate() {
            if a.type_allie == type_allie
                && a.arme.is_some()
                && a.pv > 0.0
                && choisi.is_none_or(|c| a.vie < pool[c].vie)
            {
                choisi = Some(k);
            }
        }
        choisi
    }

    fn renouveler(&mut self, type_allie: usize, duree: f32) {
        if let Some(k) = self.plus_ancien(type_allie) {
            let a = &mut self.armes.invocations.pool[k];
            a.vie = a.vie.max(duree);
        }
    }

    fn replanter(&mut self, type_allie: usize, duree: f32) {
        let Some(k) = self.plus_ancien(type_allie) else {
            return;
        };
        let (gx, gz) = self.armes.invocations.groupe;
        let a = &mut self.armes.invocations.pool[k];
        let (ax, az) = (a.x, a.z);
        a.x = gx;
        a.z = gz;
        a.vie = duree;
        self.ev.gerbe("invocation", ax, 1.0, az);
        self.ev.gerbe("invocation", gx, 1.0, gz);
    }

    fn creer(&mut self, i: usize, type_allie: usize, def: &'static DefinitionAllie) -> bool {
        if self.armes.invocations.pool.len() >= CAPACITE {
            return false;
        }
        let arme = self.arsenal.armes[i];
        let j = &self.etat.joueur;
        let angle = (self.alea.tirer() * std::f64::consts::PI * 2.0) as f32;
        let (cx, cz, r) = if def.aura.is_some() {
            let (gx, gz) = self.armes.invocations.groupe;
            (gx, gz, 0.6)
        } else {
            (j.x, j.z, 1.5)
        };
        let m = &self.etat.modificateurs;
        let pv_max = def.pv * (1.0 + m.pv_allies);
        let inv = &mut self.armes.invocations;
        let id = inv.prochain_id;
        inv.prochain_id += 1;
        let a = Allie {
            id,
            type_allie,
            def,
            arme: Some(i),
            x: cx + sin(angle) * r,
            z: cz + cos(angle) * r,
            angle: j.angle,
            vie: arme.stats.duree,
            pv: pv_max,
            pv_max,
            vitesse: def.vitesse * (1.0 + m.vitesse_allies),
            blessure: 0.0,
            minuterie: if def.soutien { 1.0 } else { 0.0 },
            vitesse_sol: 0.0,
            dernier_soin: f64::NEG_INFINITY,
        };
        inv.pool.push(a);
        self.ev.gerbe("invocation", a.x, 1.0, a.z);
        true
    }

    pub fn maj_invocations(&mut self, dt: f32) {
        let mut i = self.armes.invocations.pool.len();
        while i > 0 {
            i -= 1;
            let a = &mut self.armes.invocations.pool[i];
            a.vie -= dt;
            let Some(arme) = a.arme.filter(|_| a.vie > 0.0) else {
                let (x, z) = (a.x, a.z);
                self.armes.invocations.pool.swap_remove(i);
                self.ev.gerbe("invocation", x, 1.0, z);
                continue;
            };
            if a.minuterie > 0.0 {
                a.minuterie -= dt;
            }
            if a.blessure > 0.0 {
                a.blessure -= dt;
            }
            let def = a.def;
            if def.aura.is_some() {
                self.pulser_aura(i, arme);
            } else if def.soutien {
                self.soutenir(i, arme, dt);
            } else if def.tir.is_some() {
                self.tirer_allie(i, arme, dt);
            } else {
                self.combattre(i, arme, dt);
            }
        }
        for _ in 0..SEPARATION_PASSES {
            self.separer_allies();
        }
    }

    /// Allié assez proche (plus que le joueur) pour attirer un ennemi au corps à corps.
    pub fn allie_provocateur(&self, x: f32, z: f32, distance_joueur: f32) -> Option<(usize, u32)> {
        let mut meilleur = None;
        let mut meilleure = distance_joueur;
        for (k, a) in self.armes.invocations.pool.iter().enumerate() {
            if a.pv <= 0.0 || a.arme.is_none() {
                continue;
            }
            let d = longueur(a.x - x, a.z - z);
            if d < a.def.provocation && d < meilleure {
                meilleure = d;
                meilleur = Some((k, a.id));
            }
        }
        meilleur
    }

    /// Index actuel, position et rayon de l'allié `id` (dernier index connu `a`) s'il vit encore.
    pub fn allie_vivant(&self, a: usize, id: u32) -> Option<(usize, f32, f32, f32)> {
        let k = self.armes.invocations.retrouver(a, id)?;
        let a = &self.armes.invocations.pool[k];
        Some((k, a.x, a.z, a.def.rayon))
    }

    pub fn blesser_allie(&mut self, k: usize, degats: f32) {
        let plafond = self.d.regles.plafonds.armure;
        let a = &mut self.armes.invocations.pool[k];
        if a.pv <= 0.0 || a.arme.is_none() {
            return;
        }
        a.pv -= degats_apres_defense(degats, a.def.armure.unwrap_or(0.0), plafond);
        a.blessure = DUREE_BLESSURE;
        let def = a.def;
        if a.pv > 0.0 {
            self.ev.son(&def.son_blessure);
            return;
        }
        self.ev.son(&def.son_mort);
        self.mettre_en_rappel(k);
        self.armes.invocations.pool[k].vie = 0.0;
    }

    /// Explosion : blesse les alliés à moins de `rayon`, moins fort au bord.
    pub fn blesser_allies_zone(&mut self, x: f32, z: f32, rayon: f32, degats: f32) {
        for k in 0..self.armes.invocations.pool.len() {
            let a = &self.armes.invocations.pool[k];
            let d = longueur(a.x - x, a.z - z);
            if d < rayon {
                self.blesser_allie(k, libm::ceilf(degats * (1.0 - d / rayon)));
            }
        }
    }
}
