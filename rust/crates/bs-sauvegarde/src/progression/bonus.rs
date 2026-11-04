//! Effets en jeu des talents d'une classe (src/progression/sauvegarde.ts `bonus`) : ses
//! deux arbres jouent ensemble (points partagés). Modificateurs, rangs d'armes offerts,
//! relances, bonus d'armes précises et rang des mécaniques propres ; un talent à choix
//! joue l'option choisie. Les talents actifs (`noeuds_actifs`) sont aussi ce qu'un
//! joueur envoie à l'hôte en coop.

use bs_donnees::progression::{BonusTalents, NoeudActif};

use super::Progression;

impl Progression {
    /// Talents actifs des deux arbres du personnage, dans l'ordre des arbres.
    pub fn noeuds_actifs(&self, personnage: &str) -> Vec<NoeudActif> {
        let Some(arbres) = self.d.talents.arbres.get(personnage) else {
            return Vec::new();
        };
        let mut actifs = Vec::new();
        for (arbre, def) in arbres.iter().enumerate() {
            let noeuds = def.noeuds.iter();
            actifs.extend(
                noeuds
                    .filter(|n| self.actif(personnage, arbre, n))
                    .map(|n| NoeudActif {
                        noeud: n.id.clone(),
                        rang: self.rang(personnage, arbre, n),
                        option: if n.options.is_some() {
                            self.option(personnage, arbre, n)
                        } else {
                            0
                        },
                    }),
            );
        }
        actifs
    }

    /// Effets des talents du personnage.
    pub fn bonus(&self, personnage: &str) -> BonusTalents {
        let actifs = self.noeuds_actifs(personnage);
        BonusTalents::depuis_noeuds(
            &self.d.talents,
            personnage,
            actifs.iter().map(|n| (n.noeud.as_str(), n.rang, n.option)),
        )
    }
}
