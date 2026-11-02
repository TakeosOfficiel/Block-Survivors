//! Client du relais de la coop (docs/PLAN-RUST.md, section 4), sans Bevy :
//! - `connexion` : une WebSocket vers le relais, tenue par un fil à part ; le jeu lui passe
//!   des octets et relève ceux reçus à chaque image, sans jamais attendre ;
//! - `session` : la vie d'un salon par-dessus (créer, lister, rejoindre, échanger avec
//!   l'hôte ou les invités, reprendre sa place après une coupure grâce au jeton) ;
//! - `coop` : la partie à plusieurs par-dessus la session, côté hôte (`HoteCoop`) et côté
//!   invité (`InviteCoop`), sans fenêtre : le jeu leur passe sa `Partie` à chaque pas.
//!
//! Adresse du relais : variable d'environnement `BS_RELAIS`, sinon `ADRESSE_RELAIS`.

mod connexion;
pub mod coop;
mod session;

pub use connexion::{Connexion, Recu};
pub use coop::{HoteCoop, InviteCoop, PhaseCoop};
pub use session::{EvenementSession, SalonPublic, Session, code_lisible, code_saisi};

/// Relais par défaut (local tant que le relais n'est pas déployé : étape 7).
pub const ADRESSE_RELAIS: &str = "ws://127.0.0.1:8787/salon";

/// Adresse du relais à utiliser.
pub fn adresse_relais() -> String {
    std::env::var("BS_RELAIS").unwrap_or_else(|_| ADRESSE_RELAIS.to_owned())
}
