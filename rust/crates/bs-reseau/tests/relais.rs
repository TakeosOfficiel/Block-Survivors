//! Essais de bout en bout contre un vrai relais local : un hôte ouvre un salon public, un
//! invité le trouve dans la liste et le rejoint, ils échangent des octets ; l'invité coupé
//! reprend sa place ; un code inconnu est refusé ; le départ de l'hôte ferme le salon.

use std::time::{Duration, Instant};

use bs_reseau::{EvenementSession as E, Session, code_lisible, code_saisi};

/// Lance un relais sur un port libre ; son adresse WebSocket.
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

/// Attend l'évènement voulu (les autres sont ignorés), 5 s au plus.
fn attendre(s: &mut Session, voulu: impl Fn(&E) -> bool) -> E {
    let fin = Instant::now() + Duration::from_secs(5);
    while Instant::now() < fin {
        while let Some(e) = s.maj() {
            if voulu(&e) {
                return e;
            }
            assert!(!matches!(e, E::Perdue(_)), "session perdue : {e:?}");
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    panic!("évènement attendu non reçu");
}

#[test]
fn salon_de_bout_en_bout() {
    let adresse = relais();
    let mut hote = Session::ouvrir(&adresse);
    attendre(&mut hote, |e| *e == E::Connectee);
    hote.creer(true, b"Plaine - 1/5");
    let E::Salon { code, place } = attendre(&mut hote, |e| matches!(e, E::Salon { .. })) else {
        unreachable!()
    };
    assert_eq!(place, 0);
    assert!(hote.hote());
    assert_eq!(code_saisi(&code_lisible(&code).to_lowercase()), Some(code));

    // Un invité trouve le salon dans la liste et le rejoint
    let mut invite = Session::ouvrir(&adresse);
    invite.lister();
    let E::Liste(salons) = attendre(&mut invite, |e| matches!(e, E::Liste(_))) else {
        unreachable!()
    };
    let salon = salons.iter().find(|s| s.code == code).expect("salon listé");
    assert_eq!(salon.annonce, b"Plaine - 1/5");
    invite.rejoindre([b'Z'; 6]);
    assert!(matches!(
        attendre(&mut invite, |e| matches!(e, E::Refus(_))),
        E::Refus(bs_protocole::REFUS_SALON)
    ));
    invite.rejoindre(code);
    let E::Salon { place, .. } = attendre(&mut invite, |e| matches!(e, E::Salon { .. })) else {
        unreachable!()
    };
    assert!(place >= 1);
    assert_eq!(
        attendre(&mut hote, |e| matches!(e, E::Arrivee(_))),
        E::Arrivee(place)
    );

    // Échanges dans les deux sens
    invite.vers_hote(b"entree");
    assert_eq!(
        attendre(&mut hote, |e| matches!(e, E::Message { .. })),
        E::Message {
            place,
            octets: b"entree".to_vec()
        }
    );
    hote.vers_invites(b"instantane");
    assert_eq!(
        attendre(&mut invite, |e| matches!(e, E::Message { .. })),
        E::Message {
            place: 0,
            octets: b"instantane".to_vec()
        }
    );

    // L'hôte quitte : le salon est fermé pour l'invité
    hote.quitter();
    attendre(&mut invite, |e| *e == E::Ferme);
}

#[test]
fn coupure_puis_reprise() {
    let adresse = relais();
    let mut hote = Session::ouvrir(&adresse);
    hote.creer(false, b"");
    let E::Salon { code, .. } = attendre(&mut hote, |e| matches!(e, E::Salon { .. })) else {
        unreachable!()
    };
    let mut invite = Session::ouvrir(&adresse);
    invite.rejoindre(code);
    let E::Salon { place, .. } = attendre(&mut invite, |e| matches!(e, E::Salon { .. })) else {
        unreachable!()
    };
    attendre(&mut hote, |e| matches!(e, E::Arrivee(_)));

    // Coupure brutale (sans fermeture normale) : la session se reconnecte et reprend sa place
    invite.couper_pour_essai();
    attendre(&mut invite, |e| *e == E::Coupure);
    attendre(&mut hote, |e| *e == E::Depart(place));
    attendre(&mut invite, |e| *e == E::Reprise);
    assert_eq!(
        attendre(&mut hote, |e| matches!(e, E::Arrivee(_))),
        E::Arrivee(place)
    );
    invite.vers_hote(b"revenu");
    assert_eq!(
        attendre(&mut hote, |e| matches!(e, E::Message { .. })),
        E::Message {
            place,
            octets: b"revenu".to_vec()
        }
    );
}
