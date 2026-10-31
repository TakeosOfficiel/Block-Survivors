//! Importeur : JSON exporté par la version TypeScript → fichiers RON de `donnees/`.
//! Usage : importer-donnees <dossier JSON> <dossier RON> (voir outils/importer-donnees.sh).

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [entree, sortie] = args.as_slice() else {
        eprintln!("usage : importer-donnees <dossier JSON> <dossier RON>");
        return ExitCode::FAILURE;
    };
    let mut echecs = 0;
    for nom in bs_donnees::FICHIERS {
        let resultat = std::fs::read_to_string(entree.join(format!("{nom}.json")))
            .map_err(|e| format!("{nom}.json : {e}"))
            .and_then(|json| bs_donnees::importation::convertir(nom, &json))
            .and_then(|ron| {
                std::fs::write(sortie.join(format!("{nom}.ron")), ron)
                    .map_err(|e| format!("{nom}.ron : {e}"))
            });
        if let Err(e) = resultat {
            eprintln!("{e}");
            echecs += 1;
        }
    }
    if echecs > 0 {
        return ExitCode::FAILURE;
    }
    match bs_donnees::Donnees::charger(sortie) {
        Ok(_) => {
            println!(
                "{} fichiers écrits dans {} et validés",
                bs_donnees::FICHIERS.len(),
                sortie.display()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
