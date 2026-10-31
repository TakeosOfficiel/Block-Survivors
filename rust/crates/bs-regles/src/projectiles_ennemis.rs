//! Projectiles tirés par les ennemis (flèches) : pool, vol en ligne droite par pas
//! courts, collision avec le décor puis le joueur. Un coup de bouclier peut les
//! renvoyer : ils blessent alors les ennemis (port de src/entites/projectilesEnnemis.ts).

use bs_commun::maths::{atan2, cos, ecart_angle, longueur, sin};
use bs_donnees::entites::Projectile as DefinitionProjectile;

use crate::ennemis::Nature;
use crate::partie::Partie;

/// Projectiles simultanés au maximum.
pub const CAPACITE: usize = 256;
/// Longueur maximale d'un pas de collision (blocs).
const PAS_COLLISION: f32 = 0.25;
/// Rayon des projectiles contre le décor.
const RAYON_DECOR: f32 = 0.1;
/// Recul infligé par un projectile renvoyé.
const RECUL_RENVOI: f32 = 4.0;

#[derive(Debug, Clone, Copy)]
pub struct ProjectileEnnemi {
    pub index_type: usize,
    pub def: &'static DefinitionProjectile,
    pub x: f32,
    pub z: f32,
    pub vx: f32,
    pub vz: f32,
    pub vie: f32,
    pub degats: f32,
    /// Tiré par un boss.
    pub boss: bool,
    /// Renvoyé par le joueur : il blesse les ennemis, au nom de l'arme `arme`.
    pub renvoye: bool,
    pub arme: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ProjectilesEnnemis {
    pub pool: Vec<ProjectileEnnemi>,
    touches: [u32; 4],
}

impl ProjectilesEnnemis {
    pub fn nouveaux() -> Self {
        Self {
            pool: Vec::with_capacity(CAPACITE),
            touches: [0; 4],
        }
    }

    /// Libère `i` en l'échangeant avec le dernier (comme le pool TS).
    fn liberer(&mut self, i: usize) {
        self.pool.swap_remove(i);
    }
}

impl Partie {
    pub fn tirer_ennemi(
        &mut self,
        projectile: &'static str,
        x: f32,
        z: f32,
        dir_x: f32,
        dir_z: f32,
        degats: f32,
        boss: bool,
    ) {
        if self.projectiles_ennemis.pool.len() >= CAPACITE {
            return;
        }
        let Some((index_type, _, def)) = self.d.projectiles.get_full(projectile) else {
            return;
        };
        self.projectiles_ennemis.pool.push(ProjectileEnnemi {
            index_type,
            def,
            x,
            z,
            vx: dir_x * def.vitesse,
            vz: dir_z * def.vitesse,
            vie: def.duree,
            degats,
            boss,
            renvoye: false,
            arme: None,
        });
    }

    /// Renvoie les projectiles du cône (sommet en x, z ; demi-angle en rad ; longueur
    /// `portee`) : ils repartent vers l'avant et blessent les ennemis. Renvoie leur nombre.
    pub fn renvoyer_projectiles(
        &mut self,
        x: f32,
        z: f32,
        angle: f32,
        demi_angle: f32,
        portee: f32,
        degats: f32,
        arme: Option<usize>,
    ) -> u32 {
        let mut n = 0;
        for p in &mut self.projectiles_ennemis.pool {
            let (dx, dz) = (p.x - x, p.z - z);
            let d = longueur(dx, dz);
            if p.renvoye
                || d > portee
                || (d > 0.3 && ecart_angle(angle, atan2(dx, dz)).abs() > demi_angle)
            {
                continue;
            }
            let vitesse = p.def.vitesse * 1.2;
            p.vx = sin(angle) * vitesse;
            p.vz = cos(angle) * vitesse;
            p.vie = p.def.duree;
            p.degats = degats;
            p.renvoye = true;
            p.arme = arme;
            n += 1;
        }
        n
    }

    pub fn maj_projectiles_ennemis(&mut self, dt: f32) {
        let limite_x = self.etat.arene.largeur / 2.0;
        let limite_z = self.etat.arene.profondeur / 2.0;
        let rayon_joueur = self.d.regles.joueur.rayon;
        let mut i = self.projectiles_ennemis.pool.len();
        while i > 0 {
            i -= 1;
            let mut p = self.projectiles_ennemis.pool[i];
            p.vie -= dt;
            // Par pas courts, le décor avant la cible : jamais de coup à travers un mur
            let portee = p.def.rayon + rayon_joueur;
            let nb_pas = libm::ceilf(longueur(p.vx, p.vz) * dt / PAS_COLLISION).max(1.0) as u32;
            let mut fini = p.vie <= 0.0;
            let mut k = 0;
            while k < nb_pas && !fini {
                k += 1;
                p.x += p.vx * dt / nb_pas as f32;
                p.z += p.vz * dt / nb_pas as f32;
                let sortie = p.x < -limite_x || p.x > limite_x || p.z < -limite_z || p.z > limite_z;
                if sortie || self.carte.touche_tir(p.x, p.z, RAYON_DECOR) {
                    fini = true;
                    break;
                }
                if p.renvoye {
                    // Renvoyé : blesse le premier ennemi rencontré
                    let mut touches = self.projectiles_ennemis.touches;
                    if self.ennemis.chercher(p.x, p.z, p.def.rayon, &mut touches) > 0 {
                        self.blesser(
                            touches[0] as usize,
                            p.degats,
                            p.x - p.vx,
                            p.z - p.vz,
                            RECUL_RENVOI,
                            p.arme,
                            Nature::Arme,
                        );
                        fini = true;
                    }
                } else {
                    let j = &self.etat.joueur;
                    let (dx, dz) = (p.x - j.x, p.z - j.z);
                    if !j.mort && dx * dx + dz * dz < portee * portee {
                        self.blesser_joueur(p.degats, p.boss);
                        fini = true;
                    }
                }
            }
            if fini {
                self.projectiles_ennemis.liberer(i);
            } else {
                self.projectiles_ennemis.pool[i] = p;
            }
        }
    }
}
