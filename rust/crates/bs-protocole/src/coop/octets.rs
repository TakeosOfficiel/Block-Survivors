//! Écriture et lecture séquentielles des messages binaires de la coop (petit-boutiste),
//! dans un tampon réutilisé. Positions au sol en entiers de 16 bits au 1/64 de bloc
//! (±512 blocs), angles en entiers de 16 bits au 1/10 000 de radian, parts sur un octet.

const ECHELLE_POSITION: f32 = 64.0;
const ECHELLE_ANGLE: f32 = 10_000.0;

/// Écrit dans un `Vec` vidé d'abord (sa capacité est réutilisée).
pub struct Ecrivain<'a> {
    pub octets: &'a mut Vec<u8>,
}

impl<'a> Ecrivain<'a> {
    pub fn nouveau(octets: &'a mut Vec<u8>) -> Self {
        octets.clear();
        Self { octets }
    }

    pub fn u8(&mut self, v: u8) {
        self.octets.push(v);
    }

    pub fn u16(&mut self, v: u16) {
        self.octets.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u32(&mut self, v: u32) {
        self.octets.extend_from_slice(&v.to_le_bytes());
    }

    pub fn f32(&mut self, v: f32) {
        self.octets.extend_from_slice(&v.to_le_bytes());
    }

    pub fn f64(&mut self, v: f64) {
        self.octets.extend_from_slice(&v.to_le_bytes());
    }

    /// Position au sol (blocs), quantifiée.
    pub fn coordonnee(&mut self, v: f32) {
        let q = (v * ECHELLE_POSITION)
            .round()
            .clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16;
        self.octets.extend_from_slice(&q.to_le_bytes());
    }

    /// Angle (rad), ramené dans [−π, π].
    pub fn angle(&mut self, v: f32) {
        let a = libm::atan2f(libm::sinf(v), libm::cosf(v));
        let q = (a * ECHELLE_ANGLE).round() as i16;
        self.octets.extend_from_slice(&q.to_le_bytes());
    }

    /// Nombre de 0 à 1 sur un octet.
    pub fn part(&mut self, v: f32) {
        self.u8((v.clamp(0.0, 1.0) * 255.0).round() as u8);
    }

    /// Réserve un u16 à écrire plus tard (compte d'une liste) ; sa position.
    pub fn reserver16(&mut self) -> usize {
        let p = self.octets.len();
        self.u16(0);
        p
    }

    pub fn ecrire16(&mut self, position: usize, v: u16) {
        self.octets[position..position + 2].copy_from_slice(&v.to_le_bytes());
    }
}

/// Lecture d'un message reçu ; une lecture au-delà de la fin rend 0 et marque l'erreur.
pub struct Lecteur<'a> {
    octets: &'a [u8],
    position: usize,
    pub erreur: bool,
}

impl<'a> Lecteur<'a> {
    pub fn nouveau(octets: &'a [u8]) -> Self {
        Self {
            octets,
            position: 0,
            erreur: false,
        }
    }

    fn prendre<const N: usize>(&mut self) -> [u8; N] {
        match self.octets.get(self.position..self.position + N) {
            Some(o) => {
                self.position += N;
                o.try_into().unwrap_or([0; N])
            }
            None => {
                self.erreur = true;
                [0; N]
            }
        }
    }

    pub fn u8(&mut self) -> u8 {
        self.prendre::<1>()[0]
    }

    pub fn u16(&mut self) -> u16 {
        u16::from_le_bytes(self.prendre())
    }

    pub fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.prendre())
    }

    pub fn f32(&mut self) -> f32 {
        let v = f32::from_le_bytes(self.prendre());
        if v.is_finite() { v } else { 0.0 }
    }

    pub fn f64(&mut self) -> f64 {
        let v = f64::from_le_bytes(self.prendre());
        if v.is_finite() { v } else { 0.0 }
    }

    pub fn coordonnee(&mut self) -> f32 {
        f32::from(i16::from_le_bytes(self.prendre())) / ECHELLE_POSITION
    }

    pub fn angle(&mut self) -> f32 {
        f32::from(i16::from_le_bytes(self.prendre())) / ECHELLE_ANGLE
    }

    pub fn part(&mut self) -> f32 {
        f32::from(self.u8()) / 255.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aller_retour() {
        let mut tampon = Vec::new();
        let mut w = Ecrivain::nouveau(&mut tampon);
        w.u8(7);
        let compte = w.reserver16();
        w.u32(123_456);
        w.f32(1.5);
        w.f64(99.25);
        w.coordonnee(-12.34);
        w.angle(7.0);
        w.part(0.5);
        w.ecrire16(compte, 3);
        let mut l = Lecteur::nouveau(&tampon);
        assert_eq!(l.u8(), 7);
        assert_eq!(l.u16(), 3);
        assert_eq!(l.u32(), 123_456);
        assert_eq!(l.f32(), 1.5);
        assert_eq!(l.f64(), 99.25);
        assert!((l.coordonnee() + 12.34).abs() < 1.0 / 64.0);
        assert!((l.angle() - (7.0 - std::f32::consts::TAU)).abs() < 1e-3);
        assert!((l.part() - 0.5).abs() < 0.01);
        assert!(!l.erreur);
        l.u32();
        assert!(l.erreur, "lecture au-delà de la fin");
    }
}
