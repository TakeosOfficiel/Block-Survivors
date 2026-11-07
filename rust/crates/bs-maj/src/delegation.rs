//! Délégation (à la manière de TUF, deux niveaux) : le jeu ne contient que les clés
//! **racines**, gardées hors ligne. Une racine signe une délégation qui nomme les clés
//! autorisées à signer les manifestes (celle de la CI), avec un numéro de série et une date
//! d'expiration. La délégation voyage dans l'enveloppe de chaque manifeste.
//!
//! Si la clé de la CI est volée, on signe hors ligne une délégation de série plus grande
//! avec une nouvelle clé de CI. Un jeu qui l'a vue refuse ensuite l'ancienne délégation, et
//! donc la clé volée, sans qu'il faille publier un nouveau build. Une racine peut aussi
//! signer un manifeste elle-même, sans délégation (secours).

use base64::{Engine, engine::general_purpose::STANDARD as B64};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::manifeste::{Refus, cle_publique};

/// Délégation signée : le texte JSON de `Contenu` et sa signature par une racine.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Delegation {
    pub texte: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Contenu {
    /// Croît à chaque délégation ; le jeu refuse une série plus petite que la dernière vue.
    pub serie: u64,
    /// Secondes Unix.
    pub expire: u64,
    /// Clés publiques (base64) autorisées à signer les manifestes.
    pub cles: Vec<String>,
}

/// Délégation signée par la racine `cle`.
pub fn deleguer(c: &Contenu, cle: &SigningKey) -> Delegation {
    let texte = serde_json::to_string_pretty(c).expect("délégation sérialisable");
    let signature = B64.encode(cle.sign(texte.as_bytes()).to_bytes());
    Delegation { texte, signature }
}

/// Signature valide par l'une des `cles` (stricte : pas de signature malléable).
pub(crate) fn signee(texte: &str, signature: &str, cles: &[VerifyingKey]) -> bool {
    let Ok(o) = B64.decode(signature) else {
        return false;
    };
    let Ok(sig) = Signature::from_slice(&o) else {
        return false;
    };
    cles.iter()
        .any(|c| c.verify_strict(texte.as_bytes(), &sig).is_ok())
}

/// Clés déléguées (et série de la délégation) si elle est signée par une racine, pas plus
/// ancienne que `derniere` et pas expirée à `maintenant`.
pub fn cles_deleguees(
    d: &Delegation,
    racines: &[VerifyingKey],
    derniere: u64,
    maintenant: u64,
) -> Result<(u64, Vec<VerifyingKey>), Refus> {
    if !signee(&d.texte, &d.signature, racines) {
        return Err(Refus::Signature);
    }
    let c: Contenu = serde_json::from_str(&d.texte).map_err(|_| Refus::Illisible)?;
    if c.serie < derniere {
        return Err(Refus::DelegationAncienne {
            serie: c.serie,
            derniere,
        });
    }
    if maintenant >= c.expire {
        return Err(Refus::DelegationExpiree);
    }
    Ok((
        c.serie,
        c.cles.iter().filter_map(|k| cle_publique(k)).collect(),
    ))
}
