//! Armes posées : pluie de flèches (cercle de visée cerné de tirets qui tournent,
//! flèches qui tombent en traits de lumière puis restent plantées, petit éclat à
//! chaque impact) et pièges de ronces (buissons en croix qui poussent, cercle
//! d'épines qui pulse à chaque morsure).

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use bs_regles::combat::pose::{
    FlecheTombante, PLUIE_CERCLE_FONDU, PLUIE_CERCLE_RESTE, PLUIE_CHUTE, PLUIE_ENFONCEMENT,
    PLUIE_PLANTEE, Piege, ZonePluie,
};

use super::lueur::{Forme, MaillageLueur};
use super::outils::{au_sol, hachage, rgba, trait_au_sol};
use super::solides::{Piece, Solides};
use crate::modeles::rendu::Composants;

/// D'où viennent les flèches (décalage depuis le point d'impact, blocs).
const CIEL: Vec3 = Vec3::new(0.9, 7.0, -1.4);
/// Côté de l'image d'une flèche (blocs).
const FLECHE: f32 = 0.95;
/// Tirets du cercle de visée.
const TIRETS: u32 = 18;

const VISEE: [f32; 3] = [1.0, 0.85, 0.55];
const BAIES: [f32; 3] = [1.0, 0.2, 0.25];

/// Flèche et son image.
#[derive(Clone, Copy)]
pub struct Fleche<'a> {
    pub fleche: &'a FlecheTombante,
    pub image: &'a str,
    /// Teinte de l'arme (linéaire).
    pub teinte: [f32; 3],
}

/// Piège : ce qu'il faut pour le dessiner.
#[derive(Clone, Copy)]
pub struct Ronces<'a> {
    pub piege: &'a Piege,
    pub image: &'a str,
    pub zone: f32,
    pub duree: f32,
    pub intervalle: f32,
    /// Teinte de l'arme (linéaire).
    pub teinte: [f32; 3],
}

/// Fondu d'un objet qui vit `duree` secondes.
fn fondu(temps: f32, duree: f32, entree: f32, sortie: f32) -> f32 {
    (temps / entree).clamp(0.0, 1.0) * ((duree - temps) / sortie).clamp(0.0, 1.0)
}

/// Position de la pointe d'une flèche : tombe du ciel, puis plantée, puis s'enfonce.
fn pointe(f: &FlecheTombante) -> Vec3 {
    let sol = Vec3::new(f.x, 0.0, f.z);
    if f.age < PLUIE_CHUTE {
        let k = f.age / PLUIE_CHUTE;
        return sol + CIEL * (1.0 - k * k);
    }
    let enfonce = ((f.age - PLUIE_CHUTE - PLUIE_PLANTEE) / PLUIE_ENFONCEMENT).clamp(0.0, 1.0);
    sol - Vec3::Y * (0.15 + 0.5 * enfonce)
}

/// Buissons d'un piège : position et taille.
fn buissons(x: f32, z: f32, zone: f32) -> impl Iterator<Item = (Vec3, f32)> {
    let n = 3 + (hachage(x, z, 99) * 2.0) as u32;
    (0..n).map(move |k| {
        let a = (k as f32 + hachage(x, z, k) * 0.5) * TAU / n as f32;
        let d = if k == 0 { 0.0 } else { zone * 0.5 };
        let taille = 0.9 + 0.3 * hachage(z, x, k);
        (Vec3::new(x + a.sin() * d, 0.0, z + a.cos() * d), taille)
    })
}

/// Pièces solides : flèches (images tournées pointe en bas) et buissons en croix.
pub fn placer<'a>(
    s: &mut Solides,
    commandes: &mut Commands,
    q: &mut Composants,
    fleches: impl Iterator<Item = Fleche<'a>>,
    ronces: impl Iterator<Item = Ronces<'a>>,
    camera: Quat,
) {
    let haut = camera * Vec3::Y;
    for f in fleches {
        let fl = f.fleche;
        if fl.age >= PLUIE_CHUTE + PLUIE_PLANTEE + PLUIE_ENFONCEMENT {
            continue;
        }
        // Image de la flèche : de l'empennage (bas gauche) à la pointe (haut droit)
        let roulis = (hachage(fl.x, fl.z, 7) - 0.5) * 0.5;
        let rotation = camera * Quat::from_rotation_z(-0.75 * PI + roulis);
        let centre = pointe(fl) + haut * FLECHE * 0.45;
        let tf = Transform::from_translation(centre)
            .with_rotation(rotation)
            .with_scale(Vec3::splat(FLECHE));
        s.poser(commandes, q, f.image, Piece::Image, tf);
    }
    for r in ronces {
        let p = r.piege;
        let pousse = fondu(p.temps, r.duree, 0.25, 0.35);
        for (k, (pos, taille)) in buissons(p.x, p.z, r.zone).enumerate() {
            let t = taille * pousse;
            if t <= 0.01 {
                continue;
            }
            let lacet = hachage(p.x, p.z, k as u32 + 20) * PI;
            for croix in [PI / 4.0, -PI / 4.0] {
                let tf = Transform::from_translation(pos + Vec3::Y * (t * 0.5))
                    .with_rotation(Quat::from_rotation_y(lacet + croix))
                    .with_scale(Vec3::splat(t));
                s.poser(commandes, q, r.image, Piece::Image, tf);
            }
        }
    }
}

/// Lueurs : cercles de visée, traits des flèches, impacts ; cercles d'épines.
#[allow(clippy::too_many_arguments)]
pub fn dessiner<'a>(
    m: &mut MaillageLueur,
    pluies: impl Iterator<Item = &'a ZonePluie>,
    fleches: impl Iterator<Item = Fleche<'a>>,
    ronces: impl Iterator<Item = Ronces<'a>>,
    temps: f32,
    oeil: Vec3,
    droite: Vec3,
    haut: Vec3,
) {
    for z in pluies {
        let vie = z.duree + PLUIE_CHUTE + PLUIE_CERCLE_RESTE + PLUIE_CERCLE_FONDU;
        let f = fondu(z.temps, vie, 0.2, PLUIE_CERCLE_FONDU);
        // La visée se resserre en arrivant
        let r = z.rayon * (1.0 + 0.3 * (1.0 - (z.temps / 0.3).min(1.0)));
        au_sol(m, z.x, z.z, r, rgba(VISEE, 0.9 * f), Forme::Anneau, 0.05);
        au_sol(m, z.x, z.z, r, rgba(VISEE, 0.12 * f), Forme::Disque, 0.0);
        let centre = Vec2::new(z.x, z.z);
        let tour = temps * 0.6;
        for k in 0..TIRETS {
            let a0 = tour + k as f32 * TAU / TIRETS as f32;
            let a1 = a0 + TAU / TIRETS as f32 * 0.5;
            let p0 = centre + Vec2::new(a0.sin(), a0.cos()) * r * 0.86;
            let p1 = centre + Vec2::new(a1.sin(), a1.cos()) * r * 0.86;
            trait_au_sol(m, p0, p1, 0.06, 0.06, rgba(VISEE, 0.7 * f));
        }
    }

    for fl in fleches {
        let f = fl.fleche;
        if f.age < PLUIE_CHUTE {
            // Trait de lumière derrière la flèche
            let tete = pointe(f);
            let queue = tete + CIEL.normalize() * 2.2;
            m.faisceau(
                oeil,
                tete,
                queue,
                0.12,
                0.02,
                rgba(fl.teinte, 1.2),
                rgba(fl.teinte, 0.0),
            );
            // Point visé
            au_sol(m, f.x, f.z, 0.3, rgba(VISEE, 0.6), Forme::Disque, 0.0);
            continue;
        }
        let t = (f.age - PLUIE_CHUTE) / 0.2;
        if t < 1.0 {
            let r = 0.25 + 0.7 * t;
            au_sol(
                m,
                f.x,
                f.z,
                r,
                rgba(fl.teinte, 1.4 * (1.0 - t)),
                Forme::Anneau,
                0.2,
            );
            let p = Vec3::new(f.x, 0.25, f.z);
            m.carre(
                p,
                droite * 0.5,
                haut * 0.5,
                rgba(fl.teinte, 1.6 * (1.0 - t)),
                Forme::Etincelle,
                0.0,
            );
        }
    }

    for r in ronces {
        let p = r.piege;
        let f = fondu(p.temps, r.duree, 0.25, 0.35);
        // Morsure : juste après chaque tic (le compte à rebours repart de l'intervalle)
        let depuis = (r.intervalle - p.prochain_tic).max(0.0);
        let morsure = (-depuis * 9.0).exp();
        let zone = r.zone;
        au_sol(
            m,
            p.x,
            p.z,
            zone,
            rgba(r.teinte, (0.5 + 0.9 * morsure) * f),
            Forme::Anneau,
            0.06,
        );
        au_sol(
            m,
            p.x,
            p.z,
            zone,
            rgba(r.teinte, (0.08 + 0.25 * morsure) * f),
            Forme::Disque,
            0.0,
        );
        // Épines qui jaillissent du bord à chaque morsure
        let n = ((zone * 5.0) as u32).clamp(8, 20);
        for k in 0..n {
            let a = (k as f32 + hachage(p.x, p.z, k + 60) * 0.6) * TAU / n as f32;
            let d = zone * (0.75 + 0.2 * hachage(p.z, p.x, k + 60));
            let pied = Vec3::new(p.x + a.sin() * d, 0.0, p.z + a.cos() * d);
            let h = 0.25 + 0.45 * morsure;
            let bout = pied + Vec3::new(a.sin() * 0.15, h, a.cos() * 0.15);
            m.faisceau(
                oeil,
                pied,
                bout,
                0.1,
                0.0,
                rgba(r.teinte, 1.2 * f),
                rgba(r.teinte, 0.4 * f),
            );
        }
        // Baies qui luisent dans les buissons
        for (k, (pos, taille)) in buissons(p.x, p.z, r.zone).enumerate() {
            let b = pos + Vec3::new(0.0, taille * 0.45 * f, 0.0);
            let vacille = 0.7 + 0.3 * (temps * 3.0 + k as f32).sin();
            m.carre(
                b,
                droite * 0.35,
                haut * 0.35,
                rgba(BAIES, 0.6 * f * vacille),
                Forme::Disque,
                0.0,
            );
        }
    }
}
