//! Simulation de Block Survivors, sans rendu ni Bevy : elle reçoit les commandes du
//! joueur et avance d'un pas fixe (`bs_commun::PAS_SIMULATION`). Données en tableaux à
//! plat (une colonne par champ), préallouées : aucune allocation pendant la partie.
//! Déterministe : même graine et mêmes commandes, même état (natif comme WebAssembly).
//!
//! Prototype (étape 0 bis du plan) : une arène, le joueur, des zombies, une arme.

mod armes;
mod ennemis;
mod grille;
mod simulation;

pub use armes::{ParametresArme, Projectiles};
pub use ennemis::{Ennemis, ParametresEnnemis};
pub use grille::GrilleSpatiale;
pub use simulation::{Commande, Joueur, Parametres, Simulation};
