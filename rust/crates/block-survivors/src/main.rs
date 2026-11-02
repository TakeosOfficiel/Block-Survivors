//! Le jeu sur PC (Windows, Linux).
// Windows : pas de console noire derrière la fenêtre du jeu publié
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // En premier : lancements propres à Velopack (installation, mise à jour)
    bs_maj::demarrage();
    block_survivors::lancer();
}
