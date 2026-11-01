//! Collection : pièces gagnées (portées si l'emplacement est libre), bonus des pièces
//! portées et de l'ensemble complet, objets découverts, relus après écriture.

use bs_donnees::Donnees;
use bs_donnees::arenes::Monde;
use bs_donnees::equipement::Piece;
use bs_sauvegarde::collection::PIECES;
use bs_sauvegarde::{Collection, Stockage};

fn donnees() -> &'static Donnees {
    Box::leak(Box::new(Donnees::embarquees().unwrap()))
}

fn stockage(nom: &str) -> Stockage {
    let d = std::env::temp_dir().join(format!("bs-collection-{nom}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    Stockage::nouveau(d)
}

#[test]
fn pieces_gagnees_portees_et_relues() {
    let d = donnees();
    let s = stockage("relue");
    let mut c = Collection::charger(s.clone(), "collection".into());
    assert!(c.pieces_du_monde(Monde::Prairie).is_empty());
    assert_eq!(c.bonus(d), Default::default());
    c.ajouter(Monde::Prairie, Piece::Casque);
    c.ajouter(Monde::Mine, Piece::Casque);
    // Le premier casque gagné est porté, le second attend
    assert_eq!(c.portee(Piece::Casque), Some(Monde::Prairie));
    assert!(c.possede(Monde::Mine, Piece::Casque));
    c.decouvrir("bottes_de_vitesse");
    c.decouvrir("bottes_de_vitesse");
    let relue = Collection::charger(s, "collection".into());
    assert_eq!(relue.portee(Piece::Casque), Some(Monde::Prairie));
    assert_eq!(relue.pieces_du_monde(Monde::Mine), vec![Piece::Casque]);
    assert!(relue.decouvert("bottes_de_vitesse"));
    assert_ne!(relue.bonus(d), Default::default());
}

#[test]
fn ensemble_complet_et_piece_non_possedee() {
    let d = donnees();
    let mut c = Collection::charger(stockage("ensemble"), "collection".into());
    // Porter une pièce non possédée : sans effet
    c.porter(Piece::Cape, Some(Monde::Nether));
    assert_eq!(c.portee(Piece::Cape), None);
    for p in PIECES {
        c.ajouter(Monde::Nether, p);
    }
    assert_eq!(c.ensemble_complet(), Some(Monde::Nether));
    assert_eq!(c.avancee(d, Monde::Nether).pieces, PIECES.len());
    c.porter(Piece::Talisman, None);
    assert_eq!(c.ensemble_complet(), None);
}
