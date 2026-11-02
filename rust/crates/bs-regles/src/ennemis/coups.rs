//! Coups portés aux ennemis (dégâts, critiques, recul, mort et butin) et leurs états :
//! brûlure, poison, saignement, ralenti, enracinement, étourdissement, marque.

use bs_commun::maths::longueur;
use bs_donnees::StatsArme;

use super::DegatsPeriodiques;
use crate::evenements::{Evenement, StyleChiffre};
use crate::partie::Partie;

/// Surcouche rouge après un coup (s).
pub const DUREE_BLESSURE: f32 = 0.3;
/// s avant l'attaque suivante d'un ennemi interrompu (coup de bouclier).
const INTERRUPTION: f32 = 1.0;

/// Origine d'un coup : arme du joueur, allié invoqué (profite de la marque), ou dégâts
/// d'un état (jamais critiques, sans recul ni son de blessure).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nature {
    Arme,
    Allie,
    Brulure,
    Poison,
    Saignement,
}

impl Nature {
    pub fn periodique(self) -> bool {
        matches!(self, Self::Brulure | Self::Poison | Self::Saignement)
    }
}

/// Ajoute des cumuls (plafonnés) et remet la durée à neuf ; les dégâts les plus forts l'emportent.
fn appliquer_periodique(
    p: &mut DegatsPeriodiques,
    cumuls: f32,
    degats_par_seconde: f32,
    duree: f32,
    arme: Option<usize>,
    cumuls_max: f32,
    intervalle: f32,
) {
    if p.cumuls == 0.0 {
        p.tic = intervalle;
        p.temps = 0.0;
        p.degats_par_seconde = 0.0;
    }
    p.cumuls = cumuls_max.min(p.cumuls + cumuls);
    p.temps = p.temps.max(duree);
    if degats_par_seconde >= p.degats_par_seconde {
        p.degats_par_seconde = degats_par_seconde;
        p.arme = arme;
    }
}

impl Partie {
    /// Blesse l'ennemi si l'arme ne l'a pas touché depuis moins de `delai` secondes.
    #[allow(clippy::too_many_arguments)]
    pub fn blesser_avec_delai(
        &mut self,
        i: usize,
        arme: usize,
        delai: f32,
        degats: f32,
        sx: f32,
        sz: f32,
        recul: f32,
    ) -> bool {
        let temps = self.etat.temps_jeu;
        let e = &mut self.ennemis.pool[i];
        if !e.vivant() || f64::from(e.immunites[arme]) > temps {
            return false;
        }
        e.immunites[arme] = (temps + f64::from(delai)) as f32;
        self.blesser(i, degats, sx, sz, recul, Some(arme), Nature::Arme);
        true
    }

    /// Blesse l'ennemi et le repousse depuis (sx, sz). `arme` crédite les dégâts.
    #[allow(clippy::too_many_arguments)]
    pub fn blesser(
        &mut self,
        i: usize,
        mut degats: f32,
        sx: f32,
        sz: f32,
        recul: f32,
        arme: Option<usize>,
        nature: Nature,
    ) {
        let e = &self.ennemis.pool[i];
        // Chez un invité, les coups ne blessent personne : les vrais viennent de l'hôte
        if !e.vivant() || self.invite {
            return;
        }
        let periodique = nature.periodique();
        let m = &self.etat.modificateurs;
        if nature == Nature::Allie && e.marque_temps > 0.0 {
            degats *= 1.0 + e.marque;
        }
        if e.ralenti > 0.0 && !periodique {
            degats *= 1.0 + m.degats_ralentis;
        }
        // Équipement : bonus contre la famille de l'ennemi
        use bs_donnees::entites::FamilleEnnemi as F;
        match e.def.famille {
            Some(F::MortVivant) => degats *= 1.0 + m.degats_morts_vivants,
            Some(F::Arthropode) => degats *= 1.0 + m.degats_arthropodes,
            Some(F::Nether) => degats *= 1.0 + m.degats_nether,
            None => {}
        }
        degats *= self.talent_multiplicateur(i, arme, nature);
        // Coup critique (armes du joueur) : dégâts multipliés et étincelles du jeu
        let m = &self.etat.modificateurs;
        let mut chance = m.critique;
        let base_critique = m.degats_critiques;
        if let Some(a) = arme
            && !periodique
        {
            chance += self.talent_bonus_critique(i, a, nature);
        }
        let critique = arme.is_some()
            && !periodique
            && self.alea.tirer() < f64::from(self.d.regles.plafonds.critique.min(chance));
        let (ex, ez) = (self.ennemis.pool[i].x, self.ennemis.pool[i].z);
        if critique {
            degats *= self.talent_facteur_critique(i, arme, nature, base_critique);
            self.ev.gerbe("critique", ex, 1.0, ez);
        }
        let e = &mut self.ennemis.pool[i];
        if let Some(a) = arme {
            let style = match nature {
                Nature::Brulure => StyleChiffre::Feu,
                Nature::Poison => StyleChiffre::Poison,
                _ if critique => StyleChiffre::Critique,
                _ => StyleChiffre::Normal,
            };
            let y = self.hauteurs[e.index_type];
            self.ev.pousser(Evenement::Degats {
                x: ex,
                y,
                z: ez,
                valeur: degats,
                style,
            });
            let s = &mut self.etat.stats;
            s.degats_par_arme[a] += f64::from(degats.min(e.pv));
            s.coups_par_arme[a] += 1;
            if critique {
                s.critiques_par_arme[a] += 1;
            }
            if degats > s.plus_gros_coup_par_arme[a] {
                s.plus_gros_coup_par_arme[a] = degats;
            }
        }
        e.pv -= degats;
        // Les dégâts d'un état ne font pas rougir : on garde la teinte de l'état
        if !periodique {
            e.blessure = DUREE_BLESSURE;
        }
        let (dx, dz) = (e.x - sx, e.z - sz);
        let d = longueur(dx, dz);
        let d = if d == 0.0 { 1.0 } else { d };
        let k = recul * (1.0 - e.def.resistance_recul.unwrap_or(0.0));
        e.vx += dx / d * k;
        e.vz += dz / d * k;
        let def = e.def;
        if e.pv <= 0.0 {
            e.mort = 0.0;
            e.meche = 0.0;
            let index_type = e.index_type;
            self.etat.ennemis_tues += 1;
            let s = &mut self.etat.stats;
            s.tues_par_type[index_type] += 1;
            if let Some(a) = arme {
                s.tues_par_arme[a] += 1;
            }
            if def.boss.is_some() {
                s.boss_vaincus += 1;
                self.ev.secousse(0.7);
            }
            self.lacher_butin(def, ex, ez);
            if let Some(sons) = &def.sons {
                self.ev.pousser(Evenement::Son {
                    son: &sons.mort,
                    hauteur: def.hauteur_sons,
                });
            }
            self.talent_sur_mort(i, arme, nature);
        } else if let Some(sons) = &def.sons
            && !periodique
        {
            self.ev.pousser(Evenement::Son {
                son: &sons.blessure,
                hauteur: def.hauteur_sons,
            });
        }
        // Après la mort éventuelle : un ennemi tombé ignore les coups suivants
        self.talent_apres_coup(i, degats, arme, critique, nature);
    }

    /// Coup d'une arme du joueur : ses dégâts, puis le ralenti et l'étourdissement de ses statistiques.
    #[allow(clippy::too_many_arguments)]
    pub fn toucher(
        &mut self,
        i: usize,
        arme: usize,
        s: &StatsArme,
        sx: f32,
        sz: f32,
        recul: f32,
        facteur: f32,
    ) {
        self.blesser(
            i,
            s.degats * facteur,
            sx,
            sz,
            recul,
            Some(arme),
            Nature::Arme,
        );
        if s.ralenti > 0.0 {
            self.ralentir(i, s.ralenti, s.duree);
        }
        if s.etourdi > 0.0 {
            self.etourdir(i, s.etourdi);
        }
    }

    /// Coupe l'attaque en préparation (mèche du creeper, arc bandé) et retarde la suivante.
    pub fn interrompre(&mut self, i: usize) {
        let e = &mut self.ennemis.pool[i];
        if e.vivant() {
            e.meche = 0.0;
            e.minuterie = e.minuterie.max(INTERRUPTION);
        }
    }

    // --- États ---------------------------------------------------------------

    /// Brûlure : `cumuls` de plus (plafonnés), durée remise à neuf ; les plus forts dégâts l'emportent.
    pub fn bruler(&mut self, i: usize, cumuls: f32, dps: f32, duree: f32, arme: Option<usize>) {
        let max = self.c.etats.brulure.cumuls_max.unwrap_or(1) as f32;
        self.bruler_max(i, cumuls, dps, duree, arme, max);
    }

    pub fn bruler_max(
        &mut self,
        i: usize,
        cumuls: f32,
        dps: f32,
        duree: f32,
        arme: Option<usize>,
        cumuls_max: f32,
    ) {
        let m = &self.etat.modificateurs;
        let (dps, duree, max) = (
            dps * (1.0 + m.degats_brulure),
            duree * (1.0 + m.duree_brulure),
            cumuls_max + m.cumuls_brulure,
        );
        let tic = self.c.etats.brulure.tic.unwrap_or(1.0);
        let e = &mut self.ennemis.pool[i];
        if e.vivant() {
            appliquer_periodique(&mut e.brulure, cumuls, dps, duree, arme, max, tic);
        }
    }

    /// Saignement (Saignée) : comme le poison, avec ses propres cumuls.
    pub fn saigner(&mut self, i: usize, cumuls: f32, dps: f32, duree: f32, arme: Option<usize>) {
        let et = self.c.etats.saignement;
        let e = &mut self.ennemis.pool[i];
        if e.vivant() {
            let max = et.cumuls_max.unwrap_or(1) as f32;
            appliquer_periodique(
                &mut e.saignement,
                cumuls,
                dps,
                duree,
                arme,
                max,
                et.tic.unwrap_or(1.0),
            );
        }
    }

    /// Poison : comme la brûlure, avec ses propres cumuls.
    pub fn empoisonner(
        &mut self,
        i: usize,
        cumuls: f32,
        dps: f32,
        duree: f32,
        arme: Option<usize>,
    ) {
        let max = self.c.etats.poison.cumuls_max.unwrap_or(1) as f32;
        self.empoisonner_max(i, cumuls, dps, duree, arme, max);
    }

    pub fn empoisonner_max(
        &mut self,
        i: usize,
        cumuls: f32,
        dps: f32,
        duree: f32,
        arme: Option<usize>,
        max: f32,
    ) {
        let dps = dps * (1.0 + self.etat.modificateurs.degats_poison);
        let tic = self.c.etats.poison.tic.unwrap_or(1.0);
        let e = &mut self.ennemis.pool[i];
        if e.vivant() {
            appliquer_periodique(&mut e.poison, cumuls, dps, duree, arme, max, tic);
        }
    }

    /// Ralentit de `part` (0 à 1) ; le plus fort l'emporte. Un boss résiste.
    pub fn ralentir(&mut self, i: usize, mut part: f32, duree: f32) {
        let duree = duree * (1.0 + self.etat.modificateurs.duree_ralenti);
        let r = &self.d.regles.resistance_boss;
        let e = &mut self.ennemis.pool[i];
        if !e.vivant() {
            return;
        }
        if e.boss() {
            part = r.ralenti_max.min(part * r.ralenti);
        }
        appliquer_ralenti(e, part, duree);
    }

    /// Immobilise sans empêcher d'attaquer ; un boss est seulement ralenti.
    pub fn enraciner(&mut self, i: usize, duree: f32) {
        let part = self.d.regles.resistance_boss.enracinement;
        let e = &mut self.ennemis.pool[i];
        if !e.vivant() {
            return;
        }
        if e.boss() {
            appliquer_ralenti(e, part, duree);
        } else {
            e.enracine = e.enracine.max(duree);
        }
    }

    /// Ni marche ni attaque ; sans effet sur un boss.
    pub fn etourdir(&mut self, i: usize, duree: f32) {
        let e = &mut self.ennemis.pool[i];
        if e.vivant() && !e.boss() {
            e.etourdi = e.etourdi.max(duree);
        }
    }

    /// Marque : les alliés invoqués lui font `bonus` de dégâts en plus (la plus forte l'emporte).
    pub fn marquer(&mut self, i: usize, bonus: f32, duree: f32) {
        let e = &mut self.ennemis.pool[i];
        if e.vivant() && (bonus >= e.marque || e.marque_temps <= 0.0) {
            e.marque = bonus;
            e.marque_temps = e.marque_temps.max(duree);
        }
    }

    /// Durées des états, dégâts par seconde et particules.
    pub(super) fn maj_etats(&mut self, i: usize, dt: f32) {
        let et = self.c.etats;
        self.tiquer(i, Nature::Brulure, et.brulure.tic.unwrap_or(1.0), dt);
        self.tiquer(i, Nature::Poison, et.poison.tic.unwrap_or(1.0), dt);
        self.tiquer(i, Nature::Saignement, et.saignement.tic.unwrap_or(1.0), dt);
        let temps = self.etat.temps_jeu;
        let e = &mut self.ennemis.pool[i];
        if !e.vivant() {
            return;
        }
        if e.marque_chasseur > 0.0 {
            e.marque_chasseur -= dt;
        }
        if e.dans_piege && temps > e.piege {
            e.dans_piege = false;
            self.talent_sur_sortie_piege(i);
        }
        let e = &mut self.ennemis.pool[i];
        if e.temps_cumuls > 0.0 {
            e.temps_cumuls -= dt;
            if e.temps_cumuls <= 0.0 {
                e.cumuls_talent = 0.0;
            }
        }
        if e.encercle > 0.0 {
            e.encercle -= dt;
        }
        if e.ralenti_temps > 0.0 {
            e.ralenti_temps -= dt;
            if e.ralenti_temps <= 0.0 {
                e.ralenti = 0.0;
            }
        }
        if e.marque_temps > 0.0 {
            e.marque_temps -= dt;
            if e.marque_temps <= 0.0 {
                e.marque = 0.0;
            }
        }
        if e.enracine > 0.0 {
            e.enracine -= dt;
        }
        if e.etourdi > 0.0 {
            e.etourdi -= dt;
        }

        e.minuterie_etats -= dt;
        if e.minuterie_etats > 0.0 {
            return;
        }
        let periode = et.brulure.periode;
        e.minuterie_etats = periode;
        // Particules (les tirages comptent : ils avancent l'aléatoire de la simulation)
        let (x, z, h) = (e.x, e.z, self.hauteurs[e.index_type]);
        let (brule, empoisonne, saigne) = (
            e.brulure.cumuls > 0.0,
            e.poison.cumuls > 0.0,
            e.saignement.cumuls > 0.0,
        );
        let ralenti = e.ralenti_temps > 0.0 || e.enracine > 0.0;
        let etourdi = e.etourdi > 0.0;
        let marque = e.marque_temps > 0.0 || e.marque_chasseur > 0.0;
        if brule {
            self.ev.gerbe(&et.brulure.gerbe, x, h * 0.45, z);
        } else if empoisonne && self.alea.tirer() < f64::from(periode / et.poison.periode) {
            self.ev.gerbe(&et.poison.gerbe, x, h * 0.5, z);
        }
        if ralenti && self.alea.tirer() < f64::from(periode / et.ralenti.periode) {
            self.ev.gerbe(&et.ralenti.gerbe, x, h * 0.5, z);
        }
        if saigne && self.alea.tirer() < f64::from(periode / et.saignement.periode) {
            self.ev.gerbe(&et.saignement.gerbe, x, h * 0.5, z);
        }
        if etourdi {
            self.ev.gerbe(&et.etourdi.gerbe, x, h - 0.1, z);
        }
        if marque && self.alea.tirer() < f64::from(periode / et.marque.periode) {
            self.ev.gerbe(&et.marque.gerbe, x, h - 0.1, z);
        }
    }

    /// Dégâts d'une brûlure, d'un poison ou d'un saignement à chaque intervalle.
    fn tiquer(&mut self, i: usize, nature: Nature, intervalle: f32, dt: f32) {
        let e = &mut self.ennemis.pool[i];
        let p = match nature {
            Nature::Brulure => &mut e.brulure,
            Nature::Poison => &mut e.poison,
            _ => &mut e.saignement,
        };
        if p.cumuls == 0.0 {
            return;
        }
        p.temps -= dt;
        p.tic -= dt;
        let mut coup = None;
        if p.tic <= 0.0 {
            p.tic += intervalle;
            coup = Some((p.degats_par_seconde * p.cumuls * intervalle, p.arme));
        }
        let (x, z) = (e.x, e.z);
        if let Some((degats, arme)) = coup {
            self.blesser(i, degats, x, z, 0.0, arme, nature);
        }
        let e = &mut self.ennemis.pool[i];
        let p = match nature {
            Nature::Brulure => &mut e.brulure,
            Nature::Poison => &mut e.poison,
            _ => &mut e.saignement,
        };
        if p.temps <= 0.0 {
            p.cumuls = 0.0;
        }
    }

    /// Index des ennemis vivants en feu ; renvoie leur nombre.
    pub fn brulants(&self, sortie: &mut [u32]) -> usize {
        let mut n = 0;
        for (i, e) in self.ennemis.actifs().iter().enumerate() {
            if n >= sortie.len() {
                break;
            }
            if e.vivant() && e.brulure.cumuls > 0.0 {
                sortie[n] = i as u32;
                n += 1;
            }
        }
        n
    }
}

fn appliquer_ralenti(e: &mut super::Ennemi, part: f32, duree: f32) {
    if part >= e.ralenti || e.ralenti_temps <= 0.0 {
        e.ralenti = part;
        e.ralenti_temps = e.ralenti_temps.max(duree);
    }
}
