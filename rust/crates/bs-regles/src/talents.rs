//! Points d'accroche des talents (classes et mécaniques d'objets) dans la simulation.
//! Pour l'instant neutres : les parties du robot partent d'une sauvegarde vierge, sans
//! arbre de talents. Les mécaniques d'objets d'équipement viendront ici (étape 1b-6).

use crate::ennemis::Nature;
use crate::partie::Partie;

impl Partie {
    /// Multiplicateur de dégâts contre l'ennemi `i`.
    pub fn talent_multiplicateur(&self, _i: usize, _arme: Option<usize>, _nature: Nature) -> f32 {
        1.0
    }
    pub fn talent_bonus_critique(&self, _i: usize, _arme: usize, _nature: Nature) -> f32 {
        0.0
    }
    pub fn talent_facteur_critique(
        &self,
        _i: usize,
        _arme: Option<usize>,
        _nature: Nature,
        base: f32,
    ) -> f32 {
        base
    }
    pub fn talent_apres_coup(
        &mut self,
        _i: usize,
        _degats: f32,
        _arme: Option<usize>,
        _critique: bool,
        _nature: Nature,
    ) {
    }
    pub fn talent_sur_mort(&mut self, _i: usize, _arme: Option<usize>, _nature: Nature) {}
    pub fn talent_sur_impact_mur(&mut self, _i: usize) {}
    pub fn talent_sur_sortie_piege(&mut self, _i: usize) {}
    pub fn talent_vitesse(&self) -> f32 {
        1.0
    }
    pub fn talent_degats_recus(&mut self, degats: f32, _boss: bool) -> f32 {
        degats
    }
    pub fn talent_armure(&self) -> f32 {
        0.0
    }
    pub fn talent_eviter_mort(&mut self) -> bool {
        false
    }
    pub fn talent_apres_coup_recu(&mut self, _subis: f32, _evites: f32) {}
}
