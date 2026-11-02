//! Appli de démonstration de la chaîne de mise à jour (pari 4) : empaquetée par `vpk` en
//! version N puis N+1, elle écrit sa version dans un journal et, avec `--maj <dépôt>`,
//! cherche une mise à jour via le manifeste signé, la télécharge, l'applique et redémarre.
//! Clé publique : BS_CLE_PUBLIQUE (dans le jeu, elle sera écrite dans le code).
//!
//! Garde (retour arrière) : dossier BS_DOSSIER_GARDE. Une version empaquetée avec
//! `donnees.txt` contenant « plante » simule un plantage au démarrage.

use std::io::Write;

use bs_maj::manifeste::cle_publique;
use bs_maj::{Dossier, SourceSignee, garde};
use velopack::{UpdateCheck, UpdateManager, VelopackApp};

fn journal(ligne: &str) {
    let chemin = std::env::var("BS_JOURNAL").unwrap_or_else(|_| "/tmp/demo-maj.log".into());
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(chemin)
        .expect("journal");
    let _ = writeln!(f, "{ligne}");
}

fn main() {
    // En premier : Velopack gère ici ses propres crochets (installation, mise à jour)
    VelopackApp::build().run();
    let dossier: std::path::PathBuf = std::env::var("BS_DOSSIER_GARDE")
        .unwrap_or_else(|_| "/tmp/demo-maj-garde".into())
        .into();
    garde::verifier_demarrage(&dossier);
    if let Some((depuis, vers)) = garde::retour_a_signaler(&dossier) {
        journal(&format!(
            "retour arrière : {depuis} plantait, retour à {vers}"
        ));
    }
    let exe = std::env::current_exe().expect("exécutable");
    let donnees = std::fs::read_to_string(exe.with_file_name("donnees.txt")).unwrap_or_default();
    if donnees.contains("plante") {
        journal("plantage simulé au démarrage");
        std::process::exit(1);
    }
    garde::demarrage_reussi(&dossier);
    let args: Vec<String> = std::env::args().collect();
    let Some(depot) = args
        .iter()
        .position(|a| a == "--maj")
        .and_then(|i| args.get(i + 1))
    else {
        // Lancé sans --maj (ou relancé après la mise à jour) : dire qui on est
        match UpdateManager::new(velopack::sources::NoneSource {}, None, None) {
            Ok(um) => journal(&format!(
                "démarré : version {}",
                um.get_current_version_as_string()
            )),
            Err(e) => journal(&format!("démarré hors installation : {e}")),
        }
        return;
    };
    let cle = cle_publique(&std::env::var("BS_CLE_PUBLIQUE").expect("BS_CLE_PUBLIQUE"))
        .expect("clé publique valide");
    let serie = std::env::var("BS_FICHIER_SERIE").unwrap_or_else(|_| "/tmp/demo-maj-serie".into());
    let source = SourceSignee::new(Dossier(depot.into()), vec![cle], serie.into());
    let um = match UpdateManager::new(source, None, None) {
        Ok(um) => um,
        Err(e) => return journal(&format!("pas installé : {e}")),
    };
    journal(&format!(
        "version {} : recherche de mise à jour",
        um.get_current_version_as_string()
    ));
    match um.check_for_updates() {
        Ok(UpdateCheck::UpdateAvailable(info))
            if garde::version_refusee(&dossier).as_deref()
                == Some(info.TargetFullRelease.Version.as_str()) =>
        {
            journal(&format!(
                "à jour ({} refusée : plantait)",
                info.TargetFullRelease.Version
            ));
        }
        Ok(UpdateCheck::UpdateAvailable(info)) => {
            let t = std::time::Instant::now();
            let delta = !info.DeltasToTarget.is_empty();
            if let Err(e) = um.download_updates(&info, None) {
                return journal(&format!("téléchargement refusé : {e}"));
            }
            journal(&format!(
                "version {} téléchargée ({}) en {:.2} s, application et redémarrage",
                info.TargetFullRelease.Version,
                if delta { "delta" } else { "paquet complet" },
                t.elapsed().as_secs_f32()
            ));
            if let Err(e) = um.apply_updates_and_restart(&info.TargetFullRelease) {
                journal(&format!("application impossible : {e}"));
            }
        }
        Ok(_) => journal("à jour"),
        Err(e) => journal(&format!("refusé : {e}")),
    }
}
