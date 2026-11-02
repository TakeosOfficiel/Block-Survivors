//! Protocole de la coop : seul lien entre le jeu et le relais (le relais ne dépend que de
//! ce crate). Deux niveaux :
//! - les **trames du relais** (`Trame`) : créer, lister, rejoindre ou reprendre un salon,
//!   puis faire suivre des octets de l'hôte vers les invités et retour, sans les lire ;
//! - le **contenu** échangé entre hôte et invités, dont l'**instantané** binaire de la
//!   simulation (`instantane`), écrit dans un tampon réutilisé (aucune allocation).

pub mod coop;
mod instantane;
mod trame;

pub use instantane::{Instantane, ecrire_instantane, lire_instantane};
pub use trame::{
    ANNONCE_MAX, EntreeListe, IterListe, LISTE_MAX, ListeSalons, PLACES_SALON, REFUS_JETON,
    REFUS_PLEIN, REFUS_SALON, REFUS_SALONS, REFUS_TENTATIVES, REFUS_VERSION, Trame, lire_trame,
};

/// Version du protocole : le relais refuse un client d'une autre version, et ne met en
/// relation (`Rejoindre`, `Lister`) que des joueurs de la même version.
/// 2 : salons publics, annonces, reconnexion par jeton, exclusion.
pub const VERSION_PROTOCOLE: u16 = 2;
