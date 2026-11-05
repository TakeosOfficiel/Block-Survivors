//! Maps construites dans Minecraft (src/minecraft de la version TypeScript) : lecture
//! des schématiques (NBT), cuisson des modèles de blocs vanilla, zone jouable et carte
//! de collision, maillages par texture. Rien ici ne télécharge ni n'affiche : les
//! fichiers vanilla arrivent par une `SourceFichiers`, les maillages partent au rendu.

pub mod biomes;
pub mod blocs_entites;
pub mod cartes;
pub mod cuisson;
pub mod dossier;
pub mod litematic;
pub mod maillage;
pub mod mcstructure;
pub mod modeles;
pub mod monde_entier;
pub mod nbt;
pub mod panneaux;
pub mod proprietes;
pub mod schematique;
pub mod vent;
pub mod zone;

#[cfg(test)]
mod tests;
