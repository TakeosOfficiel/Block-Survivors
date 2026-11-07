//! Essais de la coop en jeu : une partie hôte à deux joueurs envoie ses instantanés à la
//! partie de l'invité, qui retrouve l'équipe, les ennemis, le butin et le niveau commun ;
//! les entrées et les messages fiables font l'aller-retour.

use bs_commun::PAS_SIMULATION;
use bs_donnees::Donnees;
use bs_protocole::coop::*;
use bs_regles::Partie;
use bs_regles::joueur::Commandes;

fn donnees() -> &'static Donnees {
    Box::leak(Box::new(Donnees::embarquees().expect("données embarquées")))
}

#[test]
fn entree_aller_retour() {
    let c = Commandes {
        haut: true,
        droite: true,
        lacet: 1.25,
        ..Commandes::default()
    };
    let mut octets = Vec::new();
    ecrire_entree(42, &c, &mut octets);
    let (numero, lu) = lire_entree(&octets).expect("entrée lisible");
    assert_eq!(numero, 42);
    assert_eq!(bits(&lu), bits(&c));
    assert!((lu.lacet - 1.25).abs() < 1e-6);
    assert!(lire_entree(&octets[..3]).is_none());
}

#[test]
fn instantane_vers_l_invite() {
    let d = donnees();
    // Hôte : lui (place 0) et l'invité (place 1)
    let mut hote = Partie::nouvelle(d, 0, 0, 7, false);
    hote.ajouter_combattant(1);
    for _ in 0..(40.0 / PAS_SIMULATION) as usize {
        hote.simuler(PAS_SIMULATION);
    }
    assert!(hote.ennemis.nombre > 0, "des ennemis au bout de 40 s");

    // Invité : lui-même (joueur 0, personnage 1), l'hôte (joueur 1)
    let mut invite = Partie::nouvelle(d, 0, 1, 7, true);
    invite.ajouter_combattant(0);
    invite.invite = true;
    let mut recepteur = Recepteur::nouveau(&[(1, 0), (0, 1)]);

    let chiffres = [Chiffre {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        valeur: 12.0,
        style: STYLES[1],
    }];
    let mut octets = Vec::new();
    let entete = Entete {
        numero: 1,
        rangs: &[0, 1],
        entrees: &[0, 0],
    };
    ecrire_instantane_partie(&hote, &entete, &chiffres, &mut octets);
    assert!(
        recepteur.appliquer(&mut invite, &octets),
        "instantané lisible"
    );
    // Un instantané plus ancien est ignoré
    assert!(!recepteur.appliquer(&mut invite, &octets));

    let (moi, chez_hote) = (invite.etat_joueur(0), hote.etat_joueur(1));
    assert!((moi.x - chez_hote.x).abs() < 0.5 && (moi.z - chez_hote.z).abs() < 0.5);
    assert_eq!(moi.pv, chez_hote.pv);
    assert_eq!(invite.etat_joueur(1).pv_max, hote.etat_joueur(0).pv_max);
    assert_eq!(invite.etat.temps_jeu, hote.etat.temps_jeu);
    assert_eq!(invite.etat.ennemis_tues, hote.etat.ennemis_tues);
    assert_eq!(invite.etat.joueur.niveau, hote.etat.joueur.niveau);
    assert!(invite.ennemis.nombre > 0 && invite.ennemis.nombre <= hote.ennemis.nombre);
    let premier = &invite.ennemis.pool[0];
    let source = hote
        .ennemis
        .actifs()
        .iter()
        .find(|e| e.id == premier.id)
        .expect("même ennemi");
    assert_eq!(premier.index_type, source.index_type);
    assert_eq!(premier.pv, source.pv);
    assert!(invite.ramassables.orbes.len() <= hote.ramassables.orbes.len());

    // L'invité joue en invité : ses armes tournent, les dégâts restent chez l'hôte
    let pv = invite.ennemis.pool[0].pv;
    for _ in 0..30 {
        invite.simuler_invite(PAS_SIMULATION);
        recepteur.interpoler(&mut invite, PAS_SIMULATION);
    }
    assert_eq!(invite.ennemis.pool[0].pv, pv);

    // Les instantanés suivants s'appliquent
    hote.simuler(PAS_SIMULATION);
    let entete = Entete {
        numero: 2,
        ..entete
    };
    ecrire_instantane_partie(&hote, &entete, &[], &mut octets);
    assert!(recepteur.appliquer(&mut invite, &octets));
    // Octets tronqués : refusés sans paniquer
    let mut invite2 = Partie::nouvelle(d, 0, 1, 7, false);
    invite2.ajouter_combattant(0);
    let mut r2 = Recepteur::nouveau(&[(1, 0), (0, 1)]);
    for n in [0, 1, 9, octets.len() / 2] {
        let _ = r2.appliquer(&mut invite2, &octets[..n]);
    }
}

#[test]
fn messages_fiables() {
    let d = donnees();
    let presentation = Presentation {
        version: "1.0.0".into(),
        pseudo: "Steve".into(),
        personnage: d.personnages.personnages.get_index(0).unwrap().0.clone(),
        skin: "steve".into(),
        bras_fins: false,
        skin_importe: None,
        arbre: 0,
        talents: Vec::new(),
        armures: Vec::new(),
    };
    let messages = [
        MessageFiable::Bonjour(presentation.clone()),
        MessageFiable::Salon {
            arene: "plaine".into(),
            joueurs: vec![InfoJoueur {
                index: 0,
                presentation,
            }],
        },
        MessageFiable::Carte {
            carte: CarteJouee::Soin,
        },
        MessageFiable::Coffre {
            nombre: 2,
            ender: 1,
        },
        MessageFiable::Pret,
    ];
    for m in messages {
        assert_eq!(MessageFiable::lire(&m.json()), Some(m));
    }
    assert_eq!(MessageFiable::lire(b"{\"type\":\"inconnu\"}"), None);

    // Une carte jouée chez l'hôte ou l'invité donne le même effet
    let mut p = Partie::nouvelle(d, 0, 0, 3, false);
    let avant = p.arsenal.armes.len();
    let (id, _) = d
        .armes
        .iter()
        .find(|(_, a)| p.arsenal.armes.iter().all(|x| x.def.nom != a.nom))
        .expect("une arme pas encore prise");
    let carte = CarteJouee::Arme { arme: id.clone() };
    let lue: CarteJouee = serde_json::from_slice(&serde_json::to_vec(&carte).unwrap()).unwrap();
    assert!(lue.jouer(&mut p));
    assert_eq!(p.arsenal.armes.len(), avant + 1);
    assert!(!CarteJouee::Arme { arme: "???".into() }.jouer(&mut p));
}
