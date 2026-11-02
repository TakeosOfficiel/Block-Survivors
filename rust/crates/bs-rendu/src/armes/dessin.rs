//! Systèmes de l'image : pose des pièces solides et dessin des lueurs de toutes les
//! armes, pour les objets de la partie et ceux de l'étalage de la vitrine.

use bevy::prelude::*;

use super::solides::Piece;
use super::vues::{fleches, givres, lames, orbes, pluies, ronces, texture_meteore, zone};
use super::{VisuelsArmes, arcanes, feu, foudre, melee, posees, sol};
use crate::camera::Cible;
use crate::modeles::rendu::Composants;
use crate::options::Options;
use crate::sources::sources;
use crate::{PAS, PartieEnCours};

/// Pièces solides de l'image (météores, blocs soulevés, lames, éclats de glace).
pub fn placer_solides(
    mut commandes: Commands,
    jeu: Res<PartieEnCours>,
    temps: Res<Time>,
    fixe: Res<Time<Fixed>>,
    mut v: ResMut<VisuelsArmes>,
    options: Res<Options>,
    mut q: Composants,
) {
    let allies = options.effets_allies;
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
    let meteores = sources(&jeu.partie, allies)
        .flat_map(|s| {
            s.armes
                .feu
                .meteores
                .iter()
                .map(move |m| (Some(s.arsenal), m))
        })
        .chain(etalage.meteores.iter().map(|m| (None, m)));
    for (arsenal, me) in meteores {
        let k = me.t + avance;
        let Some(texture) = texture_meteore(&jeu, arsenal, me.arme) else {
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
        lames(&jeu, allies, &etalage.chutes),
        givres(&jeu, allies, &etalage.novas),
        rotation,
    );
    let (f, r) = (
        fleches(&jeu, allies, &etalage.fleches),
        ronces(&jeu, allies, &etalage.pieges),
    );
    posees::placer(solides, &mut commandes, &mut q, f, r, rotation);
    arcanes::placer_orbes(
        solides,
        &mut commandes,
        &mut q,
        orbes(&jeu, allies, &etalage.orbes),
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
    options: Res<Options>,
    mut maillages: ResMut<Assets<Mesh>>,
    mut visibilites: Query<&mut Visibility>,
) {
    let allies = options.effets_allies;
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
    let invocations =
        crate::sources::invocations(&jeu.partie, allies).map(|(cle, a)| (cle, Vec2::new(a.x, a.z)));
    let o = orbes(&jeu, allies, &v.etalage.orbes).map(|(o, _)| o);
    v.arcanes.avancer(o, invocations, t, dt);
    let m = &mut v.lueur;
    m.commencer();
    let e = &v.etalage;
    for coups in [&v.coups, &e.coups] {
        melee::dessiner(m, coups, cible.0);
    }
    for eclairs in [&v.eclairs, &e.eclairs] {
        foudre::dessiner(m, eclairs, oeil, droite, haut);
    }
    for (n, src) in sources(&jeu.partie, allies).enumerate() {
        let f = &src.armes.feu;
        let scene = feu::Scene {
            tornades: &f.tornades,
            meteores: &f.meteores,
            feux: &f.feux,
            // Les braises (des événements) ne sont dessinées qu'une fois
            braises: if n == 0 { &v.braises } else { &[] },
            avance,
        };
        let arsenal = Some(src.arsenal);
        feu::dessiner(m, &scene, |a| zone(arsenal, a), t, oeil, droite, haut);
    }
    let scene = feu::Scene {
        tornades: &e.tornades,
        meteores: &e.meteores,
        feux: &e.feux,
        braises: &e.braises,
        avance: 0.0,
    };
    feu::dessiner(m, &scene, |a| zone(None, a), t, oeil, droite, haut);
    for blocs in [&v.blocs, &e.blocs] {
        let (l, g) = (std::iter::empty(), std::iter::empty());
        sol::dessiner(m, blocs, l, g, t, oeil, droite, haut);
    }
    let (l, g) = (
        lames(&jeu, allies, &e.chutes),
        givres(&jeu, allies, &e.novas),
    );
    sol::dessiner(m, &[], l, g, t, oeil, droite, haut);
    let (p, f) = (
        pluies(&jeu, allies, &e.zones),
        fleches(&jeu, allies, &e.fleches),
    );
    let r = ronces(&jeu, allies, &e.pieges);
    posees::dessiner(m, p, f, r, t, oeil, droite, haut);
    let o = orbes(&jeu, allies, &e.orbes).map(|(o, _)| o);
    v.arcanes.dessiner_orbes(m, o, t, oeil, droite, haut);
    v.arcanes
        .dessiner_apparitions(m, &e.apparitions, oeil, droite, haut);
    m.terminer(&mut maillages, &mut visibilites);
}
