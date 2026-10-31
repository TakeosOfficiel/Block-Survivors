//! Armes du joueur (prototype : une seule, des boules de feu tirées vers l'ennemi le plus
//! proche) et leurs projectiles, dans un tableau de capacité fixe.

use super::ennemis::Ennemis;

/// Réglages de l'arme du prototype.
#[derive(Clone, Debug)]
pub struct ParametresArme {
    /// s entre deux tirs.
    pub cadence: f32,
    pub portee: f32,
    pub vitesse: f32,
    pub degats: f32,
    pub rayon: f32,
    /// s de vol avant de disparaître.
    pub duree: f32,
    pub capacite: usize,
}

impl Default for ParametresArme {
    fn default() -> Self {
        Self {
            cadence: 0.12,
            portee: 14.0,
            vitesse: 16.0,
            degats: 10.0,
            rayon: 0.4,
            duree: 1.2,
            capacite: 128,
        }
    }
}

/// Projectiles en vol : les `nombre` premiers emplacements sont actifs.
pub struct Projectiles {
    pub parametres: ParametresArme,
    pub x: Vec<f32>,
    pub z: Vec<f32>,
    pub x_prec: Vec<f32>,
    pub z_prec: Vec<f32>,
    pub vx: Vec<f32>,
    pub vz: Vec<f32>,
    pub vie: Vec<f32>,
    pub nombre: usize,
    recharge: f32,
}

impl Projectiles {
    pub fn nouveaux(parametres: ParametresArme) -> Self {
        let n = parametres.capacite;
        Self {
            parametres,
            x: vec![0.0; n],
            z: vec![0.0; n],
            x_prec: vec![0.0; n],
            z_prec: vec![0.0; n],
            vx: vec![0.0; n],
            vz: vec![0.0; n],
            vie: vec![0.0; n],
            nombre: 0,
            recharge: 0.0,
        }
    }

    /// Tire si l'arme est prête, fait voler les projectiles, blesse les ennemis touchés ;
    /// renvoie le nombre d'ennemis tués.
    pub fn avancer(&mut self, dt: f32, jx: f32, jz: f32, ennemis: &mut Ennemis) -> u32 {
        let p = &self.parametres;
        let (cadence, portee, vitesse, degats, rayon, duree) =
            (p.cadence, p.portee, p.vitesse, p.degats, p.rayon, p.duree);
        self.recharge -= dt;
        if self.recharge <= 0.0
            && self.nombre < self.x.len()
            && let Some(cible) = plus_proche(ennemis, jx, jz, portee)
        {
            let (dx, dz) = (ennemis.x[cible] - jx, ennemis.z[cible] - jz);
            let d = (dx * dx + dz * dz).sqrt().max(1e-3);
            let k = self.nombre;
            self.nombre += 1;
            (self.x[k], self.z[k], self.x_prec[k], self.z_prec[k]) = (jx, jz, jx, jz);
            (self.vx[k], self.vz[k], self.vie[k]) = (dx / d * vitesse, dz / d * vitesse, duree);
            self.recharge = cadence;
        }
        let mut tues = 0;
        let mut k = 0;
        while k < self.nombre {
            self.x_prec[k] = self.x[k];
            self.z_prec[k] = self.z[k];
            self.x[k] += self.vx[k] * dt;
            self.z[k] += self.vz[k] * dt;
            self.vie[k] -= dt;
            let touche = premier_touche(ennemis, self.x[k], self.z[k], rayon);
            if let Some(i) = touche
                && ennemis.blesser(i, degats)
            {
                tues += 1;
            }
            if touche.is_some() || self.vie[k] <= 0.0 {
                self.liberer(k);
            } else {
                k += 1;
            }
        }
        tues
    }

    /// Libère l'emplacement `k` en y mettant le dernier actif.
    fn liberer(&mut self, k: usize) {
        self.nombre -= 1;
        let n = self.nombre;
        self.x[k] = self.x[n];
        self.z[k] = self.z[n];
        self.x_prec[k] = self.x_prec[n];
        self.z_prec[k] = self.z_prec[n];
        self.vx[k] = self.vx[n];
        self.vz[k] = self.vz[n];
        self.vie[k] = self.vie[n];
    }
}

/// Ennemi vivant le plus proche dans la portée (le plus petit indice en cas d'égalité).
fn plus_proche(e: &Ennemis, x: f32, z: f32, portee: f32) -> Option<usize> {
    let mut meilleur = None;
    let mut d_min = portee * portee;
    e.grille.pour_chaque_proche(x, z, portee, |i| {
        let d2 = (e.x[i] - x).powi(2) + (e.z[i] - z).powi(2);
        if e.vivant[i] && (d2 < d_min || (d2 == d_min && Some(i) < meilleur)) {
            d_min = d2;
            meilleur = Some(i);
        }
    });
    meilleur
}

/// Premier ennemi vivant touché (le plus petit indice, pour ne pas dépendre de l'ordre de la grille).
fn premier_touche(e: &Ennemis, x: f32, z: f32, rayon: f32) -> Option<usize> {
    let portee = rayon + e.parametres.rayon;
    let mut touche: Option<usize> = None;
    e.grille.pour_chaque_proche(x, z, portee, |i| {
        let d2 = (e.x[i] - x).powi(2) + (e.z[i] - z).powi(2);
        if e.vivant[i] && d2 <= portee * portee && touche.is_none_or(|t| i < t) {
            touche = Some(i);
        }
    });
    touche
}
