//! Lance le robot sur N graines par personnage et par arène, écrit les bilans et les
//! statistiques, puis les compare à celles de la version TypeScript.
//!
//!   cargo run --release -p bs-robot
//!   GRAINES=10 PERSONNAGES=guerrier,archer INVINCIBLE=1 cargo run --release -p bs-robot
//!
//! Variables : GRAINES (5), PERSONNAGES (guerrier,archer,mage,invocateur), ARENES
//! (prairie), DUREE (1200 s de jeu), INVINCIBLE=1, REFERENCE (dossier des références
//! TS : docs/robot, ou docs/robot-invincible), SORTIE (dossier où écrire les bilans
//! Rust, rien par défaut), VERIFIER=1 (échoue si une mesure sort de la tolérance).

use std::sync::Mutex;
use std::time::Instant;

use bs_donnees::Donnees;
use bs_regles::Partie;
use bs_robot::statistiques::{FichierReference, comparer, references};
use bs_robot::{Bilan, Robot};

fn variable(nom: &str, defaut: &str) -> String {
    std::env::var(nom).unwrap_or_else(|_| defaut.into())
}

fn main() {
    let graines: u32 = variable("GRAINES", "5").parse().expect("GRAINES");
    let personnages = variable("PERSONNAGES", "guerrier,archer,mage,invocateur");
    let arenes = variable("ARENES", "prairie");
    let duree: f64 = variable("DUREE", "1200").parse().expect("DUREE");
    let invincible = variable("INVINCIBLE", "") == "1";
    let racine = concat!(env!("CARGO_MANIFEST_DIR"), "/../../..");
    let defaut = if invincible {
        "docs/robot-invincible"
    } else {
        "docs/robot"
    };
    let dossier_ref = variable("REFERENCE", &format!("{racine}/{defaut}"));
    let d: &'static Donnees = Box::leak(Box::new(Donnees::embarquees().expect("données")));

    let mut taches = Vec::new();
    for arene in arenes.split(',') {
        let a = d
            .arenes
            .arenes
            .iter()
            .position(|x| x.id == arene)
            .expect("arène inconnue");
        for perso in personnages.split(',') {
            let p = d
                .personnages
                .personnages
                .get_index_of(perso)
                .expect("personnage inconnu");
            for g in 1..=graines {
                taches.push((a, p, g));
            }
        }
    }
    let debut = Instant::now();
    let file = Mutex::new(taches);
    let bilans = Mutex::new(Vec::new());
    let fils = std::thread::available_parallelism().map_or(4, |n| n.get());
    std::thread::scope(|s| {
        for _ in 0..fils {
            s.spawn(|| {
                loop {
                    let Some((a, p, g)) = file.lock().expect("file").pop() else {
                        break;
                    };
                    let mut partie = Partie::nouvelle(d, a, p, g, false);
                    let t = Instant::now();
                    let b = Robot::default().jouer(&mut partie, g, duree, invincible);
                    println!(
                        "{} / {} / graine {} : {} à {} s, niveau {}, {} tués ({:.1} s de calcul)",
                        b.personnage,
                        b.arene,
                        g,
                        b.issue,
                        b.temps,
                        b.niveau,
                        b.tues,
                        t.elapsed().as_secs_f32()
                    );
                    bilans.lock().expect("bilans").push(b);
                }
            });
        }
    });
    let mut bilans: Vec<Bilan> = bilans.into_inner().expect("bilans");
    bilans.sort_by(|a, b| (&a.personnage, a.graine).cmp(&(&b.personnage, b.graine)));
    println!(
        "{} parties en {:.1} s",
        bilans.len(),
        debut.elapsed().as_secs_f32()
    );
    let rust = references(&bilans);

    if let Ok(sortie) = std::env::var("SORTIE") {
        std::fs::create_dir_all(&sortie).expect("dossier de sortie");
        let fichier = FichierReference {
            graines,
            duree,
            invincible,
            reference: rust.clone(),
        };
        let json = |v: &dyn erased::Json| v.json();
        std::fs::write(format!("{sortie}/parties.json"), json(&bilans)).expect("écriture");
        std::fs::write(format!("{sortie}/reference.json"), json(&fichier)).expect("écriture");
    }

    let texte = std::fs::read_to_string(format!("{dossier_ref}/reference.json"))
        .expect("référence TS introuvable");
    let ts: FichierReference = serde_json::from_str(&texte).expect("référence TS illisible");
    let mut hors = 0;
    for (cle, r) in &rust {
        let Some(t) = ts.reference.get(cle) else {
            println!("\n{cle} : pas de référence TS");
            continue;
        };
        println!("\n{cle} ({} parties TS, {} Rust)", t.parties, r.parties);
        for e in comparer(t, r) {
            let marque = if e.ok { "  " } else { "!!" };
            hors += usize::from(!e.ok);
            println!(
                "{marque} {:<28} TS {:>8.1}   Rust {:>8.1}",
                e.mesure, e.ts, e.rust
            );
        }
    }
    println!("\n{hors} mesure(s) hors tolérance");
    if hors > 0 && variable("VERIFIER", "") == "1" {
        std::process::exit(1);
    }
}

mod erased {
    /// Sérialisation JSON indentée (une espace, comme le script TS).
    pub trait Json {
        fn json(&self) -> String;
    }
    impl<T: serde::Serialize> Json for T {
        fn json(&self) -> String {
            let mut sortie = Vec::new();
            let format = serde_json::ser::PrettyFormatter::with_indent(b" ");
            let mut s = serde_json::Serializer::with_formatter(&mut sortie, format);
            self.serialize(&mut s).expect("JSON");
            String::from_utf8(sortie).expect("UTF-8") + "\n"
        }
    }
}
