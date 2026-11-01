//! Effets en jeu de l'arbre de talents actif d'une classe (src/progression/sauvegarde.ts
//! `bonus`) : modificateurs, rangs d'armes offerts, relances, bonus d'armes précises et
//! rang des mécaniques propres. Un talent à choix joue l'option choisie.

use bs_donnees::progression::{BonusTalents, Gains};

use super::Progression;

impl Progression {
    /// Effets de l'arbre actif du personnage.
    pub fn bonus(&self, personnage: &str) -> BonusTalents {
        let mut b = BonusTalents::default();
        let Some(arbres) = self.d.talents.arbres.get(personnage) else {
            return b;
        };
        let arbre = self.arbre_actif(personnage);
        for noeud in &arbres[arbre].noeuds {
            if !self.actif(personnage, arbre, noeud) {
                continue;
            }
            let rang = self.rang(personnage, arbre, noeud);
            let effet: &'static Gains = match &noeud.options {
                Some(options) => &options[self.option(personnage, arbre, noeud)].gains,
                None => &noeud.gains,
            };
            for _ in 0..rang {
                if let Some(m) = &effet.modif {
                    b.modif.cumuler(m);
                }
                b.bonus_armes.extend(effet.armes.iter());
                b.armes.extend(effet.rangs_arme.as_deref());
                b.armes.extend(effet.arme_depart.as_deref());
                b.relances += effet.relances.unwrap_or(0.0) as u32;
            }
            if let Some(m) = &effet.mecanique {
                // Comme un dictionnaire : la dernière entrée l'emporte
                b.mecaniques.retain(|(n, _)| n != m);
                b.mecaniques.push((m.as_str(), rang));
            }
        }
        b
    }
}
