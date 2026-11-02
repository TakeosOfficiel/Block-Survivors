//! Champ de la caméra de dessus, vu par la simulation : les ennemis apparaissent hors
//! de l'écran. Reproduit le suivi lissé du joueur et le test « sphère dans le champ »
//! de la caméra three.js (src/moteur/camera.ts, sans zoom ni secousse) : vue de dessus,
//! ou à la 3e personne quand l'affichage la donne (orientation et distance).

use bs_commun::maths::{PI, cos, facteur_lissage, sin};

/// Réglages de la caméra (CONFIG.camera) et de l'écran du robot (1280 × 720).
const INCLINAISON_DEG: f32 = 55.0;
const FOV_DEG: f32 = 45.0;
const HAUTEUR_VISEE: f32 = 1.0;
const DISTANCE: f32 = 24.0;
const SUIVI: f32 = 10.0;
const PROCHE: f32 = 0.5;
const LOIN: f32 = 500.0;
/// Vue à la 3e personne (CONFIG.camera.troisieme).
const FOV_TROISIEME_DEG: f32 = 60.0;
const HAUTEUR_VISEE_TROISIEME: f32 = 1.8;
const SUIVI_TROISIEME: f32 = 30.0;

/// Caméra à la 3e personne : orientation (rad) et recul (blocs).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Troisieme {
    pub lacet: f32,
    pub tangage: f32,
    pub distance: f32,
}

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
    /// Vue à la 3e personne (`None` : de dessus).
    pub troisieme: Option<Troisieme>,
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
            troisieme: None,
        }
    }

    /// Suit le joueur (après chaque pas de simulation).
    pub fn suivre(&mut self, x: f32, z: f32, dt: f32) {
        let raideur = if self.troisieme.is_some() {
            SUIVI_TROISIEME
        } else {
            SUIVI
        };
        let k = facteur_lissage(raideur, dt);
        self.x += (x - self.x) * k;
        self.z += (z - self.z) * k;
    }

    /// Vrai si une sphère posée en (x, 1, z) de ce rayon apparaît à l'écran.
    pub fn est_visible(&self, x: f32, z: f32, rayon: f32) -> bool {
        if let Some(t) = self.troisieme {
            return self.visible_troisieme(t, x, z, rayon);
        }
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

    /// Même test pour la caméra à la 3e personne : derrière le point visé, à l'opposé du
    /// regard, tournée de `lacet` autour de la verticale puis penchée de `tangage`.
    fn visible_troisieme(&self, t: Troisieme, x: f32, z: f32, rayon: f32) -> bool {
        let (sl, cl) = (sin(t.lacet), cos(t.lacet));
        let (st, ct) = (sin(t.tangage), cos(t.tangage));
        let horizontal = ct * t.distance;
        let cy = (HAUTEUR_VISEE_TROISIEME + st * t.distance).max(0.3);
        let (rx, ry, rz) = (
            x - (self.x + sl * horizontal),
            1.0 - cy,
            z - (self.z + cl * horizontal),
        );
        // Rotation inverse : lacet autour de Y, puis tangage autour de X
        let x1 = rx * cl - rz * sl;
        let z1 = rx * sl + rz * cl;
        let y = ry * ct - z1 * st;
        let zc = ry * st + z1 * ct;
        let profondeur = -zc;
        let demi_v = FOV_TROISIEME_DEG * PI / 360.0;
        let demi_h = libm::atanf(libm::tanf(demi_v) * self.rapport);
        let (sv, cv, sh, ch) = (sin(demi_v), cos(demi_v), sin(demi_h), cos(demi_h));
        let dedans = |distance: f32| distance >= -rayon;
        dedans(profondeur - PROCHE)
            && dedans(LOIN - profondeur)
            && dedans(-y * cv + profondeur * sv)
            && dedans(y * cv + profondeur * sv)
            && dedans(-x1 * ch + profondeur * sh)
            && dedans(x1 * ch + profondeur * sh)
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

    #[test]
    fn la_troisieme_personne_voit_devant() {
        let mut v = VueCamera::nouvelle(0.0, 0.0, 16.0 / 9.0);
        // Lacet 0 : caméra au sud (+Z), regard vers −Z
        v.troisieme = Some(Troisieme {
            lacet: 0.0,
            tangage: 0.5,
            distance: 12.0,
        });
        assert!(v.est_visible(0.0, -30.0, 0.5));
        assert!(!v.est_visible(0.0, 20.0, 0.5));
        assert!(!v.est_visible(40.0, 0.0, 0.5));
        // Demi-tour : on voit le sud
        v.troisieme = Some(Troisieme {
            lacet: PI,
            tangage: 0.5,
            distance: 12.0,
        });
        assert!(v.est_visible(0.0, 30.0, 0.5));
        assert!(!v.est_visible(0.0, -20.0, 0.5));
    }
}
