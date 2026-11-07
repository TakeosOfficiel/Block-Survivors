//! Particules vanilla (src/effets/particules.ts) : explosions, nuages de mort, flammes,
//! éclats d'objets… Un pool de capacité fixe, des carrés face à la caméra lus dans un
//! atlas construit à l'entrée en partie (une case de 32 px par image, 16 colonnes).
//! Leur hasard est celui de l'affichage, jamais celui de la simulation.

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bs_commun::Alea;
use bs_donnees::Donnees;
use bs_donnees::effets::{AnimationParticule, Particule};

use super::maillage::MaillageDynamique;
use crate::arene::couleur_lineaire;

/// Particules simultanées au plus (config.ts `particules.capacite`).
const CAPACITE: usize = 1024;
/// Côté d'une case de l'atlas (px) et colonnes.
pub const CASE: usize = 32;
pub const COLONNES: usize = 16;

#[derive(Clone, Copy)]
struct Vivante {
    def: &'static Particule,
    premiere_case: u32,
    position: Vec3,
    vitesse: Vec3,
    vie: f32,
    duree: f32,
    taille: f32,
    teinte: [f32; 4],
    /// Morceau d'icône : coin du quart choisi (fraction de la case).
    morceau: Vec2,
}

pub struct Particules {
    pool: Vec<Vivante>,
    /// Case de chaque image dans l'atlas.
    cases: HashMap<&'static str, u32>,
    lignes: usize,
    alea: Alea,
    pub maillage: MaillageDynamique,
}

/// Images de l'atlas : séries des particules puis icônes (éclats).
pub fn images_atlas<'a>(d: &'a Donnees, icones: &[&'a str]) -> Vec<&'a str> {
    let mut v: Vec<&str> = Vec::new();
    for p in d.particules.particules.values() {
        for i in &p.images {
            if !v.contains(&i.as_str()) {
                v.push(i);
            }
        }
    }
    for &i in icones {
        if !v.contains(&i) {
            v.push(i);
        }
    }
    v
}

/// Atlas RGBA : chaque image (sa première case carrée) agrandie sans lissage ; une
/// image absente devient un carré blanc.
pub fn construire_atlas<'a>(
    images: &[&str],
    pixels: impl Fn(&str) -> Option<(&'a [u8], usize)>,
) -> (Vec<u8>, usize) {
    let lignes = images.len().div_ceil(COLONNES).max(1);
    let largeur = COLONNES * CASE;
    let mut atlas = vec![0u8; largeur * lignes * CASE * 4];
    for (k, chemin) in images.iter().enumerate() {
        let (x0, y0) = ((k % COLONNES) * CASE, (k / COLONNES) * CASE);
        for y in 0..CASE {
            for x in 0..CASE {
                let px = match pixels(chemin) {
                    Some((px, l)) if l > 0 => {
                        let (sx, sy) = (x * l / CASE, y * l / CASE);
                        let i = (sy * l + sx) * 4;
                        [px[i], px[i + 1], px[i + 2], px[i + 3]]
                    }
                    _ if (CASE / 4..CASE * 3 / 4).contains(&x)
                        && (CASE / 4..CASE * 3 / 4).contains(&y) =>
                    {
                        [255; 4]
                    }
                    _ => [0; 4],
                };
                let i = ((y0 + y) * largeur + x0 + x) * 4;
                atlas[i..i + 4].copy_from_slice(&px);
            }
        }
    }
    (atlas, lignes)
}

impl Particules {
    pub fn nouvelles(
        images: &[&'static str],
        lignes: usize,
        graine: u32,
        maillage: MaillageDynamique,
    ) -> Self {
        Self {
            pool: Vec::with_capacity(CAPACITE),
            cases: images
                .iter()
                .enumerate()
                .map(|(k, i)| (*i, k as u32))
                .collect(),
            lignes,
            alea: Alea::nouveau(u64::from(graine), 7),
            maillage,
        }
    }

    /// Gerbe autour d'un point ; `icone` donne les éclats des particules « morceau ».
    pub fn emettre(&mut self, d: &'static Donnees, gerbe: &str, icone: Option<&str>, centre: Vec3) {
        let Some(salves) = d.particules.gerbes.get(gerbe) else {
            return;
        };
        let a = &mut self.alea;
        for salve in salves {
            let Some(def) = d.particules.particules.get(&salve.particule) else {
                continue;
            };
            let image = match def.animation {
                AnimationParticule::Morceau => icone,
                _ => def.images.first().map(String::as_str),
            };
            let Some(&premiere_case) = image.and_then(|i| self.cases.get(i)) else {
                continue;
            };
            for _ in 0..salve.nombre {
                if self.pool.len() >= CAPACITE {
                    return;
                }
                let angle = a.unite() * std::f32::consts::TAU;
                let rayon = a.unite().sqrt() * salve.dispersion;
                let position = centre
                    + Vec3::new(
                        angle.cos() * rayon,
                        (a.unite() - 0.5) * salve.dispersion * 0.6,
                        angle.sin() * rayon,
                    );
                let v = salve.vitesse * a.entre(0.3, 1.0);
                let direction = a.unite() * std::f32::consts::TAU;
                let vitesse = Vec3::new(
                    direction.cos() * v,
                    salve.montee * a.entre(0.4, 1.0),
                    direction.sin() * v,
                );
                let duree = a.entre(def.duree[0], def.duree[1]);
                let taille = a.entre(def.taille[0], def.taille[1]);
                let teinte = match &def.teintes {
                    Some(t) if !t.is_empty() => {
                        couleur_lineaire(t[(a.unite() * t.len() as f32) as usize % t.len()])
                    }
                    _ => {
                        let g = def.gris.map_or(1.0, |[min, max]| a.entre(min, max));
                        [g, g, g, 1.0]
                    }
                };
                let morceau = Vec2::new(a.unite() * 0.75, a.unite() * 0.75);
                self.pool.push(Vivante {
                    def,
                    premiere_case,
                    position,
                    vitesse,
                    vie: duree,
                    duree,
                    taille,
                    teinte,
                    morceau,
                });
            }
        }
    }

    /// Mouvement : frottement, gravité, posées au sol.
    pub fn avancer(&mut self, dt: f32) {
        let mut i = self.pool.len();
        while i > 0 {
            i -= 1;
            let p = &mut self.pool[i];
            p.vie -= dt;
            if p.vie <= 0.0 {
                self.pool.swap_remove(i);
                continue;
            }
            let frein = 1.0 - p.def.frottement * dt;
            p.vitesse.x *= frein;
            p.vitesse.z *= frein;
            p.vitesse.y = p.vitesse.y * frein - p.def.gravite * dt;
            p.position += p.vitesse * dt;
            let sol = p.taille * 0.3;
            if p.position.y < sol {
                p.position.y = sol;
                p.vitesse.y = 0.0;
                p.vitesse.x *= 0.8;
                p.vitesse.z *= 0.8;
            }
        }
    }

    /// Carrés de l'image, face à la caméra (`droite`, `haut` : axes de l'écran).
    pub fn dessiner(&mut self, droite: Vec3, haut: Vec3) {
        let (lc, hc) = (1.0 / COLONNES as f32, 1.0 / self.lignes as f32);
        self.maillage.commencer();
        for p in &self.pool {
            let age = 1.0 - p.vie / p.duree;
            let mut s = p.taille;
            if p.def.retrecit {
                s *= 1.0 - age * age * 0.6;
            }
            let mut case = p.premiere_case;
            if p.def.animation == AnimationParticule::Age {
                let n = p.def.images.len() as u32;
                case += ((age * n as f32) as u32).min(n.saturating_sub(1));
            }
            let u0 = (case as usize % COLONNES) as f32 * lc;
            let v0 = (case as usize / COLONNES) as f32 * hc;
            let uv = if p.def.animation == AnimationParticule::Morceau {
                let (u, v) = (u0 + p.morceau.x * lc, v0 + p.morceau.y * hc);
                [u, v, u + lc / 4.0, v + hc / 4.0]
            } else {
                [u0, v0, u0 + lc, v0 + hc]
            };
            self.maillage.carre(
                p.position,
                droite * (s / 2.0),
                haut * (s / 2.0),
                uv,
                p.teinte,
            );
        }
    }
}
