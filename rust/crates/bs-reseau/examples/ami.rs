//! Un ami factice pour essayer la page Multijoueur : se connecte au relais sous un code
//! ami, suit un autre code, publie sa présence et poste un ajout, une demande ou une
//! invitation, puis reste en ligne et affiche ce qu'il reçoit.
//!
//! `cargo run -p bs-reseau --example ami -- <mon code> <code suivi> [ajout|demande|invitation]`
//! (relais : `BS_RELAIS`, sinon le relais local).

use std::time::Duration;

use bs_reseau::ServiceAmis;
use bs_reseau::amis::{Etat, EtatPresence, Lettre};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (Some(moi), Some(lui)) = (args.get(1), args.get(2)) else {
        eprintln!("usage : ami <mon code> <code suivi> [ajout|demande|invitation]");
        return;
    };
    let mut graine = [0u8; 32];
    for (g, o) in graine.iter_mut().zip(moi.bytes()) {
        *g = o;
    }
    let invitation = args.get(3).is_some_and(|a| a == "invitation");
    let etat = Etat {
        pseudo: "Factice".into(),
        etat: if invitation {
            EtatPresence::Salon
        } else {
            EtatPresence::Menu
        },
        salon: invitation.then_some(*b"KMNPQR"),
        joueurs: 1,
    };
    let mut s = ServiceAmis::nouveau(&bs_reseau::adresse_relais(), moi, &graine, etat);
    s.suivre(lui, None);
    let mut poste = false;
    loop {
        s.maj();
        while let Some(e) = s.evenement() {
            println!("{e:?}");
        }
        if s.connecte() && !poste {
            poste = true;
            std::thread::sleep(Duration::from_millis(300));
            match args.get(3).map(String::as_str) {
                Some("ajout") => s.poster(lui, &Lettre::Ajout),
                Some("demande") => s.poster(lui, &Lettre::Demande),
                Some("invitation") => s.poster(lui, &Lettre::Invitation(*b"KMNPQR")),
                _ => {}
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}
