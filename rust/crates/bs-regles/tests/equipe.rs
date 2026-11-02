//! Essais de la coopération : échange des combattants, cibles, niveau commun,
//! réanimation, défaite de toute l'équipe.

use bs_commun::PAS_SIMULATION;
use bs_donnees::Donnees;
use bs_regles::Partie;
use bs_regles::equipe::regles;
use bs_regles::etat::Issue;

fn donnees() -> &'static Donnees {
    Box::leak(Box::new(Donnees::embarquees().expect("données embarquées")))
}

#[test]
fn echanger_deux_fois_rend_l_etat() {
    let d = donnees();
    let mut p = Partie::nouvelle(d, 0, 0, 5, false);
    let k = p.ajouter_combattant(1);
    assert_eq!(k, 1);
    assert_eq!(p.nombre_joueurs(), 2);
    let hote = p.etat.joueur.clone();
    let autre = p.etat_joueur(1).clone();
    assert_ne!((hote.x, hote.z), (autre.x, autre.z), "chacun sa place");
    assert_eq!(p.equipe.membres[0].index_personnage, 1);
    p.echanger(1);
    assert_eq!(p.etat.joueur, autre);
    assert_eq!(p.etat.index_personnage, 1);
    assert_eq!(*p.etat_joueur(0), hote);
    p.echanger(1);
    assert_eq!(p.etat.joueur, hote);
    assert_eq!(p.etat.index_personnage, 0);
    // L'autre a ses armes de départ
    assert!(!p.equipe.membres[0].arsenal.armes.is_empty());
    // Plus d'ennemis à plusieurs
    assert_eq!(
        p.ennemis.multiplicateur_pv,
        regles::facteur(regles::PV_ENNEMIS, 2)
    );
}

#[test]
fn deux_joueurs_jouent_ensemble() {
    let d = donnees();
    let mut p = Partie::nouvelle(d, 0, 0, 11, false);
    p.ajouter_combattant(2);
    // L'autre s'éloigne vers la droite
    p.equipe.membres[0].joueur.commandes.droite = true;
    let x0 = p.etat_joueur(1).x;
    let pas = (90.0 / PAS_SIMULATION) as u32;
    for _ in 0..pas {
        p.pas(PAS_SIMULATION);
        if p.etat.issue.is_some() {
            break;
        }
    }
    let autre = p.etat_joueur(1);
    assert!(autre.x > x0 + 3.0, "l'autre a marché : {x0} → {}", autre.x);
    // Ses armes ont frappé (son propre bilan)
    let degats: f64 = p.equipe.membres[0].stats.degats_par_arme.iter().sum();
    assert!(degats > 0.0, "les armes de l'autre ont touché");
    // Niveau commun, cartes à choisir pour l'autre
    assert_eq!(p.etat_joueur(1).niveau, p.etat.joueur.niveau);
    if p.etat.joueur.niveau > 1 {
        assert!(p.equipe.membres[0].cartes_en_attente > 0);
    }
}

#[test]
fn reanimation_et_defaite_de_l_equipe() {
    let d = donnees();
    let mut p = Partie::nouvelle(d, 0, 0, 3, false);
    p.ajouter_combattant(0);
    // L'hôte tombe : la partie continue, l'autre le relève en restant près de lui
    p.etat.joueur.mort = true;
    p.etat.joueur.pv = 0.0;
    let (x, z) = (p.etat.joueur.x, p.etat.joueur.z);
    let j = p.etat_joueur_mut(1);
    j.x = x + 0.5;
    j.z = z;
    let pas = ((regles::DUREE_REANIMATION + 0.5) / PAS_SIMULATION) as u32;
    for _ in 0..pas {
        p.simuler(PAS_SIMULATION);
        let j = p.etat_joueur_mut(1);
        j.x = x + 0.5;
        j.z = z;
        j.pv = j.pv_max;
    }
    assert!(p.etat.issue.is_none(), "un allié debout : pas de défaite");
    assert!(!p.etat.joueur.mort, "l'hôte a été relevé");
    assert!(p.etat.joueur.pv > 0.0);
    // Les deux à terre : défaite
    p.etat.joueur.mort = true;
    p.etat_joueur_mut(1).mort = true;
    p.simuler(PAS_SIMULATION);
    assert_eq!(p.etat.issue, Some(Issue::Defaite));
}

#[test]
fn un_joueur_parti_n_est_plus_une_cible() {
    let d = donnees();
    let mut p = Partie::nouvelle(d, 0, 0, 8, false);
    p.ajouter_combattant(0);
    p.retirer_combattant(1);
    let (x, z) = (p.etat.joueur.x, p.etat.joueur.z);
    assert_eq!(p.plus_proche_joueur(1e4, 1e4), Some(0));
    assert_eq!(p.plus_proche_joueur(x, z), Some(0));
    p.etat.joueur.mort = true;
    assert_eq!(p.plus_proche_joueur(x, z), None);
}
