//! Progression lue d'un vrai profil TypeScript importé, fin de partie (maîtrise, records,
//! déblocage) et achat de talents, relus après écriture.

use bs_donnees::Donnees;
use bs_donnees::progression::TypeDeblocage;
use bs_sauvegarde::{Bilan, Profils, Progression, RefusAchat, Stockage};

const FICHIER: &str = include_str!("donnees/profil-0.50.0.bssave");

fn donnees() -> &'static Donnees {
    Box::leak(Box::new(Donnees::embarquees().unwrap()))
}

fn stockage(nom: &str) -> Stockage {
    let d = std::env::temp_dir().join(format!("bs-progression-{nom}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    Stockage::nouveau(d)
}

#[test]
fn profil_importe_puis_partie() {
    let d = donnees();
    let mut profils = Profils::charger(stockage("importe"));
    profils.importer(FICHIER).unwrap();
    let cle = profils.cle("progression");
    let p = Progression::charger(d, profils.stockage().clone(), cle.clone());
    assert_eq!(p.record("prairie"), 412.0);
    assert_eq!(p.parties(), 2);
    assert!(p.verrou(TypeDeblocage::Arene, "mine").is_none());
}

#[test]
fn fin_de_partie_et_talents() {
    let d = donnees();
    let s = stockage("partie");
    let cle = "block-survivors.p.1.progression".to_owned();
    let mut p = Progression::charger(d, s.clone(), cle.clone());
    assert!(p.verrou(TypeDeblocage::Arene, "mine").is_some());
    let (perso, _) = d.personnages.personnages.get_index(0).unwrap();
    let bilan = Bilan {
        personnage: perso,
        arene: "prairie",
        temps: 320.5,
        xp: 1e6,
        boss: 1,
        sans_fin: false,
        en_equipe: false,
    };
    let r = p.terminer_partie(&bilan);
    assert_eq!(r.xp, 1_000_000);
    assert!(r.apres.niveau > r.avant.niveau);
    let mine = d.talents.deblocages.iter().position(|b| b.id == "mine");
    assert_eq!(r.deblocages, vec![mine.unwrap()]);

    // Un talent racine s'achète jusqu'à son rang maximal
    let racine = d.talents.arbres[perso][0]
        .noeuds
        .iter()
        .find(|n| n.prerequis.is_empty() && n.ligne == 0)
        .unwrap();
    for _ in 0..racine.rang_max {
        assert!(p.acheter(perso, 0, racine));
    }
    assert_eq!(p.refus_achat(perso, 0, racine), Some(RefusAchat::Max));

    // Suite en mode sans fin : record sans fin, pas de partie de plus
    let suite = Bilan {
        temps: 700.0,
        xp: 10.0,
        sans_fin: true,
        ..bilan
    };
    assert!(p.terminer_partie(&suite).record_sans_fin);

    let relu = Progression::charger(d, s.clone(), cle.clone());
    assert_eq!(relu.rang(perso, 0, racine), racine.rang_max);
    assert_eq!((relu.record("prairie"), relu.parties()), (320.5, 1));
    assert_eq!(relu.record_sans_fin("prairie"), 700.0);
    let json: serde_json::Value = s.lire(&cle).unwrap();
    assert_eq!(json["version"], 3);
    assert_eq!(json["parties"], serde_json::json!(1));
}
