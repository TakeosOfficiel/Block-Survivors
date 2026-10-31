//! Prototype de l'étape 0 bis : une arène vide, le joueur, des zombies, une arme. Sert
//! encore aux mesures de bs-rendu et aux essais réseau, jusqu'à ce que la vraie partie
//! (module `partie`) les remplace.

mod armes;
mod ennemis;
mod grille;
mod simulation;

pub use armes::{ParametresArme, Projectiles};
pub use ennemis::{Ennemis, ParametresEnnemis};
pub use grille::GrilleSpatiale;
pub use simulation::{Commande, Joueur, Parametres, Simulation};
