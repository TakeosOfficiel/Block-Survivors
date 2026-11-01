//! Essais de la partie complète : apparitions, déplacement, coups reçus.

use bs_commun::PAS_SIMULATION;
use bs_donnees::Donnees;
use bs_regles::Partie;
use bs_regles::etat::Issue;

fn donnees() -> &'static Donnees {
    Box::leak(Box::new(Donnees::embarquees().expect("données embarquées")))
}

#[test]
fn une_minute_immobile() {
    let d = donnees();
    let mut p = Partie::nouvelle(d, 0, 0, 1234, false);
    let pas = (60.0 / PAS_SIMULATION) as u32;
    for _ in 0..pas {
        p.pas(PAS_SIMULATION);
    }
    eprintln!(
        "t = {:.1} s, ennemis : {}, PV : {:.1}/{}, subis : {:.1}, issue : {:?}",
        p.etat.temps_jeu,
        p.ennemis.nombre,
        p.etat.joueur.pv,
        p.etat.joueur.pv_max,
        p.etat.stats.degats_subis,
        p.etat.issue
    );
    assert!(p.ennemis.nombre > 0, "des ennemis doivent être apparus");
    assert!(p.etat.stats.degats_subis > 0.0 || p.etat.issue == Some(Issue::Defaite));
}

#[test]
fn deterministe() {
    let d = donnees();
    let mut a = Partie::nouvelle(d, 0, 0, 99, false);
    let mut b = Partie::nouvelle(d, 0, 0, 99, false);
    a.joueur.invincible = true;
    b.joueur.invincible = true;
    for _ in 0..(30.0 / PAS_SIMULATION) as u32 {
        a.pas(PAS_SIMULATION);
        b.pas(PAS_SIMULATION);
    }
    assert_eq!(a.ennemis.nombre, b.ennemis.nombre);
    let pos = |p: &Partie| {
        p.ennemis
            .actifs()
            .iter()
            .map(|e| (e.x.to_bits(), e.z.to_bits()))
            .collect::<Vec<_>>()
    };
    assert_eq!(pos(&a), pos(&b));
}

/// Chaque personnage, immobile, frappe avec son arme de départ.
#[test]
fn armes_de_depart() {
    let d = donnees();
    for (index, (_, perso)) in d.personnages.personnages.iter().enumerate() {
        let mut p = Partie::nouvelle(d, 0, index, 7, false);
        p.joueur.invincible = true;
        for _ in 0..(60.0 / PAS_SIMULATION) as u32 {
            p.pas(PAS_SIMULATION);
        }
        let degats: f64 = p.etat.stats.degats_par_arme.iter().sum();
        eprintln!(
            "{:?} {:?} : tués {}, dégâts {:.0}",
            perso.armes_depart,
            p.arsenal.armes.len(),
            p.etat.ennemis_tues,
            degats
        );
        assert_eq!(p.arsenal.armes.len(), perso.armes_depart.len());
    }
}

/// Chaque arme (hors invocations), seule au rang maximal, blesse des ennemis en une minute.
#[test]
fn chaque_arme_frappe() {
    use bs_donnees::armes::Mecanique;
    let d = donnees();
    for (index, (id, arme)) in d.armes.iter().enumerate() {
        // Les alliés de soutien protègent le joueur sans blesser
        let soutien = match &arme.mecanique {
            Mecanique::Invocation { allie, .. } => d.allies.allies[allie].soutien,
            _ => false,
        };
        let mut p = Partie::nouvelle(d, 0, 0, 3, false);
        p.joueur.invincible = true;
        p.arsenal.armes.clear();
        while p.ameliorer_arme(index) {}
        for _ in 0..(60.0 / PAS_SIMULATION) as u32 {
            p.pas(PAS_SIMULATION);
        }
        let degats = p.etat.stats.degats_par_arme[index];
        eprintln!("{id} : tués {}, dégâts {degats:.0}", p.etat.ennemis_tues);
        if soutien {
            assert!(
                !p.armes.invocations.pool.is_empty(),
                "{id} n'a rien invoqué"
            );
        } else {
            assert!(degats > 0.0, "{id} n'a rien blessé");
        }
    }
}

/// Cinq minutes en choisissant toujours la première carte : niveaux, éveils et coffres.
#[test]
fn cartes_et_coffres() {
    use bs_regles::equipement::ChoixEquipement;
    let d = donnees();
    let mut p = Partie::nouvelle(d, 0, 0, 2024, false);
    p.joueur.invincible = true;
    let (mut niveaux, mut objets) = (0, 0);
    for _ in 0..(300.0 / PAS_SIMULATION) as u32 {
        p.pas(PAS_SIMULATION);
        while p.etat.niveaux_en_attente > 0 {
            p.etat.niveaux_en_attente -= 1;
            let carte = p.tirer_cartes()[0];
            for a in p.cartes.proposees.clone() {
                verifier(&p.decrire_amelioration(a));
            }
            p.appliquer_amelioration(carte);
            niveaux += 1;
        }
        while p.etat.choix_coffre.restants > 0 {
            let ender = p.etat.choix_coffre.ender > 0;
            let ouverts = p.ramassables.coffres.payants_ouverts;
            let mut choix = p.tirer_equipement(ender, ouverts)[0];
            for c in p.equipement.proposees.clone() {
                verifier(&p.decrire_equipement(c));
            }
            if let ChoixEquipement::Objet(o) = choix
                && p.remplacement_necessaire(o)
            {
                choix = p.cartes_remplacement(o)[0];
                for c in p.equipement.proposees.clone() {
                    verifier(&p.decrire_equipement(c));
                }
            }
            p.appliquer_equipement(choix);
            objets += 1;
            p.etat.choix_coffre.restants -= 1;
            if ender {
                p.etat.choix_coffre.ender -= 1;
            }
        }
    }
    let armes: Vec<_> = p.arsenal.armes.iter().map(|a| (a.index, a.rang)).collect();
    let portes: Vec<_> = p.equipement.portes().collect();
    eprintln!("niveaux : {niveaux}, coffres : {objets}, armes : {armes:?}, objets : {portes:?}");
    eprintln!(
        "passifs : {:?}, tués : {}",
        p.passifs.possedes.len(),
        p.etat.ennemis_tues
    );
    assert!(niveaux > 5, "le joueur doit monter de niveau");
    assert!(p.arsenal.armes.len() > 1 || p.passifs.possedes.len() > 1);
}

/// Une carte décrite a un nom, une étiquette et une description.
fn verifier(c: &bs_regles::textes::Carte) {
    assert!(!c.nom.is_empty() && !c.etiquette.is_empty(), "{c:?}");
    assert!(!c.description.is_empty(), "{c:?}");
}

/// Talents de l'arbre actif : PV max, relances, rangs d'armes offerts, bonus d'arme.
#[test]
fn talents_appliques() {
    use bs_donnees::progression::{BonusArmeTalent, BonusTalents};
    let d = donnees();
    let mut p = Partie::nouvelle(d, 0, 0, 5, false);
    let epee = d.armes.get_index_of("epee").unwrap();
    let (pv, relances, rang) = (
        p.etat.joueur.pv_max,
        p.etat.relances_restantes,
        p.arsenal.rang(epee),
    );
    let degats = p
        .arsenal
        .armes
        .iter()
        .find(|a| a.index == epee)
        .map(|a| a.stats.degats);
    let bonus_arme: &'static BonusArmeTalent = Box::leak(Box::new(BonusArmeTalent {
        arme: "epee".into(),
        ajout: Some(bs_donnees::ModifArme {
            degats: 2.0,
            ..Default::default()
        }),
        part: None,
        pv: None,
        vitesse: None,
    }));
    let mut b = BonusTalents {
        relances: 2,
        armes: vec!["epee"],
        bonus_armes: vec![bonus_arme],
        mecaniques: vec![("enchainement", 1)],
        ..Default::default()
    };
    b.modif.pv_max = 4.0;
    p.appliquer_talents(&b);
    assert_eq!(p.etat.joueur.pv_max, pv + 4.0);
    assert_eq!(p.etat.joueur.pv, p.etat.joueur.pv_max);
    assert_eq!(p.etat.relances_restantes, relances + 2);
    assert_eq!(p.arsenal.rang(epee), rang + 1);
    assert_eq!(p.talents.rang("enchainement"), 1);
    let apres = p.arsenal.armes.iter().find(|a| a.index == epee).unwrap();
    assert_eq!(apres.talent.ajout.degats, 2.0);
    if let Some(degats) = degats {
        assert!(apres.stats.degats > degats);
    }
}
