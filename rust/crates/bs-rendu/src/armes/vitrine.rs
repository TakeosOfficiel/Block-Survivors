//! Vitrine des armes (`BS_VITRINE=1`, vérification visuelle et captures) : des effets
//! lancés à tour de rôle près du joueur, et un exemplaire figé de chacun à un instant
//! choisi, posé autour de lui.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use bs_commun::Alea;
use bs_regles::combat::feu::{Feu, Meteore, Tornade};

use super::feu::Braise;
use super::foudre::Eclair;
use super::melee::{Coup, Genre};
use super::{FOUDRE_DEFAUT, VisuelsArmes};
use crate::{PAS, PartieEnCours};

/// Arme absente de l'arsenal : la zone vaut 1 (voir `VisuelsArmes::zone`).
pub const SANS_ARME: usize = usize::MAX;

#[derive(Default)]
pub struct Etalage {
    pub coups: Vec<Coup>,
    pub eclairs: Vec<Eclair>,
    pub tornades: Vec<Tornade>,
    pub meteores: Vec<Meteore>,
    pub feux: Vec<Feu>,
    pub braises: Vec<Braise>,
    minuteur: f32,
    /// Position du joueur à la dernière image : l'étalage le suit.
    origine: Vec2,
}

fn tornade(x: f32, z: f32) -> Tornade {
    Tornade {
        arme: SANS_ARME,
        x,
        z,
        dir_x: 0.0,
        dir_z: 1.0,
        temps: 1.0,
        prochain_coup: 0.0,
        facteur: 1.0,
        zone: 1.5,
        duree: f32::MAX,
        depuis_trace: 0.0,
    }
}

fn feu(x: f32, z: f32, temps: f32) -> Feu {
    Feu {
        arme: SANS_ARME,
        x,
        z,
        temps,
        prochain_tic: 0.0,
        facteur: 1.0,
        zone: 1.6,
        duree: f32::MAX,
    }
}

impl Etalage {
    fn poser(&mut self, alea: &mut Alea, j: Vec2, couleur: [f32; 3]) {
        let fige = |genre, decalage: Vec2, portee, ouverture, age| Coup {
            genre,
            angle: PI,
            portee,
            ouverture,
            sens: 1.0,
            age,
            decalage,
        };
        self.coups.extend([
            fige(Genre::Balayage, Vec2::new(7.0, -2.0), 2.6, 2.4, 0.11),
            fige(Genre::Heurt, Vec2::new(0.0, -6.5), 3.0, 1.6, 0.1),
            fige(Genre::Tourbillon, Vec2::new(5.5, -7.5), 2.4, 0.0, 0.1),
        ]);
        let mut e = Eclair::nouveau(alea, j.x - 9.0, j.y - 2.0, 1.8, couleur);
        e.age = 0.03;
        self.eclairs.push(e);
        // Feu : tornade, météore en chute, feu au sol, impact, traînée de braises
        self.tornades.push(tornade(j.x - 4.5, j.y + 0.5));
        self.meteores.push(Meteore {
            arme: SANS_ARME,
            x: j.x - 2.0,
            z: j.y + 3.5,
            t: 0.45,
            facteur: 1.0,
            taille: 1.0,
        });
        self.feux.push(feu(j.x + 3.0, j.y + 4.0, 1.0));
        self.feux.push(feu(j.x - 7.0, j.y + 3.0, 0.12));
        for k in 0..8 {
            let (x, z) = (
                j.x - 4.5 + k as f32 * 0.5,
                j.y + 0.5 + (k as f32 * 0.7).sin(),
            );
            self.braises.push(Braise {
                x,
                z,
                age: (8 - k) as f32 * 0.22,
            });
        }
    }
}

impl Etalage {
    fn suivre(&mut self, joueur: Vec2) {
        let d = joueur - self.origine;
        self.origine = joueur;
        for e in &mut self.eclairs {
            e.deplacer(Vec3::new(d.x, 0.0, d.y));
        }
        for t in &mut self.tornades {
            t.x += d.x;
            t.z += d.y;
        }
        for m in &mut self.meteores {
            m.x += d.x;
            m.z += d.y;
        }
        for f in &mut self.feux {
            f.x += d.x;
            f.z += d.y;
        }
        for b in &mut self.braises {
            b.x += d.x;
            b.z += d.y;
        }
    }
}

/// Une image de plus de la vitrine (pendant le pas fixe).
pub fn avancer(jeu: &PartieEnCours, v: &mut VisuelsArmes) {
    let j = &jeu.partie.etat.joueur;
    if v.etalage.coups.is_empty() {
        let couleur = v.couleur_eclair(FOUDRE_DEFAUT);
        let VisuelsArmes { etalage, alea, .. } = v;
        etalage.poser(alea, Vec2::new(j.x, j.z), couleur);
        etalage.origine = Vec2::new(j.x, j.z);
    }
    v.etalage.suivre(Vec2::new(j.x, j.z));
    v.etalage.minuteur -= PAS;
    if v.etalage.minuteur > 0.0 {
        return;
    }
    v.etalage.minuteur = 0.6;
    let angle = v.alea.unite() * TAU;
    match (v.alea.unite() * 3.0) as u32 {
        0 => v.coup(Genre::Balayage, angle, 2.6, 2.4),
        1 => v.coup(Genre::Heurt, angle, 3.0, 1.6),
        _ => v.coup(Genre::Tourbillon, angle, 2.4, 0.0),
    }
    v.eclair(j.x + 3.0, j.z - 3.0, 1.8, FOUDRE_DEFAUT);
}
