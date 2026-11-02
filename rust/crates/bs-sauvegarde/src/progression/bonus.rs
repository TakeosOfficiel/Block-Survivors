//! Effets en jeu de l'arbre de talents actif d'une classe (src/progression/sauvegarde.ts
//! `bonus`) : modificateurs, rangs d'armes offerts, relances, bonus d'armes précises et
//! rang des mécaniques propres. Un talent à choix joue l'option choisie. Les talents
//! actifs (`noeuds_actifs`) sont aussi ce qu'un joueur envoie à l'hôte en coop.

use bs_donnees::progression::{BonusTalents, NoeudActif};

use super::Progression;

impl Progression {
    /// Arbre actif du personnage et ses talents actifs, dans l'ordre de l'arbre.
    pub fn noeuds_actifs(&self, personnage: &str) -> (usize, Vec<NoeudActif>) {
        let arbre = self.arbre_actif(personnage);
        let Some(arbres) = self.d.talents.arbres.get(personnage) else {
            return (arbre, Vec::new());
        };
        let actifs = arbres[arbre]
            .noeuds
            .iter()
            .filter(|n| self.actif(personnage, arbre, n))
            .map(|n| NoeudActif {
                noeud: n.id.clone(),
                rang: self.rang(personnage, arbre, n),
                option: if n.options.is_some() {
                    self.option(personnage, arbre, n)
                } else {
                    0
                },
            })
            .collect();
        (arbre, actifs)
    }

    /// Effets de l'arbre actif du personnage.
    pub fn bonus(&self, personnage: &str) -> BonusTalents {
        let (arbre, actifs) = self.noeuds_actifs(personnage);
        BonusTalents::depuis_noeuds(
            &self.d.talents,
            personnage,
            arbre,
            actifs.iter().map(|n| (n.noeud.as_str(), n.rang, n.option)),
        )
    }
}
