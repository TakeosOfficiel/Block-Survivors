//! Protocole de la coop : seul lien entre le jeu et le relais (le relais ne dépend que de
//! ce crate). Deux niveaux :
//! - les **trames du relais** (`Trame`) : créer ou rejoindre un salon, puis faire suivre
//!   des octets de l'hôte vers les invités et retour, sans les lire ;
//! - le **contenu** échangé entre hôte et invités, dont l'**instantané** binaire de la
//!   simulation (`instantane`), écrit dans un tampon réutilisé (aucune allocation).

mod instantane;
mod trame;

pub use instantane::{Instantane, ecrire_instantane, lire_instantane};
pub use trame::{REFUS_PLEIN, REFUS_SALON, REFUS_VERSION, Trame, lire_trame};

/// Version du protocole : le relais refuse un client d'une autre version.
pub const VERSION_PROTOCOLE: u16 = 1;
