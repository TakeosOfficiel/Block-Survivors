//! Visuels des armes, neufs (pas une copie de la version TS) : la simulation dit quoi
//! et où (`Evenement::Effet`), ce module choisit l'allure. Presque tout est dessiné en
//! lueurs calculées par le shader (`lueur`), ajoutées à la scène, en un seul appel de
//! dessin ; les pools ont une taille fixe et ne réallouent jamais.

mod foudre;
mod lueur;
mod melee;

use bevy::prelude::*;
use bs_commun::Alea;
use bs_donnees::armes::Mecanique;
use bs_regles::evenements::{EffetArme, Evenement};

use crate::arene::couleur_lineaire;
use crate::camera::Cible;
use crate::effets::Effets;
use crate::materiau::ParametresBloc;
use crate::{PAS, PartieEnCours};
use foudre::Eclair;
pub use lueur::PluginLueur;
use lueur::{MaillageLueur, MateriauLueur, ParametresLueur};
use melee::{Coup, Genre};

/// Couleur des éclairs sans arme de foudre (vitrine).
const FOUDRE_DEFAUT: u32 = 0x9fd8ff;
/// Intensité des éclairs au-dessus de leur couleur de base.
const FOUDRE_INTENSITE: f32 = 1.4;

#[derive(Resource)]
pub struct VisuelsArmes {
    lueur: MaillageLueur,
    coups: Vec<Coup>,
    eclairs: Vec<Eclair>,
    alea: Alea,
    /// Sens du prochain balayage (alterné).
    sens: f32,
    /// Vitrine : temps avant la prochaine démonstration.
    minuteur: f32,
    /// Vitrine : exemplaires figés à un instant choisi (captures).
    etalage_coups: Vec<Coup>,
    etalage_eclairs: Vec<Eclair>,
}

impl VisuelsArmes {
    fn coup(&mut self, genre: Genre, angle: f32, portee: f32, ouverture: f32) {
        if self.coups.len() >= melee::CAPACITE {
            return;
        }
        self.sens = -self.sens;
        self.coups.push(Coup {
            genre,
            angle,
            portee,
            ouverture,
            sens: self.sens,
            age: 0.0,
            decalage: Vec2::ZERO,
        });
    }

    fn eclair(&mut self, x: f32, z: f32, zone: f32, couleur: u32) {
        if self.eclairs.len() >= foudre::CAPACITE {
            return;
        }
        let c = couleur_lineaire(couleur);
        let rgb = [c[0], c[1], c[2]].map(|v| v * FOUDRE_INTENSITE);
        let e = Eclair::nouveau(&mut self.alea, x, z, zone, rgb);
        self.eclairs.push(e);
    }
}

pub fn construire(
    mut commandes: Commands,
    jeu: Res<PartieEnCours>,
    mut maillages: ResMut<Assets<Mesh>>,
    mut materiaux: ResMut<Assets<MateriauLueur>>,
) {
    let b = ParametresBloc::de_l_arene(jeu.partie.etat.arene).brouillard;
    let materiau = materiaux.add(MateriauLueur {
        parametres: ParametresLueur {
            brouillard: Vec4::new(b.x, b.y, 1.0, 0.0),
        },
    });
    let lueur = MaillageLueur::nouveau(&mut commandes, &mut maillages, materiau);
    commandes.insert_resource(VisuelsArmes {
        lueur,
        coups: Vec::with_capacity(melee::CAPACITE),
        eclairs: Vec::with_capacity(foudre::CAPACITE),
        alea: Alea::nouveau(u64::from(jeu.graine), 13),
        sens: 1.0,
        minuteur: 0.0,
        etalage_coups: Vec::new(),
        etalage_eclairs: Vec::new(),
    });
}

/// Après chaque pas : les effets d'armes empilés par la simulation.
pub fn recevoir(jeu: Res<PartieEnCours>, mut v: ResMut<VisuelsArmes>, effets: Res<Effets>) {
    let p = &jeu.partie;
    for ev in &p.ev.liste {
        let Evenement::Effet { arme, effet, x, z } = *ev else {
            continue;
        };
        match effet {
            EffetArme::Balayage {
                angle,
                portee,
                ouverture,
            } => v.coup(Genre::Balayage, angle, portee, ouverture),
            EffetArme::Heurt {
                angle,
                portee,
                ouverture,
            } => v.coup(Genre::Heurt, angle, portee, ouverture),
            EffetArme::Tourbillon { portee } => {
                let angle = v.alea.unite() * std::f32::consts::TAU;
                v.coup(Genre::Tourbillon, angle, portee, 0.0);
            }
            EffetArme::Eclair { zone } => {
                let couleur = match p.arsenal.armes.get(arme).map(|a| &a.def.mecanique) {
                    Some(Mecanique::Foudre { couleur }) => *couleur,
                    _ => FOUDRE_DEFAUT,
                };
                v.eclair(x, z, zone, couleur);
            }
            EffetArme::Soulevement { .. } | EffetArme::Brulure => {}
        }
    }
    if effets.vitrine {
        vitrine(&jeu, &mut v);
    }
}

/// `BS_VITRINE` : chaque effet à tour de rôle autour du joueur, et un exemplaire figé
/// de chacun derrière lui.
fn vitrine(jeu: &PartieEnCours, v: &mut VisuelsArmes) {
    let j = &jeu.partie.etat.joueur;
    if v.etalage_coups.is_empty() {
        let fige = |genre, decalage: Vec2, portee, ouverture, age| Coup {
            genre,
            angle: std::f32::consts::PI,
            portee,
            ouverture,
            sens: 1.0,
            age,
            decalage,
        };
        v.etalage_coups.extend([
            fige(Genre::Balayage, Vec2::new(7.0, -2.0), 2.6, 2.4, 0.11),
            fige(Genre::Heurt, Vec2::new(0.0, -6.5), 3.0, 1.6, 0.1),
            fige(Genre::Tourbillon, Vec2::new(5.5, -7.5), 2.4, 0.0, 0.1),
        ]);
        let couleur = couleur_lineaire(FOUDRE_DEFAUT).map(|c| c * FOUDRE_INTENSITE);
        let mut e = Eclair::nouveau(
            &mut v.alea,
            j.x - 7.0,
            j.z + 1.5,
            1.8,
            [couleur[0], couleur[1], couleur[2]],
        );
        e.age = 0.03;
        v.etalage_eclairs.push(e);
    }
    v.minuteur -= PAS;
    if v.minuteur > 0.0 {
        return;
    }
    v.minuteur = 0.6;
    let angle = v.alea.unite() * std::f32::consts::TAU;
    match (v.alea.unite() * 3.0) as u32 {
        0 => v.coup(Genre::Balayage, angle, 2.6, 2.4),
        1 => v.coup(Genre::Heurt, angle, 3.0, 1.6),
        _ => v.coup(Genre::Tourbillon, angle, 2.4, 0.0),
    }
    v.eclair(j.x + 3.0, j.z - 3.0, 1.8, FOUDRE_DEFAUT);
}

/// Vie et dessin de l'image.
pub fn dessiner(
    temps: Res<Time>,
    mut v: ResMut<VisuelsArmes>,
    cible: Res<Cible>,
    camera: Query<&Transform, With<Camera3d>>,
    mut maillages: ResMut<Assets<Mesh>>,
    mut visibilites: Query<&mut Visibility>,
) {
    let dt = temps.delta_secs();
    let tf = camera.single().copied().unwrap_or_default();
    let (oeil, droite, haut) = (tf.translation, tf.rotation * Vec3::X, tf.rotation * Vec3::Y);
    let v = &mut *v;
    melee::avancer(&mut v.coups, dt);
    foudre::avancer(&mut v.eclairs, dt);
    v.lueur.commencer();
    melee::dessiner(&mut v.lueur, &v.coups, cible.0);
    foudre::dessiner(&mut v.lueur, &v.eclairs, oeil, droite, haut);
    melee::dessiner(&mut v.lueur, &v.etalage_coups, cible.0);
    foudre::dessiner(&mut v.lueur, &v.etalage_eclairs, oeil, droite, haut);
    v.lueur.terminer(&mut maillages, &mut visibilites);
}
