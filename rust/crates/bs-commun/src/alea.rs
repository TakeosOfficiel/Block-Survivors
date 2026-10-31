//! Générateur pseudo-aléatoire à graine (PCG32, O'Neill) : même graine, même suite, en
//! natif comme en WebAssembly. Un générateur par système de la simulation ; jamais
//! d'aléatoire du système d'exploitation dans la simulation (rejeux, tests, robot).

/// PCG32 (XSH RR, état 64 bits) : petit, rapide, sans allocation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alea {
    etat: u64,
    flux: u64,
}

const MULTIPLICATEUR: u64 = 6_364_136_223_846_793_005;

impl Alea {
    /// Générateur semé ; `flux` sépare les suites de plusieurs systèmes d'une même graine.
    pub fn nouveau(graine: u64, flux: u64) -> Self {
        let mut a = Self {
            etat: 0,
            flux: (flux << 1) | 1,
        };
        a.suivant();
        a.etat = a.etat.wrapping_add(graine);
        a.suivant();
        a
    }

    /// Entier sur 32 bits.
    pub fn suivant(&mut self) -> u32 {
        let ancien = self.etat;
        self.etat = ancien.wrapping_mul(MULTIPLICATEUR).wrapping_add(self.flux);
        let melange = (((ancien >> 18) ^ ancien) >> 27) as u32;
        melange.rotate_right((ancien >> 59) as u32)
    }

    /// Nombre dans [0, 1).
    pub fn unite(&mut self) -> f32 {
        (self.suivant() >> 8) as f32 * (1.0 / 16_777_216.0)
    }

    /// Nombre dans [min, max).
    pub fn entre(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.unite()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meme_graine_meme_suite() {
        let mut a = Alea::nouveau(42, 1);
        let mut b = Alea::nouveau(42, 1);
        for _ in 0..1000 {
            assert_eq!(a.suivant(), b.suivant());
        }
        let mut c = Alea::nouveau(42, 2);
        assert_ne!(Alea::nouveau(42, 1).suivant(), c.suivant());
    }

    #[test]
    fn unite_dans_l_intervalle() {
        let mut a = Alea::nouveau(7, 0);
        for _ in 0..10_000 {
            let x = a.unite();
            assert!((0.0..1.0).contains(&x));
        }
    }
}
