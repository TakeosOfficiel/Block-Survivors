//! Visuels des armes, neufs (pas une copie de la version TS) : la simulation dit quoi
//! et où (`Evenement::Effet`), ce module choisit l'allure. Presque tout est dessiné en
//! lueurs calculées par le shader (`lueur`), ajoutées à la scène, en un seul appel de
//! dessin ; les pools ont une taille fixe et ne réallouent jamais.

mod arcanes;
mod dessin;
mod feu;
mod foudre;
mod lueur;
mod melee;
mod outils;
mod posees;
mod sol;
mod solides;
mod vitrine;
mod vues;

use bevy::prelude::*;
use bs_commun::Alea;
use bs_donnees::Donnees;
use bs_donnees::armes::Mecanique;
use bs_regles::evenements::{EffetArme, Evenement};

use crate::PartieEnCours;
use crate::arene::couleur_lineaire;
use crate::effets::Effets;
use crate::materiau::ParametresBloc;
use crate::materiau_entite::MateriauEntite;
use crate::textures::Textures;
use arcanes::Arcanes;
pub use dessin::{dessiner, placer_solides};
use feu::Braise;
use foudre::Eclair;
pub use lueur::PluginLueur;
use lueur::{MaillageLueur, MateriauLueur, ParametresLueur};
use melee::{Coup, Genre};
use sol::BlocSouleve;
use solides::{Fabrique, Solides};
use vitrine::Etalage;

/// Couleur des éclairs sans arme de foudre (vitrine).
const FOUDRE_DEFAUT: u32 = 0x9fd8ff;
/// Intensité des éclairs au-dessus de leur couleur de base.
const FOUDRE_INTENSITE: f32 = 1.4;

#[derive(Resource)]
pub struct VisuelsArmes {
    lueur: MaillageLueur,
    coups: Vec<Coup>,
    eclairs: Vec<Eclair>,
    braises: Vec<Braise>,
    blocs: Vec<BlocSouleve>,
    arcanes: Arcanes,
    solides: Solides,
    /// Orientation de la caméra à la dernière image (`dessiner`, avant `placer_solides`).
    camera: Quat,
    /// Texture des blocs soulevés (le sol de l'arène).
    sol: &'static str,
    alea: Alea,
    /// Sens du prochain balayage (alterné).
    sens: f32,
    /// Vitrine : exemplaires figés et démonstrations.
    etalage: Etalage,
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

    fn couleur_eclair(&self, couleur: u32) -> [f32; 3] {
        let c = couleur_lineaire(couleur);
        [c[0], c[1], c[2]].map(|v| v * FOUDRE_INTENSITE)
    }

    fn eclair(&mut self, x: f32, z: f32, zone: f32, couleur: u32) {
        if self.eclairs.len() >= foudre::CAPACITE {
            return;
        }
        let rgb = self.couleur_eclair(couleur);
        let e = Eclair::nouveau(&mut self.alea, x, z, zone, rgb);
        self.eclairs.push(e);
    }
}

/// Textures des pièces solides (blocs, images).
pub fn demander_textures(d: &'static Donnees, serveur: &AssetServer, t: &mut Textures) {
    for chemin in solides::textures(d) {
        t.demander(serveur, chemin);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn construire(
    mut commandes: Commands,
    jeu: Res<PartieEnCours>,
    serveur: Res<AssetServer>,
    textures: Res<Textures>,
    images: Res<Assets<Image>>,
    mut maillages: ResMut<Assets<Mesh>>,
    mut materiaux: ResMut<Assets<MateriauLueur>>,
    mut materiaux_entites: ResMut<Assets<MateriauEntite>>,
) {
    let ambiance = ParametresBloc::pour_entites(jeu.partie.etat.arene);
    let b = ambiance.brouillard;
    let materiau = materiaux.add(MateriauLueur {
        parametres: ParametresLueur {
            brouillard: Vec4::new(b.x, b.y, 1.0, 0.0),
        },
    });
    let lueur = MaillageLueur::nouveau(&mut commandes, &mut maillages, materiau);
    let sol = jeu.partie.texture_sol();
    let solides = Solides::nouveaux(
        jeu.partie.d,
        sol,
        &mut Fabrique {
            ambiance,
            serveur: &serveur,
            textures: &textures,
            images: &images,
            maillages: &mut maillages,
            materiaux: &mut materiaux_entites,
        },
    );
    commandes.insert_resource(VisuelsArmes {
        lueur,
        coups: Vec::with_capacity(melee::CAPACITE),
        eclairs: Vec::with_capacity(foudre::CAPACITE),
        braises: Vec::with_capacity(feu::BRAISES),
        blocs: Vec::with_capacity(sol::BLOCS),
        arcanes: Arcanes::nouvelles(),
        solides,
        sol,
        camera: Quat::IDENTITY,
        alea: Alea::nouveau(u64::from(jeu.graine), 13),
        sens: 1.0,
        etalage: Etalage::default(),
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
                let couleur = match p.d.armes.get_index(arme).map(|(_, a)| &a.mecanique) {
                    Some(Mecanique::Foudre { couleur }) => *couleur,
                    _ => FOUDRE_DEFAUT,
                };
                v.eclair(x, z, zone, couleur);
            }
            EffetArme::Brulure => {
                if v.braises.len() < feu::BRAISES {
                    v.braises.push(Braise { x, z, age: 0.0 });
                }
            }
            EffetArme::Soulevement { zone } => {
                let VisuelsArmes { blocs, alea, .. } = &mut *v;
                sol::soulever(blocs, alea, x, z, zone);
            }
        }
    }
    if effets.vitrine {
        vitrine::avancer(&jeu, &mut v);
    }
}
