//! Simulation de Block Survivors, sans rendu ni Bevy : elle reçoit les commandes du
//! joueur et avance d'un pas fixe (`bs_commun::PAS_SIMULATION`). Données préallouées :
//! aucune allocation pendant la partie. Déterministe : même graine et mêmes commandes,
//! même état (natif comme WebAssembly).
//!
//! `prototype` : le prototype de l'étape 0 bis (une arène, des zombies, une arme), gardé
//! pour les exemples réseau et le rendu tant que la partie complète n'est pas branchée.
//! Le reste est le port de la simulation TypeScript (étape 1), autour de `Partie`.

mod prototype;

pub mod butin;
pub mod cartes;
pub mod catalogue;
pub mod combat;
pub mod directeurs;
pub mod ennemis;
pub mod equipement;
pub mod etat;
pub mod evenements;
pub mod grille;
pub mod joueur;
pub mod partie;
pub mod passifs;
pub mod projectiles_ennemis;
pub mod ramassage;
pub mod talents;
pub mod vue;

pub use partie::Partie;
pub use prototype::Ennemis as EnnemisPrototype;
pub use prototype::{
    Commande, GrilleSpatiale, Joueur, Parametres, ParametresArme, ParametresEnnemis, Projectiles,
    Simulation,
};
