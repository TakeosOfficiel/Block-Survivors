//! Champ de flux vers le joueur : distance en cases depuis sa position (parcours en
//! largeur sur les cases franchissables), recalculée quand il change de case. Chaque
//! ennemi descend ce champ pour contourner les obstacles.

use crate::carte::{BLOQUE_MARCHE, CarteMonde};

const INATTEIGNABLE: i32 = -1;
const VOISINS_X: [i64; 8] = [1, -1, 0, 0, 1, 1, -1, -1];
const VOISINS_Z: [i64; 8] = [0, 0, 1, -1, 1, -1, 1, -1];

#[derive(Debug, Clone)]
pub struct ChampDeFlux {
    distances: Vec<i32>,
    file: Vec<u32>,
    cellule_cible: Option<usize>,
}

impl ChampDeFlux {
    pub fn nouveau(carte: &CarteMonde) -> Self {
        let cases = carte.largeur * carte.profondeur;
        Self {
            distances: vec![INATTEIGNABLE; cases],
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
        let (largeur, profondeur) = (carte.largeur as i64, carte.profondeur as i64);
        self.distances.fill(INATTEIGNABLE);
        self.distances[cible] = 0;
        let mut debut = 0;
        let mut fin = 0;
        self.file[fin] = cible as u32;
        fin += 1;
        while debut < fin {
            let c = self.file[debut] as usize;
            debut += 1;
            let ix = c as i64 % largeur;
            let iz = c as i64 / largeur;
            let d = self.distances[c] + 1;
            for v in 0..4 {
                let nx = ix + VOISINS_X[v];
                let nz = iz + VOISINS_Z[v];
                if nx < 0 || nz < 0 || nx >= largeur || nz >= profondeur {
                    continue;
                }
                let n = (nz * largeur + nx) as usize;
                if self.distances[n] != INATTEIGNABLE || carte.drapeaux[n] & BLOQUE_MARCHE != 0 {
                    continue;
                }
                self.distances[n] = d;
                self.file[fin] = n as u32;
                fin += 1;
            }
        }
    }

    /// Le joueur est-il atteignable à pied depuis (x, z) ?
    pub fn atteignable(&self, carte: &CarteMonde, x: f32, z: f32) -> bool {
        carte
            .cellule(x, z)
            .is_some_and(|c| self.distances[c] != INATTEIGNABLE)
    }

    /// Nombre de cases à parcourir à pied (sans diagonale) de (x, z) jusqu'au joueur.
    pub fn distance_marche(&self, carte: &CarteMonde, x: f32, z: f32) -> Option<u32> {
        let d = self.distances[carte.cellule(x, z)?];
        (d != INATTEIGNABLE).then_some(d as u32)
    }

    /// Direction unitaire à suivre depuis (x, z) : vers la case voisine la plus proche du
    /// joueur (diagonale seulement si elle ne coupe pas un angle). `None` si aucune.
    pub fn direction(&self, carte: &CarteMonde, x: f32, z: f32) -> Option<(f32, f32)> {
        let c = carte.cellule(x, z)?;
        if self.distances[c] == INATTEIGNABLE {
            return None;
        }
        let (largeur, profondeur) = (carte.largeur as i64, carte.profondeur as i64);
        let ix = c as i64 % largeur;
        let iz = c as i64 / largeur;
        let mut meilleure = None;
        let mut meilleure_distance = self.distances[c];
        for v in 0..8 {
            let nx = ix + VOISINS_X[v];
            let nz = iz + VOISINS_Z[v];
            if nx < 0 || nz < 0 || nx >= largeur || nz >= profondeur {
                continue;
            }
            let n = (nz * largeur + nx) as usize;
            let d = self.distances[n];
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
