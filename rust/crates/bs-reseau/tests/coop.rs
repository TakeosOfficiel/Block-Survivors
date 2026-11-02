//! Partie en coop de bout en bout contre un vrai relais local, sans fenêtre : l'hôte ouvre
//! un salon, un invité le rejoint (un autre, d'une autre version, est refusé), la partie
//! est lancée, l'invité marche chez l'hôte grâce à ses entrées, reçoit les instantanés, et
//! les cartes font l'aller-retour.

use std::time::{Duration, Instant};

use bs_commun::PAS_SIMULATION;
use bs_donnees::Donnees;
use bs_protocole::coop::{CarteJouee, Presentation};
use bs_regles::Partie;
use bs_reseau::{HoteCoop, InviteCoop, PhaseCoop, code_saisi};

fn donnees() -> &'static Donnees {
    Box::leak(Box::new(Donnees::embarquees().expect("données embarquées")))
}

fn relais() -> String {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    let ecoute = rt
        .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
        .expect("port");
    let adresse = ecoute.local_addr().expect("adresse");
    let reglages = bs_relais::Reglages {
        cloudflare: false,
        ..Default::default()
    };
    std::thread::spawn(move || {
        rt.block_on(bs_relais::servir_avec(ecoute, reglages)).ok();
    });
    format!("ws://{adresse}/salon")
}

fn presentation(d: &Donnees, pseudo: &str, personnage: usize, version: &str) -> Presentation {
    Presentation {
        version: version.into(),
        pseudo: pseudo.into(),
        personnage: d
            .personnages
            .personnages
            .get_index(personnage)
            .unwrap()
            .0
            .clone(),
        skin: "steve".into(),
        bras_fins: false,
        skin_importe: None,
        arbre: 0,
        talents: Vec::new(),
        armures: Vec::new(),
    }
}

/// Répète `f` jusqu'à ce qu'il rende vrai, 5 s au plus.
fn jusqua(mut f: impl FnMut() -> bool) {
    let fin = Instant::now() + Duration::from_secs(5);
    while !f() {
        assert!(Instant::now() < fin, "condition jamais remplie");
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn partie_en_coop() {
    let d = donnees();
    let adresse = relais();
    let mut hote = HoteCoop::nouveau(&adresse, presentation(d, "Hote", 0, "1.0.0"), false);
    jusqua(|| {
        hote.maj_salon();
        hote.phase == PhaseCoop::Salon
    });
    let code = code_saisi(&hote.code().expect("code")).expect("code lisible");

    // Un invité de la bonne version entre ; un autre, d'une autre version, est refusé
    let mut invite = InviteCoop::nouveau(&adresse, presentation(d, "Alex", 1, "1.0.0"), code);
    let mut ancien = InviteCoop::nouveau(&adresse, presentation(d, "Vieux", 0, "0.9.0"), code);
    jusqua(|| {
        hote.maj_salon();
        invite.maj_salon();
        ancien.maj_salon();
        invite.joueurs.len() == 2 && matches!(ancien.phase, PhaseCoop::Finie(_))
    });
    let PhaseCoop::Finie(raison) = &ancien.phase else {
        unreachable!()
    };
    assert!(raison.contains("Version"), "{raison}");
    assert_eq!(hote.joueurs.len(), 2);
    assert_eq!(invite.joueurs[1].presentation.pseudo, "Alex");

    // Lancement : l'invité prépare sa partie, l'hôte attend qu'il soit prêt
    let arene = d.arenes.arenes[0].id.clone();
    hote.lancer(&arene);
    jusqua(|| {
        invite.maj_salon();
        matches!(invite.phase, PhaseCoop::Chargement { .. })
    });
    let mut chez_invite = invite.preparer(d, 3).expect("arène connue");
    assert_eq!(chez_invite.nombre_joueurs(), 2);
    jusqua(|| hote.tous_prets(0.0));
    let mut chez_hote = Partie::nouvelle(d, 0, 0, 7, true);
    hote.demarrer(&mut chez_hote);
    assert_eq!(chez_hote.nombre_joueurs(), 2);
    jusqua(|| {
        invite.maj_salon();
        invite.phase == PhaseCoop::Partie
    });

    // L'invité marche : chez l'hôte, son combattant avance aussi
    let depart = (chez_hote.etat_joueur(1).x, chez_hote.etat_joueur(1).z);
    chez_invite.joueur.commandes.droite = true;
    for _ in 0..240 {
        chez_hote.ev.vider();
        hote.avant_pas(&mut chez_hote, PAS_SIMULATION);
        chez_hote.pas(PAS_SIMULATION);
        hote.apres_pas(&chez_hote, PAS_SIMULATION);
        chez_invite.ev.vider();
        invite.pas(&mut chez_invite, PAS_SIMULATION, false);
        std::thread::sleep(Duration::from_millis(1));
    }
    let j = chez_hote.etat_joueur(1);
    let parcouru = ((j.x - depart.0).powi(2) + (j.z - depart.1).powi(2)).sqrt();
    assert!(
        parcouru > 3.0,
        "l'invité a marché chez l'hôte ({parcouru} blocs)"
    );
    assert!(invite.recepteur.as_ref().is_some_and(|r| r.recu));
    assert!((chez_invite.etat.temps_jeu - chez_hote.etat.temps_jeu).abs() < 1.0);
    // Sa position prédite reste proche de la vraie
    let moi = chez_invite.etat_joueur(0);
    let ecart = ((moi.x - j.x).powi(2) + (moi.z - j.z).powi(2)).sqrt();
    assert!(ecart < 2.0, "prédiction à {ecart} blocs de l'hôte");

    // Cartes : celle de l'invité est jouée chez l'hôte, celle de l'hôte chez l'invité
    let arme = d.armes.get_index(1).unwrap().0.clone();
    chez_hote.equipe.membres[0].cartes_en_attente = 1;
    let (version_hote, version_invite) = (
        chez_hote.equipe.membres[0].arsenal.version,
        chez_invite.equipe.membres[0].arsenal.version,
    );
    invite.carte_locale(CarteJouee::Arme { arme: arme.clone() });
    hote.carte_locale(CarteJouee::Arme { arme });
    jusqua(|| {
        hote.avant_pas(&mut chez_hote, 0.0);
        invite.pas(&mut chez_invite, PAS_SIMULATION, true);
        chez_hote.equipe.membres[0].cartes_en_attente == 0
            && chez_invite.equipe.membres[0].arsenal.version != version_invite
    });
    assert_ne!(chez_hote.equipe.membres[0].arsenal.version, version_hote);

    // L'invité s'en va : l'hôte garde sa place, l'invité est immobile
    invite.quitter();
    jusqua(|| {
        hote.avant_pas(&mut chez_hote, PAS_SIMULATION);
        chez_hote.equipe.membres[0].joueur.commandes == Default::default()
    });
}
