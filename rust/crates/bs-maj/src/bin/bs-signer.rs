//! Outil de la CI pour le manifeste signé (plan, section 5) :
//!
//!   bs-signer cles
//!       nouvelle paire de clés (base64) ; la privée va dans les secrets GitHub (ou hors
//!       ligne pour la clé de secours), la publique dans le jeu.
//!   bs-signer manifeste <dossier vpk> <voie> <série> <jours de validité> <version min coop>
//!       lit `releases.<voie>.json` écrit par `vpk pack`, en fait le manifeste, le signe
//!       avec la clé privée de BS_CLE_PRIVEE et l'écrit dans `<dossier>/version.json`.

use std::path::Path;

use bs_maj::manifeste::{Manifeste, Paquet, cle_privee, nouvelle_cle, signer};
use bs_maj::source::NOM_MANIFESTE;
use serde::Deserialize;

/// Flux écrit par `vpk` (`releases.<voie>.json`), seuls les champs utiles.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Flux {
    assets: Vec<Actif>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Actif {
    package_id: String,
    version: String,
    #[serde(rename = "Type")]
    genre: String,
    file_name: String,
    #[serde(rename = "SHA1")]
    sha1: String,
    #[serde(rename = "SHA256", default)]
    sha256: String,
    size: u64,
    #[serde(default)]
    notes_markdown: String,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["cles"] => {
            let (privee, publique) = nouvelle_cle();
            println!("privée   : {privee}\npublique : {publique}");
        }
        ["manifeste", dossier, voie, serie, jours, min_coop] => {
            let dossier = Path::new(dossier);
            let flux: Flux = serde_json::from_slice(
                &std::fs::read(dossier.join(format!("releases.{voie}.json"))).expect("flux de vpk"),
            )
            .expect("flux de vpk lisible");
            let cle = cle_privee(&std::env::var("BS_CLE_PRIVEE").expect("BS_CLE_PRIVEE"))
                .expect("clé privée valide");
            // La version annoncée : la plus récente du flux (vpk y garde aussi les anciennes)
            let version = flux
                .assets
                .iter()
                .map(|a| a.version.clone())
                .max_by(|a, b| version_triee(a).cmp(&version_triee(b)))
                .expect("au moins un paquet");
            let maintenant = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let m = Manifeste {
                serie: serie.parse().expect("série : entier"),
                expire: maintenant + jours.parse::<u64>().expect("jours : entier") * 86_400,
                voie: voie.to_string(),
                version,
                version_min_coop: min_coop.to_string(),
                paquets: flux
                    .assets
                    .into_iter()
                    .map(|a| Paquet {
                        id: a.package_id,
                        version: a.version,
                        genre: a.genre,
                        fichier: a.file_name,
                        sha1: a.sha1,
                        sha256: a.sha256.to_lowercase(),
                        taille: a.size,
                        notes: a.notes_markdown,
                    })
                    .collect(),
            };
            std::fs::write(dossier.join(NOM_MANIFESTE), signer(&m, &cle))
                .expect("écriture du manifeste");
            println!(
                "{} : version {}, série {}, {} paquets",
                NOM_MANIFESTE,
                m.version,
                m.serie,
                m.paquets.len()
            );
        }
        _ => {
            eprintln!(
                "usage : bs-signer cles | bs-signer manifeste <dossier> <voie> <série> <jours> <version min coop>"
            );
            std::process::exit(2);
        }
    }
}

/// « 1.2.3 » → (1, 2, 3) pour comparer (sans pré-version : le prototype n'en a pas).
fn version_triee(v: &str) -> Vec<u64> {
    v.split(['.', '-'])
        .map(|x| x.parse().unwrap_or(0))
        .collect()
}
