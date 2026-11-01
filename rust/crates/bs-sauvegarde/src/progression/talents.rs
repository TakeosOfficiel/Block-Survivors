//! Talents achetés dans les deux arbres de chaque classe (src/progression/sauvegarde.ts) :
//! rangs, paliers, prérequis, points libres, option des talents à choix, talent final et
//! arme de départ actifs. Clés : « personnage.arbre.talent » (rangs, options) et
//! « personnage.arbre » (réglages).

use bs_donnees::progression::{Arbre, Noeud, TypeNoeud};

use super::{Progression, RefusAchat};

impl Progression {
    fn arbre(&self, personnage: &str, arbre: usize) -> Option<&'static Arbre> {
        self.d
            .talents
            .arbres
            .get(personnage)
            .map(|a| &a[arbre.min(1)])
    }

    fn noeuds(&self, personnage: &str, arbre: usize) -> &'static [Noeud] {
        self.arbre(personnage, arbre).map_or(&[], |a| &a.noeuds)
    }

    fn cle_arbre(&self, personnage: &str, arbre: usize) -> String {
        let id = self.arbre(personnage, arbre).map_or("", |a| a.id.as_str());
        format!("{personnage}.{id}")
    }

    fn cle_noeud(&self, personnage: &str, arbre: usize, noeud: &Noeud) -> String {
        format!("{}.{}", self.cle_arbre(personnage, arbre), noeud.id)
    }

    pub fn arbre_actif(&self, personnage: &str) -> usize {
        usize::from(self.c.arbres.get(personnage) == Some(&1.0))
    }

    pub fn choisir_arbre(&mut self, personnage: &str, arbre: usize) {
        self.c.arbres.insert(personnage.into(), arbre as f64);
        self.enregistrer();
    }

    pub fn rang(&self, personnage: &str, arbre: usize, noeud: &Noeud) -> u32 {
        let cle = self.cle_noeud(personnage, arbre, noeud);
        self.c.rangs.get(&cle).map_or(0, |r| *r as u32)
    }

    pub fn complet(&self, personnage: &str, arbre: usize, noeud: &Noeud) -> bool {
        self.rang(personnage, arbre, noeud) >= noeud.rang_max
    }

    /// Rangs achetés dans l'arbre (au-dessus de la ligne `ligne` si elle est donnée).
    pub fn points(&self, personnage: &str, arbre: usize, ligne: Option<u32>) -> u32 {
        let noeuds = self.noeuds(personnage, arbre).iter();
        noeuds
            .filter(|n| ligne.is_none_or(|l| n.ligne < l))
            .map(|n| self.rang(personnage, arbre, n))
            .sum()
    }

    /// Le palier de cette ligne est-il atteint (assez de rangs achetés au-dessus) ?
    pub fn palier_atteint(&self, personnage: &str, arbre: usize, ligne: u32) -> bool {
        let paliers = self.d.talents.paliers.iter();
        paliers
            .into_iter()
            .all(|p| ligne < p.ligne || self.points(personnage, arbre, Some(p.ligne)) >= p.rangs)
    }

    /// Palier atteint, et racine, libre ou relié à un talent complet au-dessus.
    pub fn ouvert(&self, personnage: &str, arbre: usize, noeud: &Noeud) -> bool {
        if !self.palier_atteint(personnage, arbre, noeud.ligne) {
            return false;
        }
        if noeud.libre || noeud.prerequis.is_empty() {
            return true;
        }
        let noeuds = self.noeuds(personnage, arbre);
        noeud.prerequis.iter().any(|id| {
            let parent = noeuds.iter().find(|n| n.id == *id);
            parent.is_some_and(|p| self.complet(personnage, arbre, p))
        })
    }

    pub fn refus_achat(&self, personnage: &str, arbre: usize, noeud: &Noeud) -> Option<RefusAchat> {
        if self.complet(personnage, arbre, noeud) {
            Some(RefusAchat::Max)
        } else if !self.ouvert(personnage, arbre, noeud) {
            Some(RefusAchat::Verrou)
        } else if self.points_libres(personnage, arbre) < 1 {
            Some(RefusAchat::Points)
        } else {
            None
        }
    }

    pub fn acheter(&mut self, personnage: &str, arbre: usize, noeud: &Noeud) -> bool {
        if self.refus_achat(personnage, arbre, noeud).is_some() {
            return false;
        }
        let rang = self.rang(personnage, arbre, noeud) + 1;
        let cle = self.cle_noeud(personnage, arbre, noeud);
        self.c.rangs.insert(cle, f64::from(rang));
        self.enregistrer();
        true
    }

    /// Rend tous les points de l'arbre et efface ses réglages.
    pub fn reinitialiser(&mut self, personnage: &str, arbre: usize) {
        for n in self.noeuds(personnage, arbre) {
            let cle = self.cle_noeud(personnage, arbre, n);
            self.c.rangs.remove(&cle);
            self.c.options.remove(&cle);
        }
        let cle = self.cle_arbre(personnage, arbre);
        self.c.finals.remove(&cle);
        self.c.armes_depart.remove(&cle);
        self.enregistrer();
    }

    /// Option choisie (0 ou 1) d'un talent à choix.
    pub fn option(&self, personnage: &str, arbre: usize, noeud: &Noeud) -> usize {
        let cle = self.cle_noeud(personnage, arbre, noeud);
        usize::from(self.c.options.get(&cle) == Some(&1.0))
    }

    pub fn choisir_option(&mut self, personnage: &str, arbre: usize, noeud: &Noeud, option: usize) {
        let cle = self.cle_noeud(personnage, arbre, noeud);
        self.c.options.insert(cle, f64::from(u8::from(option == 1)));
        self.enregistrer();
    }

    /// Talent choisi parmi les achetés qui vérifient `genre`, sinon le premier acheté.
    fn choisi(
        &self,
        personnage: &str,
        arbre: usize,
        id: Option<&String>,
        genre: impl Fn(&Noeud) -> bool,
    ) -> Option<&'static Noeud> {
        let noeuds = self.noeuds(personnage, arbre);
        let achete = |n: &&Noeud| genre(n) && self.rang(personnage, arbre, n) > 0;
        noeuds
            .iter()
            .filter(achete)
            .find(|n| Some(&n.id) == id)
            .or_else(|| noeuds.iter().find(achete))
    }

    /// Talent final actif (le choisi s'il est acheté, sinon le premier acheté).
    pub fn final_actif(&self, personnage: &str, arbre: usize) -> Option<&'static Noeud> {
        let id = self.c.finals.get(&self.cle_arbre(personnage, arbre));
        self.choisi(personnage, arbre, id, |n| n.genre == TypeNoeud::Final)
    }

    pub fn choisir_final(&mut self, personnage: &str, arbre: usize, noeud: &Noeud) {
        if noeud.genre != TypeNoeud::Final || self.rang(personnage, arbre, noeud) == 0 {
            return;
        }
        let cle = self.cle_arbre(personnage, arbre);
        self.c.finals.insert(cle, noeud.id.clone());
        self.enregistrer();
    }

    /// Talent d'arme de départ actif, ou `None` (aucun acheté, ou « aucune arme en plus »).
    pub fn arme_depart_active(&self, personnage: &str, arbre: usize) -> Option<&'static Noeud> {
        let id = self.c.armes_depart.get(&self.cle_arbre(personnage, arbre));
        if id.is_some_and(String::is_empty) {
            return None;
        }
        self.choisi(personnage, arbre, id, |n| n.gains.arme_depart.is_some())
    }

    pub fn choisir_arme_depart(&mut self, personnage: &str, arbre: usize, noeud: Option<&Noeud>) {
        let cle = self.cle_arbre(personnage, arbre);
        let id = noeud.map_or_else(String::new, |n| n.id.clone());
        self.c.armes_depart.insert(cle, id);
        self.enregistrer();
    }

    /// Le talent acheté joue-t-il (final choisi, arme de départ active) ?
    pub fn actif(&self, personnage: &str, arbre: usize, noeud: &Noeud) -> bool {
        if self.rang(personnage, arbre, noeud) == 0 {
            return false;
        }
        if noeud.genre == TypeNoeud::Final {
            return self
                .final_actif(personnage, arbre)
                .is_some_and(|n| n.id == noeud.id);
        }
        if noeud.gains.arme_depart.is_some() {
            let actif = self.arme_depart_active(personnage, arbre);
            return actif.is_some_and(|n| n.id == noeud.id);
        }
        true
    }
}
