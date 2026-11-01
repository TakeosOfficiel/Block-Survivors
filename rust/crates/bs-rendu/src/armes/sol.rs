//! Armes qui frappent le sol : blocs soulevés par l'onde de choc (vrais blocs du sol
//! qui jaillissent en basculant, onde de poussière), lame géante (cible marquée, chute
//! avec sillage, impact : éclair, onde, fissures lumineuses, puis elle s'enfonce) et
//! nova de glace (anneau qui s'étend, éclats de glace qui jaillissent sur son passage,
//! sol gelé scintillant).

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use bs_commun::Alea;
use bs_regles::combat::sol::{
    CHUTE_DUREE, CHUTE_FONDU, CHUTE_TENUE, Chute, FISSURE_DUREE_BLOC, NOVA_ECLATS_RESTENT, Nova,
};

use super::lueur::{Forme, MaillageLueur};
use super::outils::{au_sol, hachage, rgba, trait_au_sol};
use super::solides::{Piece, Solides};
use crate::modeles::rendu::Composants;

/// Blocs soulevés : capacité, au plus par soulèvement.
pub const BLOCS: usize = 240;
const BLOCS_PAR_COUP: u32 = 4;
/// Hauteur de départ de la lame (blocs au-dessus du sol).
const LAME_HAUTEUR: f32 = 14.0;
/// Durée des fissures lumineuses après l'impact (s).
const FISSURES: f32 = 0.9;
/// Éclats de glace par nova.
const ECLATS: u32 = 18;

const POUSSIERE: [f32; 3] = [0.9, 0.75, 0.55];
const CIBLE: [f32; 3] = [1.0, 0.25, 0.15];
const ACIER: [f32; 3] = [0.75, 0.85, 1.0];
const IMPACT: [f32; 3] = [1.0, 0.7, 0.35];
const GLACE: [f32; 3] = [0.35, 0.75, 1.0];
const GIVRE: [f32; 3] = [0.7, 0.9, 1.0];

#[derive(Clone, Copy)]
pub struct BlocSouleve {
    pub x: f32,
    pub z: f32,
    pub age: f32,
    angle: f32,
    penche: f32,
    taille: f32,
}

/// Fait jaillir des blocs autour de (x, z).
pub fn soulever(blocs: &mut Vec<BlocSouleve>, alea: &mut Alea, x: f32, z: f32, zone: f32) {
    let n = ((zone / 0.9).round() as u32).clamp(1, BLOCS_PAR_COUP);
    for _ in 0..n {
        if blocs.len() >= BLOCS {
            return;
        }
        let a = alea.unite() * TAU;
        let r = alea.unite().sqrt() * zone * 0.45;
        blocs.push(BlocSouleve {
            x: x + a.sin() * r,
            z: z + a.cos() * r,
            age: 0.0,
            angle: alea.unite() * TAU,
            penche: 0.15 + alea.unite() * 0.4,
            taille: 0.55 + alea.unite() * 0.3,
        });
    }
}

pub fn avancer(blocs: &mut Vec<BlocSouleve>, dt: f32) {
    blocs.retain_mut(|b| {
        b.age += dt;
        b.age < FISSURE_DUREE_BLOC
    });
}

/// Sortie d'un bloc soulevé (0 enfoui, 1 dehors) selon son âge : jaillit, retombe.
fn sortie(age: f32) -> f32 {
    let monte = (age / 0.12).min(1.0);
    let monte = 1.0 - (1.0 - monte).powi(3);
    let rebond = (age / 0.25).min(1.0);
    let rebond = (rebond * PI).sin() * 0.35;
    let descend = ((age - 0.45) / (FISSURE_DUREE_BLOC - 0.45)).clamp(0.0, 1.0);
    (monte + rebond) * (1.0 - descend * descend)
}

/// Lame géante : ce qu'il faut pour la poser.
#[derive(Clone, Copy)]
pub struct Lame<'a> {
    pub chute: &'a Chute,
    pub image: &'a str,
    pub taille: f32,
    pub zone: f32,
}

/// Nova : ce qu'il faut pour la poser.
#[derive(Clone, Copy)]
pub struct Givre<'a> {
    pub nova: &'a Nova,
    pub texture: &'a str,
    pub zone: f32,
    pub expansion: f32,
    pub duree_sol: f32,
}

/// Centre de l'image de la lame : la pointe descend vers la cible puis s'y plante et,
/// à la fin, s'enfonce. `haut` : haut de l'écran (la lame fait face à la caméra).
fn centre_lame(l: &Lame, haut: Vec3) -> Vec3 {
    let c = l.chute;
    let k = (c.t / CHUTE_DUREE).clamp(0.0, 1.0);
    let hauteur = LAME_HAUTEUR * (1.0 - k * k);
    let enfonce = ((c.t - CHUTE_DUREE - CHUTE_TENUE) / CHUTE_FONDU).clamp(0.0, 1.0);
    let pointe = Vec3::new(c.x, hauteur, c.z);
    // La pointe est dans le coin de l'image : à 0,6 côté du centre
    pointe + haut * l.taille * (0.42 - 0.5 * enfonce)
}

/// Position du `k`-ième éclat d'une nova : direction, distance (part de la zone), taille.
fn eclat(n: &Nova, k: u32) -> (f32, f32, f32) {
    let a = (k as f32 + hachage(n.x, n.z, k) * 0.6) * TAU / ECLATS as f32;
    let d = 0.45 + 0.5 * hachage(n.z, n.x, k);
    let h = 1.0 + 1.0 * hachage(n.x + 3.0, n.z, k);
    (a, d, h)
}

/// Hauteur relative d'un éclat (0 : rentré) : jaillit au passage de l'anneau.
fn pousse_eclat(g: &Givre, d: f32) -> f32 {
    let passage = g.expansion * d;
    let age = g.nova.temps - passage;
    let fin = g.expansion + NOVA_ECLATS_RESTENT;
    let monte = (age / 0.08).clamp(0.0, 1.0);
    let rentre = ((fin - g.nova.temps) / 0.15).clamp(0.0, 1.0);
    monte * rentre
}

/// Pièces solides : blocs soulevés, lames, éclats.
#[allow(clippy::too_many_arguments)]
pub fn placer<'a>(
    s: &mut Solides,
    commandes: &mut Commands,
    q: &mut Composants,
    blocs: &[BlocSouleve],
    sol: &str,
    lames: impl Iterator<Item = Lame<'a>>,
    givres: impl Iterator<Item = Givre<'a>>,
    camera: Quat,
) {
    for b in blocs {
        let h = sortie(b.age);
        if h <= 0.0 {
            continue;
        }
        let rotation = Quat::from_rotation_y(b.angle) * Quat::from_rotation_x(b.penche);
        let y = -b.taille * 0.55 + h * b.taille * 0.9;
        let tf = Transform::from_xyz(b.x, y, b.z)
            .with_rotation(rotation)
            .with_scale(Vec3::splat(b.taille));
        s.poser(commandes, q, sol, Piece::Cube, tf);
    }
    let haut = camera * Vec3::Y;
    // L'image de l'épée va de la garde (bas gauche) à la pointe (haut droit) : on la
    // tourne pour que la pointe regarde le sol
    let rotation = camera * Quat::from_rotation_z(-0.75 * PI);
    for l in lames {
        if l.chute.t < 0.0 {
            continue;
        }
        let tf = Transform::from_translation(centre_lame(&l, haut))
            .with_rotation(rotation)
            .with_scale(Vec3::splat(l.taille));
        s.poser(commandes, q, l.image, Piece::Image, tf);
    }
    for g in givres {
        for k in 0..ECLATS {
            let (a, d, h) = eclat(g.nova, k);
            let p = pousse_eclat(&g, d);
            if p <= 0.0 {
                continue;
            }
            let r = g.zone * d;
            let (dx, dz) = (a.sin(), a.cos());
            let hauteur = h * p;
            // Penché vers l'extérieur, la base au sol
            let rotation = Quat::from_rotation_y(a) * Quat::from_rotation_x(0.45);
            let base = Vec3::new(g.nova.x + dx * r, 0.0, g.nova.z + dz * r);
            let tf = Transform::from_translation(base + rotation * Vec3::Y * (hauteur * 0.5))
                .with_rotation(rotation * Quat::from_rotation_y(PI / 4.0))
                .with_scale(Vec3::new(0.55 * p, hauteur, 0.55 * p));
            s.poser(commandes, q, g.texture, Piece::Cube, tf);
        }
    }
}

/// Lueurs : poussière des blocs, cible, sillage et impact des lames, anneau et givre
/// des novas.
#[allow(clippy::too_many_arguments)]
pub fn dessiner<'a>(
    m: &mut MaillageLueur,
    blocs: &[BlocSouleve],
    lames: impl Iterator<Item = Lame<'a>>,
    givres: impl Iterator<Item = Givre<'a>>,
    temps: f32,
    oeil: Vec3,
    droite: Vec3,
    haut: Vec3,
) {
    for b in blocs {
        let k = (b.age / 0.35).min(1.0);
        if k < 1.0 {
            let r = b.taille * (0.6 + 1.2 * k);
            au_sol(
                m,
                b.x,
                b.z,
                r,
                rgba(POUSSIERE, 0.55 * (1.0 - k)),
                Forme::Anneau,
                0.3,
            );
        }
    }

    for l in lames {
        let c = l.chute;
        let k = (c.t / CHUTE_DUREE).clamp(0.0, 1.0);
        let fin = CHUTE_DUREE + CHUTE_TENUE + CHUTE_FONDU;
        if c.t < CHUTE_DUREE {
            // Cible qui se resserre, de plus en plus vive
            let r = l.zone * (1.0 + 0.4 * (1.0 - k));
            let a = 0.5 + 1.2 * k.max(0.0);
            au_sol(m, c.x, c.z, r, rgba(CIBLE, a), Forme::Anneau, 0.07);
            au_sol(
                m,
                c.x,
                c.z,
                l.zone,
                rgba(CIBLE, 0.3 * k),
                Forme::Disque,
                0.0,
            );
            if c.t >= 0.0 {
                // Sillage : le long de la lame, au-dessus d'elle
                let pointe = Vec3::new(c.x, LAME_HAUTEUR * (1.0 - k * k), c.z);
                let queue = pointe + haut * l.taille * 2.2;
                m.faisceau(
                    oeil,
                    pointe,
                    queue,
                    0.5,
                    0.05,
                    rgba(ACIER, 1.1),
                    rgba(ACIER, 0.0),
                );
            }
            continue;
        }
        let t = c.t - CHUTE_DUREE;
        // Éclair blanc, onde, poussière
        let flash = (1.0 - t / 0.2).max(0.0);
        au_sol(
            m,
            c.x,
            c.z,
            l.zone * 1.6,
            rgba(IMPACT, 2.2 * flash),
            Forme::Disque,
            0.0,
        );
        let o = (t / 0.45).min(1.0);
        if o < 1.0 {
            let r = l.zone * (0.3 + 1.6 * (1.0 - (1.0 - o).powi(3)));
            au_sol(
                m,
                c.x,
                c.z,
                r,
                rgba(IMPACT, 2.0 * (1.0 - o)),
                Forme::Anneau,
                0.12,
            );
            let r2 = l.zone * (0.2 + 1.1 * o);
            au_sol(
                m,
                c.x,
                c.z,
                r2,
                rgba(POUSSIERE, 0.6 * (1.0 - o)),
                Forme::Anneau,
                0.35,
            );
        }
        // Fissures lumineuses en étoile, qui refroidissent
        let f = 1.0 - (t / FISSURES).min(1.0);
        if f > 0.0 {
            let centre = Vec2::new(c.x, c.z);
            for k in 0..9u32 {
                let a = (k as f32 + hachage(c.x, c.z, k) * 0.7) * TAU / 9.0;
                let long = l.zone * (0.7 + 0.6 * hachage(c.z, c.x, k));
                let dir = Vec2::new(a.sin(), a.cos());
                let coude = centre
                    + dir * long * 0.5
                    + dir.perp() * (hachage(c.x, k as f32, 1) - 0.5) * 0.6;
                let bout = centre + dir * long;
                let couleur = rgba([1.0, 0.45 + 0.35 * f, 0.15 * f], 1.6 * f);
                trait_au_sol(m, centre, coude, 0.16, 0.11, couleur);
                trait_au_sol(m, coude, bout, 0.11, 0.03, couleur);
            }
        }
        // Reflet sur la lame plantée, qui s'éteint en s'enfonçant
        let reste = ((fin - c.t) / CHUTE_FONDU).clamp(0.0, 1.0);
        let lueur = Vec3::new(c.x, 0.3, c.z) + haut * l.taille * 0.15;
        m.carre(
            lueur,
            droite * 1.6,
            haut * 1.6,
            rgba(IMPACT, 0.5 * reste),
            Forme::Disque,
            0.0,
        );
    }

    for g in givres {
        let n = g.nova;
        let x = (n.temps / g.expansion).min(1.0);
        if x < 1.0 {
            let r = g.zone * (1.0 - (1.0 - x).powi(2));
            au_sol(m, n.x, n.z, r, rgba(GLACE, 1.8), Forme::Anneau, 0.1);
            au_sol(m, n.x, n.z, r * 1.05, rgba(GIVRE, 0.6), Forme::Anneau, 0.35);
            au_sol(m, n.x, n.z, r, rgba(GLACE, 0.18), Forme::Disque, 0.0);
        }
        // Reflets au bout des éclats
        for k in 0..ECLATS {
            let (a, d, h) = eclat(n, k);
            let p = pousse_eclat(&g, d);
            if p <= 0.0 {
                continue;
            }
            let age = n.temps - g.expansion * d;
            let eclair = (1.0 - age / 0.3).max(0.0);
            let r = g.zone * d + h * p * 0.45;
            let pointe = Vec3::new(n.x + a.sin() * r, h * p * 0.85, n.z + a.cos() * r);
            m.carre(
                pointe,
                droite * 0.35,
                haut * 0.35,
                rgba(GIVRE, 1.5 * eclair + 0.25),
                Forme::Etincelle,
                0.0,
            );
        }
        // Sol gelé : nappe pâle, bord net, paillettes qui scintillent
        if !n.sol {
            continue;
        }
        let debut = g.expansion;
        let fin = debut + NOVA_ECLATS_RESTENT + g.duree_sol;
        let f = ((n.temps - debut * 0.5) / 0.3).clamp(0.0, 1.0)
            * ((fin - n.temps) / 0.5).clamp(0.0, 1.0);
        if f <= 0.0 {
            continue;
        }
        au_sol(
            m,
            n.x,
            n.z,
            g.zone * 1.1,
            rgba(GIVRE, 0.45 * f),
            Forme::Disque,
            0.0,
        );
        au_sol(
            m,
            n.x,
            n.z,
            g.zone,
            rgba(GLACE, 0.7 * f),
            Forme::Anneau,
            0.05,
        );
        for k in 0..14u32 {
            let a = hachage(n.x, n.z, k + 40) * TAU;
            let d = hachage(n.z, n.x, k + 40).sqrt() * g.zone * 0.95;
            let scintille = (temps * 3.0 + k as f32 * 1.3).sin().max(0.0).powi(4);
            let p = Vec3::new(n.x + a.sin() * d, 0.12, n.z + a.cos() * d);
            m.carre(
                p,
                droite * 0.22,
                haut * 0.22,
                rgba(GIVRE, 1.6 * scintille * f),
                Forme::Etincelle,
                0.0,
            );
        }
    }
}
