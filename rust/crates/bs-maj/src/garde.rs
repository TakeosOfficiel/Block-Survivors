//! Retour arrière automatique (plan, section 5) : une version qui plante deux fois de suite
//! au démarrage est remplacée par la dernière qui a marché, puis refusée par la recherche
//! de mises à jour jusqu'à la suivante.
//!
//! - Au lancement (`verifier_demarrage`, juste après les crochets de Velopack), le jeu note
//!   un démarrage en cours.
//! - Une fois le jeu lancé depuis 20 s (`demarrage_reussi`, appelé par le jeu), le démarrage
//!   compte comme réussi. Le paquet complet de cette version est alors gardé à part
//!   (`bonne/`) : les deltas ne vont que vers l'avant, et Velopack ne garde que le dernier.
//! - Au troisième lancement après deux échecs, ce paquet est remis dans le dossier des
//!   paquets de Velopack. Les plus récents en sont retirés (Velopack les réappliquerait
//!   au lancement), puis l'installeur applique le paquet gardé et relance le jeu, qui le
//!   signale au menu.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use velopack::locator::{LocationContext, VelopackLocator, auto_locate_app_manifest};
use velopack::{UpdateManager, VelopackAsset};

/// Démarrages ratés de suite avant le retour arrière.
pub const ECHECS_MAX: u32 = 2;
const FICHIER: &str = "demarrage.json";
const DOSSIER_BONNE: &str = "bonne";

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct EtatDemarrage {
    pub version: String,
    /// Démarrages commencés et jamais réussis, de suite.
    pub echecs: u32,
    /// Version qui plantait : la recherche de mises à jour l'ignore.
    pub refusee: Option<String>,
    /// Retour fait (depuis, vers), à signaler au menu.
    pub retour: Option<(String, String)>,
}

#[derive(Debug, PartialEq)]
pub enum Decision {
    Continuer,
    /// Revenir à cette version (celle du paquet gardé).
    Revenir(String),
}

/// Le cœur, sans fichiers : met l'état à jour pour un démarrage de `version`, sachant la
/// version du paquet gardé.
pub fn decider(e: &mut EtatDemarrage, version: &str, bonne: Option<&str>) -> Decision {
    if e.version != version {
        e.version = version.to_owned();
        e.echecs = 0;
    }
    if e.echecs >= ECHECS_MAX
        && let Some(b) = bonne.filter(|b| *b != version)
    {
        e.refusee = Some(version.to_owned());
        e.retour = Some((version.to_owned(), b.to_owned()));
        e.version = b.to_owned();
        e.echecs = 0;
        return Decision::Revenir(b.to_owned());
    }
    e.echecs += 1;
    Decision::Continuer
}

fn lire(dossier: &Path) -> EtatDemarrage {
    std::fs::read(dossier.join(FICHIER))
        .ok()
        .and_then(|o| serde_json::from_slice(&o).ok())
        .unwrap_or_default()
}

fn ecrire(dossier: &Path, e: &EtatDemarrage) {
    let _ = std::fs::create_dir_all(dossier);
    if let Ok(texte) = serde_json::to_vec_pretty(e) {
        let _ = std::fs::write(dossier.join(FICHIER), texte);
    }
}

fn installation() -> Option<VelopackLocator> {
    auto_locate_app_manifest(LocationContext::FromCurrentExe).ok()
}

/// Paquet complet gardé (chemin, version).
fn paquet_garde(dossier: &Path) -> Option<(PathBuf, String)> {
    velopack::locator::find_latest_full_package(&dossier.join(DOSSIER_BONNE))
        .map(|(p, m)| (p, m.version.to_string()))
}

/// Au lancement, avant le jeu : note le démarrage ou, après deux échecs, revient à la
/// dernière version qui a marché (le jeu se ferme alors aussitôt). `dossier` : celui des
/// sauvegardes. Rien hors installation Velopack.
pub fn verifier_demarrage(dossier: &Path) {
    let Some(loc) = installation() else {
        return;
    };
    let version = loc.get_manifest_version_full_string();
    let mut e = lire(dossier);
    let garde = paquet_garde(dossier);
    match decider(&mut e, &version, garde.as_ref().map(|g| g.1.as_str())) {
        Decision::Continuer => ecrire(dossier, &e),
        Decision::Revenir(vers) => {
            ecrire(dossier, &e);
            if let Some((chemin, _)) = garde
                && let Err(raison) = revenir(&loc, &chemin, &vers)
            {
                // Retour impossible : on joue la version actuelle, sans recommencer
                e.retour = None;
                ecrire(dossier, &e);
                eprintln!("retour à la version {vers} impossible : {raison}");
                return;
            }
            std::process::exit(0);
        }
    }
}

fn revenir(loc: &VelopackLocator, garde: &Path, vers: &str) -> Result<(), String> {
    let paquets = loc.get_packages_dir();
    let cible = semver(vers)?;
    for (p, m) in velopack::locator::find_local_full_packages(&paquets) {
        if m.version > cible {
            let _ = std::fs::remove_file(p);
        }
    }
    let nom = garde.file_name().ok_or("paquet sans nom")?;
    std::fs::copy(garde, paquets.join(nom)).map_err(|e| e.to_string())?;
    let um = UpdateManager::new(velopack::sources::NoneSource {}, None, None)
        .map_err(|e| e.to_string())?;
    let asset = VelopackAsset {
        PackageId: loc.get_manifest_id(),
        Version: vers.to_owned(),
        Type: "Full".into(),
        FileName: nom.to_string_lossy().into_owned(),
        ..Default::default()
    };
    um.wait_exit_then_apply_updates(&asset, true, true, Vec::<String>::new())
        .map_err(|e| e.to_string())
}

fn semver(v: &str) -> Result<semver::Version, String> {
    semver::Version::parse(v).map_err(|e| e.to_string())
}

/// Démarrage réussi (20 s de jeu) : compteur remis à zéro et paquet
/// de cette version gardé pour un retour futur.
pub fn demarrage_reussi(dossier: &Path) {
    let Some(loc) = installation() else {
        return;
    };
    let mut e = lire(dossier);
    let version = loc.get_manifest_version_full_string();
    if e.version == version && e.echecs == 0 {
        return;
    }
    e.version = version.clone();
    e.echecs = 0;
    ecrire(dossier, &e);
    if paquet_garde(dossier).is_some_and(|g| g.1 == version) {
        return;
    }
    let actuel = velopack::locator::find_local_full_packages(&loc.get_packages_dir())
        .into_iter()
        .find(|(_, m)| m.version.to_string() == version);
    if let Some((chemin, _)) = actuel
        && let Some(nom) = chemin.file_name()
    {
        let bonne = dossier.join(DOSSIER_BONNE);
        let _ = std::fs::remove_dir_all(&bonne);
        let _ = std::fs::create_dir_all(&bonne);
        let _ = std::fs::copy(&chemin, bonne.join(nom));
    }
}

/// Version que la recherche doit ignorer (elle plantait au démarrage).
pub fn version_refusee(dossier: &Path) -> Option<String> {
    lire(dossier).refusee
}

/// Retour arrière fait au dernier lancement (depuis, vers), rendu une seule fois.
pub fn retour_a_signaler(dossier: &Path) -> Option<(String, String)> {
    let mut e = lire(dossier);
    let r = e.retour.take()?;
    ecrire(dossier, &e);
    Some(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deux_plantages_puis_retour() {
        let mut e = EtatDemarrage::default();
        // 0.2.0 démarre et plante deux fois (jamais réussi)
        assert_eq!(decider(&mut e, "0.2.0", Some("0.1.0")), Decision::Continuer);
        assert_eq!(decider(&mut e, "0.2.0", Some("0.1.0")), Decision::Continuer);
        assert_eq!(e.echecs, 2);
        // Troisième lancement : retour à 0.1.0, 0.2.0 refusée
        assert_eq!(
            decider(&mut e, "0.2.0", Some("0.1.0")),
            Decision::Revenir("0.1.0".into())
        );
        assert_eq!(e.refusee.as_deref(), Some("0.2.0"));
        assert_eq!(e.retour, Some(("0.2.0".into(), "0.1.0".into())));
        // 0.1.0 relancée : compteur neuf
        assert_eq!(decider(&mut e, "0.1.0", Some("0.1.0")), Decision::Continuer);
        assert_eq!((e.version.as_str(), e.echecs), ("0.1.0", 1));
    }

    #[test]
    fn sans_paquet_garde_pas_de_boucle() {
        let mut e = EtatDemarrage::default();
        for _ in 0..5 {
            assert_eq!(decider(&mut e, "0.2.0", None), Decision::Continuer);
        }
        // Le paquet gardé est celui de la version qui plante : rien à faire non plus
        assert_eq!(decider(&mut e, "0.2.0", Some("0.2.0")), Decision::Continuer);
    }

    #[test]
    fn nouvelle_version_compteur_neuf() {
        let mut e = EtatDemarrage {
            version: "0.1.0".into(),
            echecs: 1,
            ..Default::default()
        };
        decider(&mut e, "0.2.0", Some("0.1.0"));
        assert_eq!((e.version.as_str(), e.echecs), ("0.2.0", 1));
    }

    #[test]
    fn fichier_et_signalement() {
        let d = std::env::temp_dir().join(format!("bs-garde-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        assert_eq!(version_refusee(&d), None);
        ecrire(
            &d,
            &EtatDemarrage {
                version: "0.1.0".into(),
                refusee: Some("0.2.0".into()),
                retour: Some(("0.2.0".into(), "0.1.0".into())),
                ..Default::default()
            },
        );
        assert_eq!(version_refusee(&d).as_deref(), Some("0.2.0"));
        assert_eq!(
            retour_a_signaler(&d),
            Some(("0.2.0".into(), "0.1.0".into()))
        );
        assert_eq!(retour_a_signaler(&d), None);
        let _ = std::fs::remove_dir_all(&d);
    }
}
