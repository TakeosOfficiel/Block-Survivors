//! Effets de l'arbre de talents actif au début d'une partie (src/jeu.ts) : bonus
//! permanents ajoutés à ceux du personnage, relances en plus, bonus d'armes précises,
//! rangs d'armes offerts (après les armes de départ du personnage) et rang des
//! mécaniques propres, lues par les points d'accroche de talents.rs.

use bs_donnees::progression::BonusTalents;

use crate::partie::Partie;

impl Partie {
    /// Applique les talents du joueur ; à appeler avant le premier pas de simulation.
    pub fn appliquer_talents(&mut self, b: &BonusTalents) {
        let d = self.d;
        let avant = self.etat.modificateurs.pv_max;
        self.passifs.ajouter_bonus(&b.modif);
        self.etat.modificateurs = self.passifs.calculer(d, None);
        let j = &mut self.etat.joueur;
        j.pv_max += self.etat.modificateurs.pv_max - avant;
        j.pv = j.pv_max;
        self.etat.relances_restantes += b.relances;
        for bonus in &b.bonus_armes {
            if let Some(i) = d.armes.get_index_of(&bonus.arme) {
                self.talents.armes[i].cumuler(bonus);
            }
        }
        for arme in &mut self.arsenal.armes {
            arme.talent = self.talents.armes[arme.index];
        }
        self.recalculer_armes();
        self.talents.mecaniques.clone_from(&b.mecaniques);
        let rangs = crate::classes::guerrier::Rangs::lire(&self.talents);
        if let Some(g) = self.talents.classes.guerrier.as_mut() {
            g.r = rangs;
        }
        for id in &b.armes {
            if let Some(i) = d.armes.get_index_of(*id) {
                self.ameliorer_arme(i);
            }
        }
    }
}
