//! Tests des trames : aller-retour de chaque trame, liste lue sans allocation, rejet
//! des trames mal formées.

use super::*;

fn liste_de_test(o: &mut Vec<u8>) {
    Trame::Liste {
        salons: ListeSalons::default(),
    }
    .ecrire(o);
    for (k, annonce) in [&b"plaine"[..], &[], &[7; ANNONCE_MAX]]
        .into_iter()
        .enumerate()
    {
        EntreeListe {
            code: *b"ABCDEF",
            joueurs: k as u8 + 1,
            places: PLACES_SALON,
            annonce,
        }
        .ajouter(o);
    }
}

#[test]
fn aller_retour() {
    let mut o = Vec::new();
    let mut liste = Vec::new();
    liste_de_test(&mut liste);
    let Some(Trame::Liste { salons }) = lire_trame(&liste) else {
        panic!("liste illisible")
    };
    for t in [
        Trame::Creer {
            version: 7,
            public: true,
            annonce: b"arene=plaine",
        },
        Trame::Creer {
            version: 7,
            public: false,
            annonce: &[],
        },
        Trame::Rejoindre {
            version: 7,
            code: *b"ABCDEF",
        },
        Trame::Salon {
            code: *b"ABCDEF",
            place: 2,
            jeton: 0x0123_4567_89ab_cdef,
        },
        Trame::Refus {
            raison: REFUS_PLEIN,
        },
        Trame::VersHote {
            place: 1,
            charge: &[9, 8],
        },
        Trame::VersInvites { charge: &[1, 2, 3] },
        Trame::Reprendre {
            version: 7,
            code: *b"XYZXYZ",
            jeton: u64::MAX,
        },
        Trame::Annoncer {
            annonce: &[3; ANNONCE_MAX],
        },
        Trame::Lister { version: 7 },
        Trame::Liste { salons },
        Trame::Liste {
            salons: ListeSalons::default(),
        },
        Trame::Arrivee { place: 3 },
        Trame::Depart { place: 4 },
        Trame::HotePerdu,
        Trame::HoteRevenu,
        Trame::Exclure { place: 2 },
        Trame::Exclu,
        Trame::Ferme,
    ] {
        t.ecrire(&mut o);
        assert_eq!(lire_trame(&o), Some(t));
    }
}

#[test]
fn liste_parcourue_sans_allocation() {
    let mut o = Vec::new();
    liste_de_test(&mut o);
    let Some(Trame::Liste { salons }) = lire_trame(&o) else {
        panic!()
    };
    let lus: Vec<_> = salons.iter().collect();
    assert_eq!(lus.len(), 3);
    assert_eq!(lus[0].annonce, b"plaine");
    assert_eq!(lus[1].joueurs, 2);
    assert_eq!(lus[2].annonce.len(), ANNONCE_MAX);
    assert!(lus.iter().all(|e| e.code == *b"ABCDEF"));
}

#[test]
fn trames_mal_formees() {
    let trop_longue = [&[ANNONCER][..], &[0; ANNONCE_MAX + 1]].concat();
    let creer_trop_longue = [&[CREER, 1, 0, 1][..], &[0; ANNONCE_MAX + 1]].concat();
    let mut liste = Vec::new();
    liste_de_test(&mut liste);
    let mut liste_trop_longue = liste.clone();
    liste_trop_longue.push(0);
    // Taille d'annonce annoncée > ANNONCE_MAX
    let mut entree_geante = vec![LISTE, b'A', b'B', b'C', b'D', b'E', b'F', 1, 5];
    entree_geante.extend_from_slice(&(ANNONCE_MAX as u16 + 1).to_le_bytes());
    entree_geante.extend_from_slice(&[0; ANNONCE_MAX + 1]);
    for o in [
        &[][..],
        &[200],
        &[CREER, 1],
        &[CREER, 1, 0],
        &[CREER, 1, 0, 2],
        &creer_trop_longue,
        &[REJOINDRE, 0],
        &[REJOINDRE, 1, 0, b'A', b'B', b'C', b'D', b'E', b'F', 0],
        &[SALON, b'A', b'B', b'C', b'D', b'E', b'F', 1, 0, 0],
        &[REFUS],
        &[REFUS, 1, 1],
        &[VERS_HOTE],
        &[REPRENDRE, 1, 0, b'A', b'B', b'C', b'D', b'E', b'F', 1, 2, 3],
        &trop_longue,
        &[LISTER, 1],
        &[LISTER, 1, 0, 0],
        &liste[..liste.len() - 1],
        &liste_trop_longue,
        &entree_geante,
        &[ARRIVEE],
        &[DEPART, 1, 1],
        &[EXCLURE],
        &[HOTE_PERDU, 0],
        &[HOTE_REVENU, 0],
        &[EXCLU, 0],
        &[FERME, 0],
    ] {
        assert_eq!(lire_trame(o), None, "{o:?}");
    }
}
