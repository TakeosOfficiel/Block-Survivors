//! Embarque les maps Minecraft du dossier cartes/ (à la racine du dépôt) : chaque
//! .schem, .litematic, .nbt ou .mcstructure devient une arène, réglée par le .json du même nom s'il existe.

use std::fmt::Write;
use std::path::Path;

fn main() {
    let dossier = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../cartes");
    println!("cargo:rerun-if-changed={}", dossier.display());
    let mut fichiers: Vec<_> = std::fs::read_dir(&dossier)
        .map(|d| d.filter_map(Result::ok).map(|e| e.path()).collect())
        .unwrap_or_default();
    fichiers.sort();
    let mut code = String::from("&[\n");
    for f in &fichiers {
        let Some(ext) = f.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !["schem", "litematic", "nbt", "mcstructure"].contains(&ext) {
            continue;
        }
        let id = f
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("nom de carte");
        let fichier = f.canonicalize().expect("chemin de carte");
        let reglages = f.with_extension("json");
        let reglages = if reglages.exists() {
            format!(
                "Some(include_str!({:?}))",
                reglages
                    .canonicalize()
                    .expect("chemin")
                    .display()
                    .to_string()
            )
        } else {
            "None".into()
        };
        writeln!(
            code,
            "    Carte {{ id: {id:?}, fichier: include_bytes!({:?}), reglages: {reglages} }},",
            fichier.display().to_string()
        )
        .expect("écriture");
    }
    code.push(']');
    let sortie = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("cartes.rs");
    std::fs::write(sortie, code).expect("écriture de cartes.rs");
}
