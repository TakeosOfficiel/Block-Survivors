//! Monde d'une partie, sans rendu : génération d'une arène à partir de sa définition,
//! carte de collision (marche, tirs, effets de sol) et champ de flux que suivent les
//! ennemis. Reprend src/monde de la version TypeScript (carte.ts, navigation.ts,
//! generation.ts). Les arènes importées de Minecraft (.schem) viendront à l'étape 2.

mod carte;
mod generation;
mod navigation;

pub use carte::{BLOQUE_MARCHE, BLOQUE_TIR, CarteMonde};
pub use generation::{MondeGenere, generer_monde};
pub use navigation::ChampDeFlux;
