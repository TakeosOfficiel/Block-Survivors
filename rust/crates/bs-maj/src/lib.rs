//! Mises à jour sur PC (plan, section 5). Velopack installe, télécharge les deltas et bascule
//! de version ; il ne signe pas son flux. D'où notre **manifeste signé** (ed25519) : le jeu
//! ne connaît que les paquets listés dans un manifeste dont la signature, le numéro de série
//! (jamais en arrière) et la date d'expiration sont valides, et chaque fichier téléchargé est
//! vérifié contre son empreinte SHA-256 signée.
//!
//! - `manifeste` : format, signature, vérification ;
//! - `source` : `SourceSignee`, la source Velopack qui n'expose que le manifeste vérifié ;
//! - `bin/bs-signer` : outil de la CI (clés, manifeste depuis la sortie de `vpk`, signature).

pub mod manifeste;
pub mod source;

pub use manifeste::{Enveloppe, Manifeste, Paquet, Refus, signer, verifier};
pub use source::{Depot, Dossier, SourceSignee};
