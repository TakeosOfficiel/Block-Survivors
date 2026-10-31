//! Socle commun à tous les crates de Block Survivors : version du jeu et pas de temps de
//! la simulation. Aucune dépendance au moteur, pour que la simulation, le robot et le
//! relais puissent l'utiliser sans Bevy.

mod alea;
pub use alea::Alea;

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
