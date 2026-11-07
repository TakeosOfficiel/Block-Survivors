//! Relief de la partie en cours, pour poser armes, lueurs et effets sur le sol des maps
//! à plusieurs niveaux. Une copie de la carte de collision, prise une fois par partie
//! (aucune si l'arène est plate) ; chaque dessin en prend une vue en commençant l'image.

use std::sync::{Arc, RwLock};

use bevy::prelude::*;
use bs_monde::CarteMonde;

use crate::PartieEnCours;

/// Carte suivie (adresse et taille de ses hauteurs) et sa copie si elle a du relief.
static RELIEF: RwLock<(usize, usize, Option<Arc<CarteMonde>>)> = RwLock::new((0, 0, None));

/// Vue du relief pour une image (plate par défaut).
#[derive(Clone, Default)]
pub struct Relief(Option<Arc<CarteMonde>>);

impl Relief {
    pub fn actuel() -> Self {
        Self(RELIEF.read().ok().and_then(|r| r.2.clone()))
    }

    /// Hauteur du sol en (x, z).
    pub fn sol(&self, x: f32, z: f32) -> f32 {
        self.0.as_ref().map_or(0.0, |c| c.hauteur_lissee(x, z))
    }
}

/// Recopie la carte de la partie quand elle change (nouvelle partie, autre map).
pub fn suivre(jeu: Option<Res<PartieEnCours>>) {
    let carte = jeu.as_ref().map(|j| &j.partie.carte);
    let cle = carte.map_or((0, 0), |c| (c.hauteurs.as_ptr() as usize, c.hauteurs.len()));
    let Ok(mut r) = RELIEF.write() else {
        return;
    };
    if (r.0, r.1) == cle {
        return;
    }
    let plate = carte.is_none_or(|c| c.hauteurs.iter().all(|&h| h == 0.0));
    *r = (
        cle.0,
        cle.1,
        if plate {
            None
        } else {
            carte.cloned().map(Arc::new)
        },
    );
}
