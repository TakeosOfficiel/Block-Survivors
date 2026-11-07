//! Foudre : éclairs sur des ennemis au hasard autour du joueur, qui blessent tout dans leur
//! zone (port de src/combat/foudre.ts). Le tracé des éclairs est visuel, mais il tire dans
//! l'aléa du jeu : on garde le compte des éclairs affichés pour tirer pareil.
//! Talents du Mage lus ici : Haute tension (un arc vers un 2e ennemi), Conduction (arcs
//! depuis un ennemi ralenti), Tempête d'orage (éclairs de talent, `eclair_talent`).

use bs_donnees::armes::Mecanique;

use super::arme::ArmeEquipee;
use super::tours;
use crate::classes::valeur;
use crate::ennemis::Nature;
use crate::evenements::EffetArme;
use crate::partie::Partie;

/// Éclairs affichés à la fois, segments par éclair, durée d'affichage (s).
const CAPACITE: usize = 48;
const SEGMENTS: u32 = 10;
const DUREE: f32 = 0.25;
/// Talents : portée des arcs (blocs), Haute tension (part par rang), Conduction (voisins
/// par rang, part des dégâts) ; ennemis touchés au plus par les arcs d'un éclair.
const PORTEE_ARC: f32 = 4.0;
const HAUTE_TENSION: [f32; 2] = [0.3, 0.5];
const CONDUCTION_VOISINS: [f32; 2] = [2.0, 3.0];
const CONDUCTION_DEGATS: f32 = 0.6;
const ARCS_MAX: usize = 6;

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
        let tension = valeur(self.talents.rang("haute_tension"), &HAUTE_TENSION);
        let conduction = valeur(self.talents.rang("conduction"), &CONDUCTION_VOISINS);
        let mut tampon = [0u32; 64];
        for _ in 0..tours(s.nombre) {
            let Some(c) = self.ennemi_au_hasard(jx, jz, s.portee) else {
                break;
            };
            frappe = true;
            // Conduction : la cible était ralentie avant le coup
            let cible = &self.ennemis.pool[c];
            let ralentie = cible.ralenti_temps > 0.0 || cible.enracine > 0.0;
            let (x, z) = (cible.x, cible.z);
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
            if tension > 0.0 {
                self.arcs(arme.index, (x, z), c, 1, degats * tension);
            }
            if conduction > 0.0 && ralentie {
                let degats = degats * CONDUCTION_DEGATS;
                self.arcs(arme.index, (x, z), c, conduction as usize, degats);
            }
        }
        frappe
    }

    /// Arcs vers les `nombre` ennemis les plus proches de (x, z), sauf `exclu` : réactions,
    /// sans rebond.
    fn arcs(&mut self, arme: usize, (x, z): (f32, f32), exclu: usize, nombre: usize, degats: f32) {
        let mut voisins = [0u32; 64];
        let n = self.ennemis.chercher(x, z, PORTEE_ARC, &mut voisins);
        let mut frappes = [usize::MAX; ARCS_MAX];
        frappes[0] = exclu;
        let mut fait = 0;
        while fait < nombre && fait < ARCS_MAX - 1 {
            let mut meilleur = None;
            let mut meilleure_d2 = f32::INFINITY;
            for &k in &voisins[..n] {
                let k = k as usize;
                let e = &self.ennemis.pool[k];
                let d2 = (e.x - x) * (e.x - x) + (e.z - z) * (e.z - z);
                if !e.vivant()
                    || d2 >= meilleure_d2
                    || frappes[..=fait].contains(&k)
                    || self.a_couvert(x, z, k)
                {
                    continue;
                }
                meilleur = Some(k);
                meilleure_d2 = d2;
            }
            let Some(m) = meilleur else {
                return;
            };
            frappes[fait + 1] = m;
            self.blesser_reaction(m, degats, x, z, 0.0, Some(arme), Nature::Arme);
            let (mx, mz) = (self.ennemis.pool[m].x, self.ennemis.pool[m].z);
            self.creer_arc();
            self.ev.effet(arme, EffetArme::Eclair { zone: 0.5 }, mx, mz);
            self.ev.gerbe("impactFoudre", mx, 0.8, mz);
            fait += 1;
        }
    }

    /// Éclair d'un talent (Tempête d'orage) en (x, z) : réaction, critiques compris.
    pub(crate) fn eclair_talent(
        &mut self,
        arme: usize,
        (x, z): (f32, f32),
        degats: f32,
        zone: f32,
    ) {
        let mut tampon = [0u32; 64];
        let n = self.ennemis.chercher(x, z, zone, &mut tampon);
        for &e in &tampon[..n] {
            self.blesser_reaction(e as usize, degats, x, z, 1.0, Some(arme), Nature::Arme);
        }
        self.creer_eclair();
        self.ev.effet(arme, EffetArme::Eclair { zone }, x, z);
        self.ev.gerbe("impactFoudre", x, 0.3, z);
        self.ev.son("tonnerre");
    }

    /// Un arc à l'écran : ses points tirent trois fois l'aléa chacun.
    fn creer_arc(&mut self) {
        if self.armes.eclairs.vies.len() >= CAPACITE {
            return;
        }
        self.armes.eclairs.vies.push(DUREE);
        for _ in 0..=SEGMENTS * 3 + 2 {
            self.alea.tirer();
        }
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
