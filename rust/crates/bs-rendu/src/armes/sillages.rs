//! Sillages : les projectiles du joueur (et des coéquipiers) laissent une queue de
//! comète à la teinte de leur arme, avec un halo autour de la tête ; les flèches
//! ricocheuses aussi ; les armes en orbite tracent un arc lumineux derrière elles et
//! baignent dans un halo scintillant.
//! Tout vient de l'état de la partie : chacun voit les sillages de tous.

use bevy::prelude::*;
use bs_donnees::armes::Mecanique;
use bs_regles::combat::tours;

use super::lueur::{Forme, MaillageLueur};
use super::outils::{blanchi, rgba};
use crate::sources::Source;

/// Hauteur des sillages (au niveau des projectiles).
const HAUTEUR: f32 = 1.0;
/// Longueur de la queue : durée de vol représentée (s), bornée.
const QUEUE_TEMPS: f32 = 0.09;
const QUEUE_MAX: f32 = 2.4;
/// Arc tracé derrière une arme en orbite (rad) et nombre de segments.
const ARC_ORBITE: f32 = 1.3;
const SEGMENTS_ORBITE: usize = 8;

/// Queue de comète de `tete` vers l'arrière (`dir` : sens du vol, au sol).
#[allow(clippy::too_many_arguments)]
fn comete(
    m: &mut MaillageLueur,
    tete: Vec3,
    dir: Vec2,
    longueur: f32,
    largeur: f32,
    c: [f32; 3],
    oeil: Vec3,
    droite: Vec3,
    haut: Vec3,
) {
    let queue = tete - Vec3::new(dir.x, 0.0, dir.y) * longueur;
    m.faisceau(oeil, tete, queue, largeur, 0.0, rgba(c, 1.3), rgba(c, 0.0));
    m.faisceau(
        oeil,
        tete,
        tete.lerp(queue, 0.5),
        largeur * 0.35,
        0.0,
        rgba(blanchi(c, 0.7), 1.6),
        rgba(c, 0.0),
    );
    let h = largeur * 2.2;
    m.carre(tete, droite * h, haut * h, rgba(c, 0.7), Forme::Disque, 0.0);
}

/// Sillages d'une source. `teinte` : couleur d'une arme du catalogue ; `avance` :
/// fraction de pas écoulée (mêmes positions que les modèles des projectiles).
#[allow(clippy::too_many_arguments)]
pub fn dessiner(
    m: &mut MaillageLueur,
    src: &Source,
    teinte: impl Fn(usize) -> [f32; 3],
    avance: f32,
    sol: impl Fn(f32, f32) -> f32,
    oeil: Vec3,
    droite: Vec3,
    haut: Vec3,
) {
    for p in &src.armes.projectiles.pool {
        let (x, z) = (
            p.x + p.dir_x * p.vitesse * avance,
            p.z + p.dir_z * p.vitesse * avance,
        );
        let tete = Vec3::new(x, sol(x, z) + p.def.hauteur.max(0.4), z);
        let longueur = (p.vitesse * QUEUE_TEMPS)
            .min(QUEUE_MAX)
            .min(p.parcouru + 0.3);
        let largeur = if p.zone > 0.0 { 0.4 } else { 0.25 };
        let dir = Vec2::new(p.dir_x, p.dir_z).normalize_or_zero();
        comete(
            m,
            tete,
            dir,
            longueur,
            largeur,
            teinte(p.arme),
            oeil,
            droite,
            haut,
        );
    }
    for (i, [x, z, dx, dz]) in src.armes.traque.ricochets_armes() {
        let Some(arme) = src.arsenal.armes.get(i) else {
            continue;
        };
        let tete = Vec3::new(x, sol(x, z) + HAUTEUR, z);
        let dir = Vec2::new(dx, dz).normalize_or_zero();
        comete(
            m,
            tete,
            dir,
            1.6,
            0.16,
            teinte(arme.index),
            oeil,
            droite,
            haut,
        );
    }
    if src.joueur.mort {
        return;
    }
    let j = src.joueur;
    for arme in &src.arsenal.armes {
        if !matches!(arme.def.mecanique, Mecanique::Orbite { .. }) {
            continue;
        }
        let s = arme.stats;
        let c = teinte(arme.index);
        let sens = s.vitesse.signum();
        let a0 = src.armes.angles_orbite[arme.index] + s.vitesse * avance;
        let y = sol(j.x, j.z) + HAUTEUR * 0.8;
        for k in 0..tours(s.nombre) {
            let tete = a0 + k as f32 * std::f32::consts::TAU / s.nombre;
            let point = |a: f32| Vec3::new(j.x + a.sin() * s.zone, y, j.z + a.cos() * s.zone);
            // Halo autour de l'objet, éclat blanc qui scintille au centre
            let centre = point(tete);
            m.carre(
                centre,
                droite * 0.9,
                haut * 0.9,
                rgba(c, 0.55),
                Forme::Disque,
                0.0,
            );
            let vif = 0.8 + 0.4 * (tete * 3.0).sin().abs();
            let e = 0.55 * vif;
            m.carre(
                centre,
                droite * e,
                haut * e,
                rgba(blanchi(c, 0.6), 1.2 * vif),
                Forme::Etincelle,
                0.0,
            );
            for n in 0..SEGMENTS_ORBITE {
                let (t0, t1) = (
                    n as f32 / SEGMENTS_ORBITE as f32,
                    (n + 1) as f32 / SEGMENTS_ORBITE as f32,
                );
                let (p0, p1) = (
                    point(tete - sens * ARC_ORBITE * t0),
                    point(tete - sens * ARC_ORBITE * t1),
                );
                let (f0, f1) = (1.0 - t0, 1.0 - t1);
                m.faisceau(
                    oeil,
                    p0,
                    p1,
                    0.3 * f0,
                    0.3 * f1,
                    rgba(c, 1.0 * f0 * f0),
                    rgba(c, 1.0 * f1 * f1),
                );
            }
        }
    }
}

/// Vitrine : une comète par arme à projectiles, en ronde autour de `centre`.
#[allow(clippy::too_many_arguments)]
pub fn vitrine(
    m: &mut MaillageLueur,
    d: &bs_donnees::Donnees,
    teinte: impl Fn(usize) -> [f32; 3],
    centre: Vec3,
    temps: f32,
    oeil: Vec3,
    droite: Vec3,
    haut: Vec3,
) {
    let armes = d.armes.values().enumerate();
    let tireuses = armes.filter(|(_, a)| a.mecanique.projectile().is_some());
    for (n, (i, _)) in tireuses.enumerate() {
        let a = temps * 1.5 + n as f32 * 0.7;
        let r = 4.0 + (n % 3) as f32 * 0.8;
        let tete = centre + Vec3::new(a.sin() * r, HAUTEUR, a.cos() * r);
        let dir = Vec2::new(a.cos(), -a.sin());
        comete(m, tete, dir, 2.0, 0.2, teinte(i), oeil, droite, haut);
    }
}
