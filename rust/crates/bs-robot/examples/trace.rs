//! Trace pas à pas d'une partie du robot (tirages d'aléa, position, tués, PV), à comparer
//! avec celle du jeu TS pour trouver le premier pas où les deux divergent.
//!
//!   cargo run --release -p bs-robot --example trace -- archer 1 60 > trace.json

use bs_donnees::Donnees;
use bs_regles::Partie;
use bs_robot::Robot;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let perso = args.get(1).map_or("archer", |s| s.as_str());
    let graine: u32 = args.get(2).map_or(1, |s| s.parse().expect("graine"));
    let duree: f64 = args.get(3).map_or(60.0, |s| s.parse().expect("durée"));
    let d: &'static Donnees = Box::leak(Box::new(Donnees::embarquees().expect("données")));
    let a = d
        .arenes
        .arenes
        .iter()
        .position(|x| x.id == "prairie")
        .expect("prairie");
    let i = d
        .personnages
        .personnages
        .get_index_of(perso)
        .expect("personnage");
    let mut p = Partie::nouvelle(d, a, i, graine, false);
    let mut robot = Robot::default();
    let arrondi = |v: f32, n: f32| (v * n).round() / n;
    let mut lignes = Vec::new();
    let mut pas = 0;
    while p.etat.issue.is_none() && p.etat.temps_jeu < duree {
        robot.avancer(&mut p);
        let j = &p.etat.joueur;
        lignes.push(format!(
            "[{pas},{},{},{},{},{},[{}]]",
            p.alea.tirages(graine),
            arrondi(j.x, 1000.0),
            arrondi(j.z, 1000.0),
            p.etat.ennemis_tues,
            arrondi(j.pv, 100.0),
            p.ennemis
                .actifs()
                .iter()
                .map(|e| format!(
                    "[{},{},{},{},{},{},{},{},{}]",
                    arrondi(e.x, 1000.0),
                    arrondi(e.z, 1000.0),
                    e.index_type,
                    arrondi(e.dir_x, 1000.0),
                    arrondi(e.dir_z, 1000.0),
                    u8::from(e.voit_joueur),
                    arrondi(e.minuterie, 1000.0),
                    arrondi(e.minuterie_vue, 1000.0),
                    arrondi(e.pv, 100.0)
                ))
                .collect::<Vec<_>>()
                .join(",")
        ));
        pas += 1;
    }
    println!("[{}]", lignes.join(","));
}
