//! Sauvegarde de Block Survivors, au format de la version TypeScript pour qu'un joueur
//! garde tout en changeant de version :
//! - `stockage` : le stockage local (une clé, une valeur JSON), en fichiers sur disque ;
//! - `profils` : plusieurs profils par installation, chacun sous ses propres clés ;
//! - `progression` : maîtrise des classes, talents achetés, records, déblocages ;
//! - `collection` : pièces d'armure gagnées et portées, objets de coffre découverts ;
//! - `bssave` : fichiers `.bssave` chiffrés (export et import d'un profil).

mod bssave;
pub mod collection;
pub mod profils;
pub mod progression;
pub mod stockage;

pub use bssave::{DONNEES_PROFIL, Erreur, Profil, chiffrer, dechiffrer, exporter, importer};
pub use collection::Collection;
pub use profils::Profils;
pub use progression::{Bilan, EtatMaitrise, Progression, Recompense, RefusAchat};
pub use stockage::Stockage;
