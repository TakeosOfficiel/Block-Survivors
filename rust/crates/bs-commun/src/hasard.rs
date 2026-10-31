//! Hasard reproductible repris de la version TypeScript (utils/aleatoire.ts et
//! utils/maths.ts) : graine tirée d'un texte (FNV-1a), suite mulberry32 et hachage de
//! position. Les mêmes valeurs qu'en TypeScript : une arène générée a le même décor.

/// Graine numérique tirée d'un texte (FNV-1a sur les unités UTF-16, comme `charCodeAt`).
pub fn graine(texte: &str) -> u32 {
    let mut h: u32 = 2_166_136_261;
    for unite in texte.encode_utf16() {
        h ^= u32::from(unite);
        h = h.wrapping_mul(16_777_619);
    }
    h
}

/// Suite mulberry32 : un nombre dans [0, 1) à chaque tirage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mulberry {
    etat: u32,
}

impl Mulberry {
    pub fn nouveau(depart: u32) -> Self {
        Self { etat: depart }
    }

    /// Nombre dans [0, 1), en double précision comme en TypeScript.
    pub fn tirer(&mut self) -> f64 {
        self.etat = self.etat.wrapping_add(0x6d2b_79f5);
        let mut t = self.etat;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        f64::from(t ^ (t >> 14)) / 4_294_967_296.0
    }

    /// Nombre dans [0, 1) en simple précision (simulation).
    pub fn unite(&mut self) -> f32 {
        // Arrondi vers le bas : 0,999 999 99 ne doit pas devenir 1
        let x = self.tirer() as f32;
        if x >= 1.0 {
            1.0 - f32::EPSILON / 2.0
        } else {
            x
        }
    }
}

/// Pseudo-aléatoire déterministe dans [0, 1) à partir d'une position entière.
pub fn hachage3(x: i32, y: i32, z: i32) -> f64 {
    let mut h =
        x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663) ^ z.wrapping_mul(83_492_791);
    h = (h ^ ((h as u32) >> 13) as i32).wrapping_mul(0x5bd1_e995);
    h ^= ((h as u32) >> 15) as i32;
    f64::from(h as u32) / 4_294_967_296.0
}

/// Index d'un élément d'une liste pondérée, choisi par un tirage dans [0, 1).
/// Liste vide : 0 (l'appelant garantit une liste non vide, validée au chargement).
pub fn choisir_pondere<T>(choix: &[T], poids: impl Fn(&T) -> f32, tirage: f64) -> usize {
    let total: f64 = choix.iter().map(|c| f64::from(poids(c))).sum();
    let mut seuil = tirage * total;
    for (i, c) in choix.iter().enumerate() {
        seuil -= f64::from(poids(c));
        if seuil < 0.0 {
            return i;
        }
    }
    choix.len().saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memes_valeurs_qu_en_typescript() {
        // Valeurs relevées avec la version TypeScript (node)
        assert_eq!(graine("prairie"), 2_963_214_559);
        let mut m = Mulberry::nouveau(1);
        assert_eq!(m.tirer(), 0.627_073_940_588_161_3);
        assert_eq!(m.tirer(), 0.002_735_721_180_215_478);
        assert_eq!(hachage3(-5, 0, 7), 0.161_332_551_855_593_92);
        assert_eq!(hachage3(123, 4, -99), 0.829_226_707_108_318_8);
    }

    #[test]
    fn pondere() {
        let poids = [1.0_f32, 3.0];
        assert_eq!(choisir_pondere(&poids, |p| *p, 0.1), 0);
        assert_eq!(choisir_pondere(&poids, |p| *p, 0.5), 1);
        assert_eq!(choisir_pondere(&poids, |p| *p, 0.999), 1);
    }
}
