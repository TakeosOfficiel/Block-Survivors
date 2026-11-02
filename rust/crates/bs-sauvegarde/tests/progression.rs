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
        joueurs: 1,
        meme_classe: false,
        victoire: false,
    };
    let r = p.terminer_partie(&bilan);
    assert_eq!(r.xp, 1_000_000);
    assert!(r.apres.niveau > r.avant.niveau);
    let mine = d.talents.deblocages.iter().position(|b| b.id == "mine");
    assert_eq!(r.deblocages, vec![mine.unwrap()]);
    assert!(r.succes.is_empty(), "seul, aucun succès de coop");

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

    // Effets en jeu de l'arbre actif (guerrier : Enchaînement, Lame affûtée, Maître d'armes)
    let noeud = |id| {
        let noeuds = &d.talents.arbres[perso][0].noeuds;
        noeuds.iter().find(|n| n.id == id).unwrap()
    };
    for id in ["lame", "lame", "lame", "maitre", "maitre"] {
        assert!(p.acheter(perso, 0, noeud(id)), "{id}");
    }
    let b = p.bonus(perso);
    assert_eq!(b.mecaniques, vec![("enchainement", 1)]);
    assert!((b.modif.degats_melee - 0.12).abs() < 1e-6);
    assert_eq!(b.armes, ["epee", "epee"]);
    assert_eq!(p.bonus("inconnu").armes.len(), 0);

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

#[test]
fn succes_de_coop() {
    let d = donnees();
    let s = stockage("succes");
    let cle = "block-survivors.p.1.progression".to_owned();
    let mut p = Progression::charger(d, s.clone(), cle.clone());
    let (perso, _) = d.personnages.personnages.get_index(0).unwrap();
    let index = |id: &str| d.talents.succes.iter().position(|s| s.id == id).unwrap();
    let a_deux = Bilan {
        personnage: perso,
        arene: "prairie",
        temps: 900.0,
        xp: 10.0,
        boss: 1,
        sans_fin: false,
        joueurs: 2,
        meme_classe: false,
        victoire: true,
    };
    let r = p.terminer_partie(&a_deux);
    let mut attendus = vec![
        index("ensemble"),
        index("chasse_groupe"),
        index("tenir_bon"),
    ];
    attendus.sort_unstable();
    assert_eq!(r.succes, attendus);
    // À plusieurs, les records du solo ne bougent pas
    assert_eq!((p.record("prairie"), p.parties()), (0.0, 0));
    // Un succès ne se gagne qu'une fois
    assert!(p.terminer_partie(&a_deux).succes.is_empty());
    let a_quatre = Bilan {
        joueurs: 4,
        meme_classe: true,
        victoire: false,
        ..a_deux
    };
    let r = p.terminer_partie(&a_quatre);
    assert_eq!(r.succes, vec![index("meme_uniforme")]);
    let relu = Progression::charger(d, s, cle);
    assert!(relu.succes("ensemble") && relu.succes("meme_uniforme"));
    assert!(!relu.succes("escouade"));
}
