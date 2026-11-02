//! Amis de bout en bout contre un vrai relais local : deux profils amis se voient en ligne,
//! s'écrivent ; un imposteur sous le code d'un ami est ignoré ; le départ se voit.

use std::time::{Duration, Instant};

use bs_reseau::amis::{Etat, EtatPresence, Lettre};
use bs_reseau::{EvenementAmis as E, ServiceAmis};

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

fn etat(pseudo: &str) -> Etat {
    Etat {
        pseudo: pseudo.into(),
        joueurs: 1,
        ..Default::default()
    }
}

/// Fait tourner les services jusqu'à ce que `s` donne l'évènement voulu (5 s au plus).
fn attendre(s: &mut ServiceAmis, autres: &mut [&mut ServiceAmis], voulu: impl Fn(&E) -> bool) -> E {
    let fin = Instant::now() + Duration::from_secs(5);
    while Instant::now() < fin {
        for a in autres.iter_mut() {
            a.maj();
            while a.evenement().is_some() {}
        }
        s.maj();
        while let Some(e) = s.evenement() {
            if voulu(&e) {
                return e;
            }
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    panic!("évènement attendu non reçu");
}

/// Laisse tourner un moment sans rien attendre ; les évènements de `s`.
fn tourner(s: &mut ServiceAmis, autres: &mut [&mut ServiceAmis], duree: u64) -> Vec<E> {
    let mut vus = Vec::new();
    let fin = Instant::now() + Duration::from_millis(duree);
    while Instant::now() < fin {
        for a in autres.iter_mut() {
            a.maj();
            while a.evenement().is_some() {}
        }
        s.maj();
        while let Some(e) = s.evenement() {
            vus.push(e);
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    vus
}

#[test]
fn amis_de_bout_en_bout() {
    let adresse = relais();
    let mut steve = ServiceAmis::nouveau(&adresse, "AAA-AAA-AAA", &[1; 32], etat("Steve"));
    let mut alex = ServiceAmis::nouveau(&adresse, "BBB-BBB-BBB", &[2; 32], etat("Alex"));

    // Steve ajoute Alex et le lui dit ; Alex reçoit l'ajout (Steve inconnu de lui)
    steve.suivre("BBB-BBB-BBB", None);
    tourner(&mut steve, &mut [&mut alex], 300);
    steve.poster("BBB-BBB-BBB", &Lettre::Ajout);
    let e = attendre(&mut alex, &mut [&mut steve], |e| {
        matches!(e, E::Ajout { .. })
    });
    assert_eq!(
        e,
        E::Ajout {
            code: "AAA-AAA-AAA".into(),
            pseudo: "Steve".into()
        }
    );
    // Steve voit Alex en ligne et a appris sa clé (évènement `Cle` relevé plus haut)
    tourner(&mut steve, &mut [&mut alex], 200);
    let a = steve.ami("BBB-BBB-BBB").unwrap();
    assert!(a.presence.is_some());
    let cle = a.cle.expect("clé apprise");
    // Une clé apprise se signale
    let mut zoe = ServiceAmis::nouveau(&adresse, "CCC-CCC-CCC", &[3; 32], etat("Zoé"));
    zoe.suivre("BBB-BBB-BBB", None);
    let e = attendre(&mut zoe, &mut [&mut alex], |e| matches!(e, E::Cle { .. }));
    assert_eq!(
        e,
        E::Cle {
            code: "BBB-BBB-BBB".into(),
            cle
        }
    );
    drop(zoe);

    // Alex ajoute Steve à son tour, ouvre un salon et l'invite
    alex.suivre("AAA-AAA-AAA", None);
    attendre(&mut alex, &mut [&mut steve], |e| matches!(e, E::Cle { .. }));
    tourner(&mut steve, &mut [&mut alex], 100);
    alex.publier(Etat {
        etat: EtatPresence::Salon,
        salon: Some(*b"KMNPQR"),
        ..etat("Alex")
    });
    alex.poster("AAA-AAA-AAA", &Lettre::Invitation(*b"KMNPQR"));
    let e = attendre(&mut steve, &mut [&mut alex], |e| {
        matches!(e, E::Invitation { .. })
    });
    assert_eq!(
        e,
        E::Invitation {
            code: "BBB-BBB-BBB".into(),
            salon: *b"KMNPQR"
        }
    );
    tourner(&mut steve, &mut [&mut alex], 200);
    let p = steve.ami("BBB-BBB-BBB").unwrap().presence.clone().unwrap();
    assert_eq!(p.etat, EtatPresence::Salon);
    assert_eq!(p.salon, Some(*b"KMNPQR"));

    // Un imposteur se dit Alex : ses lettres (autre clé) sont ignorées
    let mut faux = ServiceAmis::nouveau(&adresse, "BBB-BBB-BBB", &[9; 32], etat("Alex"));
    tourner(&mut faux, &mut [&mut steve, &mut alex], 300);
    faux.poster("AAA-AAA-AAA", &Lettre::Demande);
    let vus = tourner(&mut steve, &mut [&mut alex, &mut faux], 400);
    assert!(
        !vus.iter().any(|e| matches!(e, E::Demande { .. })),
        "{vus:?}"
    );
    assert_eq!(steve.ami("BBB-BBB-BBB").unwrap().cle, Some(cle));
    drop(faux);

    // La vraie demande passe
    alex.poster("AAA-AAA-AAA", &Lettre::Demande);
    attendre(&mut steve, &mut [&mut alex], |e| {
        matches!(e, E::Demande { .. })
    });

    // Alex part : Steve le voit hors ligne
    alex.fermer();
    let fin = Instant::now() + Duration::from_secs(5);
    while steve.ami("BBB-BBB-BBB").unwrap().presence.is_some() {
        assert!(Instant::now() < fin, "départ non vu");
        steve.maj();
        while steve.evenement().is_some() {}
        std::thread::sleep(Duration::from_millis(2));
    }
}
