//! Les données de rust/donnees se chargent et se valident ; les erreurs disent
//! où corriger (fichier, ligne, entrée).

use std::borrow::Cow;

use bs_donnees::{Donnees, ErreurDonnees};

#[test]
fn les_donnees_se_chargent() {
    let d = Donnees::embarquees().unwrap_or_else(|e| panic!("{e}"));
    assert!(d.armes.len() >= 20, "{} armes", d.armes.len());
    assert!(d.ennemis.contains_key("zombie"));
    assert_eq!(d.personnages.par_defaut, "guerrier");
    assert_eq!(d.arenes.par_defaut, "prairie");
    // L'ordre du fichier est gardé (listes du jeu, tirages)
    assert_eq!(d.armes.keys().next().map(String::as_str), Some("epee"));
}

#[test]
fn le_dossier_et_l_embarque_sont_identiques() {
    let dossier = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../donnees");
    assert_eq!(
        Donnees::charger(&dossier).unwrap(),
        Donnees::embarquees().unwrap()
    );
}

#[test]
fn regles_reprises_de_la_version_typescript() {
    let d = Donnees::embarquees().unwrap();
    // talents.ts : xpPourNiveau, niveauMaitrise
    let m = &d.talents.maitrise;
    assert_eq!(m.xp_pour_niveau(1), 300);
    assert_eq!(m.xp_pour_niveau(2), 675);
    assert_eq!(m.niveau(674), 1);
    assert_eq!(m.niveau(675), 2);
    // blocs.ts : bloqueLePassage
    assert!(d.blocs["stone"].bloque_le_passage());
    assert!(d.blocs["water"].bloque_le_passage());
    // Dernier palier d'XP sans fin (Infinity en TypeScript)
    assert_eq!(
        d.regles.progression.paliers.last().unwrap().jusqu_au_niveau,
        None
    );
}

/// Les données embarquées, avec un remplacement de texte dans un fichier.
fn avec(fichier: &'static str, avant: &str, apres: &str) -> Result<Donnees, ErreurDonnees> {
    let dossier = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../donnees");
    Donnees::depuis(|nom| {
        let texte = std::fs::read_to_string(dossier.join(format!("{nom}.ron"))).unwrap();
        if nom == fichier {
            assert!(texte.contains(avant), "{avant} absent de {fichier}");
            Ok(Cow::Owned(texte.replacen(avant, apres, 1)))
        } else {
            Ok(Cow::Owned(texte))
        }
    })
}

#[test]
fn reference_inconnue_signalee() {
    let e = avec(
        "armes",
        "projectile: \"fleche\"",
        "projectile: \"flechette\"",
    )
    .unwrap_err();
    let texte = e.to_string();
    assert!(
        texte.contains("armes.ron, arc : projectile inconnu(e) « flechette »"),
        "{texte}"
    );
}

#[test]
fn champ_inconnu_signale_avec_sa_ligne() {
    let e = avec("ennemis", "    pv: 12.0,", "    pv: 12.0,\n    vie: 3.0,").unwrap_err();
    match e {
        ErreurDonnees::Format {
            fichier,
            ligne,
            message,
            ..
        } => {
            assert_eq!(fichier, "ennemis.ron");
            assert!(ligne > 3);
            assert!(message.contains("vie"), "{message}");
        }
        autre => panic!("{autre}"),
    }
}
