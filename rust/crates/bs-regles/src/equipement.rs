//! Équipement de la partie : 2 bagues et 1 amulette au plus. Les coffres en proposent
//! (objets du monde et de tous les mondes, rareté tirée selon le coffre, ou renforcement
//! d'un objet porté). Tout est porté au maximum : le coffre garde son intérêt, il offre
//! un niveau, des soins complets ou un sac d'émeraudes. Emplacements pleins : on choisit
//! ensuite l'objet à remplacer. Les bonus passent par les passifs, les effets propres
//! par les talents (port de src/progression/equipement.ts, sans les textes).

use bs_donnees::Donnees;
use bs_donnees::equipement::{Emplacement, PoidsRaretes, Rarete};

use crate::partie::Partie;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjetPorte {
    /// Index dans la table des objets d'équipement.
    pub objet: usize,
    pub niveau: u32,
}

/// Une carte de coffre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoixEquipement {
    Objet(usize),
    Renfort(usize),
    /// Prendre `objet` à la place de l'objet porté à l'emplacement `index`.
    Remplacer {
        objet: usize,
        index: usize,
    },
    /// Ne rien remplacer : revenir aux propositions du coffre.
    Retour,
    Emeraudes(u32),
    /// Équipement au maximum : passer au niveau suivant (cartes de niveau).
    Niveau,
    /// Équipement au maximum : PV au maximum.
    Soin,
}

#[derive(Debug, Clone)]
pub struct Equipement {
    pub bagues: Vec<Option<ObjetPorte>>,
    pub amulettes: Vec<Option<ObjetPorte>>,
    /// Incrémenté à chaque changement (interface).
    pub version: u32,
    /// Dernières cartes tirées (coffre, puis remplacement).
    pub proposees: Vec<ChoixEquipement>,
    reserve: Vec<usize>,
    candidats: Vec<usize>,
    pris: Vec<usize>,
}

impl Equipement {
    pub fn nouveau(d: &Donnees) -> Self {
        let r = &d.equipement.regles;
        let n = d.equipement.objets.len();
        Self {
            bagues: vec![None; r.bagues as usize],
            amulettes: vec![None; r.amulettes as usize],
            version: 0,
            proposees: Vec::with_capacity(8),
            reserve: Vec::with_capacity(n),
            candidats: Vec::with_capacity(n),
            pris: Vec::with_capacity(8),
        }
    }

    fn emplacements(&mut self, e: Emplacement) -> &mut Vec<Option<ObjetPorte>> {
        match e {
            Emplacement::Bague => &mut self.bagues,
            Emplacement::Amulette => &mut self.amulettes,
        }
    }

    /// Objets portés, bagues puis amulette.
    pub fn portes(&self) -> impl Iterator<Item = ObjetPorte> + '_ {
        self.bagues.iter().chain(&self.amulettes).flatten().copied()
    }

    fn porte_mut(&mut self, objet: usize) -> Option<&mut ObjetPorte> {
        let liste = self.bagues.iter_mut().chain(&mut self.amulettes);
        liste.flatten().find(|o| o.objet == objet)
    }
}

pub fn niveau_max_objet(d: &Donnees, objet: usize) -> u32 {
    if d.equipement.objets[objet].rarete == Rarete::Legendaire {
        1
    } else {
        d.equipement.regles.niveaux.len() as u32
    }
}

fn facteur(d: &Donnees, o: ObjetPorte) -> f32 {
    d.equipement.regles.niveaux[o.niveau as usize - 1]
}

impl Partie {
    /// Vrai si prendre cet objet oblige à en retirer un (emplacements pleins).
    pub fn remplacement_necessaire(&mut self, objet: usize) -> bool {
        let e = self.d.equipement.objets[objet].emplacement;
        !self.equipement.emplacements(e).contains(&None)
    }

    /// Propositions d'un coffre (`ender` : coffre de boss ; `ouverts` : coffres payants
    /// déjà ouverts), dans `equipement.proposees`.
    pub fn tirer_equipement(&mut self, ender: bool, ouverts: u32) -> &[ChoixEquipement] {
        let d = self.d;
        let r = &d.equipement.regles;
        let monde = self.etat.arene.monde;
        let eq = &mut self.equipement;
        eq.reserve.clear();
        for (k, def) in d.equipement.objets.values().enumerate() {
            let porte = eq
                .bagues
                .iter()
                .chain(&eq.amulettes)
                .flatten()
                .any(|o| o.objet == k);
            if def.monde.is_none_or(|m| m == monde)
                && !porte
                && (ender || def.rarete != Rarete::Legendaire)
            {
                eq.reserve.push(k);
            }
        }
        eq.pris.clear();
        eq.proposees.clear();
        // Coffre de boss : la légendaire du monde, tant qu'on ne la porte pas
        if ender
            && let Some(&legendaire) = eq
                .reserve
                .iter()
                .find(|&&k| d.equipement.objets[k].rarete == Rarete::Legendaire)
        {
            eq.pris.push(legendaire);
            eq.proposees.push(ChoixEquipement::Objet(legendaire));
        }
        while self.equipement.proposees.len() < r.propositions as usize {
            let eq = &self.equipement;
            let renfort = eq
                .portes()
                .find(|o| o.niveau < niveau_max_objet(d, o.objet) && !eq.pris.contains(&o.objet));
            if let Some(o) = renfort
                && self.alea.tirer() < f64::from(r.chance_renfort)
            {
                self.equipement.pris.push(o.objet);
                self.equipement
                    .proposees
                    .push(ChoixEquipement::Renfort(o.objet));
                continue;
            }
            let rarete = self.tirer_rarete(ender, ouverts);
            let eq = &mut self.equipement;
            let objets = &d.equipement.objets;
            let libre = |k: &usize, pris: &[usize]| !pris.contains(k);
            eq.candidats.clear();
            for &k in &eq.reserve {
                if libre(&k, &eq.pris) && objets[k].rarete == rarete {
                    eq.candidats.push(k);
                }
            }
            // Plus rien de cette rareté : une autre permise par ce coffre, sinon n'importe laquelle
            let permises = if ender { &r.ender.poids } else { &r.bois.poids };
            if eq.candidats.is_empty() {
                for &k in &eq.reserve {
                    if libre(&k, &eq.pris) && permises.contains_key(&objets[k].rarete) {
                        eq.candidats.push(k);
                    }
                }
            }
            if eq.candidats.is_empty() {
                for &k in &eq.reserve {
                    if libre(&k, &eq.pris) {
                        eq.candidats.push(k);
                    }
                }
            }
            if eq.candidats.is_empty() {
                let Some(o) = renfort else {
                    break;
                };
                eq.pris.push(o.objet);
                eq.proposees.push(ChoixEquipement::Renfort(o.objet));
                continue;
            }
            let n = eq.candidats.len();
            let k = eq.candidats[(self.alea.tirer() * n as f64) as usize];
            let eq = &mut self.equipement;
            eq.pris.push(k);
            eq.proposees.push(ChoixEquipement::Objet(k));
        }
        if self.equipement.proposees.is_empty() {
            // Plus rien à trouver : des trésors à la place
            let nombre = r.emeraudes_consolation as u32;
            let tresors = [
                ChoixEquipement::Niveau,
                ChoixEquipement::Soin,
                ChoixEquipement::Emeraudes(nombre),
            ];
            self.equipement.proposees.extend(tresors);
            return &self.equipement.proposees;
        }
        // Ordre mélangé (la légendaire n'est pas toujours la première)
        for i in (1..self.equipement.proposees.len()).rev() {
            let j = (self.alea.tirer() * (i + 1) as f64) as usize;
            self.equipement.proposees.swap(i, j);
        }
        // Rien ne plaît : quelques émeraudes à la place, toujours en dernier
        if !matches!(self.equipement.proposees[0], ChoixEquipement::Emeraudes(_)) {
            let refus = &r.emeraudes_refus;
            let nombre = if ender { refus.ender } else { refus.bois } as u32;
            self.equipement
                .proposees
                .push(ChoixEquipement::Emeraudes(nombre));
        }
        &self.equipement.proposees
    }

    /// Cartes de la 2e étape : quel objet porté retirer pour prendre `objet`, ou revenir.
    pub fn cartes_remplacement(&mut self, objet: usize) -> &[ChoixEquipement] {
        let e = self.d.equipement.objets[objet].emplacement;
        let eq = &mut self.equipement;
        eq.proposees.clear();
        let n = eq.emplacements(e).len();
        for index in 0..n {
            if eq.emplacements(e)[index].is_some() {
                eq.proposees
                    .push(ChoixEquipement::Remplacer { objet, index });
            }
        }
        eq.proposees.push(ChoixEquipement::Retour);
        &self.equipement.proposees
    }

    pub fn appliquer_equipement(&mut self, choix: ChoixEquipement) {
        let d = self.d;
        let eq = &mut self.equipement;
        match choix {
            ChoixEquipement::Objet(objet) => {
                let places = eq.emplacements(d.equipement.objets[objet].emplacement);
                let Some(libre) = places.iter_mut().find(|p| p.is_none()) else {
                    return;
                };
                *libre = Some(ObjetPorte { objet, niveau: 1 });
                self.trouver_objet(objet);
            }
            ChoixEquipement::Remplacer { objet, index } => {
                let places = eq.emplacements(d.equipement.objets[objet].emplacement);
                let Some(place) = places.get_mut(index) else {
                    return;
                };
                *place = Some(ObjetPorte { objet, niveau: 1 });
                self.trouver_objet(objet);
            }
            ChoixEquipement::Renfort(objet) => {
                let Some(o) = eq.porte_mut(objet) else {
                    return;
                };
                if o.niveau >= niveau_max_objet(d, objet) {
                    return;
                }
                o.niveau += 1;
            }
            ChoixEquipement::Emeraudes(nombre) => {
                self.etat.emeraudes += nombre;
                return;
            }
            ChoixEquipement::Niveau => {
                let j = &self.etat.joueur;
                let manque = (j.xp_niveau - j.xp).max(0.0);
                crate::etat::gagner_xp(d, &mut self.etat, manque);
                return;
            }
            ChoixEquipement::Soin => {
                let j = &mut self.etat.joueur;
                self.etat.stats.pv_soignes += j.pv_max - j.pv;
                j.pv = j.pv_max;
                return;
            }
            ChoixEquipement::Retour => return,
        }
        self.recalculer_equipement();
    }

    /// Bonus et effets des objets portés, puis PV : les PV gagnés arrivent pleins, ceux
    /// perdus bornent les PV actuels.
    fn recalculer_equipement(&mut self) {
        let d = self.d;
        let pv_max_avant = self.etat.modificateurs.pv_max;
        let objets = &d.equipement.objets;
        let bonus = self.equipement.portes().filter_map(|o| {
            let modif = objets[o.objet].modif.as_ref()?;
            Some(modif.fois(facteur(d, o)))
        });
        self.passifs.definir_equipement(bonus);
        self.etat.modificateurs = self.passifs.calculer(d, None);
        let gain = self.etat.modificateurs.pv_max - pv_max_avant;
        let j = &mut self.etat.joueur;
        j.pv_max += gain;
        j.pv = if gain > 0.0 && !j.mort {
            j.pv + gain
        } else {
            j.pv.min(j.pv_max)
        };
        self.recalculer_armes();
        let effets = &mut self.talents.effets;
        effets.clear();
        for o in self.equipement.portes() {
            if let Some(effet) = &objets[o.objet].effet {
                effets.push((effet, facteur(d, o)));
            }
        }
        self.equipement.version += 1;
    }

    fn tirer_rarete(&mut self, ender: bool, ouverts: u32) -> Rarete {
        let r = &self.d.equipement.regles;
        let poids = |rarete: &Rarete, p: f32| -> f64 {
            if ender {
                return f64::from(p);
            }
            let b = &r.bois;
            let lire = |t: &PoidsRaretes| t.get(rarete).copied().map(f64::from);
            let valeur = f64::from(p) + lire(&b.par_coffre).unwrap_or(0.0) * f64::from(ouverts);
            let bas = lire(&b.minimum).unwrap_or(0.0);
            let haut = lire(&b.maximum).unwrap_or(f64::INFINITY);
            haut.min(bas.max(valeur))
        };
        let table = if ender { &r.ender.poids } else { &r.bois.poids };
        let total: f64 = table.iter().map(|(k, &p)| poids(k, p)).sum();
        let mut seuil = self.alea.tirer() * total;
        for (k, &p) in table {
            seuil -= poids(k, p);
            if seuil < 0.0 {
                return *k;
            }
        }
        *table.keys().next().unwrap_or(&Rarete::Commune)
    }
}
