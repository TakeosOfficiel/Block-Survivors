//! Foudre : éclairs sur des ennemis au hasard autour du joueur, qui blessent tout dans leur
//! zone (port de src/combat/foudre.ts). Le tracé des éclairs est visuel, mais il tire dans
//! l'aléa du jeu : on garde le compte des éclairs affichés pour tirer pareil.

use bs_donnees::armes::Mecanique;

use super::arme::ArmeEquipee;
use super::tours;
use crate::ennemis::Nature;
use crate::evenements::EffetArme;
use crate::partie::Partie;

/// Éclairs affichés à la fois, segments par éclair, durée d'affichage (s).
const CAPACITE: usize = 48;
const SEGMENTS: u32 = 10;
const DUREE: f32 = 0.25;

/// Durées de vie restantes des éclairs affichés.
#[derive(Debug, Clone)]
pub struct Eclairs {
    vies: Vec<f32>,
}

impl Default for Eclairs {
    fn default() -> Self {
        Self {
            vies: Vec::with_capacity(CAPACITE),
        }
    }
}

impl Partie {
    pub(super) fn frapper_foudre(&mut self, arme: &ArmeEquipee) -> bool {
        if !matches!(arme.def.mecanique, Mecanique::Foudre { .. }) {
            return false;
        }
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let s = arme.stats;
        let degats = s.degats * self.talent_facteur_declenchement();
        let mut frappe = false;
        let mut tampon = [0u32; 64];
        for _ in 0..tours(s.nombre) {
            let Some(c) = self.ennemi_au_hasard(jx, jz, s.portee) else {
                break;
            };
            frappe = true;
            let (x, z) = (self.ennemis.pool[c].x, self.ennemis.pool[c].z);
            let n = self.ennemis.chercher(x, z, s.zone, &mut tampon);
            for &e in &tampon[..n] {
                self.blesser(
                    e as usize,
                    degats,
                    x,
                    z,
                    s.recul,
                    Some(arme.index),
                    Nature::Arme,
                );
            }
            self.creer_eclair();
            self.ev
                .effet(arme.index, EffetArme::Eclair { zone: s.zone }, x, z);
            self.ev.gerbe("impactFoudre", x, 0.3, z);
        }
        frappe
    }

    /// Un éclair de plus à l'écran : ses points en zigzag tirent deux fois l'aléa chacun.
    fn creer_eclair(&mut self) {
        if self.armes.eclairs.vies.len() >= CAPACITE {
            return;
        }
        self.armes.eclairs.vies.push(DUREE);
        for _ in 0..=SEGMENTS {
            self.alea.tirer();
            self.alea.tirer();
        }
    }

    pub fn maj_foudre(&mut self, dt: f32) {
        let vies = &mut self.armes.eclairs.vies;
        let mut i = vies.len();
        while i > 0 {
            i -= 1;
            vies[i] -= dt;
            if vies[i] <= 0.0 {
                vies.swap_remove(i);
            }
        }
    }
}
