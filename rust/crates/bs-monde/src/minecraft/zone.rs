//! Zone jouable d'une schématique (src/minecraft/importation.ts) : à partir du point de
//! départ (panneau « depart », sinon le centre), toutes les colonnes reliées à pied, sur
//! un sol plein avec deux blocs libres au-dessus. On monte d'un bloc (ou d'une dalle)
//! tout seul et l'on saute d'une falaise d'au plus `CHUTE` blocs ; une colonne n'a qu'un
//! sol, le premier atteint. Ce qui reste hors d'atteinte (murs, clôtures, champignons
//! géants…) devient le décor. Puis la carte de collision du rectangle de la zone.

use std::collections::VecDeque;

use super::proprietes::{Collision, Physique};
use super::schematique::Schematique;
use crate::carte::{BLOQUE_MARCHE, BLOQUE_TIR, CarteMonde, MARCHE};

const PRATICABLE: u8 = 1;
const LIQUIDE: u8 = 2;
/// Plus haute falaise dont on saute (blocs).
const CHUTE: i32 = 6;

#[derive(Debug, Clone)]
pub struct Zone {
    /// Niveau des pieds au départ (y de la première case libre au-dessus du sol) : la
    /// hauteur 0 du jeu.
    pub niveau: i32,
    /// Rectangle englobant, en cases de la schématique.
    pub x0: i32,
    pub z0: i32,
    pub largeur: usize,
    pub profondeur: usize,
    /// Cases du rectangle atteintes (1 : sol praticable, 2 : liquide au sol).
    pub cases: Vec<u8>,
    /// Niveau des pieds de chaque case atteinte (celui d'une voisine atteinte sinon).
    pub niveaux: Vec<i32>,
    /// Hauteur du sol de chaque case par rapport à `niveau` (dalles comprises).
    pub sols: Vec<f32>,
    pub depart_x: i32,
    pub depart_z: i32,
}

struct Volume<'a> {
    s: &'a Schematique,
    physiques: &'a [Physique],
}

impl Volume<'_> {
    fn physique(&self, x: i32, y: i32, z: i32) -> Option<&Physique> {
        self.s.bloc(x, y, z).map(|i| &self.physiques[i as usize])
    }

    fn collision(&self, x: i32, y: i32, z: i32) -> Option<Collision> {
        self.physique(x, y, z).map(|p| p.collision)
    }

    fn libre(&self, x: i32, y: i32, z: i32) -> bool {
        match self.collision(x, y, z) {
            Some(c) => c == Collision::Vide,
            None => y >= self.s.hauteur as i32,
        }
    }

    fn debout(&self, x: i32, y: i32, z: i32) -> bool {
        self.collision(x, y - 1, z) == Some(Collision::Plein)
            && self.libre(x, y, z)
            && self.libre(x, y + 1, z)
    }

    /// Hauteur absolue du sol sous des pieds au niveau `y`.
    fn sol(&self, x: i32, y: i32, z: i32) -> f32 {
        (y - 1) as f32 + self.physique(x, y - 1, z).map_or(1.0, |p| p.dessus)
    }

    /// Niveau des pieds en arrivant en (x, z) depuis une voisine aux pieds en `y` et au
    /// sol à `sol` : une marche plus haut, au même niveau ou en contrebas.
    fn arrivee(&self, x: i32, z: i32, y: i32, sol: f32) -> Option<i32> {
        if self.debout(x, y + 1, z) && self.sol(x, y + 1, z) <= sol + MARCHE {
            return Some(y + 1);
        }
        for yy in (y - CHUTE..=y).rev() {
            if !self.libre(x, yy + 1, z) {
                return None;
            }
            if self.debout(x, yy, z) {
                return (self.sol(x, yy, z) <= sol + MARCHE).then_some(yy);
            }
        }
        None
    }
}

pub fn trouver_zone(s: &Schematique, physiques: &[Physique]) -> Result<Zone, String> {
    let v = Volume { s, physiques };
    let (l, h, p) = (s.largeur as i32, s.hauteur as i32, s.profondeur as i32);
    let panneau = s.panneaux.iter().find(|p| {
        let t = p.texte.to_lowercase();
        t.contains("depart") || t.contains("départ")
    });
    let depart_x = panneau.map_or(l / 2, |p| p.x);
    let depart_z = panneau.map_or(p / 2, |p| p.z);
    // Le niveau le plus haut où l'on tient debout (près du panneau d'abord)
    let mut niveau = panneau.and_then(|pn| {
        (0..=2)
            .map(|dy| pn.y - dy)
            .find(|&y| v.debout(depart_x, y, depart_z))
    });
    if niveau.is_none() {
        niveau = (1..=h).rev().find(|&y| v.debout(depart_x, y, depart_z));
    }
    let niveau = niveau.ok_or(
        "Aucun sol praticable au point de départ : pose un panneau « depart » au sol, dans l'arène.",
    )?;

    // Parcours en largeur des colonnes reliées à pied (on traverse les étangs)
    let (lu, pu) = (s.largeur, s.profondeur);
    let mut etat = vec![0u8; lu * pu];
    let mut niveaux = vec![niveau; lu * pu];
    let mut file = VecDeque::new();
    let c0 = depart_z as usize * lu + depart_x as usize;
    etat[c0] = PRATICABLE;
    file.push_back((depart_x, depart_z));
    let (mut min_x, mut max_x, mut min_z, mut max_z) = (depart_x, depart_x, depart_z, depart_z);
    while let Some((x, z)) = file.pop_front() {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_z = min_z.min(z);
        max_z = max_z.max(z);
        let c = z as usize * lu + x as usize;
        let y = niveaux[c];
        let sol = if etat[c] == LIQUIDE {
            (y - 1) as f32 + 1.0
        } else {
            v.sol(x, y, z)
        };
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, nz) = (x + dx, z + dz);
            if nx < 0 || nz < 0 || nx >= l || nz >= p {
                continue;
            }
            let n = nz as usize * lu + nx as usize;
            if etat[n] != 0 {
                continue;
            }
            if let Some(yy) = v.arrivee(nx, nz, y, sol) {
                etat[n] = PRATICABLE;
                niveaux[n] = yy;
            } else if v.collision(nx, y - 1, nz) == Some(Collision::Liquide)
                && v.libre(nx, y, nz)
                && v.libre(nx, y + 1, nz)
            {
                etat[n] = LIQUIDE;
                niveaux[n] = y;
            } else {
                continue;
            }
            file.push_back((nx, nz));
        }
    }
    let largeur = (max_x - min_x + 1) as usize;
    let profondeur = (max_z - min_z + 1) as usize;
    let mut cases = vec![0u8; largeur * profondeur];
    let mut niv = vec![niveau; largeur * profondeur];
    let mut sols = vec![0.0; largeur * profondeur];
    for z in 0..profondeur {
        for x in 0..largeur {
            let i = (min_z as usize + z) * lu + min_x as usize + x;
            cases[z * largeur + x] = etat[i];
            niv[z * largeur + x] = niveaux[i];
        }
    }
    // Hors parcours : le niveau d'une voisine atteinte (pour les murets et les tirs)
    for _ in 0..2 {
        for z in 0..profondeur {
            for x in 0..largeur {
                let c = z * largeur + x;
                if cases[c] != 0 {
                    continue;
                }
                let voisins = [(1, 0), (-1, 0), (0, 1), (0, -1)].map(|(dx, dz)| {
                    let (nx, nz) = (x as i64 + dx, z as i64 + dz);
                    (nx >= 0 && nz >= 0 && nx < largeur as i64 && nz < profondeur as i64)
                        .then(|| nz as usize * largeur + nx as usize)
                });
                if let Some(n) = voisins.into_iter().flatten().find(|&n| cases[n] != 0) {
                    niv[c] = niv[n];
                }
            }
        }
    }
    for z in 0..profondeur {
        for x in 0..largeur {
            let c = z * largeur + x;
            let (wx, wz) = (min_x + x as i32, min_z + z as i32);
            sols[c] = match cases[c] {
                PRATICABLE => v.sol(wx, niv[c], wz) - niveau as f32,
                _ => (niv[c] - niveau) as f32,
            };
        }
    }
    Ok(Zone {
        niveau,
        x0: min_x,
        z0: min_z,
        largeur,
        profondeur,
        cases,
        niveaux: niv,
        sols,
        depart_x,
        depart_z,
    })
}

/// Carte de collision du rectangle de la zone : hors parcours, une case est
/// infranchissable (et arrête les tirs si un bloc plein s'y dresse) ; dans le parcours,
/// le sol et les blocs traversés donnent leurs effets.
pub fn construire_carte(s: &Schematique, physiques: &[Physique], zone: &Zone) -> CarteMonde {
    let v = Volume { s, physiques };
    let mut carte = CarteMonde::nouvelle(zone.largeur, zone.profondeur);
    for iz in 0..zone.profondeur {
        for ix in 0..zone.largeur {
            let c = iz * zone.largeur + ix;
            let (x, z) = (zone.x0 + ix as i32, zone.z0 + iz as i32);
            let y = zone.niveaux[c];
            carte.hauteurs[c] = zone.sols[c];
            let corps = [v.physique(x, y, z), v.physique(x, y + 1, z)];
            let sol = v.physique(x, y - 1, z);
            if zone.cases[c] != PRATICABLE {
                carte.drapeaux[c] |= BLOQUE_MARCHE;
                if corps
                    .iter()
                    .flatten()
                    .any(|p| p.collision == Collision::Plein)
                {
                    carte.drapeaux[c] |= BLOQUE_TIR;
                }
                if zone.cases[c] == LIQUIDE {
                    carte.degats[c] = sol.and_then(|p| p.degats).unwrap_or(0.0);
                }
                continue;
            }
            let mut ralentit = sol.and_then(|p| p.ralentit).unwrap_or(1.0);
            let mut hors_toiles = ralentit;
            let mut degats = sol.and_then(|p| p.degats).unwrap_or(0.0);
            for p in corps.into_iter().flatten() {
                if let Some(r) = p.ralentit {
                    ralentit = ralentit.min(r);
                    if !p.toile {
                        hors_toiles = hors_toiles.min(r);
                    }
                }
                degats = degats.max(p.degats.unwrap_or(0.0));
            }
            carte.ralentissement[c] = ralentit;
            carte.ralentissement_hors_toiles[c] = hors_toiles;
            carte.degats[c] = degats;
        }
    }
    carte
}
