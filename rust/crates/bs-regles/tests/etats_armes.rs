//! Essais de l'état propre d'une arme (`etat` dans armes.ron) : infligé dès le rang 1,
//! sans éveil, comme celui d'un éveil.

use bs_commun::PAS_SIMULATION;
use bs_donnees::Donnees;
use bs_regles::Partie;

fn donnees() -> &'static Donnees {
    Box::leak(Box::new(Donnees::embarquees().expect("données embarquées")))
}

/// La Fiole de poison de l'Alchimiste empoisonne dès sa première fiole.
#[test]
fn fiole_de_poison_empoisonne_au_rang_1() {
    let d = donnees();
    let alchimiste = d
        .personnages
        .personnages
        .get_index_of("alchimiste")
        .unwrap();
    let fiole = d.armes.get_index_of("fiole_poison").unwrap();
    assert!(d.armes[fiole].etat.is_some());
    let mut p = Partie::nouvelle(d, 0, alchimiste, 11, false);
    p.joueur.invincible = true;
    assert_eq!(p.arsenal.rang(fiole), 1);
    let mut empoisonnes = 0;
    for _ in 0..(40.0 / PAS_SIMULATION) as u32 {
        p.pas(PAS_SIMULATION);
        empoisonnes += p.ennemis.pool[..p.ennemis.nombre]
            .iter()
            .filter(|e| e.poison.arme == Some(fiole) && e.poison.temps > 0.0)
            .count();
    }
    assert!(empoisonnes > 0, "aucun ennemi empoisonné par la fiole");
}

/// Une arme sans état propre n'en inflige pas avant son éveil.
#[test]
fn sans_etat_ni_eveil_rien() {
    let d = donnees();
    let epee = d.armes.get_index_of("epee").unwrap();
    assert!(d.armes[epee].etat.is_none());
    let mut p = Partie::nouvelle(d, 0, 0, 11, false);
    p.joueur.invincible = true;
    for _ in 0..(30.0 / PAS_SIMULATION) as u32 {
        p.pas(PAS_SIMULATION);
        assert!(
            p.ennemis.pool[..p.ennemis.nombre]
                .iter()
                .all(|e| e.poison.arme != Some(epee) && e.brulure.arme != Some(epee))
        );
    }
}
