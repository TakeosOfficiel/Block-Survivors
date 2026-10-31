//! Manifeste d'une version : numéro de série croissant, expiration, voie, version minimale
//! pour la coop, et la liste des paquets Velopack (complet et deltas) avec leurs empreintes.
//! Il voyage dans une enveloppe JSON : `{"manifeste": "<le JSON, en texte>", "signature":
//! "<base64>"}`. On signe les octets exacts du texte : pas de question de mise en forme.

use base64::{Engine, engine::general_purpose::STANDARD as B64};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Manifeste {
    /// Croît à chaque manifeste publié ; le jeu refuse un numéro plus petit que le dernier vu.
    pub serie: u64,
    /// Secondes Unix ; passé cette date, le manifeste est refusé (on ne fige pas un joueur
    /// sur une version périmée en lui resservant un vieux manifeste).
    pub expire: u64,
    /// « stable » ou « beta » : canal Velopack.
    pub voie: String,
    pub version: String,
    /// Plus vieille version avec laquelle la coop reste possible.
    pub version_min_coop: String,
    pub paquets: Vec<Paquet>,
}

/// Un fichier de la version (paquet complet ou delta), tel que `vpk` le décrit.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Paquet {
    pub id: String,
    pub version: String,
    /// « Full » ou « Delta ».
    pub genre: String,
    pub fichier: String,
    pub sha1: String,
    pub sha256: String,
    pub taille: u64,
    pub notes: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Enveloppe {
    pub manifeste: String,
    pub signature: String,
}

#[derive(Debug, PartialEq)]
pub enum Refus {
    Illisible,
    /// Aucune des clés publiques du jeu ne reconnaît la signature.
    Signature,
    /// Numéro de série plus petit que le dernier vu : retour forcé en arrière.
    Ancien {
        serie: u64,
        derniere: u64,
    },
    Expire,
}

impl std::fmt::Display for Refus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refus::Illisible => write!(f, "manifeste illisible"),
            Refus::Signature => write!(f, "signature du manifeste invalide"),
            Refus::Ancien { serie, derniere } => {
                write!(f, "manifeste ancien (série {serie} < {derniere})")
            }
            Refus::Expire => write!(f, "manifeste expiré"),
        }
    }
}

/// Enveloppe signée (JSON) du manifeste.
pub fn signer(m: &Manifeste, cle: &SigningKey) -> String {
    let texte = serde_json::to_string_pretty(m).expect("manifeste sérialisable");
    let signature = B64.encode(cle.sign(texte.as_bytes()).to_bytes());
    serde_json::to_string_pretty(&Enveloppe {
        manifeste: texte,
        signature,
    })
    .expect("enveloppe sérialisable")
}

/// Vérifie l'enveloppe : signature par l'une des `cles` (CI ou secours), série au moins
/// égale à `derniere_serie`, pas expirée à `maintenant` (secondes Unix).
pub fn verifier(
    octets: &[u8],
    cles: &[VerifyingKey],
    derniere_serie: u64,
    maintenant: u64,
) -> Result<Manifeste, Refus> {
    let e: Enveloppe = serde_json::from_slice(octets).map_err(|_| Refus::Illisible)?;
    let sig = B64.decode(&e.signature).map_err(|_| Refus::Signature)?;
    let sig = Signature::from_slice(&sig).map_err(|_| Refus::Signature)?;
    // verify_strict : refuse aussi les signatures malléables
    if !cles.iter().any(|c| {
        c.verify_strict(e.manifeste.as_bytes(), &sig).is_ok()
            && c.verify(e.manifeste.as_bytes(), &sig).is_ok()
    }) {
        return Err(Refus::Signature);
    }
    let m: Manifeste = serde_json::from_str(&e.manifeste).map_err(|_| Refus::Illisible)?;
    if m.serie < derniere_serie {
        return Err(Refus::Ancien {
            serie: m.serie,
            derniere: derniere_serie,
        });
    }
    if maintenant >= m.expire {
        return Err(Refus::Expire);
    }
    Ok(m)
}

/// Clé publique depuis son écriture en base64 (32 octets).
pub fn cle_publique(b64: &str) -> Option<VerifyingKey> {
    let o: [u8; 32] = B64.decode(b64.trim()).ok()?.try_into().ok()?;
    VerifyingKey::from_bytes(&o).ok()
}

/// Clé privée depuis son écriture en base64 (32 octets).
pub fn cle_privee(b64: &str) -> Option<SigningKey> {
    let o: [u8; 32] = B64.decode(b64.trim()).ok()?.try_into().ok()?;
    Some(SigningKey::from_bytes(&o))
}

/// Nouvelle paire (privée, publique) en base64, tirée de l'aléa du système.
pub fn nouvelle_cle() -> (String, String) {
    let mut o = [0u8; 32];
    getrandom::getrandom(&mut o).expect("aléa du système");
    let cle = SigningKey::from_bytes(&o);
    (
        B64.encode(cle.to_bytes()),
        B64.encode(cle.verifying_key().to_bytes()),
    )
}
