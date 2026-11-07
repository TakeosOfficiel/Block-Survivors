//! Stockage local, comme `window.localStorage` de la version TypeScript : une clé
//! (« block-survivors.p.1.progression ») et sa valeur JSON en texte, une par fichier
//! `<clé>.json` du dossier de sauvegarde. Écriture dans un fichier temporaire puis
//! renommage : un arrêt brutal ne laisse pas de fichier à moitié écrit. Comme dans le jeu
//! TS, une erreur d'écriture ne fait que perdre la valeur pour cette session.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Dossier des sauvegardes de l'installation.
#[derive(Debug, Clone)]
pub struct Stockage {
    dossier: PathBuf,
}

impl Stockage {
    pub fn nouveau(dossier: impl Into<PathBuf>) -> Self {
        Self {
            dossier: dossier.into(),
        }
    }

    /// Dossier du système (`BS_DOSSIER` pour un autre) : `%APPDATA%\block-survivors`
    /// sous Windows, `~/Library/Application Support/block-survivors` sous macOS,
    /// `$XDG_DATA_HOME/block-survivors` (ou `~/.local/share/…`) ailleurs.
    pub fn du_systeme() -> Self {
        let var = |n: &str| {
            std::env::var_os(n)
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
        };
        if let Some(d) = var("BS_DOSSIER") {
            return Self::nouveau(d);
        }
        let base = if cfg!(windows) {
            var("APPDATA")
        } else if cfg!(target_os = "macos") {
            var("HOME").map(|h| h.join("Library/Application Support"))
        } else {
            var("XDG_DATA_HOME").or_else(|| var("HOME").map(|h| h.join(".local/share")))
        };
        Self::nouveau(
            base.unwrap_or_else(|| PathBuf::from("."))
                .join("block-survivors"),
        )
    }

    pub fn dossier(&self) -> &Path {
        &self.dossier
    }

    fn chemin(&self, cle: &str) -> PathBuf {
        // Les clés du jeu n'ont que des lettres, chiffres, points et tirets
        let nom: String = cle
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '.' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        self.dossier.join(format!("{nom}.json"))
    }

    /// Valeur brute (JSON en texte), ou `None`.
    pub fn lire_brut(&self, cle: &str) -> Option<String> {
        std::fs::read_to_string(self.chemin(cle)).ok()
    }

    /// Valeur lue, ou `None` si absente ou illisible.
    pub fn lire<T: DeserializeOwned>(&self, cle: &str) -> Option<T> {
        serde_json::from_str(&self.lire_brut(cle)?).ok()
    }

    pub fn ecrire_brut(&self, cle: &str, valeur: &str) {
        let chemin = self.chemin(cle);
        let temporaire = chemin.with_extension("json.tmp");
        let ecrit = std::fs::create_dir_all(&self.dossier)
            .and_then(|()| std::fs::write(&temporaire, valeur))
            .and_then(|()| std::fs::rename(&temporaire, &chemin));
        if let Err(e) = ecrit {
            eprintln!("sauvegarde : {} non écrit ({e})", chemin.display());
        }
    }

    pub fn ecrire<T: Serialize>(&self, cle: &str, valeur: &T) {
        let texte = serde_json::to_string(valeur).expect("valeur en JSON");
        self.ecrire_brut(cle, &texte);
    }

    pub fn retirer(&self, cle: &str) {
        let _ = std::fs::remove_file(self.chemin(cle));
    }
}

/// Dossier vide et propre à un essai.
#[cfg(test)]
pub(crate) fn essai(nom: &str) -> Stockage {
    let d = std::env::temp_dir().join(format!("bs-sauvegarde-{nom}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    Stockage::nouveau(d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecrit_relit_retire() {
        let s = essai("stockage");
        assert_eq!(s.lire_brut("block-survivors.profils"), None);
        s.ecrire(
            "block-survivors.p.1.choix",
            &serde_json::json!({ "arene": "prairie" }),
        );
        let v: serde_json::Value = s.lire("block-survivors.p.1.choix").unwrap();
        assert_eq!(v["arene"], "prairie");
        s.retirer("block-survivors.p.1.choix");
        assert_eq!(s.lire_brut("block-survivors.p.1.choix"), None);
    }
}
