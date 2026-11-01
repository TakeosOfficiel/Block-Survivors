//! Systèmes de l'image : pose des pièces solides et dessin des lueurs de toutes les
//! armes, pour les objets de la partie et ceux de l'étalage de la vitrine.

use bevy::prelude::*;

use super::solides::Piece;
use super::vues::{fleches, givres, lames, orbes, pluies, ronces, texture_meteore, zone};
use super::{VisuelsArmes, arcanes, feu, foudre, melee, posees, sol};
use crate::camera::Cible;
use crate::modeles::rendu::Composants;
use crate::{PAS, PartieEnCours};

/// Pièces solides de l'image (météores, blocs soulevés, lames, éclats de glace).
pub fn placer_solides(
    mut commandes: Commands,
    jeu: Res<PartieEnCours>,
    temps: Res<Time>,
    fixe: Res<Time<Fixed>>,
    mut v: ResMut<VisuelsArmes>,
    mut q: Composants,
) {
    let avance = fixe.overstep_fraction() * PAS;
    let t = temps.elapsed_secs();
    let rotation = v.camera;
    let VisuelsArmes {
        solides,
        etalage,
        blocs,
        sol,
        ..
    } = &mut *v;
    solides.commencer();
    for me in jeu
        .partie
        .armes
        .feu
        .meteores
        .iter()
        .chain(&etalage.meteores)
    {
        let k = me.t + avance;
        let Some(texture) = texture_meteore(&jeu, me.arme) else {
            continue;
        };
        if k >= 0.0 {
            let pos = feu::position_meteore(me.x, me.z, k, me.taille);
            let tf = feu::bloc_meteore(pos, me.taille, t);
            solides.poser(&mut commandes, &mut q, texture, Piece::Cube, tf);
        }
    }
    for b in [&*blocs, &etalage.blocs] {
        sol::placer(
            solides,
            &mut commandes,
            &mut q,
            b,
            sol,
            std::iter::empty(),
            std::iter::empty(),
            rotation,
        );
    }
    sol::placer(
        solides,
        &mut commandes,
        &mut q,
        &[],
        sol,
        lames(&jeu, &etalage.chutes),
        givres(&jeu, &etalage.novas),
        rotation,
    );
    let (f, r) = (
        fleches(&jeu, &etalage.fleches),
        ronces(&jeu, &etalage.pieges),
    );
    posees::placer(solides, &mut commandes, &mut q, f, r, rotation);
    arcanes::placer_orbes(
        solides,
        &mut commandes,
        &mut q,
        orbes(&jeu, &etalage.orbes),
        t,
    );
    solides.terminer(&mut q);
}

/// Vie et dessin de l'image.
#[allow(clippy::too_many_arguments)]
pub fn dessiner(
    temps: Res<Time>,
    fixe: Res<Time<Fixed>>,
    jeu: Res<PartieEnCours>,
    mut v: ResMut<VisuelsArmes>,
    cible: Res<Cible>,
    camera: Query<&Transform, With<Camera3d>>,
    mut maillages: ResMut<Assets<Mesh>>,
    mut visibilites: Query<&mut Visibility>,
) {
    let dt = temps.delta_secs();
    let t = temps.elapsed_secs();
    let avance = fixe.overstep_fraction() * PAS;
    let tf = camera.single().copied().unwrap_or_default();
    let (oeil, droite, haut) = (tf.translation, tf.rotation * Vec3::X, tf.rotation * Vec3::Y);
    let v = &mut *v;
    v.camera = tf.rotation;
    melee::avancer(&mut v.coups, dt);
    foudre::avancer(&mut v.eclairs, dt);
    feu::avancer(&mut v.braises, dt);
    sol::avancer(&mut v.blocs, dt);
    let allies = jeu.partie.armes.invocations.pool.iter();
    let allies = allies.map(|a| (a.id, Vec2::new(a.x, a.z)));
    let o = orbes(&jeu, &v.etalage.orbes).map(|(o, _)| o);
    v.arcanes.avancer(o, allies, t, dt);
    let m = &mut v.lueur;
    m.commencer();
    let e = &v.etalage;
    for coups in [&v.coups, &e.coups] {
        melee::dessiner(m, coups, cible.0);
    }
    for eclairs in [&v.eclairs, &e.eclairs] {
        foudre::dessiner(m, eclairs, oeil, droite, haut);
    }
    let f = &jeu.partie.armes.feu;
    let scenes = [
        feu::Scene {
            tornades: &f.tornades,
            meteores: &f.meteores,
            feux: &f.feux,
            braises: &v.braises,
            avance,
        },
        feu::Scene {
            tornades: &e.tornades,
            meteores: &e.meteores,
            feux: &e.feux,
            braises: &e.braises,
            avance: 0.0,
        },
    ];
    for scene in &scenes {
        feu::dessiner(m, scene, |a| zone(&jeu, a), t, oeil, droite, haut);
    }
    for blocs in [&v.blocs, &e.blocs] {
        let (l, g) = (std::iter::empty(), std::iter::empty());
        sol::dessiner(m, blocs, l, g, t, oeil, droite, haut);
    }
    let (l, g) = (lames(&jeu, &e.chutes), givres(&jeu, &e.novas));
    sol::dessiner(m, &[], l, g, t, oeil, droite, haut);
    let (p, f) = (pluies(&jeu, &e.zones), fleches(&jeu, &e.fleches));
    let r = ronces(&jeu, &e.pieges);
    posees::dessiner(m, p, f, r, t, oeil, droite, haut);
    let o = orbes(&jeu, &e.orbes).map(|(o, _)| o);
    v.arcanes.dessiner_orbes(m, o, t, oeil, droite, haut);
    v.arcanes
        .dessiner_apparitions(m, &e.apparitions, oeil, droite, haut);
    m.terminer(&mut maillages, &mut visibilites);
}
