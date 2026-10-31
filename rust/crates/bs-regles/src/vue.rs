//! Champ de la caméra de dessus, vu par la simulation : les ennemis apparaissent hors
//! de l'écran. Reproduit le suivi lissé du joueur et le test « sphère dans le champ »
//! de la caméra three.js (src/moteur/camera.ts, vue de dessus, sans zoom ni secousse).

use bs_commun::maths::{PI, cos, facteur_lissage, sin};

/// Réglages de la caméra (CONFIG.camera) et de l'écran du robot (1280 × 720).
const INCLINAISON_DEG: f32 = 55.0;
const FOV_DEG: f32 = 45.0;
const HAUTEUR_VISEE: f32 = 1.0;
const DISTANCE: f32 = 24.0;
const SUIVI: f32 = 10.0;
const PROCHE: f32 = 0.5;
const LOIN: f32 = 500.0;

#[derive(Debug, Clone)]
pub struct VueCamera {
    /// Point suivi (lissé).
    pub x: f32,
    pub z: f32,
    /// Largeur / hauteur de l'écran.
    pub rapport: f32,
    sin_inclinaison: f32,
    cos_inclinaison: f32,
    /// Demi-angles (rad) vertical et horizontal du champ.
    cos_v: f32,
    sin_v: f32,
    cos_h: f32,
    sin_h: f32,
}

impl VueCamera {
    pub fn nouvelle(x: f32, z: f32, rapport: f32) -> Self {
        let inclinaison = INCLINAISON_DEG * PI / 180.0;
        let demi_v = FOV_DEG * PI / 360.0;
        let demi_h = libm::atanf(libm::tanf(demi_v) * rapport);
        Self {
            x,
            z,
            rapport,
            sin_inclinaison: sin(inclinaison),
            cos_inclinaison: cos(inclinaison),
            cos_v: cos(demi_v),
            sin_v: sin(demi_v),
            cos_h: cos(demi_h),
            sin_h: sin(demi_h),
        }
    }

    /// Suit le joueur (après chaque pas de simulation).
    pub fn suivre(&mut self, x: f32, z: f32, dt: f32) {
        let k = facteur_lissage(SUIVI, dt);
        self.x += (x - self.x) * k;
        self.z += (z - self.z) * k;
    }

    /// Vrai si une sphère posée en (x, 1, z) de ce rayon apparaît à l'écran.
    pub fn est_visible(&self, x: f32, z: f32, rayon: f32) -> bool {
        // Position relative à la caméra, puis dans son repère (elle regarde vers −Z, penchée)
        let rx = x - self.x;
        let ry = 1.0 - (HAUTEUR_VISEE + DISTANCE * self.sin_inclinaison);
        let rz = z - (self.z + DISTANCE * self.cos_inclinaison);
        let (s, c) = (self.sin_inclinaison, self.cos_inclinaison);
        // Rotation inverse de celle de la caméra (−inclinaison autour de X)
        let y = ry * c - rz * s;
        let zc = ry * s + rz * c;
        let profondeur = -zc;
        let dedans = |distance: f32| distance >= -rayon;
        dedans(profondeur - PROCHE)
            && dedans(LOIN - profondeur)
            && dedans(-y * self.cos_v + profondeur * self.sin_v)
            && dedans(y * self.cos_v + profondeur * self.sin_v)
            && dedans(-rx * self.cos_h + profondeur * self.sin_h)
            && dedans(rx * self.cos_h + profondeur * self.sin_h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_joueur_est_visible_et_le_lointain_non() {
        let v = VueCamera::nouvelle(0.0, 0.0, 16.0 / 9.0);
        assert!(v.est_visible(0.0, 0.0, 0.5));
        assert!(!v.est_visible(40.0, 0.0, 0.5));
        assert!(!v.est_visible(0.0, -40.0, 0.5));
        // Le haut de l'écran voit plus loin que le bas (caméra penchée vers −Z)
        assert!(v.est_visible(0.0, -12.0, 0.5));
        assert!(!v.est_visible(0.0, 12.0, 0.5));
    }
}
