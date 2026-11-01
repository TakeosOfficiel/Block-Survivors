//! Invocations et talents de l'Invocateur : alliés temporaires hors quota (Lune de sang,
//! loups fantômes), allié proche du joueur (Garde rapprochée) et haie de racines entre
//! deux totems (Bosquet), port de src/combat/invocations.ts.

use bs_commun::maths::longueur;
use bs_donnees::armes::Mecanique;

use crate::partie::Partie;

/// Haie du Bosquet : distance max entre totems, ralenti, espacement des points, demi-largeur
/// (blocs), période des ralentissements (s).
pub const HAIE_DISTANCE: f32 = 8.0;
const HAIE_RALENTI: f32 = 0.4;
pub const HAIE_ESPACEMENT: f32 = 0.7;
const HAIE_LARGEUR: f32 = 0.9;
const HAIE_PERIODE: f32 = 0.5;

impl Partie {
    /// Invoque pour `duree` s `nombre` alliés de l'arme `i` hors quota, avec une part de
    /// leurs PV ; ils ne sont jamais renouvelés et leur fin n'est pas une mort.
    pub(crate) fn invoquer_temporaire(&mut self, i: usize, nombre: f32, duree: f32, part_pv: f32) {
        let Mecanique::Invocation { allie, .. } = &self.arsenal.armes[i].def.mecanique else {
            return;
        };
        let Some((type_allie, _, def)) = self.d.allies.allies.get_full(allie) else {
            return;
        };
        let mut k = 0.0;
        while k < nombre {
            k += 1.0;
            let Some(a) = self.creer(i, type_allie, def, true) else {
                return;
            };
            let a = &mut self.armes.invocations.pool[a];
            a.vie = duree;
            a.pv_max *= part_pv;
            a.pv = a.pv_max;
        }
    }

    /// Un allié vivant à moins de `rayon` de (x, z) ?
    pub(crate) fn allie_proche(&self, x: f32, z: f32, rayon: f32) -> bool {
        self.armes.invocations.pool.iter().any(|a| {
            a.arme.is_some()
                && a.pv > 0.0
                && (a.x - x) * (a.x - x) + (a.z - z) * (a.z - z) <= rayon * rayon
        })
    }

    /// Bosquet : deux totems assez proches font pousser entre eux une haie qui ralentit.
    pub(super) fn maj_haies(&mut self, dt: f32) {
        if self.talents.rang("bosquet") == 0 {
            return;
        }
        let inv = &mut self.armes.invocations;
        inv.minuterie_haie -= dt;
        if inv.minuterie_haie > 0.0 {
            return;
        }
        inv.minuterie_haie = HAIE_PERIODE;
        let mut tampon = [0u32; 128];
        let n = self.armes.invocations.pool.len();
        for i in 0..n {
            let a = self.armes.invocations.pool[i];
            if a.def.aura.is_none() || a.arme.is_none() {
                continue;
            }
            for k in i + 1..n {
                let b = self.armes.invocations.pool[k];
                if b.def.aura.is_none() || b.arme.is_none() {
                    continue;
                }
                let d = longueur(b.x - a.x, b.z - a.z);
                if d > HAIE_DISTANCE {
                    continue;
                }
                let pas = libm::ceilf(d / HAIE_ESPACEMENT).max(1.0) as u32;
                for s in 0..=pas {
                    let x = a.x + (b.x - a.x) * s as f32 / pas as f32;
                    let z = a.z + (b.z - a.z) * s as f32 / pas as f32;
                    let touches = self.ennemis.chercher(x, z, HAIE_LARGEUR, &mut tampon);
                    for &e in &tampon[..touches] {
                        self.ralentir(e as usize, HAIE_RALENTI, HAIE_PERIODE * 2.0);
                    }
                }
            }
        }
    }
}
