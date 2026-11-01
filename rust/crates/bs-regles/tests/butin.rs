//! Butin des boss : jamais une pièce déjà possédée (Collection), un événement par boss
//! récompensé qui nomme les pièces gagnées.

use bs_donnees::Donnees;
use bs_donnees::equipement::Piece;
use bs_regles::Partie;
use bs_regles::evenements::Evenement;

fn donnees() -> &'static Donnees {
    Box::leak(Box::new(Donnees::embarquees().expect("données embarquées")))
}

#[test]
fn butin_sans_pieces_possedees() {
    let d = donnees();
    let possedees = [Piece::Casque, Piece::Bottes, Piece::Jambieres];
    let mut gagnees = 0;
    for graine in 0..200 {
        let mut p = Partie::nouvelle(d, 0, 0, graine, true);
        p.posseder(&possedees);
        for boss in 1..=4 {
            p.ev.vider();
            p.etat.stats.boss_vaincus = boss;
            p.recompenser_boss();
            for ev in &p.ev.liste {
                if let Evenement::ButinBoss { monde, pieces } = *ev {
                    assert_eq!(monde, p.etat.arene.monde);
                    for piece in pieces.into_iter().flatten() {
                        assert!(!possedees.contains(&piece), "{piece:?} déjà possédée");
                        gagnees += 1;
                    }
                }
            }
        }
        // Chaque pièce gagnée l'est une seule fois
        let g = &p.butin.gagnees;
        for (k, piece) in g.iter().enumerate() {
            assert!(!g[k + 1..].contains(piece));
        }
    }
    assert!(gagnees > 50, "{gagnees} pièces gagnées sur 200 parties");
}
