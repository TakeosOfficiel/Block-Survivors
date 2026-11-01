//! Visuels des armes, neufs (pas une copie de la version TS) : la simulation dit quoi
//! et où (`Evenement::Effet`), ce module choisit l'allure. Presque tout est dessiné en
//! lueurs calculées par le shader (`lueur`), ajoutées à la scène, en un seul appel de
//! dessin ; les pools ont une taille fixe et ne réallouent jamais.

mod feu;
mod foudre;
mod lueur;
mod melee;
mod sol;
mod solides;
mod vitrine;

use bevy::prelude::*;
use bs_commun::Alea;
use bs_donnees::armes::{Arme, Mecanique};
use bs_donnees::{Donnees, StatsArme};
use bs_regles::combat::sol::{Chute, Nova};
use bs_regles::evenements::{EffetArme, Evenement};

use crate::arene::couleur_lineaire;
use crate::camera::Cible;
use crate::effets::Effets;
use crate::materiau::ParametresBloc;
use crate::materiau_entite::MateriauEntite;
use crate::modeles::rendu::Composants;
use crate::textures::Textures;
use crate::{PAS, PartieEnCours};
use feu::Braise;
use foudre::Eclair;
pub use lueur::PluginLueur;
use lueur::{MaillageLueur, MateriauLueur, ParametresLueur};
use melee::{Coup, Genre};
use sol::{BlocSouleve, Givre, Lame};
use solides::{Fabrique, Piece, Solides};
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

/// Zone de l'arme `arme` de l'arsenal (1 si elle n'y est pas : vitrine).
fn zone(jeu: &PartieEnCours, arme: usize) -> f32 {
    jeu.partie
        .arsenal
        .armes
        .get(arme)
        .map_or(1.0, |a| a.stats.zone)
}

/// L'arme `arme` de l'arsenal et ses stats ; hors de l'arsenal (vitrine), la première
/// arme du genre `genre` dans les données, avec ses stats de base.
fn definition(jeu: &PartieEnCours, arme: usize, genre: &str) -> Option<(&'static Arme, StatsArme)> {
    match jeu.partie.arsenal.armes.get(arme) {
        Some(a) if a.def.mecanique.nom() == genre => Some((a.def, a.stats)),
        _ => jeu
            .partie
            .d
            .armes
            .values()
            .find(|a| a.mecanique.nom() == genre)
            .map(|a| (a, a.stats)),
    }
}

/// Texture du météore de l'arme `arme`.
fn texture_meteore(jeu: &PartieEnCours, arme: usize) -> Option<&'static str> {
    match definition(jeu, arme, "meteore").map(|(a, _)| &a.mecanique) {
        Some(Mecanique::Meteore { texture, .. }) => Some(texture.as_str()),
        _ => None,
    }
}

/// Lames géantes de la simulation et de l'étalage.
fn lames<'a>(jeu: &'a PartieEnCours, etalage: &'a [Chute]) -> impl Iterator<Item = Lame<'a>> {
    let p = &jeu.partie;
    p.armes
        .sol
        .chutes
        .iter()
        .chain(etalage)
        .filter_map(move |c| {
            let (a, s) = definition(jeu, c.arme, "chute")?;
            let Mecanique::Chute { image, taille } = &a.mecanique else {
                return None;
            };
            Some(Lame {
                chute: c,
                image,
                taille: *taille,
                zone: s.zone,
            })
        })
}

/// Novas de la simulation et de l'étalage.
fn givres<'a>(jeu: &'a PartieEnCours, etalage: &'a [Nova]) -> impl Iterator<Item = Givre<'a>> {
    let p = &jeu.partie;
    p.armes
        .sol
        .novas
        .iter()
        .chain(etalage)
        .filter_map(move |n| {
            let (a, s) = definition(jeu, n.arme, "nova")?;
            let Mecanique::Nova {
                texture,
                expansion,
                duree_sol,
                ..
            } = &a.mecanique
            else {
                return None;
            };
            Some(Givre {
                nova: n,
                texture,
                zone: s.zone,
                expansion: *expansion,
                duree_sol: *duree_sol,
            })
        })
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
    let ambiance = ParametresBloc::de_l_arene(jeu.partie.etat.arene);
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
                let couleur = match p.arsenal.armes.get(arme).map(|a| &a.def.mecanique) {
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
    m.terminer(&mut maillages, &mut visibilites);
}
