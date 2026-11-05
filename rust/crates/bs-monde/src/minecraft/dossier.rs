//! Maps du joueur : un .schem ou .nbt glissé sur la fenêtre est copié (avec le .json du
//! même nom, s'il est à côté) dans le dossier `cartes` des données du jeu, puis lu au
//! démarrage comme les maps embarquées. Les fichiers lus restent en mémoire jusqu'à la
//! fin du programme (comme les maps embarquées).

use std::path::{Path, PathBuf};

use super::cartes::Carte;
use super::schematique::lire_schematique;

/// Extensions reconnues comme des maps.
const EXTENSIONS: [&str; 2] = ["schem", "nbt"];

/// Le fichier a-t-il l'extension d'une map ?
pub fn est_une_map(chemin: &Path) -> bool {
    chemin
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| EXTENSIONS.contains(&e.to_lowercase().as_str()))
}

/// Lit une map (schématique vérifiée, .json voisin facultatif).
pub fn lire_carte(chemin: &Path) -> Result<&'static Carte, String> {
    let nom = chemin
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("map")
        .to_owned();
    let id = chemin
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("{nom} : nom de fichier invalide."))?;
    let octets = std::fs::read(chemin).map_err(|e| format!("{nom} : illisible ({e})."))?;
    lire_schematique(&octets).map_err(|e| format!("{nom} : {e}"))?;
    let reglages = std::fs::read_to_string(chemin.with_extension("json")).ok();
    Ok(Box::leak(Box::new(Carte {
        id: Box::leak(id.to_owned().into_boxed_str()),
        fichier: Box::leak(octets.into_boxed_slice()),
        reglages: reglages.map(|r| &*Box::leak(r.into_boxed_str())),
    })))
}

/// Maps du dossier, par ordre alphabétique ; les fichiers illisibles sont écartés (et
/// signalés dans `erreurs`).
pub fn cartes_du_dossier(dossier: &Path, erreurs: &mut Vec<String>) -> Vec<&'static Carte> {
    let mut chemins: Vec<PathBuf> = std::fs::read_dir(dossier)
        .map(|d| d.filter_map(Result::ok).map(|e| e.path()).collect())
        .unwrap_or_default();
    chemins.retain(|c| est_une_map(c));
    chemins.sort();
    chemins
        .iter()
        .filter_map(|c| lire_carte(c).map_err(|e| erreurs.push(e)).ok())
        .collect()
}

/// Copie la map `source` (et son .json) dans `dossier`, puis la lit depuis sa copie.
pub fn ajouter_carte(dossier: &Path, source: &Path) -> Result<&'static Carte, String> {
    let carte = lire_carte(source)?;
    let nom = source.file_name().ok_or("Nom de fichier invalide.")?;
    std::fs::create_dir_all(dossier).map_err(|e| format!("Dossier des maps : {e}"))?;
    let cible = dossier.join(nom);
    if cible != source {
        std::fs::copy(source, &cible).map_err(|e| format!("Copie de la map : {e}"))?;
        let json = source.with_extension("json");
        if json.exists() {
            std::fs::copy(&json, cible.with_extension("json"))
                .map_err(|e| format!("Copie des réglages : {e}"))?;
        }
    }
    Ok(carte)
}

#[cfg(test)]
mod essais {
    use super::*;

    #[test]
    fn extensions() {
        assert!(est_une_map(Path::new("a/Village.SCHEM")));
        assert!(est_une_map(Path::new("tour.nbt")));
        assert!(!est_une_map(Path::new("skin.png")));
    }

    #[test]
    fn fichier_qui_n_est_pas_une_map() {
        let dossier = std::env::temp_dir().join(format!("bs-cartes-{}", std::process::id()));
        std::fs::create_dir_all(&dossier).unwrap();
        let faux = dossier.join("faux.schem");
        std::fs::write(&faux, b"pas une map").unwrap();
        let mut erreurs = Vec::new();
        assert!(cartes_du_dossier(&dossier, &mut erreurs).is_empty());
        assert_eq!(erreurs.len(), 1);
        assert!(ajouter_carte(&dossier.join("copie"), &faux).is_err());
        std::fs::remove_dir_all(&dossier).unwrap();
    }
}
