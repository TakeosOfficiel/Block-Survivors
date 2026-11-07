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

#[test]
fn guerrier_defense() {
    use bs_donnees::progression::BonusTalents;
    let d = donnees();
    let mut p = Partie::nouvelle(d, 0, 0, 5, false);
    assert!(
        p.talents.classes.guerrier.is_some(),
        "personnage 0 = guerrier"
    );
    let b = BonusTalents {
        mecaniques: vec![
            ("inebranlable", 1),
            ("tenir_la_ligne", 2),
            ("peau_de_fer", 2),
            ("dernier_rempart", 1),
        ],
        ..Default::default()
    };
    p.appliquer_talents(&b);
    // Boss : −10 % ; immobile depuis 1 s : −20 %
    assert!((p.talent_degats_recus(10.0, true) - 9.0).abs() < 1e-4);
    p.talents.classes.temps_immobile = 1.0;
    assert!((p.talent_degats_recus(10.0, false) - 8.0).abs() < 1e-4);
    // Peau de fer : +2 d'armure sous la moitié des PV
    assert_eq!(p.talent_armure(), 0.0);
    p.etat.joueur.pv = p.etat.joueur.pv_max * 0.4;
    assert_eq!(p.talent_armure(), 2.0);
    // Dernier rempart : sauve une seule fois
    assert!(p.talent_eviter_mort());
    assert_eq!(p.etat.joueur.pv, 1.0);
    assert!(p.etat.joueur.protection >= 3.0);
    assert!(!p.talent_eviter_mort());
}

#[test]
fn archere_talents() {
    use bs_donnees::progression::BonusTalents;
    let d = donnees();
    let archere = d.personnages.personnages.get_index_of("archer").unwrap();
    let mut p = Partie::nouvelle(d, 0, archere, 7, false);
    assert!(p.talents.classes.archere.is_some());
    let max = p.talent_cumuls_poison();
    let b = BonusTalents {
        mecaniques: vec![
            ("pas_chasseur", 1),
            ("concentration", 2),
            ("venin", 2),
            ("venin_concentre", 1),
            ("contagion", 2),
            ("nuage_toxique", 1),
            ("marque_chasseur", 2),
            ("premier_sang", 2),
            ("oeil_aigle", 1),
            ("executrice", 1),
            ("puissance", 1),
            ("course_chargee", 2),
            ("perforation_nette", 2),
            ("visee_stable", 1),
        ],
        ..Default::default()
    };
    p.appliquer_talents(&b);
    // Venin concentré : 3 cumuls au plus
    assert_eq!(p.talent_cumuls_poison(), 3.0);
    assert_ne!(max, 3.0);
    // Pas du chasseur : +10 % après 2 s de marche
    assert_eq!(p.talent_vitesse(), 1.0);
    p.talents.classes.temps_mouvement = 2.0;
    assert!((p.talent_vitesse() - 1.1).abs() < 1e-6);
    // Concentration : armes de tir −15 % de délai à l'arrêt
    let arc = d.armes.get_index_of("arc").unwrap();
    p.talents.classes.temps_immobile = 1.0;
    assert!((p.talent_delai(arc) - 0.85).abs() < 1e-6);
    // Une minute de jeu avec tous ces talents : rien ne casse
    let pas = (60.0 / PAS_SIMULATION) as u32;
    for _ in 0..pas {
        p.pas(PAS_SIMULATION);
    }
    assert!(p.etat.stats.degats_par_arme[arc] > 0.0);
}

#[test]
fn mage_talents() {
    use bs_donnees::progression::BonusTalents;
    let d = donnees();
    let mage = d.personnages.personnages.get_index_of("mage").unwrap();
    let mut p = Partie::nouvelle(d, 0, mage, 11, false);
    assert!(p.talents.classes.mage.is_some());
    let b = BonusTalents {
        mecaniques: vec![
            ("coeur_blaze", 1),
            ("cendres_protectrices", 1),
            ("propagation", 2),
            ("brasier", 1),
            ("fournaise", 1),
            ("embrasement", 1),
            ("phenix", 1),
            ("resonance", 1),
            ("haute_tension", 2),
            ("conduction", 2),
            ("orbes_givrees", 1),
            ("flux_arcanique", 2),
            ("surcharge", 2),
            ("acceleration", 2),
            ("tempete_orage", 1),
            ("echo_arcanique", 1),
            ("zero_absolu", 1),
            ("lave_fusion", 1),
            ("tempete_feu", 1),
            ("pluie_feu", 1),
        ],
        ..Default::default()
    };
    p.appliquer_talents(&b);
    // Écho arcanique : le facteur de déclenchement vaut 1 hors répétition
    assert_eq!(p.talent_facteur_declenchement(), 1.0);
    // Une minute avec tous les sorts : rien ne casse, l'orage tombe
    p.joueur.invincible = true;
    for id in [
        "eclairs",
        "nova_glace",
        "meteore",
        "tornade_feu",
        "orbes_arcaniques",
    ] {
        p.ameliorer_arme(d.armes.get_index_of(id).unwrap());
    }
    for _ in 0..(60.0 / PAS_SIMULATION) as u32 {
        p.pas(PAS_SIMULATION);
    }
    let eclairs = d.armes.get_index_of("eclairs").unwrap();
    assert!(p.etat.stats.degats_par_arme[eclairs] > 0.0);
    // Phénix : sauve une seule fois, à 40 % des PV
    assert!(p.talent_eviter_mort());
    assert_eq!(p.etat.joueur.pv, (p.etat.joueur.pv_max * 0.4).ceil());
    assert!(!p.talent_eviter_mort());
}

/// Talents de l'Invocateur : alpha, Lune de sang (loups hors quota), crocs et ravageur
/// crédités aux loups, haie du Bosquet ; une minute et quelques sans rien casser.
#[test]
fn invocateur_talents() {
    use bs_donnees::progression::BonusTalents;
    let d = donnees();
    let invocateur = d
        .personnages
        .personnages
        .get_index_of("invocateur")
        .unwrap();
    let mut p = Partie::nouvelle(d, 0, invocateur, 13, false);
    assert!(p.talents.classes.invocateur.is_some());
    let b = BonusTalents {
        mecaniques: vec![
            ("chef_meute", 1),
            ("morsure_marquante", 2),
            ("reactivite", 2),
            ("meute_soudee", 2),
            ("traque", 1),
            ("curee", 1),
            ("hurlement", 2),
            ("encerclement", 2),
            ("braises_blaze", 2),
            ("coordination", 2),
            ("alpha", 2),
            ("ruee_meute", 1),
            ("lune_de_sang", 1),
            ("gardien_alpha", 1),
            ("lien_spirituel", 1),
            ("garde_rapprochee", 2),
            ("allay_chanteur", 1),
            ("essaim", 2),
            ("evocation_superieure", 1),
            ("crocs_evocateur", 1),
            ("bosquet", 1),
        ],
        ..Default::default()
    };
    p.appliquer_talents(&b);
    p.joueur.invincible = true;
    for id in [
        "loups",
        "vex",
        "esprit_blaze",
        "totem_racines",
        "esprit_protecteur",
    ] {
        p.ameliorer_arme(d.armes.get_index_of(id).unwrap());
    }
    let loup = d.allies.allies.get_index_of("loup").unwrap();
    let loups_vivants = |p: &Partie| {
        p.armes
            .invocations
            .pool
            .iter()
            .filter(|a| a.type_allie == loup && a.arme.is_some() && a.pv > 0.0)
            .count()
    };
    let mut alpha = false;
    let mut crocs = false;
    let mut ravageur = false;
    let mut max_loups = 0;
    for _ in 0..(62.0 / PAS_SIMULATION) as u32 {
        p.pas(PAS_SIMULATION);
        alpha |= p.armes.invocations.pool.iter().any(|a| a.alpha);
        let v = p.talents.classes.invocateur.as_ref().unwrap();
        crocs |= !v.crocs.is_empty();
        ravageur |= v.ravageur.actif;
        max_loups = max_loups.max(loups_vivants(&p));
    }
    assert!(alpha, "aucun loup alpha");
    assert!(crocs, "aucun croc");
    assert!(ravageur, "pas de ravageur");
    // Lune de sang : plus de loups que le quota normal (6)
    assert!(max_loups > 6, "loups au plus : {max_loups}");
    let loups = d.armes.get_index_of("loups").unwrap();
    assert!(p.etat.stats.degats_par_arme[loups] > 0.0);
}

/// Entouré d'un anneau de murets (franchissables à pied, mais qui arrêtent les tirs), le
/// joueur blesse moins avec la flèche à ricochets : elle s'arrête sur le muret (sans le
/// test du décor, les dégâts sont identiques avec et sans murets).
#[test]
fn ricochets_arretes_par_les_murs() {
    use bs_monde::BLOQUE_TIR;
    let d = donnees();
    let index = d
        .armes
        .get_index_of("fleche_ricochet")
        .expect("flèche à ricochets");
    let degats = |murets: bool| {
        let mut p = Partie::nouvelle(d, 0, 0, 11, false);
        p.joueur.invincible = true;
        p.arsenal.armes.clear();
        while p.ameliorer_arme(index) {}
        let c = &mut p.carte;
        for iz in 0..c.profondeur {
            for ix in 0..c.largeur {
                let (x, z) = (ix as f32 + 0.5 - c.demi_x, iz as f32 + 0.5 - c.demi_z);
                let r = (x * x + z * z).sqrt();
                if murets && (2.5..4.0).contains(&r) {
                    c.drapeaux[iz * c.largeur + ix] |= BLOQUE_TIR;
                }
            }
        }
        for _ in 0..(40.0 / PAS_SIMULATION) as u32 {
            p.pas(PAS_SIMULATION);
        }
        p.etat.stats.degats_par_arme[index]
    };
    let (libre, mure) = (degats(false), degats(true));
    eprintln!("ricochets : {libre:.0} sans murets, {mure:.0} avec");
    assert!(
        libre > 0.0 && mure < libre * 0.9,
        "les murets n'arrêtent pas les flèches"
    );
}

/// Un anneau de murets (qui arrêtent les tirs, pas la marche) protège les ennemis des coups
/// portés au sol : épée et bouclier font nettement moins de dégâts.
#[test]
fn coups_au_sol_arretes_par_les_murs() {
    use bs_monde::BLOQUE_TIR;
    let d = donnees();
    for (id, anneau) in [("epee", 1.5..2.5), ("heurt_bouclier", 1.5..2.5)] {
        let index = d.armes.get_index_of(id).expect("arme connue");
        let degats = |murets: bool| {
            let mut p = Partie::nouvelle(d, 0, 0, 11, false);
            p.joueur.invincible = true;
            p.arsenal.armes.clear();
            while p.ameliorer_arme(index) {}
            let c = &mut p.carte;
            for iz in 0..c.profondeur {
                for ix in 0..c.largeur {
                    let (x, z) = (ix as f32 + 0.5 - c.demi_x, iz as f32 + 0.5 - c.demi_z);
                    let r = (x * x + z * z).sqrt();
                    if murets && anneau.contains(&r) {
                        c.drapeaux[iz * c.largeur + ix] |= BLOQUE_TIR;
                    }
                }
            }
            for _ in 0..(40.0 / PAS_SIMULATION) as u32 {
                p.pas(PAS_SIMULATION);
            }
            p.etat.stats.degats_par_arme[index]
        };
        let (libre, mure) = (degats(false), degats(true));
        eprintln!("{id} : {libre:.0} sans murets, {mure:.0} avec");
        assert!(
            libre > 0.0 && mure < libre * 0.95,
            "les murets n'arrêtent pas {id}"
        );
    }
}

/// Une explosion au sol épargne l'ennemi caché derrière un mur plein, pas celui à découvert.
#[test]
fn explosion_arretee_par_un_mur() {
    use bs_monde::BLOQUE_TIR;
    let d = donnees();
    let mut p = Partie::nouvelle(d, 0, 0, 11, false);
    let zombie = d.ennemis.get_index_of("zombie").expect("zombie");
    assert!(p.apparaitre(zombie, 3.0, 0.0, 1.0));
    assert!(p.apparaitre(zombie, -3.0, 0.0, 1.0));
    let c = &mut p.carte;
    for iz in 0..c.profondeur {
        let ix = (1.5 + c.demi_x) as usize;
        c.drapeaux[iz * c.largeur + ix] |= BLOQUE_TIR;
    }
    p.ennemis.reconstruire_grille();
    let pv: Vec<f32> = p.ennemis.actifs().iter().map(|e| e.pv).collect();
    p.blesser_zone(0.0, 0.0, 5.0, 5.0, 0.0, 0);
    let apres: Vec<f32> = p.ennemis.actifs().iter().map(|e| e.pv).collect();
    let (cache, decouvert) = if p.ennemis.actifs()[0].x > 0.0 {
        (0, 1)
    } else {
        (1, 0)
    };
    assert_eq!(apres[cache], pv[cache], "le mur ne protège pas");
    assert!(
        apres[decouvert] < pv[decouvert],
        "l'ennemi à découvert n'est pas touché"
    );
}
