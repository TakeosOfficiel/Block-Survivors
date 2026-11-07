//! Contenu échangé entre l'hôte et les invités pendant une vraie partie (src/reseau) :
//! - `instantane` : entrées des invités et instantanés binaires de la partie, écrits par
//!   l'hôte 15 fois par seconde ;
//! - `reception` : côté invité, applique les instantanés à sa propre `Partie` (prédiction
//!   de sa marche, glissement des autres) ;
//! - `messages` : messages fiables en JSON (salon, cartes, coffres, bilans, fin) ;
//! - `octets` : lecture et écriture petit-boutiste, coordonnées compactées.

pub mod instantane;
pub mod messages;
pub mod octets;
pub mod reception;

pub use instantane::*;
pub use messages::{CarteJouee, InfoJoueur, IssueReseau, MessageFiable, Presentation, StatsReseau};
pub use reception::Recepteur;
