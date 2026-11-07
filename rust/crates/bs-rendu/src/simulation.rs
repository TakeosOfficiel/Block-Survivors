//! Pas de la simulation (`FixedUpdate`) : le clavier conduit le joueur, ou le robot de
//! démonstration tant que personne n'y a touché ; pas de pas tant que des cartes
//! attendent un choix ni pendant la pause. Début de partie accéléré pour les captures
//! (`BS_DEBUT`, `BS_COFFRE`, `BS_FIN`).

use bevy::prelude::*;

use crate::ecran_cartes::{self, EcranCartes};
use crate::ecrans::Ecrans;
use bs_regles::joueur::Commandes;

use crate::coop::{self, Coop, MessageCoop};
use crate::{ATTENTE_RELANCE, Mesure, PAS, PartieEnCours, Phase};

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
        if crate::scene::immobile() {
            // Capture de scène : le joueur attend au point de départ, désarmé, que les
            // mobs l'entourent
            jeu.partie.etat.niveaux_en_attente = 0;
            jeu.partie.arsenal.armes.clear();
            jeu.partie.pas(PAS);
        } else {
            jeu.robot.avancer(&mut jeu.partie);
        }
    }
    if crate::scene::immobile() {
        encercler(jeu);
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
/// En coop, rien ne se fige : cartes ou menu ouverts, le joueur reste juste immobile.
#[allow(clippy::too_many_arguments)]
pub(crate) fn avancer(
    mut jeu: ResMut<PartieEnCours>,
    touches: Res<ButtonInput<KeyCode>>,
    mut ecran: ResMut<EcranCartes>,
    ecrans: Res<Ecrans>,
    mut mesure: ResMut<Mesure>,
    coop: Option<ResMut<Coop>>,
    mut message: ResMut<MessageCoop>,
    mut phase: ResMut<NextState<Phase>>,
) {
    let debut = bevy::platform::time::Instant::now();
    // Les événements du pas précédent ont été lus par `entites::animer`
    jeu.partie.ev.vider();
    if let Some(mut coop) = coop {
        ecran.humain = true;
        let libre = !ecran.ouvert() && !ecrans.ouvert();
        lire_clavier(&mut jeu.partie.joueur.commandes, &touches, libre);
        if let Err(raison) = coop::pas(&mut coop, &mut jeu, PAS) {
            warn!("coop terminée : {raison}");
            message.0 = Some(raison);
            phase.set(Phase::Menu);
        }
        mesure.ajouter_pas(debut.elapsed());
        return;
    }
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
    let c = &mut partie.joueur.commandes;
    lire_clavier(c, &touches, true);
    if c.haut || c.bas || c.gauche || c.droite {
        ecran.humain = true;
    }
    if crate::scene::scene().is_some() {
        // Capture de scène : la partie reste figée sur l'instant photographié
    } else if !ecran.humain {
        robot.avancer(partie);
    } else if !ecran_cartes::en_attente(partie) {
        // Cartes à choisir : partie en pause
        partie.pas(PAS);
    }
    mesure.ajouter_pas(debut.elapsed());
}

/// Touches de déplacement (`libre` faux : aucune, le lacet de la caméra reste).
fn lire_clavier(c: &mut Commandes, touches: &ButtonInput<KeyCode>, libre: bool) {
    let appui = |codes: &[KeyCode]| libre && codes.iter().any(|c| touches.pressed(*c));
    c.haut = appui(&[KeyCode::KeyW, KeyCode::ArrowUp]);
    c.bas = appui(&[KeyCode::KeyS, KeyCode::ArrowDown]);
    c.gauche = appui(&[KeyCode::KeyA, KeyCode::ArrowLeft]);
    c.droite = appui(&[KeyCode::KeyD, KeyCode::ArrowRight]);
}

/// Capture de scène : les mobs collés au joueur reculent sur un cercle autour de lui,
/// pour que le héros de l'accueil, posé au centre, reste visible.
fn encercler(jeu: &mut PartieEnCours) {
    let (jx, jz) = (jeu.partie.etat.joueur.x, jeu.partie.etat.joueur.z);
    let n = jeu.partie.ennemis.nombre;
    for (i, e) in jeu.partie.ennemis.pool[..n].iter_mut().enumerate() {
        let (dx, dz) = (e.x - jx, e.z - jz);
        let d = (dx * dx + dz * dz).sqrt();
        if d > 3.5 {
            continue;
        }
        // Angle d'arrivée, écarté d'un angle d'or pour ne pas les empiler
        let a = dz.atan2(dx) + i as f32 * 2.4;
        let r = 2.4 + (i % 3) as f32 * 0.8;
        e.x = jx + a.cos() * r;
        e.z = jz + a.sin() * r;
        // Tourné vers le joueur
        e.angle = bs_commun::maths::atan2(jx - e.x, jz - e.z);
    }
}
