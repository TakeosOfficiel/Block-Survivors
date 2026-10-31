//! Instantané binaire de la simulation, envoyé par l'hôte aux invités. Positions en
//! entiers de 16 bits au 1/256 de bloc (précision ≈ 4 mm, arène de ±128 blocs), angle sur
//! un octet, vie en pourcentage : 6 octets par zombie vivant.

use bs_regles::Simulation;

const ECHELLE: f32 = 256.0;

fn position(v: f32) -> [u8; 2] {
    ((v * ECHELLE)
        .round()
        .clamp(i16::MIN as f32, i16::MAX as f32) as i16)
        .to_le_bytes()
}

fn angle(a: f32) -> u8 {
    ((a / core::f32::consts::TAU).rem_euclid(1.0) * 256.0) as u8
}

/// Écrit l'instantané de `s` dans `sortie` (vidé d'abord ; sa capacité est réutilisée).
pub fn ecrire_instantane(s: &Simulation, sortie: &mut Vec<u8>) {
    sortie.clear();
    sortie.extend_from_slice(&(s.pas as u32).to_le_bytes());
    sortie.extend_from_slice(&s.joueur.x.to_le_bytes());
    sortie.extend_from_slice(&s.joueur.z.to_le_bytes());
    sortie.extend_from_slice(&s.joueur.pv.to_le_bytes());
    sortie.extend_from_slice(&s.tues.to_le_bytes());
    let e = &s.ennemis;
    let vivants = e.vivant.iter().filter(|v| **v).count() as u16;
    sortie.extend_from_slice(&vivants.to_le_bytes());
    for i in 0..e.x.len() {
        if e.vivant[i] {
            sortie.extend_from_slice(&position(e.x[i]));
            sortie.extend_from_slice(&position(e.z[i]));
            sortie.push(angle(e.angle[i]));
            sortie.push((e.pv[i] / e.parametres.pv * 100.0).clamp(0.0, 255.0) as u8);
        }
    }
    let p = &s.projectiles;
    sortie.extend_from_slice(&(p.nombre as u16).to_le_bytes());
    for k in 0..p.nombre {
        sortie.extend_from_slice(&position(p.x[k]));
        sortie.extend_from_slice(&position(p.z[k]));
    }
}

/// Ce qu'un invité retrouve d'un instantané (tableaux réutilisés d'un instantané à l'autre).
#[derive(Default, Debug)]
pub struct Instantane {
    pub pas: u32,
    pub joueur: [f32; 3],
    pub tues: u32,
    /// x, z, angle (rad), vie (0 à 1) de chaque zombie vivant.
    pub ennemis: Vec<[f32; 4]>,
    pub projectiles: Vec<[f32; 2]>,
}

struct Lecteur<'a>(&'a [u8]);

impl Lecteur<'_> {
    fn octets<const N: usize>(&mut self) -> Option<[u8; N]> {
        let (a, reste) = self.0.split_at_checked(N)?;
        self.0 = reste;
        a.try_into().ok()
    }
    fn f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.octets()?))
    }
    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.octets()?))
    }
    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.octets()?))
    }
    fn position(&mut self) -> Option<f32> {
        Some(i16::from_le_bytes(self.octets()?) as f32 / ECHELLE)
    }
}

/// Lit un instantané dans `dans` ; faux s'il est tronqué.
pub fn lire_instantane(o: &[u8], dans: &mut Instantane) -> bool {
    lire(o, dans).is_some()
}

fn lire(o: &[u8], d: &mut Instantane) -> Option<()> {
    let mut l = Lecteur(o);
    d.pas = l.u32()?;
    d.joueur = [l.f32()?, l.f32()?, l.f32()?];
    d.tues = l.u32()?;
    d.ennemis.clear();
    for _ in 0..l.u16()? {
        let (x, z) = (l.position()?, l.position()?);
        let [a, v] = l.octets()?;
        d.ennemis.push([
            x,
            z,
            a as f32 / 256.0 * core::f32::consts::TAU,
            v as f32 / 100.0,
        ]);
    }
    d.projectiles.clear();
    for _ in 0..l.u16()? {
        d.projectiles.push([l.position()?, l.position()?]);
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bs_regles::{Commande, Parametres};

    #[test]
    fn relit_l_etat_a_4_mm_pres() {
        let mut s = Simulation::nouvelle(5, Parametres::default());
        for _ in 0..300 {
            s.avancer(Commande { dx: 1.0, dz: 0.3 });
        }
        let mut o = Vec::new();
        ecrire_instantane(&s, &mut o);
        let mut lu = Instantane::default();
        assert!(lire_instantane(&o, &mut lu));
        assert_eq!(lu.pas, 300);
        let vivants: Vec<usize> = (0..s.ennemis.x.len())
            .filter(|&i| s.ennemis.vivant[i])
            .collect();
        assert_eq!(lu.ennemis.len(), vivants.len());
        for (k, &i) in vivants.iter().enumerate() {
            assert!((lu.ennemis[k][0] - s.ennemis.x[i]).abs() <= 0.5 / ECHELLE);
        }
        assert!(!lire_instantane(&o[..o.len() - 1], &mut lu));
    }
}
