//! Profils venus de la version Electron (plan, section 5, « Passage depuis Electron »).
//!
//! La dernière mise à jour Electron exporte chacun de ses profils en `.bssave` dans le
//! dossier `depuis-electron/` de nos sauvegardes, avec `actif.txt` (nom du fichier du
//! profil qui était actif), puis installe cette version. Au lancement suivant, chaque
//! fichier devient un profil, et l'ancien profil actif le redevient.
//!
//! Les fichiers importés vont dans `depuis-electron/importes/`, ceux refusés dans
//! `refuses/`. Un fichier refusé parce qu'il y a déjà trop de profils reste en place :
//! il sera repris quand un profil aura été supprimé.

use std::path::{Path, PathBuf};

use crate::profils::{Profils, RefusImport};

pub const DOSSIER: &str = "depuis-electron";
const ACTIF: &str = "actif.txt";

/// Ce qu'a donné le passage.
#[derive(Debug, Default, PartialEq)]
pub struct Passage {
    /// Profils créés (identifiants).
    pub importes: Vec<String>,
    /// Fichiers refusés (nom, raison).
    pub refuses: Vec<(String, &'static str)>,
}

/// Importe les profils déposés par la version Electron. Rien à faire : `Passage` vide.
pub fn importer(profils: &mut Profils) -> Passage {
    let dossier = profils.stockage().dossier().join(DOSSIER);
    let mut passage = Passage::default();
    let Ok(entrees) = std::fs::read_dir(&dossier) else {
        return passage;
    };
    let mut fichiers: Vec<PathBuf> = entrees
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == "bssave"))
        .collect();
    if fichiers.is_empty() {
        return passage;
    }
    fichiers.sort();
    let actif = std::fs::read_to_string(dossier.join(ACTIF)).unwrap_or_default();
    let mut id_actif = None;
    for chemin in &fichiers {
        let nom = chemin
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let contenu = std::fs::read_to_string(chemin).unwrap_or_default();
        match profils.importer(&contenu) {
            Ok(id) => {
                ranger(chemin, &dossier.join("importes"));
                if nom == actif.trim() {
                    id_actif = Some(id.clone());
                }
                passage.importes.push(id);
            }
            Err(RefusImport::Complet) => {
                passage.refuses.push((nom, RefusImport::Complet.message()))
            }
            Err(e) => {
                ranger(chemin, &dossier.join("refuses"));
                passage.refuses.push((nom, e.message()));
            }
        }
    }
    if let Some(id) = id_actif.or_else(|| passage.importes.first().cloned()) {
        profils.changer(&id);
    }
    passage
}

fn ranger(fichier: &Path, vers: &Path) {
    let _ = std::fs::create_dir_all(vers);
    if let Some(nom) = fichier.file_name() {
        let _ = std::fs::rename(fichier, vers.join(nom));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bssave::{self, Profil};
    use crate::stockage::essai;

    fn fichier(pseudo: &str) -> String {
        let mut p = Profil::default();
        p.donnees
            .insert("profil".into(), format!("{{\"pseudo\":\"{pseudo}\"}}"));
        bssave::exporter(&p, "1.0.0", "2026-11-02T00:00:00Z")
    }

    #[test]
    fn deux_profils_electron_au_premier_lancement() {
        let s = essai("electron");
        let mut profils = Profils::charger(s.clone());
        assert_eq!(importer(&mut profils), Passage::default());
        let d = s.dossier().join(DOSSIER);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("1-Alice.bssave"), fichier("Alice")).unwrap();
        std::fs::write(d.join("2-Bob.bssave"), fichier("Bob")).unwrap();
        std::fs::write(d.join("3-abime.bssave"), "pas une sauvegarde").unwrap();
        std::fs::write(d.join(ACTIF), "2-Bob.bssave\n").unwrap();
        let passage = importer(&mut profils);
        // Alice prend la place du profil vierge, Bob est ajouté et redevient actif
        assert_eq!(passage.importes, vec!["1".to_string(), "2".to_string()]);
        assert_eq!(passage.refuses.len(), 1);
        assert_eq!(profils.ids(), ["1", "2"]);
        assert_eq!(profils.actif(), "2");
        let lu = s.lire_brut(&crate::profils::cle("2", "profil")).unwrap();
        assert!(lu.contains("Bob"));
        assert!(d.join("importes/2-Bob.bssave").exists());
        assert!(d.join("refuses/3-abime.bssave").exists());
        // Rien ne se refait au lancement suivant
        assert_eq!(
            importer(&mut Profils::charger(s.clone())),
            Passage::default()
        );
        let _ = std::fs::remove_dir_all(s.dossier());
    }
}
