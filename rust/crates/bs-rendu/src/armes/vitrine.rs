//! Vitrine des armes (`BS_VITRINE=1`, vérification visuelle et captures) : des effets
//! lancés à tour de rôle près du joueur, et un exemplaire figé de chacun à un instant
//! choisi, posé autour de lui. `BS_ETALAGE=melee|foudre|feu|sol` n'étale qu'un groupe.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use bs_commun::Alea;
use bs_regles::combat::feu::{Feu, Meteore, Tornade};
use bs_regles::combat::pose::{FlecheTombante, PLUIE_CHUTE, Piege, ZonePluie};
use bs_regles::combat::sol::{CHUTE_DUREE, Chute, Nova};
use bs_regles::combat::traque::Orbe;

use super::arcanes::Apparition;
use super::feu::Braise;
use super::foudre::Eclair;
use super::melee::{Coup, Genre};
use super::sol::{self, BlocSouleve};
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
    pub blocs: Vec<BlocSouleve>,
    pub chutes: Vec<Chute>,
    pub novas: Vec<Nova>,
    pub zones: Vec<ZonePluie>,
    pub fleches: Vec<FlecheTombante>,
    pub pieges: Vec<Piege>,
    pub orbes: Vec<Orbe>,
    pub apparitions: Vec<Apparition>,
    minuteur: f32,
    pose: bool,
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

fn chute(x: f32, z: f32, t: f32) -> Chute {
    Chute {
        arme: SANS_ARME,
        x,
        z,
        t,
        tombee: t >= CHUTE_DUREE,
    }
}

fn nova(x: f32, z: f32, temps: f32, au_sol: bool) -> Nova {
    Nova {
        arme: SANS_ARME,
        x,
        z,
        temps,
        sol: au_sol,
        prochain_sol: 0.0,
        facteur: 1.0,
    }
}

impl Etalage {
    fn poser(&mut self, alea: &mut Alea, j: Vec2, couleur: [f32; 3]) {
        let groupe = std::env::var("BS_ETALAGE").ok();
        let montrer = |g: &str| groupe.as_deref().is_none_or(|x| x == g);
        if montrer("sol") {
            self.poser_sol(alea, j);
        }
        if montrer("pose") {
            self.poser_pose(alea, j);
        }
        if montrer("arcanes") {
            self.poser_arcanes(j);
        }
        self.poser_autres(alea, j, couleur, &montrer);
    }

    /// Sol : une onde de blocs soulevés, une lame qui tombe et une plantée, une nova
    /// qui s'étend et une autre qui a gelé le sol.
    fn poser_sol(&mut self, alea: &mut Alea, j: Vec2) {
        for k in 0..6 {
            let (x, z) = (j.x + 0.8 + k as f32 * 0.8, j.y + 1.0);
            sol::soulever(&mut self.blocs, alea, x, z, 1.8);
            let n = self.blocs.len();
            let debut = n.saturating_sub(2);
            for b in &mut self.blocs[debut..] {
                b.age = (5 - k) as f32 * 0.09 + 0.04;
            }
        }
        self.chutes
            .push(chute(j.x - 5.0, j.y + 3.0, CHUTE_DUREE + 0.25));
        self.chutes.push(chute(j.x + 3.0, j.y - 3.5, 0.3));
        self.novas.push(nova(j.x - 3.5, j.y - 4.5, 0.3, false));
        self.novas.push(nova(j.x + 1.5, j.y + 4.0, 1.2, true));
    }

    /// Pluie de flèches (en chute et plantées), piège qui vient de mordre.
    fn poser_pose(&mut self, alea: &mut Alea, j: Vec2) {
        let (cx, cz) = (j.x - 4.0, j.y + 2.5);
        self.zones.push(ZonePluie {
            arme: SANS_ARME,
            x: cx,
            z: cz,
            rayon: 2.5,
            duree: f32::MAX,
            temps: 1.0,
            prochaine_fleche: 0.0,
        });
        for k in 0..16 {
            let a = alea.unite() * TAU;
            let r = 2.3 * alea.unite().sqrt();
            self.fleches.push(FlecheTombante {
                arme: SANS_ARME,
                x: cx + a.sin() * r,
                z: cz + a.cos() * r,
                age: k as f32 * 0.06,
                touche: k as f32 * 0.06 >= PLUIE_CHUTE,
            });
        }
        self.pieges.push(Piege {
            arme: SANS_ARME,
            x: j.x + 2.0,
            z: j.y + 4.3,
            temps: 1.0,
            prochain_tic: 0.42,
        });
    }

    /// Orbes (en orbite autour du joueur, voir `avancer`) et apparitions d'alliés.
    fn poser_arcanes(&mut self, j: Vec2) {
        for k in 0..5 {
            let a = k as f32 * TAU / 5.0;
            self.orbes
                .push(Orbe::figee(SANS_ARME, j.x, j.y, a, 0.0, false));
        }
        self.orbes
            .push(Orbe::figee(SANS_ARME, j.x, j.y, 0.123, 0.0, true));
        let apparition = |x: f32, z: f32, age: f32, depart: bool| Apparition { x, z, age, depart };
        self.apparitions.extend([
            apparition(j.x - 4.5, j.y + 2.5, 0.15, false),
            apparition(j.x + 3.0, j.y + 3.5, 0.4, false),
            apparition(j.x - 3.5, j.y - 4.0, 0.12, true),
        ]);
    }

    fn poser_autres(
        &mut self,
        alea: &mut Alea,
        j: Vec2,
        couleur: [f32; 3],
        montrer: &dyn Fn(&str) -> bool,
    ) {
        let fige = |genre, decalage: Vec2, portee, ouverture, age| Coup {
            genre,
            angle: PI,
            portee,
            ouverture,
            sens: 1.0,
            age,
            decalage,
        };
        if montrer("melee") {
            self.coups.extend([
                fige(Genre::Balayage, Vec2::new(7.0, -2.0), 2.6, 2.4, 0.11),
                fige(Genre::Heurt, Vec2::new(0.0, -6.5), 3.0, 1.6, 0.1),
                fige(Genre::Tourbillon, Vec2::new(5.5, -7.5), 2.4, 0.0, 0.1),
            ]);
        }
        if montrer("foudre") {
            let mut e = Eclair::nouveau(alea, j.x - 9.0, j.y - 2.0, 1.8, couleur);
            e.age = 0.03;
            self.eclairs.push(e);
        }
        if !montrer("feu") {
            return;
        }
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
        for b in &mut self.blocs {
            b.x += d.x;
            b.z += d.y;
        }
        for c in &mut self.chutes {
            c.x += d.x;
            c.z += d.y;
        }
        for n in &mut self.novas {
            n.x += d.x;
            n.z += d.y;
        }
        for z in &mut self.zones {
            z.x += d.x;
            z.z += d.y;
        }
        for f in &mut self.fleches {
            f.x += d.x;
            f.z += d.y;
        }
        for p in &mut self.pieges {
            p.x += d.x;
            p.z += d.y;
        }
        for a in &mut self.apparitions {
            a.x += d.x;
            a.z += d.y;
        }
    }
}

/// Une image de plus de la vitrine (pendant le pas fixe).
pub fn avancer(jeu: &PartieEnCours, v: &mut VisuelsArmes) {
    let j = &jeu.partie.etat.joueur;
    if !v.etalage.pose {
        v.etalage.pose = true;
        let couleur = v.couleur_eclair(FOUDRE_DEFAUT);
        let VisuelsArmes { etalage, alea, .. } = v;
        etalage.poser(alea, Vec2::new(j.x, j.z), couleur);
        etalage.origine = Vec2::new(j.x, j.z);
    }
    v.etalage.suivre(Vec2::new(j.x, j.z));
    // Orbes : en orbite autour du joueur ; la lancée file sur un grand cercle
    for o in &mut v.etalage.orbes {
        o.temps += PAS;
        let (a, r) = if o.lance {
            (o.temps * 1.6, 5.5)
        } else {
            (o.angle + o.temps * 3.0, 2.2)
        };
        o.x = j.x + a.sin() * r;
        o.z = j.z + a.cos() * r;
    }
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
