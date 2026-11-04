//! Directeurs d'apparition (port de src/entites/directeur.ts et directeurBoss.ts) :
//! - ordinaire : des groupes d'ennemis hors du champ de la caméra, sur des cases libres
//!   d'où le joueur est atteignable, de plus en plus nombreux et résistants ; le type
//!   dépend de la zone de l'arène où le groupe apparaît ;
//! - boss et mini-boss : à intervalles réguliers (leur calendrier), hors champ.

use std::f64::consts::PI;

use bs_commun::maths::{cos, longueur, sin};
use bs_donnees::Intervalle;
use bs_donnees::arenes::ChoixEnnemi;
use bs_donnees::regles::{Apparitions, CalendrierBoss};
use bs_monde::{BLOQUE_MARCHE, CarteMonde};

use crate::evenements::Evenement;
use crate::partie::Partie;

/// Tirages pour trouver un point hors champ.
const ESSAIS_APPARITION: u32 = 16;

fn interpoler(p: &Intervalle, t: f32) -> f32 {
    p.debut + (p.fin - p.debut) * t
}

/// Arrondi de JavaScript (moitiés vers +∞).
fn arrondi_js(x: f32) -> f32 {
    libm::floorf(x + 0.5)
}

/// Directeur des apparitions ordinaires.
#[derive(Debug, Clone)]
pub struct DirecteurApparitions {
    minuterie: f32,
    /// Part du plafond d'ennemis simultanés, selon la surface marchable de la map.
    facteur_surface: f32,
}

impl Default for DirecteurApparitions {
    fn default() -> Self {
        Self {
            minuterie: 0.0,
            facteur_surface: 1.0,
        }
    }
}

impl DirecteurApparitions {
    /// Plafond réduit au prorata sur une map plus petite que `surface_reference`.
    pub fn selon_surface(carte: &CarteMonde, a: &Apparitions) -> Self {
        let marchables = carte
            .drapeaux
            .iter()
            .filter(|&&f| f & BLOQUE_MARCHE == 0)
            .count() as f32;
        let facteur = (marchables / a.surface_reference).clamp(a.facteur_surface_min, 1.0);
        Self {
            facteur_surface: facteur,
            ..Self::default()
        }
    }
}

/// Boss ou mini-boss.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SorteBoss {
    Boss,
    MiniBoss,
}

/// Calendrier d'arrivée des boss (ou des mini-boss).
#[derive(Debug, Clone)]
pub struct DirecteurBoss {
    pub sorte: SorteBoss,
    /// Index dans l'ordre d'arrivée de l'arène (puis en boucle).
    prochain: usize,
    moment: f32,
    annonce: bool,
}

impl DirecteurBoss {
    pub fn nouveau(sorte: SorteBoss, calendrier: &CalendrierBoss) -> Self {
        Self {
            sorte,
            prochain: 0,
            moment: calendrier.premier_passage,
            annonce: false,
        }
    }
}

impl Partie {
    pub fn maj_directeur(&mut self, dt: f32) {
        if self.etat.issue.is_some() || self.equipe_vaincue() {
            return;
        }
        let a = &self.d.regles.apparitions;
        let t = (self.etat.temps_jeu as f32 / a.duree_montee).min(1.0);
        self.directeur.minuterie -= dt;
        if self.directeur.minuterie > 0.0 {
            return;
        }
        self.directeur.minuterie += interpoler(&a.intervalle, t);

        let places = libm::floorf(interpoler(&a.max_simultanes, t) * self.directeur.facteur_surface)
            as i32
            - self.ennemis.nombre as i32;
        if places <= 0 {
            return;
        }
        let Some((px, pz)) = self.trouver_point_hors_champ() else {
            return;
        };
        let Some(index_type) = self.choisir_type(px, pz) else {
            return;
        };

        // Mode sans fin : groupes plus gros et ennemis plus robustes à chaque minute
        let s = &self.d.regles.progression.sans_fin;
        let minutes = self
            .etat
            .minutes_sans_fin(self.d.regles.progression.duree_partie);
        let groupe = arrondi_js(interpoler(&a.taille_groupe, t) + minutes * s.groupe_par_minute)
            .min(s.groupe_max);
        let taille = (groupe as i32).min(places);
        let pv = interpoler(&a.multiplicateur_pv, t) * (1.0 + minutes * s.pv_par_minute);
        for _ in 0..taille {
            let angle = (self.alea.tirer() * PI * 2.0) as f32;
            let r = (self.alea.tirer() * f64::from(a.dispersion_groupe)) as f32;
            let mut x = px + cos(angle) * r;
            let mut z = pz + sin(angle) * r;
            if self.carte.bloque_marche(x, z) {
                x = px;
                z = pz;
            }
            self.apparaitre(index_type, x, z, pv);
        }
    }

    /// Type tiré au hasard, selon les poids, parmi ceux déjà débloqués de la population
    /// de la première zone qui contient le point (sinon celle de l'arène).
    fn choisir_type(&mut self, x: f32, z: f32) -> Option<usize> {
        let arene = self.etat.arene;
        let population: &[ChoixEnnemi] = arene
            .zones
            .iter()
            .find(|zone| {
                let [x0, z0, x1, z1] = zone.rect;
                x >= x0 && x < x1 && z >= z0 && z < z1
            })
            .map_or(&arene.ennemis, |zone| &zone.ennemis);
        let temps = self.etat.temps_jeu;
        let debloque = |e: &ChoixEnnemi| f64::from(e.depuis) <= temps;
        let total: f64 = population
            .iter()
            .filter(|e| debloque(e))
            .map(|e| f64::from(e.poids))
            .sum();
        let mut seuil = self.alea.tirer() * total;
        for e in population.iter().filter(|e| debloque(e)) {
            seuil -= f64::from(e.poids);
            if seuil < 0.0 {
                return self.d.ennemis.get_index_of(&e.ennemi);
            }
        }
        None
    }

    /// Case libre, loin du joueur, d'où il est atteignable, et hors champ. Si la caméra
    /// voit toute la zone accessible, garde le point valable le plus éloigné.
    fn trouver_point_hors_champ(&mut self) -> Option<(f32, f32)> {
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let limite_x = self.etat.arene.largeur / 2.0 - 1.0;
        let limite_z = self.etat.arene.profondeur / 2.0 - 1.0;
        let distance_min = self.d.regles.apparitions.distance_min;
        let mut meilleure = -1.0;
        let mut point = None;
        for _ in 0..ESSAIS_APPARITION {
            let x = ((self.alea.tirer() * 2.0 - 1.0) * f64::from(limite_x)) as f32;
            let z = ((self.alea.tirer() * 2.0 - 1.0) * f64::from(limite_z)) as f32;
            let distance = longueur(x - jx, z - jz);
            if distance < distance_min
                || self.carte.bloque_marche(x, z)
                || !self.flux.atteignable(&self.carte, x, z)
            {
                continue;
            }
            if !self.vue.est_visible(x, z, 2.0) && !self.vu_par_un_autre(x, z, 2.0) {
                return Some((x, z));
            }
            if distance > meilleure {
                meilleure = distance;
                point = Some((x, z));
            }
        }
        point
    }

    /// Fait avancer le calendrier des boss puis celui des mini-boss.
    pub fn maj_directeurs_boss(&mut self) {
        for sorte in [SorteBoss::Boss, SorteBoss::MiniBoss] {
            self.maj_directeur_boss(sorte);
        }
    }

    fn directeur_boss(&mut self, sorte: SorteBoss) -> &mut DirecteurBoss {
        match sorte {
            SorteBoss::Boss => &mut self.directeur_boss,
            SorteBoss::MiniBoss => &mut self.directeur_mini_boss,
        }
    }

    fn calendrier(&self, sorte: SorteBoss) -> &'static CalendrierBoss {
        match sorte {
            SorteBoss::Boss => &self.d.regles.boss,
            SorteBoss::MiniBoss => &self.d.regles.mini_boss,
        }
    }

    fn liste_boss(&self, sorte: SorteBoss) -> &'static [String] {
        match sorte {
            SorteBoss::Boss => &self.etat.arene.boss,
            SorteBoss::MiniBoss => &self.etat.arene.mini_boss,
        }
    }

    /// Plus aucun boss avant la fin de la partie (sauf en mode sans fin).
    fn boss_termine(&self, sorte: SorteBoss) -> bool {
        let d = match sorte {
            SorteBoss::Boss => &self.directeur_boss,
            SorteBoss::MiniBoss => &self.directeur_mini_boss,
        };
        !self.etat.sans_fin && d.moment >= self.d.regles.progression.duree_partie
    }

    /// Secondes avant le prochain boss, ou None s'il n'en viendra plus avant la fin.
    pub fn temps_avant_boss(&self) -> Option<f32> {
        if self.boss_termine(SorteBoss::Boss) {
            return None;
        }
        Some((self.directeur_boss.moment - self.etat.temps_jeu as f32).max(0.0))
    }

    fn maj_directeur_boss(&mut self, sorte: SorteBoss) {
        let c = self.calendrier(sorte);
        if self.equipe_vaincue()
            || self.etat.issue.is_some()
            || self.boss_termine(sorte)
            || self.liste_boss(sorte).is_empty()
        {
            return;
        }
        let t = self.etat.temps_jeu as f32;
        let d = self.directeur_boss(sorte);
        if !d.annonce && t >= d.moment - c.avertissement {
            d.annonce = true;
            let prochain = d.prochain;
            let liste = self.liste_boss(sorte);
            let def = self.d.ennemis.get(&liste[prochain % liste.len()]);
            if let (SorteBoss::Boss, Some(def)) = (sorte, def) {
                self.ev.pousser(Evenement::AnnonceBoss {
                    nom: &def.nom,
                    duree: c.avertissement,
                });
            }
        }
        let d = self.directeur_boss(sorte);
        if t >= d.moment {
            self.faire_apparaitre_boss(sorte);
            let d = self.directeur_boss(sorte);
            d.moment += c.intervalle;
            d.annonce = false;
        }
    }

    /// « Continuer sans fin » après la victoire : la partie reprend, les boss reviennent.
    pub fn continuer_sans_fin(&mut self) {
        self.etat.issue = None;
        self.etat.sans_fin = true;
        self.reprendre_directeurs_boss();
    }

    /// Mode sans fin : les calendriers reprennent, le prochain venant au plus tôt un intervalle après maintenant.
    pub fn reprendre_directeurs_boss(&mut self) {
        let t = self.etat.temps_jeu as f32;
        for sorte in [SorteBoss::Boss, SorteBoss::MiniBoss] {
            let intervalle = self.calendrier(sorte).intervalle;
            let d = self.directeur_boss(sorte);
            d.moment = d.moment.max(t + intervalle);
            d.annonce = false;
        }
    }

    /// Apparition hors champ sur une case libre que le joueur peut atteindre (un boss ne
    /// traverse pas les murs : enfermé, il rendrait la victoire impossible) ; à défaut,
    /// sur n'importe quelle case atteignable, même en vue.
    fn faire_apparaitre_boss(&mut self, sorte: SorteBoss) {
        let liste = self.liste_boss(sorte);
        let d = self.directeur_boss(sorte);
        let id = &liste[d.prochain % liste.len()];
        d.prochain += 1;
        let Some((index_type, _, def)) = self.d.ennemis.get_full(id) else {
            return;
        };
        let (distance_min, son, secousse) = match sorte {
            SorteBoss::Boss => (12.0, Some("boss"), 0.6),
            SorteBoss::MiniBoss => (10.0, None, 0.0),
        };
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let limite_x = self.etat.arene.largeur / 2.0 - def.rayon - 1.0;
        let limite_z = self.etat.arene.profondeur / 2.0 - def.rayon - 1.0;
        let mut x = if jx > 0.0 { -limite_x } else { limite_x };
        let mut z = if jz > 0.0 { -limite_z } else { limite_z };
        for essai in 0..64 {
            let cx = ((self.alea.tirer() * 2.0 - 1.0) * f64::from(limite_x)) as f32;
            let cz = ((self.alea.tirer() * 2.0 - 1.0) * f64::from(limite_z)) as f32;
            if self.carte.bloque_marche(cx, cz) || !self.flux.atteignable(&self.carte, cx, cz) {
                continue;
            }
            let cache = longueur(cx - jx, cz - jz) > distance_min
                && !self.vue.est_visible(cx, cz, 3.0)
                && !self.vu_par_un_autre(cx, cz, 3.0);
            if cache || essai >= 48 {
                x = cx;
                z = cz;
                break;
            }
        }
        let a = &self.d.regles.apparitions;
        let montee = (self.etat.temps_jeu as f32 / a.duree_montee).min(1.0);
        let selon_temps = if self.calendrier(sorte).pv_selon_temps {
            interpoler(&a.multiplicateur_pv, montee)
        } else {
            1.0
        };
        let p = &self.d.regles.progression;
        let pv = selon_temps
            * (1.0 + self.etat.minutes_sans_fin(p.duree_partie) * p.sans_fin.pv_par_minute);
        self.apparaitre(index_type, x, z, pv);
        if let Some(son) = son {
            self.ev.son(son);
        }
        if secousse > 0.0 {
            self.ev.secousse(secousse);
        }
    }
}

#[cfg(test)]
mod essais {
    use super::*;

    /// Une arène du jeu garde tout son plafond ; une petite map en reçoit moins, sans
    /// descendre sous le minimum.
    #[test]
    fn plafond_selon_surface() {
        let d = bs_donnees::Donnees::embarquees().expect("données embarquées");
        let a = &d.regles.apparitions;
        let grande = DirecteurApparitions::selon_surface(&CarteMonde::nouvelle(60, 60), a);
        assert_eq!(grande.facteur_surface, 1.0);
        let petite = DirecteurApparitions::selon_surface(&CarteMonde::nouvelle(30, 30), a);
        assert!((petite.facteur_surface - 900.0 / a.surface_reference).abs() < 1e-6);
        let minuscule = DirecteurApparitions::selon_surface(&CarteMonde::nouvelle(5, 5), a);
        assert_eq!(minuscule.facteur_surface, a.facteur_surface_min);
    }
}
