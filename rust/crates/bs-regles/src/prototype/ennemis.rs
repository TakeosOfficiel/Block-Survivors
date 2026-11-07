//! Ennemis : une colonne par champ, capacité fixe. Ils marchent vers le joueur en
//! s'écartant les uns des autres, frappent au contact, et réapparaissent hors de vue
//! après leur mort (le prototype garde une foule constante).

use bs_commun::Alea;

use super::grille::GrilleSpatiale;

/// Réglages des ennemis (données du prototype : des zombies).
#[derive(Clone, Debug)]
pub struct ParametresEnnemis {
    pub nombre: usize,
    pub vitesse: f32,
    pub pv: f32,
    pub rayon: f32,
    pub degats: f32,
    /// s entre deux coups au contact.
    pub cadence: f32,
    /// Distance d'apparition autour du joueur (blocs).
    pub distance_apparition: (f32, f32),
    /// s avant qu'un ennemi tué ne revienne.
    pub delai_retour: f32,
}

impl Default for ParametresEnnemis {
    fn default() -> Self {
        Self {
            nombre: 300,
            vitesse: 2.2,
            pv: 20.0,
            rayon: 0.35,
            degats: 1.0,
            cadence: 1.0,
            distance_apparition: (16.0, 24.0),
            delai_retour: 2.0,
        }
    }
}

pub struct Ennemis {
    pub parametres: ParametresEnnemis,
    pub x: Vec<f32>,
    pub z: Vec<f32>,
    /// Positions au pas précédent (le rendu interpole entre les deux).
    pub x_prec: Vec<f32>,
    pub z_prec: Vec<f32>,
    /// Orientation (rad), 0 = vers +Z.
    pub angle: Vec<f32>,
    pub pv: Vec<f32>,
    pub vivant: Vec<bool>,
    /// s avant le prochain coup (vivant) ou avant le retour (mort).
    pub minuterie: Vec<f32>,
    pub grille: GrilleSpatiale,
    alea: Alea,
}

impl Ennemis {
    pub fn nouveaux(
        parametres: ParametresEnnemis,
        largeur: f32,
        profondeur: f32,
        graine: u64,
    ) -> Self {
        let n = parametres.nombre;
        Self {
            parametres,
            x: vec![0.0; n],
            z: vec![0.0; n],
            x_prec: vec![0.0; n],
            z_prec: vec![0.0; n],
            angle: vec![0.0; n],
            pv: vec![0.0; n],
            vivant: vec![false; n],
            minuterie: vec![0.0; n],
            grille: GrilleSpatiale::nouvelle(largeur, profondeur, 2.0, n),
            alea: Alea::nouveau(graine, 2),
        }
    }

    /// Fait apparaître l'ennemi `i` autour de (jx, jz), dans l'arène.
    pub fn apparaitre(&mut self, i: usize, jx: f32, jz: f32, demi_l: f32, demi_p: f32) {
        let (dmin, dmax) = self.parametres.distance_apparition;
        let a = self.alea.entre(0.0, core::f32::consts::TAU);
        let d = self.alea.entre(dmin, dmax);
        let marge = 1.0;
        self.x[i] = (jx + libm::cosf(a) * d).clamp(-demi_l + marge, demi_l - marge);
        self.z[i] = (jz + libm::sinf(a) * d).clamp(-demi_p + marge, demi_p - marge);
        self.x_prec[i] = self.x[i];
        self.z_prec[i] = self.z[i];
        self.pv[i] = self.parametres.pv;
        self.vivant[i] = true;
        self.minuterie[i] = 0.0;
    }

    /// Un pas : marche, écartement, coups au contact ; renvoie les dégâts faits au joueur.
    pub fn avancer(&mut self, dt: f32, jx: f32, jz: f32, demi_l: f32, demi_p: f32) -> f32 {
        self.x_prec.copy_from_slice(&self.x);
        self.z_prec.copy_from_slice(&self.z);
        let p = self.parametres.clone_leger();
        let mut degats = 0.0;
        for i in 0..self.x.len() {
            if !self.vivant[i] {
                self.minuterie[i] -= dt;
                if self.minuterie[i] <= 0.0 {
                    self.apparaitre(i, jx, jz, demi_l, demi_p);
                }
                continue;
            }
            let (dx, dz) = (jx - self.x[i], jz - self.z[i]);
            let d = (dx * dx + dz * dz).sqrt();
            // Écartement : chaque voisin trop proche repousse
            let (mut ex, mut ez) = (0.0f32, 0.0f32);
            let (xi, zi, diametre) = (self.x[i], self.z[i], p.rayon * 2.0);
            let (xs, zs, vivants) = (&self.x, &self.z, &self.vivant);
            self.grille.pour_chaque_proche(xi, zi, diametre, |k| {
                if k == i || !vivants[k] {
                    return;
                }
                let (ox, oz) = (xi - xs[k], zi - zs[k]);
                let d2 = ox * ox + oz * oz;
                if d2 < diametre * diametre && d2 > 1e-6 {
                    let f = (diametre - d2.sqrt()) / diametre;
                    ex += ox * f;
                    ez += oz * f;
                }
            });
            if d > 0.6 {
                self.x[i] += (dx / d * p.vitesse + ex * 4.0) * dt;
                self.z[i] += (dz / d * p.vitesse + ez * 4.0) * dt;
                self.angle[i] = libm::atan2f(dx, dz);
            }
            self.x[i] = self.x[i].clamp(-demi_l, demi_l);
            self.z[i] = self.z[i].clamp(-demi_p, demi_p);
            // Coup au contact
            self.minuterie[i] -= dt;
            if d < 0.6 + p.rayon && self.minuterie[i] <= 0.0 {
                degats += p.degats;
                self.minuterie[i] = p.cadence;
            }
        }
        self.grille.reconstruire(&self.x, &self.z, &self.vivant);
        degats
    }

    /// Blesse l'ennemi `i` ; renvoie vrai s'il meurt.
    pub fn blesser(&mut self, i: usize, degats: f32) -> bool {
        self.pv[i] -= degats;
        if self.pv[i] > 0.0 {
            return false;
        }
        self.vivant[i] = false;
        self.minuterie[i] = self.parametres.delai_retour;
        true
    }
}

/// Copie des seuls réglages utiles dans la boucle (types simples, pas d'allocation).
struct Reglages {
    vitesse: f32,
    rayon: f32,
    degats: f32,
    cadence: f32,
}

impl ParametresEnnemis {
    fn clone_leger(&self) -> Reglages {
        Reglages {
            vitesse: self.vitesse,
            rayon: self.rayon,
            degats: self.degats,
            cadence: self.cadence,
        }
    }
}
