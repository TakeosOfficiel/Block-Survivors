//! Socle commun à tous les crates de Block Survivors : version du jeu, pas de temps de
//! la simulation, hasard reproductible et fonctions mathématiques identiques en natif et
//! en WebAssembly. Aucune dépendance au moteur, pour que la simulation, le robot et le
//! relais puissent l'utiliser sans Bevy.

mod alea;
mod hasard;
pub mod maths;
pub use alea::Alea;
pub use hasard::{Mulberry, choisir_pondere, graine, hachage3};

/// Version du jeu (une seule source : le `Cargo.toml` de l'espace de travail).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Pas de la simulation, en secondes : fixe, pour une partie identique partout (coop, rejeux).
pub const PAS_SIMULATION: f32 = 1.0 / 60.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_lisible() {
        assert!(VERSION.split('.').count() == 3);
    }
}
