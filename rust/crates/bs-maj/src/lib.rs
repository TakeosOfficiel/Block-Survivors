//! Mises à jour sur PC (plan, section 5). Velopack installe, télécharge les paquets et bascule
//! de version ; il ne signe pas son flux. D'où notre **manifeste signé** (ed25519) : le jeu
//! ne connaît que les paquets listés dans un manifeste dont la signature, le numéro de série
//! (jamais en arrière) et la date d'expiration sont valides, et chaque fichier téléchargé est
//! vérifié contre son empreinte SHA-256 signée.
//!
//! - `manifeste` : format, signature, vérification ;
//! - `source` : `SourceSignee`, la source Velopack qui n'expose que le manifeste vérifié,
//!   sur un dossier ou en HTTPS ;
//! - `cles` : clés publiques écrites dans le build ;
//! - `garde` : retour à la dernière version qui marchait après deux plantages au démarrage ;
//! - `service` : recherche en arrière-plan pour le jeu (état, voie, redémarrage) ;
//! - `bin/bs-signer` : outil de la CI (clés, manifeste depuis la sortie de `vpk`, signature).

pub mod cles;
pub mod garde;
pub mod manifeste;
pub mod service;
pub mod source;

pub use manifeste::{Enveloppe, Manifeste, Paquet, Refus, signer, verifier};
pub use service::{Etat, MiseAJour, Reglages};
pub use source::{Depot, DepotHttps, Dossier, SourceSignee};

/// À appeler en tout premier dans `main` : Velopack y traite ses propres lancements
/// (installation, désinstallation, mise à jour) et applique une version téléchargée
/// mais pas encore installée.
pub fn demarrage() {
    velopack::VelopackApp::build().run();
}
