//! Pas de la simulation (`FixedUpdate`) : le clavier conduit le joueur, ou le robot de
//! démonstration tant que personne n'y a touché ; pas de pas tant que des cartes
//! attendent un choix ni pendant la pause. Début de partie accéléré pour les captures
//! (`BS_DEBUT`, `BS_COFFRE`, `BS_FIN`).

use bevy::prelude::*;

use crate::ecran_cartes::{self, EcranCartes};
use crate::ecrans::Ecrans;
use crate::{ATTENTE_RELANCE, Mesure, PAS, PartieEnCours};

/// `BS_DEBUT=<s>` (essais) : le robot joue ces secondes avant l'affichage, sans mourir.
pub(crate) fn avancer_debut(jeu: &mut PartieEnCours) {
    let secondes: f32 = std::env::var("BS_DEBUT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);
    for _ in 0..(secondes / PAS) as u32 {
        jeu.partie.ev.vider();
        let j = &mut jeu.partie.etat.joueur;
        j.pv = j.pv_max;
        jeu.robot.avancer(&mut jeu.partie);
    }
    // Essai de l'écran des coffres : un coffre (de boss avec « ender ») à ouvrir
    if let Ok(v) = std::env::var("BS_COFFRE") {
        let choix = &mut jeu.partie.etat.choix_coffre;
        choix.restants += 1;
        choix.ender += u32::from(v == "ender");
    }
    // Essai de l'écran de fin : `BS_FIN=mort` ou `victoire`
    if let Ok(v) = std::env::var("BS_FIN") {
        jeu.partie.etat.issue = Some(if v == "victoire" {
            bs_regles::etat::Issue::Victoire
        } else {
            bs_regles::etat::Issue::Defaite
        });
    }
}

/// Un pas : le clavier (ZQSD, WASD, flèches) conduit le joueur ; sans joueur, le robot.
pub(crate) fn avancer(
    mut jeu: ResMut<PartieEnCours>,
    touches: Res<ButtonInput<KeyCode>>,
    mut ecran: ResMut<EcranCartes>,
    ecrans: Res<Ecrans>,
    mut mesure: ResMut<Mesure>,
) {
    let debut = bevy::platform::time::Instant::now();
    // Les événements du pas précédent ont été lus par `entites::animer`
    jeu.partie.ev.vider();
    if ecrans.fige() {
        return;
    }
    if jeu.partie.etat.issue.is_some() {
        // Le joueur choisit sur l'écran de fin ; le robot relance seul
        jeu.fin += PAS;
        if !ecran.humain && jeu.fin >= ATTENTE_RELANCE {
            jeu.relancer();
        }
        return;
    }
    let PartieEnCours { partie, robot, .. } = &mut *jeu;
    let appui = |codes: &[KeyCode]| codes.iter().any(|c| touches.pressed(*c));
    let c = &mut partie.joueur.commandes;
    c.haut = appui(&[KeyCode::KeyW, KeyCode::ArrowUp]);
    c.bas = appui(&[KeyCode::KeyS, KeyCode::ArrowDown]);
    c.gauche = appui(&[KeyCode::KeyA, KeyCode::ArrowLeft]);
    c.droite = appui(&[KeyCode::KeyD, KeyCode::ArrowRight]);
    if c.haut || c.bas || c.gauche || c.droite {
        ecran.humain = true;
    }
    if !ecran.humain {
        robot.avancer(partie);
    } else if !ecran_cartes::en_attente(partie) {
        // Cartes à choisir : partie en pause
        partie.pas(PAS);
    }
    mesure.ajouter_pas(debut.elapsed());
}
