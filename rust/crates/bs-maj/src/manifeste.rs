//! Manifeste d'une version : numéro de série croissant, expiration, voie, version minimale
//! pour la coop, et la liste des paquets Velopack (complet et deltas) avec leurs empreintes.
//! Il voyage dans une enveloppe JSON : `{"manifeste": "<le JSON, en texte>", "signature":
//! "<base64>", "delegation": {…}}`. On signe les octets exacts du texte : pas de question
//! de mise en forme. La signature vient d'une clé racine du jeu, ou d'une clé que la
//! délégation jointe autorise (`delegation`).

use base64::{Engine, engine::general_purpose::STANDARD as B64};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::delegation::{Delegation, cles_deleguees, signee};

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
    /// Installeurs de la version (`Setup.exe`, `.AppImage`) : la version Electron les
    /// vérifie avant de lancer le passage. Absents des vieux manifestes.
    #[serde(default)]
    pub installeurs: Vec<Installeur>,
}

/// Un installeur publié avec la version.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Installeur {
    pub fichier: String,
    pub sha256: String,
    pub taille: u64,
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
    /// Délégation qui autorise la clé signataire (absente : signé par une racine).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation: Option<Delegation>,
}

/// Dernières séries vues (manifeste, délégation) : jamais en arrière.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Series {
    pub manifeste: u64,
    pub delegation: u64,
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
    DelegationAncienne {
        serie: u64,
        derniere: u64,
    },
    DelegationExpiree,
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
            Refus::DelegationAncienne { serie, derniere } => {
                write!(f, "délégation ancienne (série {serie} < {derniere})")
            }
            Refus::DelegationExpiree => write!(f, "délégation expirée"),
        }
    }
}

/// Enveloppe signée (JSON) du manifeste, avec la délégation qui autorise `cle` (aucune si
/// `cle` est une racine).
pub fn signer(m: &Manifeste, cle: &SigningKey, delegation: Option<&Delegation>) -> String {
    let texte = serde_json::to_string_pretty(m).expect("manifeste sérialisable");
    let signature = B64.encode(cle.sign(texte.as_bytes()).to_bytes());
    serde_json::to_string_pretty(&Enveloppe {
        manifeste: texte,
        signature,
        delegation: delegation.cloned(),
    })
    .expect("enveloppe sérialisable")
}

/// Vérifie l'enveloppe : signature par l'une des `racines`, ou par une clé de sa délégation
/// (elle-même signée par une racine, pas plus ancienne que la dernière vue, pas expirée) ;
/// série au moins égale à la dernière vue ; pas expirée à `maintenant` (secondes Unix).
/// Rend aussi les séries à retenir.
pub fn verifier(
    octets: &[u8],
    racines: &[VerifyingKey],
    vues: Series,
    maintenant: u64,
) -> Result<(Manifeste, Series), Refus> {
    let e: Enveloppe = serde_json::from_slice(octets).map_err(|_| Refus::Illisible)?;
    let mut series = vues;
    if !signee(&e.manifeste, &e.signature, racines) {
        let d = e.delegation.as_ref().ok_or(Refus::Signature)?;
        let (serie, cles) = cles_deleguees(d, racines, vues.delegation, maintenant)?;
        if !signee(&e.manifeste, &e.signature, &cles) {
            return Err(Refus::Signature);
        }
        series.delegation = serie;
    }
    let m: Manifeste = serde_json::from_str(&e.manifeste).map_err(|_| Refus::Illisible)?;
    if m.serie < vues.manifeste {
        return Err(Refus::Ancien {
            serie: m.serie,
            derniere: vues.manifeste,
        });
    }
    if maintenant >= m.expire {
        return Err(Refus::Expire);
    }
    series.manifeste = m.serie;
    Ok((m, series))
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
