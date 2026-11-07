//! Projectiles des armes du joueur : pool unique pour les mécaniques « tir » (ligne
//! droite, perforation), « retour » (aller puis retour au joueur, traverse tout) et
//! « explosif » (explose au premier contact). Port de src/combat/projectilesJoueur.ts.

use std::f64::consts::PI;

use bs_commun::maths::{cos, longueur, sin};
use bs_donnees::armes::{FamilleArme, Mecanique};
use bs_donnees::entites::Projectile as DefinitionProjectile;

use super::arme::ArmeEquipee;
use crate::classes::valeur;
use crate::ennemis::Nature;
use crate::partie::Partie;

const CAPACITE: usize = 256;
const MAX_TOUCHES: usize = 32;
const PAS_COLLISION: f32 = 0.25;
const RAYON_DECOR: f32 = 0.1;
/// Part des dégâts perdue par ennemi traversé (flèches).
const PERTE_TRAVERSEE: f32 = 0.25;
/// Perforation nette : perte par rang, et rang qui donne une perforation de plus.
const PERFORATION_NETTE: [f32; 2] = [0.12, 0.0];
const PERFORATION_NETTE_RANG: u32 = 2;
/// Visée stable : distance parcourue (blocs) et bonus de dégâts au-delà.
const VISEE_DISTANCE: f32 = 8.0;
const VISEE_BONUS: f32 = 0.15;
/// Le trident disparaît à cette distance du joueur (blocs).
const DISTANCE_RETOUR: f32 = 0.7;
/// s de brûlure des tirs qui enflamment (blazes alliés).
pub const BRULURE_TIRS: f32 = 3.0;
/// Éclats de la Fusée explosive : nombre, distance (blocs), rayon et dégâts (parts de l'explosion).
const ECLATS_NOMBRE: u32 = 4;
const ECLATS_DISTANCE: f32 = 1.6;
const ECLATS_RAYON: f32 = 0.5;
const ECLATS_DEGATS: f32 = 0.35;

/// `Ligne` : s'arrête après `perforation` ennemis ; `Traversant` : traverse tout ;
/// `Retour` : traverse tout et revient.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeProjectile {
    Ligne,
    Traversant,
    Retour,
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectileJoueur {
    pub def: &'static DefinitionProjectile,
    /// Crédite les dégâts à l'arme dans les statistiques.
    pub arme: usize,
    pub retour: bool,
    /// Phase de retour en cours.
    pub revient: bool,
    pub x: f32,
    pub z: f32,
    pub dir_x: f32,
    pub dir_z: f32,
    pub vitesse: f32,
    pub parcouru: f32,
    pub portee: f32,
    pub degats: f32,
    pub recul: f32,
    /// Ennemis encore traversables.
    pub restants: f32,
    /// Part des dégâts perdue à chaque ennemi traversé.
    pub perte: f32,
    /// Visée stable : plus de dégâts au-delà d'une distance.
    pub visee: bool,
    /// Rayon d'explosion (0 = n'explose pas).
    pub zone: f32,
    pub gerbe: &'static str,
    pub son_explosion: &'static str,
    pub eclats: u32,
    /// Dégâts par seconde de la brûlure infligée (0 : aucune).
    pub brulure: f32,
    pub nature: Nature,
    /// id des ennemis déjà touchés (pendant cette phase).
    touches: [u32; MAX_TOUCHES],
    nb_touches: usize,
}

#[derive(Debug, Clone)]
pub struct ProjectilesJoueur {
    pub pool: Vec<ProjectileJoueur>,
    tampon: [u32; 64],
}

impl Default for ProjectilesJoueur {
    fn default() -> Self {
        Self {
            pool: Vec::with_capacity(CAPACITE),
            tampon: [0; 64],
        }
    }
}

/// Origine d'un tir et brûlure : depuis le joueur, ou depuis un allié qui tire.
#[derive(Debug, Clone, Copy)]
pub struct Origine {
    pub x: f32,
    pub z: f32,
    pub brulure: f32,
    pub nature: Nature,
}

impl Partie {
    /// Lance un projectile dans la direction (dir_x, dir_z) unitaire depuis le joueur.
    pub fn lancer(
        &mut self,
        arme: &ArmeEquipee,
        projectile: &str,
        mode: ModeProjectile,
        dir_x: f32,
        dir_z: f32,
        explose: bool,
    ) {
        let j = &self.etat.joueur;
        let origine = Origine {
            x: j.x,
            z: j.z,
            brulure: 0.0,
            nature: Nature::Arme,
        };
        self.lancer_depuis(arme, projectile, mode, dir_x, dir_z, explose, origine);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn lancer_depuis(
        &mut self,
        arme: &ArmeEquipee,
        projectile: &str,
        mode: ModeProjectile,
        dir_x: f32,
        dir_z: f32,
        explose: bool,
        o: Origine,
    ) {
        if self.armes.projectiles.pool.len() >= CAPACITE {
            return;
        }
        let Some(def) = self.d.projectiles.get(projectile) else {
            return;
        };
        let s = &arme.stats;
        // Une flèche perd une part de ses dégâts à chaque ennemi traversé
        let fleche = mode == ModeProjectile::Ligne
            && arme.def.famille == Some(FamilleArme::Projectile)
            && o.nature == Nature::Arme;
        let (mut gerbe, mut son, mut eclats) = ("explosion", "explosion", 0);
        if let Mecanique::Explosif {
            gerbe: g,
            son: s,
            eclats_au_rang,
            ..
        } = &arme.def.mecanique
        {
            if let Some(g) = g {
                gerbe = g;
            }
            if let Some(s) = s {
                son = s;
            }
            if eclats_au_rang.is_some_and(|r| arme.rang as f32 >= r) {
                eclats = ECLATS_NOMBRE;
            }
        }
        let nette = self.talents.rang("perforation_nette");
        let perte = match (fleche, nette) {
            (false, _) => 0.0,
            (true, 0) => PERTE_TRAVERSEE,
            (true, r) => valeur(r, &PERFORATION_NETTE),
        };
        let perforation_nette = fleche && nette >= PERFORATION_NETTE_RANG;
        let visee = self.est_arme(arme.index, "arc") && self.talents.rang("visee_stable") > 0;
        self.armes.projectiles.pool.push(ProjectileJoueur {
            def,
            arme: arme.index,
            retour: mode == ModeProjectile::Retour,
            revient: false,
            x: o.x,
            z: o.z,
            dir_x,
            dir_z,
            vitesse: s.vitesse,
            parcouru: 0.0,
            portee: s.portee,
            degats: s.degats * self.talent_facteur_declenchement(),
            recul: s.recul,
            restants: if mode == ModeProjectile::Ligne {
                s.perforation.max(1.0) + if perforation_nette { 1.0 } else { 0.0 }
            } else {
                f32::INFINITY
            },
            perte,
            visee,
            zone: if explose { s.zone } else { 0.0 },
            gerbe,
            son_explosion: son,
            eclats,
            brulure: o.brulure,
            nature: o.nature,
            touches: [0; MAX_TOUCHES],
            nb_touches: 0,
        });
    }

    /// Projectile spécial d'un talent (flèche géante de la Puissance) : traverse tout,
    /// avec ses propres dégâts et sa portée.
    pub(crate) fn lancer_special(
        &mut self,
        arme: &ArmeEquipee,
        projectile: &str,
        (dir_x, dir_z): (f32, f32),
        degats: f32,
        portee: f32,
    ) {
        let avant = self.armes.projectiles.pool.len();
        self.lancer(
            arme,
            projectile,
            ModeProjectile::Traversant,
            dir_x,
            dir_z,
            false,
        );
        let Some(def) = self.d.projectiles.get(projectile) else {
            return;
        };
        if self.armes.projectiles.pool.len() > avant
            && let Some(p) = self.armes.projectiles.pool.last_mut()
        {
            p.degats = degats;
            p.portee = portee;
            p.vitesse = def.vitesse;
            p.perte = 0.0;
            p.visee = false;
        }
    }

    pub fn maj_projectiles_joueur(&mut self, dt: f32) {
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let limite_x = self.etat.arene.largeur / 2.0;
        let limite_z = self.etat.arene.profondeur / 2.0;
        let mut i = self.armes.projectiles.pool.len();
        while i > 0 {
            i -= 1;
            let mut p = self.armes.projectiles.pool[i];

            // Retour : se dirige vers le joueur et disparaît en l'atteignant
            if p.revient {
                let (dx, dz) = (jx - p.x, jz - p.z);
                let d = longueur(dx, dz);
                if d < DISTANCE_RETOUR {
                    self.armes.projectiles.pool.swap_remove(i);
                    continue;
                }
                p.dir_x = dx / d;
                p.dir_z = dz / d;
            }

            // Par pas courts, le décor avant les ennemis (le trident, magique, traverse le décor)
            let pas = p.vitesse * dt;
            let nb_pas = libm::ceilf(pas / PAS_COLLISION).max(1.0) as u32;
            let mut detruit = false;
            let mut sortie = false;
            let mut s = 0;
            while s < nb_pas && !detruit && !sortie {
                s += 1;
                p.x += p.dir_x * pas / nb_pas as f32;
                p.z += p.dir_z * pas / nb_pas as f32;
                p.parcouru += pas / nb_pas as f32;
                sortie = p.x < -limite_x
                    || p.x > limite_x
                    || p.z < -limite_z
                    || p.z > limite_z
                    || (!p.retour
                        && self.carte.touche_tir(
                            p.x,
                            p.z,
                            RAYON_DECOR,
                            self.carte.hauteur(p.x, p.z),
                        ));
                if sortie {
                    break;
                }
                // Collisions avec les ennemis
                let mut tampon = self.armes.projectiles.tampon;
                let n = self.ennemis.chercher(p.x, p.z, p.def.rayon, &mut tampon);
                for &index in &tampon[..n] {
                    let e = index as usize;
                    let id = self.ennemis.pool[e].id;
                    if p.touches[..p.nb_touches].contains(&id) {
                        continue;
                    }
                    if p.zone > 0.0 {
                        self.exploser_projectile(&p);
                        detruit = true;
                        break;
                    }
                    let degats = if p.visee && p.parcouru > VISEE_DISTANCE {
                        p.degats * (1.0 + VISEE_BONUS)
                    } else {
                        p.degats
                    };
                    self.blesser(
                        e,
                        degats,
                        p.x - p.dir_x,
                        p.z - p.dir_z,
                        p.recul,
                        Some(p.arme),
                        p.nature,
                    );
                    p.degats *= 1.0 - p.perte;
                    if p.brulure > 0.0 {
                        self.bruler(e, 1.0, p.brulure, BRULURE_TIRS, Some(p.arme));
                    }
                    if p.nb_touches < MAX_TOUCHES {
                        p.touches[p.nb_touches] = id;
                        p.nb_touches += 1;
                    }
                    p.restants -= 1.0;
                    if p.restants <= 0.0 {
                        detruit = true;
                        break;
                    }
                }
            }

            // Fin de course
            if !detruit && (p.parcouru >= p.portee || sortie) {
                if p.retour {
                    if !p.revient {
                        p.revient = true;
                        // Peut retoucher les mêmes ennemis au retour
                        p.nb_touches = 0;
                    }
                } else {
                    if p.zone > 0.0 {
                        self.exploser_projectile(&p);
                    }
                    detruit = true;
                }
            }

            if detruit {
                self.armes.projectiles.pool.swap_remove(i);
            } else {
                self.armes.projectiles.pool[i] = p;
            }
        }
    }

    fn exploser_projectile(&mut self, p: &ProjectileJoueur) {
        self.blesser_zone(p.x, p.z, p.zone, p.degats, p.recul, p.arme);
        self.ev.gerbe(p.gerbe, p.x, p.def.hauteur, p.z);
        self.ev.son_hauteur(p.son_explosion, 1.3);
        // Éclats : petites explosions tout autour (dernier rang de la Fusée explosive)
        for k in 0..p.eclats {
            let a = (f64::from(k) / f64::from(p.eclats) * PI * 2.0 + self.alea.tirer()) as f32;
            let x = p.x + sin(a) * ECLATS_DISTANCE;
            let z = p.z + cos(a) * ECLATS_DISTANCE;
            self.blesser_zone(
                x,
                z,
                p.zone * ECLATS_RAYON,
                p.degats * ECLATS_DEGATS,
                p.recul * 0.5,
                p.arme,
            );
            self.ev.gerbe("artifice", x, p.def.hauteur, z);
        }
    }

    /// Blesse les ennemis à moins de `rayon` de (x, z), au nom de l'arme `arme` (au plus 64,
    /// comme le tampon du jeu TS).
    pub fn blesser_zone(
        &mut self,
        x: f32,
        z: f32,
        rayon: f32,
        degats: f32,
        recul: f32,
        arme: usize,
    ) {
        let mut tampon = [0u32; 64];
        let n = self.ennemis.chercher(x, z, rayon, &mut tampon);
        for &e in &tampon[..n] {
            if self.a_couvert(x, z, e as usize) {
                continue;
            }
            self.blesser(e as usize, degats, x, z, recul, Some(arme), Nature::Arme);
        }
    }
}
