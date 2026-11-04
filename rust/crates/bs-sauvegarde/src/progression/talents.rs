//! Talents achetés dans les deux arbres de chaque classe (src/progression/sauvegarde.ts) :
//! rangs, paliers, prérequis, points libres, option des talents à choix, talent final et
//! arme de départ actifs. Clés : « personnage.arbre.talent » (rangs, options) et
//! « personnage.arbre » (réglages). Les points sont rares (maîtrise plafonnée) : un seul
//! talent final par arbre, et l'option d'un talent à choix se fixe à l'achat.

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
        } else if noeud.genre == TypeNoeud::Final && self.autre_final(personnage, arbre, noeud) {
            Some(RefusAchat::Exclusif)
        } else if self.points_libres(personnage) < 1 {
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

    /// Un autre talent final que `noeud` est-il appris dans l'arbre ?
    fn autre_final(&self, personnage: &str, arbre: usize, noeud: &Noeud) -> bool {
        let noeuds = self.noeuds(personnage, arbre).iter();
        noeuds
            .filter(|n| n.genre == TypeNoeud::Final && n.id != noeud.id)
            .any(|n| self.rang(personnage, arbre, n) > 0)
    }

    /// Rend les points de l'arbre et efface ses réglages (l'autre arbre garde les siens).
    pub fn reinitialiser(&mut self, personnage: &str, arbre: usize) {
        self.vider_arbre(personnage, arbre);
        self.enregistrer();
    }

    fn vider_arbre(&mut self, personnage: &str, arbre: usize) {
        for n in self.noeuds(personnage, arbre) {
            let cle = self.cle_noeud(personnage, arbre, n);
            self.c.rangs.remove(&cle);
            self.c.options.remove(&cle);
        }
        let cle = self.cle_arbre(personnage, arbre);
        self.c.finals.remove(&cle);
        self.c.armes_depart.remove(&cle);
    }

    /// Le rang acheté respecte-t-il les règles actuelles (rangée ouverte par les points
    /// des rangées au-dessus, prérequis) ?
    fn regulier(&self, personnage: &str, arbre: usize, noeud: &Noeud) -> bool {
        self.rang(personnage, arbre, noeud) == 0 || self.ouvert(personnage, arbre, noeud)
    }

    /// Vide les talents d'une classe qui sort des règles (plus de rangs que de points,
    /// plusieurs talents finals, rangée achetée sans les points au-dessus : sauvegarde
    /// d'avant ces règles) ; vrai si l'une l'a été.
    pub(super) fn assainir(&mut self) -> bool {
        let d = self.d;
        let mut vide = false;
        for personnage in d.talents.arbres.keys() {
            let niveau = self.maitrise(personnage).niveau;
            let hors_regles = (0..2).any(|arbre| {
                let noeuds = self.noeuds(personnage, arbre);
                let appris = |n: &&Noeud| self.rang(personnage, arbre, n) > 0;
                let finals = noeuds
                    .iter()
                    .filter(|n| n.genre == TypeNoeud::Final)
                    .filter(appris);
                finals.count() > 1 || !noeuds.iter().all(|n| self.regulier(personnage, arbre, n))
            });
            if hors_regles || self.points_classe(personnage) > niveau {
                for arbre in 0..2 {
                    self.vider_arbre(personnage, arbre);
                }
                vide = true;
            }
        }
        if vide {
            self.c.reinitialise = true;
        }
        vide
    }

    /// Option choisie (0 ou 1) d'un talent à choix.
    pub fn option(&self, personnage: &str, arbre: usize, noeud: &Noeud) -> usize {
        let cle = self.cle_noeud(personnage, arbre, noeud);
        usize::from(self.c.options.get(&cle) == Some(&1.0))
    }

    /// Choisit l'option d'un talent à choix pas encore appris (fixée ensuite).
    pub fn choisir_option(&mut self, personnage: &str, arbre: usize, noeud: &Noeud, option: usize) {
        if self.rang(personnage, arbre, noeud) > 0 {
            return;
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stockage::Stockage;

    fn progression(nom: &str) -> Progression {
        let d = Box::leak(Box::new(bs_donnees::Donnees::embarquees().unwrap()));
        let dossier = std::env::temp_dir().join(format!("bs-talents-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dossier);
        let mut p = Progression::charger(d, Stockage::nouveau(dossier), "p".into());
        p.c.maitrise.insert("guerrier".into(), 1e9);
        p
    }

    fn poser(p: &mut Progression, id: &str, rang: u32) {
        let n = p.noeuds("guerrier", 0).iter().find(|n| n.id == id).unwrap();
        let cle = p.cle_noeud("guerrier", 0, n);
        p.c.rangs.insert(cle, f64::from(rang));
    }

    #[test]
    fn points_rares_et_choix_definitifs() {
        let mut p = progression("rares");
        let max = p.d.talents.maitrise.niveau_max;
        assert_eq!(p.maitrise("guerrier").niveau, max);
        assert!(p.maitrise("guerrier").au_max());
        let noeuds = p.noeuds("guerrier", 0);
        let total: u32 = (0..2)
            .flat_map(|a| p.noeuds("guerrier", a))
            .map(|n| n.rang_max)
            .sum();
        assert!(total > 2 * max, "on ne prend pas tout : {total} rangs");

        // Une rangée s'ouvre avec les points mis au-dessus dans l'arbre
        let n = |id: &str| noeuds.iter().find(|n| n.id == id).unwrap();
        assert_eq!(
            p.refus_achat("guerrier", 0, n("lame")),
            Some(RefusAchat::Verrou)
        );
        assert!(p.acheter("guerrier", 0, n("enchainement")));
        assert!(p.acheter("guerrier", 0, n("lame")));

        // Plus de rangs que de points, ou une rangée sans ses points : classe vidée
        for x in noeuds.iter().filter(|n| n.genre != TypeNoeud::Final) {
            poser(&mut p, &x.id, x.rang_max);
        }
        poser(&mut p, "f_fureur", 1);
        assert!(p.assainir());
        assert_eq!(p.points_classe("guerrier"), 0);
        assert!(p.annoncer_reinitialisation());

        // Un talent final ferme les autres
        for x in noeuds.iter().filter(|n| n.genre != TypeNoeud::Final) {
            poser(&mut p, &x.id, x.rang_max);
        }
        poser(&mut p, "f_fureur", 1);
        let champion = n("f_champion");
        assert_eq!(
            p.refus_achat("guerrier", 0, champion),
            Some(RefusAchat::Exclusif)
        );

        // L'option se choisit avant l'achat, puis ne bouge plus
        p.reinitialiser("guerrier", 0);
        p.choisir_option("guerrier", 0, n("elan"), 1);
        poser(&mut p, "elan", 1);
        p.choisir_option("guerrier", 0, n("elan"), 0);
        assert_eq!(p.option("guerrier", 0, n("elan")), 1);

        // Les deux arbres se partagent les points : tout acheter s'arrête au plafond
        p.reinitialiser("guerrier", 0);
        let mut achats = 0;
        loop {
            let libre = (0..2).find_map(|a| {
                let noeuds = p.noeuds("guerrier", a).iter();
                noeuds
                    .clone()
                    .find(|n| p.refus_achat("guerrier", a, n).is_none())
                    .map(|n| (a, n))
            });
            let Some((a, n)) = libre else { break };
            assert!(p.acheter("guerrier", a, n));
            achats += 1;
        }
        assert_eq!(achats, max);
        assert_eq!(p.points_libres("guerrier"), 0);
    }
}
