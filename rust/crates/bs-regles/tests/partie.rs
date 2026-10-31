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
