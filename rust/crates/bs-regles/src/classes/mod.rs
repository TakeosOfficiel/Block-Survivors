//! Socle des mécaniques propres des talents de classe (src/combat/talents/talents.ts) :
//! mouvements du joueur (temps immobile, distance parcourue), soins plafonnés, coups
//! « en réaction » qui ne relancent pas les talents, cercles de particules, ordres aux
//! systèmes d'armes (Séisme, Zéro absolu, Écho). Chaque classe a son état et ses mécaniques
//! (guerrier/, archere/, mage/, invocateur/) ;
//! leurs rangs sont lus une fois, au début de la partie.

pub mod archere;
pub mod guerrier;
pub mod invocateur;
pub mod mage;

use bs_commun::maths::{cos, longueur, sin};
use bs_donnees::armes::FamilleArme;

use crate::ennemis::Nature;
use crate::partie::Partie;

pub use archere::Archere;
pub use guerrier::Guerrier;
pub use invocateur::Invocateur;
pub use mage::Mage;

/// Vitesse (blocs/s) sous laquelle le joueur compte comme immobile.
const SEUIL_MOUVEMENT: f32 = 0.5;

/// Valeur selon le rang : `valeurs[rang − 1]`, 0 sans le talent.
pub fn valeur(rang: u32, valeurs: &[f32]) -> f32 {
    if rang == 0 {
        return 0.0;
    }
    valeurs[(rang as usize).min(valeurs.len()) - 1]
}

/// État commun des mécaniques de classe.
#[derive(Debug, Clone, Default)]
pub struct Classes {
    /// s depuis le dernier pas, et s de marche continue.
    pub temps_immobile: f32,
    pub temps_mouvement: f32,
    /// Blocs parcourus depuis le début de la partie.
    pub parcouru: f32,
    pub dernier: (f32, f32),
    /// Soins des talents déjà rendus dans la seconde (plafond de régénération).
    soins_seconde: f32,
    minuterie_soins: f32,
    /// La prochaine Onde de choc part dans 6 directions (Séisme).
    pub onde_circulaire: bool,
    /// Dégâts de la flèche géante que tire ce coup de l'Arc (Puissance ; 0 : aucune), et
    /// sa portée.
    pub fleche_geante: f32,
    pub portee_fleche_geante: f32,
    /// Cette volée part aussi vers l'arrière (Vent arrière).
    pub volee_arriere: bool,
    /// La prochaine Nova de glace gèle (Zéro absolu).
    pub nova_gel: bool,
    /// Le sort déclenché est la répétition de l'Écho arcanique (dégâts réduits).
    pub echo: bool,
    pub guerrier: Option<Guerrier>,
    pub archere: Option<Archere>,
    pub mage: Option<Mage>,
    pub invocateur: Option<Invocateur>,
}

impl Partie {
    /// Mouvements du joueur et minuterie des soins, à chaque pas (même mort).
    pub(crate) fn maj_socle_classes(&mut self, dt: f32) {
        let (x, z) = (self.etat.joueur.x, self.etat.joueur.z);
        let c = &mut self.talents.classes;
        let pas = longueur(x - c.dernier.0, z - c.dernier.1);
        c.dernier = (x, z);
        c.parcouru += pas;
        if pas > SEUIL_MOUVEMENT * dt {
            c.temps_mouvement += dt;
            c.temps_immobile = 0.0;
        } else {
            c.temps_immobile += dt;
            c.temps_mouvement = 0.0;
        }
        c.minuterie_soins -= dt;
        if c.minuterie_soins <= 0.0 {
            c.minuterie_soins += 1.0;
            c.soins_seconde = 0.0;
        }
    }

    /// L'arme d'index `arme` a l'identifiant `id`.
    pub(crate) fn est_arme(&self, arme: usize, id: &str) -> bool {
        self.d.armes.get_index(arme).is_some_and(|(n, _)| n == id)
    }

    /// Famille de l'arme d'index `arme`.
    pub(crate) fn famille(&self, arme: Option<usize>) -> Option<FamilleArme> {
        arme.and_then(|a| self.d.armes[a].famille)
    }

    /// Un coup causé par un talent (réaction : ne relance pas les talents).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn blesser_reaction(
        &mut self,
        i: usize,
        degats: f32,
        sx: f32,
        sz: f32,
        recul: f32,
        arme: Option<usize>,
        nature: Nature,
    ) {
        let avant = self.talents.en_reaction;
        self.talents.en_reaction = true;
        self.blesser(i, degats, sx, sz, recul, arme, nature);
        self.talents.en_reaction = avant;
    }

    /// Dégâts de zone causés par un talent (réaction) ; renvoie le nombre de touchés.
    pub(crate) fn blesser_zone_reaction(
        &mut self,
        (x, z, rayon): (f32, f32, f32),
        degats: f32,
        arme: Option<usize>,
        recul: f32,
    ) -> usize {
        let mut tampon = [0u32; 256];
        let n = self.ennemis.chercher(x, z, rayon, &mut tampon);
        let mut touches = 0;
        for &e in &tampon[..n] {
            if self.a_couvert(x, z, e as usize) {
                continue;
            }
            self.blesser_reaction(e as usize, degats, x, z, recul, arme, Nature::Arme);
            touches += 1;
        }
        touches
    }

    /// Cercle de particules au sol (anneau, onde).
    pub(crate) fn cercle(&mut self, x: f32, z: f32, rayon: f32, gerbe: &'static str, points: u32) {
        for k in 0..points {
            let a = k as f32 / points as f32 * std::f32::consts::TAU;
            self.ev
                .gerbe(gerbe, x + sin(a) * rayon, 0.3, z + cos(a) * rayon);
        }
    }

    /// Soigne le joueur (statistiques comprises).
    pub(crate) fn soigner(&mut self, pv: f32) {
        let j = &mut self.etat.joueur;
        if j.mort || pv <= 0.0 {
            return;
        }
        let avant = j.pv;
        j.pv = j.pv_max.min(j.pv + pv);
        self.etat.stats.pv_soignes += j.pv - avant;
    }

    /// Soin limité par le plafond de régénération (par seconde, régénération comprise).
    pub(crate) fn soigner_plafonne(&mut self, pv: f32) {
        let j = &self.etat.joueur;
        let m = &self.etat.modificateurs;
        let regen = m.regeneration + m.regeneration_part * j.pv_max;
        let plafond = self.d.regles.plafonds.regeneration * j.pv_max;
        let c = &mut self.talents.classes;
        let soin = pv.min((plafond - regen - c.soins_seconde).max(0.0));
        c.soins_seconde += soin;
        self.soigner(soin);
    }
}
