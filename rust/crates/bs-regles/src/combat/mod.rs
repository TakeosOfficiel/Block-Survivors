//! Arsenal du joueur : armes possédées, minuteries, et déclenchement de chaque mécanique
//! vers son système (port de src/combat/arsenal.ts). Les systèmes d'armes sont des blocs
//! `impl Partie` dans les modules voisins.

pub mod arme;
pub mod feu;
pub mod foudre;
pub mod melee;
pub mod pose;
pub mod projectiles;
pub mod sol;
pub mod traque;

use bs_commun::maths::{atan2, cos, sin};
use bs_donnees::Donnees;
use bs_donnees::armes::Mecanique;

use crate::partie::Partie;
use arme::ArmeEquipee;
use feu::ArmesFeu;
use foudre::Eclairs;
use melee::Tourbillon;
use pose::ArmesPosees;
use projectiles::{ModeProjectile, ProjectilesJoueur};
use sol::ArmesSol;
use traque::ArmesTraque;

/// s avant de réessayer une arme qui n'a trouvé aucune cible.
const NOUVEL_ESSAI: f32 = 0.2;
/// Pas de nouvelle vague d'alliés avant `PAUSE` s après le départ de la précédente.
pub const PAUSE_INVOCATIONS: f32 = 3.0;

/// Nombre de tours d'une boucle TS `for (k = 0; k < nombre; k++)` sur un nombre décimal.
pub fn tours(nombre: f32) -> u32 {
    libm::ceilf(nombre.max(0.0)) as u32
}

#[derive(Debug, Clone)]
pub struct Arsenal {
    pub armes: Vec<ArmeEquipee>,
    /// Change à chaque arme gagnée ou améliorée (affichage).
    pub version: u32,
}

impl Arsenal {
    pub fn nouveau() -> Self {
        Self {
            armes: Vec::with_capacity(16),
            version: 0,
        }
    }

    /// Rang de l'arme `index` (0 = absente).
    pub fn rang(&self, index: usize) -> u32 {
        self.armes
            .iter()
            .find(|a| a.index == index)
            .map_or(0, |a| a.rang)
    }
}

/// États des systèmes d'armes sans pool propre.
#[derive(Debug, Clone)]
pub struct SystemesArmes {
    pub projectiles: ProjectilesJoueur,
    /// Angle courant de chaque arme en orbite, par index d'arme.
    pub angles_orbite: Vec<f32>,
    pub eclairs: Eclairs,
    pub tourbillon: Tourbillon,
    pub sol: ArmesSol,
    pub feu: ArmesFeu,
    pub traque: ArmesTraque,
    pub posees: ArmesPosees,
}

impl SystemesArmes {
    pub fn nouveaux(d: &Donnees) -> Self {
        Self {
            projectiles: ProjectilesJoueur::default(),
            angles_orbite: vec![0.0; d.armes.len()],
            eclairs: Eclairs::default(),
            tourbillon: Tourbillon::default(),
            sol: ArmesSol::default(),
            feu: ArmesFeu::default(),
            traque: ArmesTraque::default(),
            posees: ArmesPosees::default(),
        }
    }
}

impl Partie {
    /// Gagne l'arme `index` au rang 1, ou la monte d'un rang ; faux si elle est au maximum.
    pub fn ameliorer_arme(&mut self, index: usize) -> bool {
        let m = self.etat.modificateurs;
        match self.arsenal.armes.iter().position(|a| a.index == index) {
            None => {
                let arme = ArmeEquipee::nouvelle(self.d, index, &m);
                self.arsenal.armes.push(arme);
                self.etat.stats.acquisition_arme[index] = self.etat.temps_jeu;
            }
            Some(k) => {
                let arme = &mut self.arsenal.armes[k];
                if arme.rang >= arme.rang_max() {
                    return false;
                }
                arme.rang += 1;
                arme.recalculer(self.d, &m);
            }
        }
        self.arsenal.version += 1;
        true
    }

    /// Recalcule toutes les armes après un changement des modificateurs.
    pub fn recalculer_armes(&mut self) {
        let m = self.etat.modificateurs;
        for arme in &mut self.arsenal.armes {
            arme.recalculer(self.d, &m);
        }
        self.arsenal.version += 1;
    }

    pub fn maj_arsenal(&mut self, dt: f32) {
        if self.etat.joueur.mort || self.etat.issue.is_some() {
            return;
        }
        for i in 0..self.arsenal.armes.len() {
            let arme = self.arsenal.armes[i];
            if matches!(arme.def.mecanique, Mecanique::Orbite { .. }) {
                self.maj_orbite(&arme, dt);
                continue;
            }
            let mut minuterie = arme.minuterie - dt;
            if minuterie < PAUSE_INVOCATIONS && self.vague_en_place(&arme) {
                minuterie = PAUSE_INVOCATIONS;
            }
            self.arsenal.armes[i].minuterie = minuterie;
            if minuterie > 0.0 {
                continue;
            }
            let declenchee = self.declencher(i);
            let arme = self.arsenal.armes[i];
            self.arsenal.armes[i].minuterie = if declenchee {
                arme.stats.cadence * self.talent_delai(arme.index)
            } else {
                NOUVEL_ESSAI
            };
            if declenchee
                && let Some(son) = &arme.def.son
                && !matches!(arme.def.mecanique, Mecanique::Invocation { .. })
            {
                self.ev.son(son);
            }
        }
    }

    /// Déclenche l'arme `i` de l'arsenal ; faux si elle n'a rien trouvé à frapper.
    fn declencher(&mut self, i: usize) -> bool {
        let arme = self.arsenal.armes[i];
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let s = arme.stats;
        match &arme.def.mecanique {
            Mecanique::Balayage { angle, .. } => self.frapper_balayage(&arme, *angle),
            Mecanique::Tir {
                projectile,
                dispersion,
            }
            | Mecanique::Retour {
                projectile,
                dispersion,
                ..
            } => {
                let Some(c) = self.ennemis.plus_proche(jx, jz, s.portee) else {
                    return false;
                };
                let cible = &self.ennemis.pool[c];
                let base = atan2(cible.x - jx, cible.z - jz);
                let mode = match &arme.def.mecanique {
                    Mecanique::Retour { loyaute, .. } if arme.rang as f32 >= *loyaute => {
                        ModeProjectile::Retour
                    }
                    Mecanique::Retour { .. } => ModeProjectile::Traversant,
                    _ => ModeProjectile::Ligne,
                };
                for k in 0..tours(s.nombre) {
                    let a = base + (k as f32 - (s.nombre - 1.0) / 2.0) * dispersion;
                    self.lancer(&arme, projectile, mode, sin(a), cos(a), false);
                }
                true
            }
            Mecanique::Explosif { projectile, .. } => {
                let mut lance = false;
                for _ in 0..tours(s.nombre) {
                    let Some(c) = self.ennemi_au_hasard(jx, jz, s.portee) else {
                        break;
                    };
                    let cible = &self.ennemis.pool[c];
                    let a = atan2(cible.x - jx, cible.z - jz);
                    self.lancer(
                        &arme,
                        projectile,
                        ModeProjectile::Ligne,
                        sin(a),
                        cos(a),
                        true,
                    );
                    lance = true;
                }
                lance
            }
            Mecanique::Foudre { .. } => self.frapper_foudre(&arme),
            Mecanique::Invocation { .. } => {
                if self.invoquer(i) > 0
                    && let Some(son) = &arme.def.son
                {
                    self.ev.son(son);
                }
                true
            }
            Mecanique::Orbite { .. } => true,
            Mecanique::Tourbillon { intervalle, .. } => self.frapper_tourbillon(i, *intervalle),
            Mecanique::Heurt { angle, renvoi } => self.frapper_heurt(&arme, *angle, *renvoi),
            Mecanique::Fissure { ecart, .. } => self.frapper_fissure(i, *ecart),
            Mecanique::Chute { .. } => self.frapper_chute(i),
            Mecanique::Tornade { .. } => self.frapper_tornade(i),
            Mecanique::Meteore { .. } => self.frapper_meteore(i),
            Mecanique::Orbes { .. } => self.frapper_orbes(i),
            Mecanique::Ricochet { .. } => self.frapper_ricochet(i),
            Mecanique::Pluie { .. } => self.frapper_pluie(i),
            Mecanique::Piege { .. } => self.frapper_piege(i),
            Mecanique::Nova { sol_au_rang, .. } => self.frapper_nova(i, *sol_au_rang),
        }
    }

    // --- En attente des invocations ---

    /// La vague d'alliés de cette arme est encore en place.
    fn vague_en_place(&self, _arme: &ArmeEquipee) -> bool {
        false
    }
    /// Invoque les alliés de l'arme `i` de l'arsenal ; renvoie leur nombre.
    fn invoquer(&mut self, _i: usize) -> u32 {
        0
    }
}
