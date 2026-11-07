//! Le jeu sur PC (Windows, Linux).
// Windows : pas de console noire derrière la fenêtre du jeu publié
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // En premier : lancements propres à Velopack (installation, mise à jour)
    bs_maj::demarrage();
    // Puis le garde : retour à la version d'avant si celle-ci a planté deux fois au démarrage
    let dossier = bs_sauvegarde::stockage::Stockage::du_systeme()
        .dossier()
        .to_path_buf();
    bs_maj::garde::verifier_demarrage(&dossier);
    block_survivors::lancer();
}
