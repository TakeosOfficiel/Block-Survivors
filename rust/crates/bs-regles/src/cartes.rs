//! Cartes d'amélioration : tirage des propositions à chaque niveau (nouvelle arme, rang
//! d'arme, éveil d'une arme au rang maximal, nouveau passif, rang de passif, ou soin si
//! tout est au maximum) et application du choix (port de src/progression/ameliorations.ts,
//! sans les textes affichés, qui viendront avec l'interface). Tampons préparés à
//! l'avance : rien n'est alloué pendant la partie.

use bs_donnees::Donnees;

use crate::partie::Partie;

/// Une carte de niveau.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Amelioration {
    /// Nouvelle arme ou rang de plus (index d'arme).
    Arme(usize),
    /// Éveil (variante 0 ou 1) d'une arme au rang maximal ; l'autre n'est plus proposée.
    Eveil {
        arme: usize,
        variante: usize,
    },
    /// Nouveau passif ou rang de plus (index de passif).
    Passif(usize),
    Soin,
}

#[derive(Debug, Clone)]
pub struct Cartes {
    /// Améliorations possibles, dans l'ordre du jeu TS (armes, éveils, passifs).
    possibles: Vec<Amelioration>,
    /// Dernières cartes tirées.
    pub proposees: Vec<Amelioration>,
    /// Par arme : réservée aux personnages à arsenal complet.
    reservees: Vec<bool>,
}

impl Cartes {
    pub fn nouvelles(d: &Donnees) -> Self {
        let personnages = d.personnages.personnages.values();
        let complets: Vec<_> = personnages.filter(|p| p.arsenal_complet).collect();
        let reservees = d
            .armes
            .keys()
            .map(|id| complets.iter().any(|p| p.arsenal.contains(id)))
            .collect();
        Self {
            possibles: Vec::with_capacity(d.armes.len() * 2 + d.passifs.len()),
            proposees: Vec::with_capacity(8),
            reservees,
        }
    }
}

impl Partie {
    /// L'arme `index` peut-elle être proposée au personnage joué ?
    fn arme_proposable(&self, index: usize) -> bool {
        let p = self.etat.personnage;
        let Some((id, _)) = self.d.armes.get_index(index) else {
            return false;
        };
        p.arsenal.contains(id) || (!p.arsenal_complet && !self.cartes.reservees[index])
    }

    /// Vies (totems) gagnées mais pas encore utilisées.
    fn reserve_vies(&self) -> f32 {
        self.etat.modificateurs.vies - self.etat.joueur.vies_utilisees as f32
    }

    fn ameliorations_possibles(&mut self) {
        let d = self.d;
        let progression = &d.regles.progression;
        let mut liste = std::mem::take(&mut self.cartes.possibles);
        liste.clear();
        let place_arme = (self.arsenal.armes.len() as u32) < progression.emplacements_armes;
        let possedes = &self.passifs.possedes;
        let occupes = possedes
            .iter()
            .filter(|p| d.passifs[p.index].consommable.is_none())
            .count();
        let place_passif = (occupes as u32) < progression.emplacements_passifs;
        for (index, def) in d.armes.values().enumerate() {
            if !self.arme_proposable(index) {
                continue;
            }
            let rang = self.arsenal.rang(index);
            let possible = if rang == 0 {
                place_arme
            } else {
                rang < crate::combat::arme::rang_max(def)
            };
            if possible {
                liste.push(Amelioration::Arme(index));
            }
        }
        // Éveils : les deux variantes d'une arme au rang maximal pas encore éveillée
        for arme in &self.arsenal.armes {
            let Some(eveils) = &arme.def.eveils else {
                continue;
            };
            if arme.eveil.is_some() || arme.rang < arme.rang_max() {
                continue;
            }
            for variante in 0..eveils.len() {
                liste.push(Amelioration::Eveil {
                    arme: arme.index,
                    variante,
                });
            }
        }
        let exclusifs = &self.etat.personnage.cartes_exclusives;
        for (index, (id, def)) in d.passifs.iter().enumerate() {
            if def.exclusif && !exclusifs.contains(id) {
                continue;
            }
            if let Some(c) = &def.consommable {
                if self.reserve_vies() < c.reserve_max {
                    liste.push(Amelioration::Passif(index));
                }
                continue;
            }
            let rang = self.passifs.rang(index);
            let possible = if rang == 0 {
                place_passif
            } else {
                (rang as usize) < def.niveaux.len()
            };
            if possible {
                liste.push(Amelioration::Passif(index));
            }
        }
        self.cartes.possibles = liste;
    }

    /// Propose jusqu'à `cartes_par_niveau` cartes différentes, tirées au hasard parmi les
    /// améliorations possibles (dans `cartes.proposees`).
    pub fn tirer_cartes(&mut self) -> &[Amelioration] {
        self.ameliorations_possibles();
        let nombre = self.d.regles.progression.cartes_par_niveau as usize;
        self.cartes.proposees.clear();
        while self.cartes.proposees.len() < nombre && !self.cartes.possibles.is_empty() {
            let n = self.cartes.possibles.len();
            let k = (self.alea.tirer() * n as f64) as usize;
            let choix = self.cartes.possibles.remove(k);
            self.cartes.proposees.push(choix);
        }
        if self.cartes.proposees.is_empty() {
            self.cartes.proposees.push(Amelioration::Soin);
        }
        &self.cartes.proposees
    }

    pub fn appliquer_amelioration(&mut self, amelioration: Amelioration) {
        match amelioration {
            Amelioration::Arme(index) => {
                self.ameliorer_arme(index);
            }
            Amelioration::Eveil { arme, variante } => {
                self.eveiller_arme(arme, variante);
            }
            Amelioration::Passif(index) => {
                let pv_max_avant = self.etat.modificateurs.pv_max;
                self.passifs.ameliorer(self.d, index);
                self.etat.modificateurs = self.passifs.calculer(self.d, None);
                // Les cœurs gagnés arrivent pleins (vides pour un joueur à terre)
                let gain = self.etat.modificateurs.pv_max - pv_max_avant;
                let j = &mut self.etat.joueur;
                j.pv_max += gain;
                if !j.mort {
                    j.pv += gain;
                }
                self.recalculer_armes();
            }
            Amelioration::Soin => {
                let j = &mut self.etat.joueur;
                let avant = j.pv;
                if !j.mort {
                    j.pv = j.pv_max.min(j.pv + self.d.regles.progression.soin.pv);
                }
                self.etat.stats.pv_soignes += j.pv - avant;
            }
        }
    }

    /// Éveille l'arme `index` (au rang maximal) avec la variante `variante` ; faux si
    /// c'est impossible.
    pub fn eveiller_arme(&mut self, index: usize, variante: usize) -> bool {
        let Some(k) = self.arsenal.armes.iter().position(|a| a.index == index) else {
            return false;
        };
        let arme = &mut self.arsenal.armes[k];
        let Some(eveil) = arme.def.eveils.as_ref().and_then(|e| e.get(variante)) else {
            return false;
        };
        if arme.eveil.is_some() || arme.rang < arme.rang_max() {
            return false;
        }
        arme.eveil = Some(variante);
        let m = self.etat.modificateurs;
        arme.recalculer(self.d, &m);
        self.talents.definir_eveil(index, eveil);
        self.arsenal.version += 1;
        true
    }
}
