//! Bonus passifs possédés par le joueur et leur effet sur les modificateurs
//! (port de src/progression/passifs.ts).

use bs_donnees::{Donnees, Modifs};

use crate::etat::{Modificateurs, modificateurs_de_base};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PassifPossede {
    /// Index dans la table des passifs.
    pub index: usize,
    pub rang: u32,
}

#[derive(Debug, Clone)]
pub struct Passifs {
    pub possedes: Vec<PassifPossede>,
    /// Bonus permanents du personnage, toujours comptés.
    bonus: Modifs,
    /// Bonus de l'équipement porté (bagues, amulette), remplacés à chaque changement.
    equipement: Vec<Modifs>,
}

impl Passifs {
    pub fn nouveaux(bonus: Modifs) -> Self {
        Self {
            possedes: Vec::with_capacity(16),
            bonus,
            equipement: Vec::with_capacity(8),
        }
    }

    /// Ajoute des bonus permanents (talents de l'arbre actif).
    pub fn ajouter_bonus(&mut self, bonus: &Modifs) {
        self.bonus.cumuler(bonus);
    }

    /// Remplace les bonus de l'équipement (pris en compte au prochain `calculer`).
    pub fn definir_equipement(&mut self, bonus: impl IntoIterator<Item = Modifs>) {
        self.equipement.clear();
        self.equipement.extend(bonus);
    }

    /// Rang possédé (0 = absent).
    pub fn rang(&self, index: usize) -> u32 {
        self.possedes
            .iter()
            .find(|p| p.index == index)
            .map_or(0, |p| p.rang)
    }

    /// Ajoute le passif au rang 1, ou le monte d'un rang ; faux s'il est déjà au maximum
    /// (un consommable s'empile sans limite ici).
    pub fn ameliorer(&mut self, d: &Donnees, index: usize) -> bool {
        let def = &d.passifs[index];
        match self.possedes.iter_mut().find(|p| p.index == index) {
            None => self.possedes.push(PassifPossede { index, rang: 1 }),
            Some(p) => {
                if def.consommable.is_none() && p.rang as usize >= def.niveaux.len() {
                    return false;
                }
                p.rang += 1;
            }
        }
        true
    }

    /// Modificateurs depuis zéro : bonus du personnage, équipement, puis passifs ;
    /// `ajout` simule un passif monté d'un rang.
    pub fn calculer(&self, d: &Donnees, ajout: Option<usize>) -> Modificateurs {
        let mut sortie = modificateurs_de_base();
        sortie.cumuler(&self.bonus);
        for bonus in &self.equipement {
            sortie.cumuler(bonus);
        }
        let mut cumuler = |index: usize, rang: u32| {
            let niveaux = &d.passifs[index].niveaux;
            for r in 0..rang as usize {
                sortie.cumuler(&niveaux[r.min(niveaux.len() - 1)].modif);
            }
        };
        for p in &self.possedes {
            cumuler(p.index, p.rang + u32::from(ajout == Some(p.index)));
        }
        if let Some(a) = ajout
            && self.rang(a) == 0
        {
            cumuler(a, 1);
        }
        sortie
    }
}
