//! Fichiers `.bssave` de Block Survivors, au format de la version TypeScript
//! (`src/progression/transfert.ts`) : « BSSAVE1: » puis base64 de (IV de 12 octets +
//! données chiffrées AES-256-GCM), clé = SHA-256 du secret du jeu. Le clair est un JSON :
//! un profil (`block-survivors-profil`) ou toute une sauvegarde d'avant les profils
//! (`block-survivors-sauvegarde`, clés préfixées). Les valeurs restent du JSON en texte :
//! leur lecture typée viendra avec bs-donnees (étape 4 du plan).

use std::collections::BTreeMap;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const ENTETE: &str = "BSSAVE1:";
const SECRET: &str = "Block Survivors · sauvegarde · 7c1e5a93d2f84b60";
const TAILLE_IV: usize = 12;
const FORMAT_PROFIL: &str = "block-survivors-profil";
const FORMAT_ANCIEN: &str = "block-survivors-sauvegarde";
const PREFIXE_ANCIEN: &str = "block-survivors.";

/// Données d'un profil, dans l'ordre de la version TypeScript (`DONNEES_PROFIL`).
pub const DONNEES_PROFIL: [&str; 6] = [
    "profil",
    "progression",
    "progression.multi",
    "collection",
    "choix",
    "amis",
];

/// Pourquoi un fichier est refusé.
#[derive(Debug, PartialEq, Eq)]
pub enum Erreur {
    /// Pas une sauvegarde, ou retouchée (GCM vérifie l'intégrité).
    Illisible,
    /// Une sauvegarde, mais sans aucune donnée de profil.
    Vide,
}

/// Un profil lu : valeur brute (JSON en texte) de chaque donnée présente.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Profil {
    /// Version du jeu qui a écrit le fichier (absente dans les fichiers d'avant les profils).
    pub version: Option<String>,
    pub donnees: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct Fichier {
    format: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    date: Option<String>,
    donnees: BTreeMap<String, serde_json::Value>,
}

fn chiffreur() -> Aes256Gcm {
    let cle = Sha256::digest(SECRET.as_bytes());
    Aes256Gcm::new_from_slice(&cle).expect("clé de 32 octets")
}

/// Texte clair d'un fichier `.bssave`.
pub fn dechiffrer(contenu: &str) -> Result<String, Erreur> {
    let b64 = contenu
        .strip_prefix(ENTETE)
        .ok_or(Erreur::Illisible)?
        .trim();
    let tout = STANDARD.decode(b64).map_err(|_| Erreur::Illisible)?;
    if tout.len() < TAILLE_IV {
        return Err(Erreur::Illisible);
    }
    let (iv, chiffre) = tout.split_at(TAILLE_IV);
    let clair = chiffreur()
        .decrypt(Nonce::from_slice(iv), chiffre)
        .map_err(|_| Erreur::Illisible)?;
    String::from_utf8(clair).map_err(|_| Erreur::Illisible)
}

/// Chiffre un texte clair en `.bssave` (IV tiré au hasard).
pub fn chiffrer(texte: &str) -> String {
    let mut iv = [0u8; TAILLE_IV];
    getrandom::getrandom(&mut iv).expect("aléa du système");
    let chiffre = chiffreur()
        .encrypt(Nonce::from_slice(&iv), texte.as_bytes())
        .expect("chiffrement AES-GCM");
    let mut tout = iv.to_vec();
    tout.extend_from_slice(&chiffre);
    format!("{ENTETE}{}", STANDARD.encode(tout))
}

/// Lit un `.bssave` (profil, ou sauvegarde d'avant les profils) comme le fait l'import TS.
pub fn importer(contenu: &str) -> Result<Profil, Erreur> {
    let fichier: Fichier =
        serde_json::from_str(&dechiffrer(contenu)?).map_err(|_| Erreur::Illisible)?;
    let prefixe = match fichier.format.as_str() {
        FORMAT_PROFIL => "",
        FORMAT_ANCIEN => PREFIXE_ANCIEN,
        _ => return Err(Erreur::Illisible),
    };
    let mut donnees = BTreeMap::new();
    for nom in DONNEES_PROFIL {
        if let Some(serde_json::Value::String(v)) = fichier.donnees.get(&format!("{prefixe}{nom}"))
        {
            donnees.insert(nom.to_string(), v.clone());
        }
    }
    if donnees.is_empty() {
        return Err(Erreur::Vide);
    }
    Ok(Profil {
        version: fichier.version,
        donnees,
    })
}

/// Écrit un profil en `.bssave` lisible par la version TypeScript.
pub fn exporter(profil: &Profil, version: &str, date: &str) -> String {
    let fichier = Fichier {
        format: FORMAT_PROFIL.into(),
        version: Some(version.into()),
        date: Some(date.into()),
        donnees: profil
            .donnees
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect(),
    };
    chiffrer(&serde_json::to_string(&fichier).expect("JSON"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aller_retour() {
        let mut p = Profil::default();
        p.donnees
            .insert("profil".into(), r#"{"pseudo":"Éloïse"}"#.into());
        let lu = importer(&exporter(&p, "0.50.0", "2026-10-31T00:00:00Z")).unwrap();
        assert_eq!(lu.donnees, p.donnees);
        assert_eq!(lu.version.as_deref(), Some("0.50.0"));
    }

    #[test]
    fn refuse_un_fichier_retouche() {
        let mut p = Profil::default();
        p.donnees.insert("profil".into(), "{}".into());
        let f = exporter(&p, "1", "d");
        let mut octets = STANDARD.decode(&f[ENTETE.len()..]).unwrap();
        *octets.last_mut().unwrap() ^= 1;
        assert_eq!(
            importer(&format!("{ENTETE}{}", STANDARD.encode(octets))),
            Err(Erreur::Illisible)
        );
        assert_eq!(importer("pas une sauvegarde"), Err(Erreur::Illisible));
    }
}
