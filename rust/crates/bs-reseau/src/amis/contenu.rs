//! Contenu des présences et des lettres des amis : du JSON court (comme la version TS),
//! signé tel quel par son auteur. Tout ce qui est lu vient d'un autre joueur : chaque champ
//! est vérifié et borné.

use serde_json::{Value, json};

use crate::session::{code_lisible, code_saisi};

/// Longueur maximale d'un pseudo reçu.
const LONGUEUR_PSEUDO: usize = 16;

/// Où en est un joueur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EtatPresence {
    #[default]
    Menu,
    Salon,
    Partie,
}

impl EtatPresence {
    fn nom(self) -> &'static str {
        match self {
            Self::Menu => "menu",
            Self::Salon => "salon",
            Self::Partie => "partie",
        }
    }

    fn lu(v: &Value) -> Self {
        match v.as_str() {
            Some("salon") => Self::Salon,
            Some("partie") => Self::Partie,
            _ => Self::Menu,
        }
    }
}

/// Ce qu'un joueur publie de lui.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Etat {
    pub pseudo: String,
    pub etat: EtatPresence,
    /// Son salon s'il en héberge un qui a encore de la place.
    pub salon: Option<[u8; 6]>,
    /// Joueurs dans son salon ou sa partie.
    pub joueurs: u8,
}

/// Une lettre reçue ou à poster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lettre {
    /// « Je vous ai ajouté » (acceptée même d'un inconnu).
    Ajout,
    /// « Rejoignez mon salon ».
    Invitation([u8; 6]),
    /// « Je voudrais jouer avec vous ».
    Demande,
}

fn pseudo(v: &Value) -> String {
    v.as_str()
        .unwrap_or("?")
        .chars()
        .filter(|c| !c.is_control())
        .take(LONGUEUR_PSEUDO)
        .collect()
}

fn salon(v: &Value) -> Option<[u8; 6]> {
    v.as_str().and_then(code_saisi)
}

impl Etat {
    pub fn ecrire(&self) -> Vec<u8> {
        let v = json!({
            "pseudo": self.pseudo,
            "etat": self.etat.nom(),
            "salon": self.salon.as_ref().map(code_lisible),
            "joueurs": self.joueurs,
        });
        serde_json::to_vec(&v).unwrap_or_default()
    }

    pub fn lire(o: &[u8]) -> Option<Self> {
        let v: Value = serde_json::from_slice(o).ok()?;
        let joueurs = v["joueurs"].as_u64().unwrap_or(1).clamp(1, 16);
        Some(Self {
            pseudo: pseudo(&v["pseudo"]),
            etat: EtatPresence::lu(&v["etat"]),
            salon: salon(&v["salon"]),
            joueurs: joueurs as u8,
        })
    }
}

impl Lettre {
    /// La lettre, de la part de `de` (code ami) qui s'appelle `pseudo`.
    pub fn ecrire(&self, de: &str, pseudo: &str) -> Vec<u8> {
        let mut v = json!({ "de": de, "pseudo": pseudo });
        match self {
            Self::Ajout => v["t"] = "ajout".into(),
            Self::Demande => v["t"] = "demande".into(),
            Self::Invitation(s) => {
                v["t"] = "invitation".into();
                v["salon"] = code_lisible(s).into();
            }
        }
        serde_json::to_vec(&v).unwrap_or_default()
    }

    /// (lettre, code de l'auteur, son pseudo).
    pub fn lire(o: &[u8]) -> Option<(Self, String, String)> {
        let v: Value = serde_json::from_slice(o).ok()?;
        let de = v["de"].as_str()?.to_owned();
        let lettre = match v["t"].as_str()? {
            "ajout" => Self::Ajout,
            "demande" => Self::Demande,
            "invitation" => Self::Invitation(salon(&v["salon"])?),
            _ => return None,
        };
        Some((lettre, de, pseudo(&v["pseudo"])))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aller_retour() {
        let e = Etat {
            pseudo: "Steve".into(),
            etat: EtatPresence::Salon,
            salon: Some(*b"ABCDEF"),
            joueurs: 2,
        };
        assert_eq!(Etat::lire(&e.ecrire()), Some(e));
        for l in [
            Lettre::Ajout,
            Lettre::Demande,
            Lettre::Invitation(*b"KMNPQR"),
        ] {
            let o = l.ecrire("ABC-DEF-GHJ", "Alex");
            assert_eq!(
                Lettre::lire(&o),
                Some((l, "ABC-DEF-GHJ".into(), "Alex".into()))
            );
        }
        // Champs bornés
        let lu = Etat::lire(br#"{"pseudo":"abcdefghijklmnopqrstu","joueurs":99}"#).unwrap();
        assert_eq!(lu.pseudo.len(), 16);
        assert_eq!(lu.joueurs, 16);
        assert_eq!(lu.etat, EtatPresence::Menu);
        assert_eq!(Lettre::lire(br#"{"t":"invitation","de":"x"}"#), None);
    }
}
