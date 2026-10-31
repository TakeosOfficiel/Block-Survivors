//! Pari 5 du prototype : un vrai `.bssave`, exporté par la version TypeScript 0.50.0
//! (scripts/exporter-bssave.mjs), se lit en Rust ; et un fichier écrit en Rust se relit.

use bs_sauvegarde::{exporter, importer};

const FICHIER: &str = include_str!("donnees/profil-0.50.0.bssave");

#[test]
fn lit_un_profil_exporte_par_la_version_typescript() {
    let p = importer(FICHIER).expect("sauvegarde lisible");
    assert_eq!(p.version.as_deref(), Some("0.50.0"));
    let profil: serde_json::Value = serde_json::from_str(&p.donnees["profil"]).unwrap();
    assert_eq!(profil["pseudo"], "Éloïse");
    let progression: serde_json::Value = serde_json::from_str(&p.donnees["progression"]).unwrap();
    assert_eq!(progression["records"]["prairie"], 412);
    assert_eq!(progression["parties"], 2);
}

#[test]
fn reecrit_et_relit_a_l_identique() {
    let p = importer(FICHIER).unwrap();
    assert_eq!(
        importer(&exporter(&p, "0.50.0", "2026-10-31T00:00:00Z")).unwrap(),
        p
    );
}
