//! Champ de flux vers le joueur : distance en cases depuis sa position (parcours en
//! largeur sur les cases franchissables), recalculée quand il change de case. Chaque
//! ennemi descend ce champ pour contourner les obstacles.
//!
//! Un champ par largeur de passage (1 à 3 cases) : les gros ennemis suivent celui de leur
//! taille, qui ne passe que là où un carré de cette largeur tient, et ne se faufilent par
//! un passage étroit que s'il n'existe aucun chemin assez large.

use crate::carte::{BLOQUE_MARCHE, CarteMonde};

const INATTEIGNABLE: i32 = -1;
/// Plus grande largeur de passage suivie (en cases).
pub const TAILLE_MAX: usize = 3;
const VOISINS_X: [i64; 8] = [1, -1, 0, 0, 1, 1, -1, -1];
const VOISINS_Z: [i64; 8] = [0, 0, 1, -1, 1, -1, 1, -1];

#[derive(Debug, Clone)]
pub struct ChampDeFlux {
    /// Distances pour chaque largeur de passage (index : largeur - 1).
    distances: [Vec<i32>; TAILLE_MAX],
    /// Largeur du plus grand carré libre (au plus `TAILLE_MAX`) contenant chaque case.
    degagement: Vec<u8>,
    /// Plus grand carré libre dont la case est le coin bas-droit (calcul du dégagement).
    carres: Vec<u8>,
    file: Vec<u32>,
    cellule_cible: Option<usize>,
}

impl ChampDeFlux {
    pub fn nouveau(carte: &CarteMonde) -> Self {
        let cases = carte.largeur * carte.profondeur;
        Self {
            distances: std::array::from_fn(|_| vec![INATTEIGNABLE; cases]),
            degagement: vec![0; cases],
            carres: vec![0; cases],
            file: vec![0; cases],
            cellule_cible: None,
        }
    }

    /// Recalcule le champ si la cible a changé de case.
    pub fn maj(&mut self, carte: &CarteMonde, x: f32, z: f32) {
        let Some(cible) = carte.cellule(x, z) else {
            return;
        };
        if self.cellule_cible == Some(cible) {
            return;
        }
        self.cellule_cible = Some(cible);
        self.calculer_degagement(carte);
        for taille in 1..=TAILLE_MAX {
            self.parcourir(carte, cible, taille);
        }
    }

    /// Dégagement de chaque case (la carte peut avoir changé depuis le dernier calcul).
    fn calculer_degagement(&mut self, carte: &CarteMonde) {
        let (l, p) = (carte.largeur, carte.profondeur);
        for iz in 0..p {
            for ix in 0..l {
                let c = iz * l + ix;
                self.carres[c] = if carte.drapeaux[c] & BLOQUE_MARCHE != 0 {
                    0
                } else if ix == 0 || iz == 0 {
                    1
                } else {
                    let m = self.carres[c - 1]
                        .min(self.carres[c - l])
                        .min(self.carres[c - l - 1]);
                    (m + 1).min(TAILLE_MAX as u8)
                };
            }
        }
        for iz in 0..p {
            for ix in 0..l {
                let mut d = 0;
                for bz in iz..(iz + TAILLE_MAX).min(p) {
                    for bx in ix..(ix + TAILLE_MAX).min(l) {
                        let carre = self.carres[bz * l + bx];
                        // Le carré de côté `carre` finissant en (bx, bz) couvre-t-il la case ?
                        if carre as usize > (bx - ix).max(bz - iz) {
                            d = d.max(carre);
                        }
                    }
                }
                self.degagement[iz * l + ix] = d;
            }
        }
    }

    /// Parcours en largeur depuis `cible` sur les cases où tient un carré de `taille`.
    fn parcourir(&mut self, carte: &CarteMonde, cible: usize, taille: usize) {
        let (largeur, profondeur) = (carte.largeur as i64, carte.profondeur as i64);
        let distances = &mut self.distances[taille - 1];
        distances.fill(INATTEIGNABLE);
        distances[cible] = 0;
        let mut debut = 0;
        let mut fin = 0;
        self.file[fin] = cible as u32;
        fin += 1;
        while debut < fin {
            let c = self.file[debut] as usize;
            debut += 1;
            let ix = c as i64 % largeur;
            let iz = c as i64 / largeur;
            let d = distances[c] + 1;
            for v in 0..4 {
                let nx = ix + VOISINS_X[v];
                let nz = iz + VOISINS_Z[v];
                if nx < 0 || nz < 0 || nx >= largeur || nz >= profondeur {
                    continue;
                }
                let n = (nz * largeur + nx) as usize;
                if distances[n] != INATTEIGNABLE || (self.degagement[n] as usize) < taille {
                    continue;
                }
                distances[n] = d;
                self.file[fin] = n as u32;
                fin += 1;
            }
        }
    }

    /// Le joueur est-il atteignable à pied depuis (x, z) ?
    pub fn atteignable(&self, carte: &CarteMonde, x: f32, z: f32) -> bool {
        carte
            .cellule(x, z)
            .is_some_and(|c| self.distances[0][c] != INATTEIGNABLE)
    }

    /// Nombre de cases à parcourir à pied (sans diagonale) de (x, z) jusqu'au joueur.
    pub fn distance_marche(&self, carte: &CarteMonde, x: f32, z: f32) -> Option<u32> {
        let d = self.distances[0][carte.cellule(x, z)?];
        (d != INATTEIGNABLE).then_some(d as u32)
    }

    /// Direction unitaire à suivre depuis (x, z) : vers la case voisine la plus proche du
    /// joueur (diagonale seulement si elle ne coupe pas un angle). `None` si aucune.
    pub fn direction(&self, carte: &CarteMonde, x: f32, z: f32) -> Option<(f32, f32)> {
        self.direction_dans(&self.distances[0], carte, x, z)
    }

    /// Comme `direction`, pour un ennemi large de `taille` cases : par un passage assez
    /// large s'il y en a un, sinon par le plus court.
    pub fn direction_taille(
        &self,
        carte: &CarteMonde,
        x: f32,
        z: f32,
        taille: usize,
    ) -> Option<(f32, f32)> {
        let t = taille.clamp(1, TAILLE_MAX);
        (2..=t)
            .rev()
            .find_map(|t| self.direction_dans(&self.distances[t - 1], carte, x, z))
            .or_else(|| self.direction(carte, x, z))
    }

    fn direction_dans(
        &self,
        distances: &[i32],
        carte: &CarteMonde,
        x: f32,
        z: f32,
    ) -> Option<(f32, f32)> {
        let c = carte.cellule(x, z)?;
        if distances[c] == INATTEIGNABLE {
            return None;
        }
        let (largeur, profondeur) = (carte.largeur as i64, carte.profondeur as i64);
        let ix = c as i64 % largeur;
        let iz = c as i64 / largeur;
        let mut meilleure = None;
        let mut meilleure_distance = distances[c];
        for v in 0..8 {
            let nx = ix + VOISINS_X[v];
            let nz = iz + VOISINS_Z[v];
            if nx < 0 || nz < 0 || nx >= largeur || nz >= profondeur {
                continue;
            }
            let n = (nz * largeur + nx) as usize;
            let d = distances[n];
            if d == INATTEIGNABLE || d >= meilleure_distance {
                continue;
            }
            if v >= 4
                && (carte.drapeaux[(iz * largeur + nx) as usize] & BLOQUE_MARCHE != 0
                    || carte.drapeaux[(nz * largeur + ix) as usize] & BLOQUE_MARCHE != 0)
            {
                continue;
            }
            meilleure = Some(n);
            meilleure_distance = d;
        }
        let (cx, cz) = carte.centre(meilleure?);
        let dx = cx - x;
        let dz = cz - z;
        let mut l = (dx * dx + dz * dz).sqrt();
        if l == 0.0 {
            l = 1.0;
        }
        Some((dx / l, dz / l))
    }
}
