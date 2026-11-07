//! Carte de l'arène, une case par bloc au sol : ce qui bloque la marche, ce qui arrête
//! les projectiles, et les effets de sol (ralentissement, dégâts). Fournit les
//! collisions cercle / blocs et les lignes de vue et de tir.
//!
//! Chaque case a la hauteur de son sol (blocs, par rapport au niveau de départ) : on
//! monte tout seul d'au plus `MARCHE` (un bloc, une dalle), on saute de n'importe quelle
//! falaise ; un projectile suit le relief à hauteur de corps et s'arrête sur une paroi.

/// Case infranchissable à pied (bloc plein ou liquide à hauteur de corps).
pub const BLOQUE_MARCHE: u8 = 1;
/// Case qui arrête les projectiles (bloc plein, même un muret d'un bloc).
pub const BLOQUE_TIR: u8 = 2;
/// Plus grande marche que l'on monte sans s'arrêter (un bloc).
pub const MARCHE: f32 = 1.05;
/// Hauteur de vol d'un projectile au-dessus du sol qu'il survole : il passe une marche
/// d'un bloc, pas un relief de deux.
pub const VOL: f32 = 1.2;

#[derive(Debug, Clone)]
pub struct CarteMonde {
    /// Cases d'ouest en est, et du nord au sud.
    pub largeur: usize,
    pub profondeur: usize,
    /// Demi-côtés : l'arène va de −demi_x à demi_x et de −demi_z à demi_z.
    pub demi_x: f32,
    pub demi_z: f32,
    pub drapeaux: Vec<u8>,
    pub ralentissement: Vec<f32>,
    /// Même chose sans les toiles (pour les araignées).
    pub ralentissement_hors_toiles: Vec<f32>,
    pub degats: Vec<f32>,
    /// Hauteur du sol de chaque case (0 : niveau de départ).
    pub hauteurs: Vec<f32>,
}

impl CarteMonde {
    pub fn nouvelle(largeur: usize, profondeur: usize) -> Self {
        let cases = largeur * profondeur;
        Self {
            largeur,
            profondeur,
            demi_x: largeur as f32 / 2.0,
            demi_z: profondeur as f32 / 2.0,
            drapeaux: vec![0; cases],
            ralentissement: vec![1.0; cases],
            ralentissement_hors_toiles: vec![1.0; cases],
            degats: vec![0.0; cases],
            hauteurs: vec![0.0; cases],
        }
    }

    /// Index de la case contenant (x, z), ou `None` hors de l'arène.
    pub fn cellule(&self, x: f32, z: f32) -> Option<usize> {
        let ix = (x + self.demi_x).floor();
        let iz = (z + self.demi_z).floor();
        if ix < 0.0 || iz < 0.0 || ix >= self.largeur as f32 || iz >= self.profondeur as f32 {
            return None;
        }
        Some(iz as usize * self.largeur + ix as usize)
    }

    /// Centre de la case `index`.
    pub fn centre(&self, index: usize) -> (f32, f32) {
        (
            (index % self.largeur) as f32 - self.demi_x + 0.5,
            (index / self.largeur) as f32 - self.demi_z + 0.5,
        )
    }

    /// Hauteur du sol en (x, z) (0 hors de l'arène).
    pub fn hauteur(&self, x: f32, z: f32) -> f32 {
        self.cellule(x, z).map_or(0.0, |c| self.hauteurs[c])
    }

    /// Hauteur du sol lissée sur les bords des cases (pour l'affichage : on monte une
    /// marche au lieu d'y sauter d'un coup).
    pub fn hauteur_lissee(&self, x: f32, z: f32) -> f32 {
        let h = self.hauteur(x, z);
        let mut m = h;
        for (dx, dz) in [(0.2, 0.0), (-0.2, 0.0), (0.0, 0.2), (0.0, -0.2)] {
            let v = self.hauteur(x + dx, z + dz);
            if v > m && v - h <= MARCHE {
                m = v;
            }
        }
        // Monte en douceur à l'approche d'une marche, tombe net d'une falaise
        h + (m - h) * 0.5
    }

    /// Peut-on passer de la case `de` à la case `vers` à pied (marche assez basse) ?
    pub fn pas_possible(&self, de: usize, vers: usize) -> bool {
        self.drapeaux[vers] & BLOQUE_MARCHE == 0
            && self.hauteurs[vers] <= self.hauteurs[de] + MARCHE
    }

    pub fn bloque_marche(&self, x: f32, z: f32) -> bool {
        self.cellule(x, z)
            .is_none_or(|c| self.drapeaux[c] & BLOQUE_MARCHE != 0)
    }

    pub fn bloque_tir(&self, x: f32, z: f32) -> bool {
        self.cellule(x, z)
            .is_none_or(|c| self.drapeaux[c] & BLOQUE_TIR != 0)
    }

    /// Multiplicateur de vitesse sur cette case ; `traverse_toiles` : sans tenir compte des toiles.
    pub fn vitesse_sol(&self, x: f32, z: f32, traverse_toiles: bool) -> f32 {
        match self.cellule(x, z) {
            None => 1.0,
            Some(c) if traverse_toiles => self.ralentissement_hors_toiles[c],
            Some(c) => self.ralentissement[c],
        }
    }

    pub fn degats_sol(&self, x: f32, z: f32) -> f32 {
        self.cellule(x, z).map_or(0.0, |c| self.degats[c])
    }

    /// Cases (ix0..=ix1, iz0..=iz1) touchées par un cercle, bornées à l'arène
    /// (intervalles vides si le cercle est dehors).
    fn cases_touchees(&self, x: f32, z: f32, rayon: f32) -> (i64, i64, i64, i64) {
        (
            ((x - rayon + self.demi_x).floor() as i64).max(0),
            ((x + rayon + self.demi_x).floor() as i64).min(self.largeur as i64 - 1),
            ((z - rayon + self.demi_z).floor() as i64).max(0),
            ((z + rayon + self.demi_z).floor() as i64).min(self.profondeur as i64 - 1),
        )
    }

    /// Repousse un cercle hors des cases infranchissables voisines (ou trop hautes pour
    /// qui se tient à la hauteur `h`) et le garde dans l'arène ; il glisse le long des
    /// obstacles.
    pub fn resoudre_collision(&self, x: f32, z: f32, rayon: f32, h: f32) -> (f32, f32) {
        let limite_x = self.demi_x - rayon;
        let limite_z = self.demi_z - rayon;
        let mut px = x.clamp(-limite_x, limite_x);
        let mut pz = z.clamp(-limite_z, limite_z);
        let (ix0, ix1, iz0, iz1) = self.cases_touchees(px, pz, rayon);
        for iz in iz0..=iz1 {
            for ix in ix0..=ix1 {
                let c = iz as usize * self.largeur + ix as usize;
                if self.drapeaux[c] & BLOQUE_MARCHE == 0 && self.hauteurs[c] <= h + MARCHE {
                    continue;
                }
                // Point de la case le plus proche du centre du cercle
                let x0 = ix as f32 - self.demi_x;
                let z0 = iz as f32 - self.demi_z;
                let qx = px.clamp(x0, x0 + 1.0);
                let qz = pz.clamp(z0, z0 + 1.0);
                let dx = px - qx;
                let dz = pz - qz;
                let d2 = dx * dx + dz * dz;
                if d2 >= rayon * rayon {
                    continue;
                }
                if d2 > 1e-10 {
                    let d = d2.sqrt();
                    px += dx / d * (rayon - d);
                    pz += dz / d * (rayon - d);
                } else {
                    // Centre dans la case : sortie par le côté le plus proche
                    let gauche = px - x0;
                    let droite = x0 + 1.0 - px;
                    let haut = pz - z0;
                    let bas = z0 + 1.0 - pz;
                    let m = gauche.min(droite).min(haut).min(bas);
                    if m == gauche {
                        px = x0 - rayon;
                    } else if m == droite {
                        px = x0 + 1.0 + rayon;
                    } else if m == haut {
                        pz = z0 - rayon;
                    } else {
                        pz = z0 + 1.0 + rayon;
                    }
                }
            }
        }
        (px, pz)
    }

    /// Vrai si un cercle en (x, z), au-dessus d'un sol à la hauteur `h`, touche une case
    /// qui arrête les tirs ou un relief plus haut que lui (hors de l'arène : faux, la
    /// sortie est gérée à part).
    pub fn touche_tir(&self, x: f32, z: f32, rayon: f32, h: f32) -> bool {
        let (ix0, ix1, iz0, iz1) = self.cases_touchees(x, z, rayon);
        for iz in iz0..=iz1 {
            for ix in ix0..=ix1 {
                let c = iz as usize * self.largeur + ix as usize;
                if self.drapeaux[c] & BLOQUE_TIR == 0 && self.hauteurs[c] < h + VOL {
                    continue;
                }
                let x0 = ix as f32 - self.demi_x;
                let z0 = iz as f32 - self.demi_z;
                let dx = x - x.clamp(x0, x0 + 1.0);
                let dz = z - z.clamp(z0, z0 + 1.0);
                if dx * dx + dz * dz < rayon * rayon {
                    return true;
                }
            }
        }
        false
    }

    /// Vrai si un projectile tiré de (x0, z0) vers (x1, z1) n'est arrêté par aucun bloc.
    pub fn ligne_de_tir(&self, x0: f32, z0: f32, x1: f32, z1: f32, rayon: f32) -> bool {
        let dx = x1 - x0;
        let dz = z1 - z0;
        let pas = ((dx * dx + dz * dz).sqrt() * 4.0).ceil() as u32;
        (1..pas).all(|i| {
            let t = i as f32 / pas as f32;
            let (x, z) = (x0 + dx * t, z0 + dz * t);
            !self.touche_tir(x, z, rayon, self.hauteur(x, z))
        })
    }

    /// Vrai si l'on va à pied en ligne droite de (x0, z0) à (x1, z1) : aucune case
    /// infranchissable ni marche trop haute (pas de 0,5 bloc).
    pub fn ligne_de_vue(&self, x0: f32, z0: f32, x1: f32, z1: f32) -> bool {
        let dx = x1 - x0;
        let dz = z1 - z0;
        let pas = ((dx * dx + dz * dz).sqrt() * 2.0).ceil() as u32;
        let Some(mut c) = self.cellule(x0, z0) else {
            return false;
        };
        (1..=pas).all(|i| {
            let t = i as f32 / pas.max(1) as f32;
            let Some(n) = self.cellule(x0 + dx * t, z0 + dz * t) else {
                return false;
            };
            let ok = n == c || self.pas_possible(c, n);
            c = n;
            ok
        })
    }
}
