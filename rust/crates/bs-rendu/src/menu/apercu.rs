//! Aperçu 3D du héros de la page Profil (src/ui/apercuHeros.ts) : le personnage choisi
//! avec la skin du profil et l'armure portée, qui tourne lentement sur lui-même. Une
//! caméra 3D du menu le dessine sous l'interface (la page Profil laisse son centre
//! transparent). Refait quand la page change ; ses textures sont attendues avant.

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bs_donnees::Apparence;
use bs_donnees::arenes::Monde;

use super::{ElementMenu, Menu, Page};
use crate::lancement::DonneesJeu;
use crate::materiau::ParametresBloc;
use crate::materiau_entite::MateriauEntite;
use crate::modeles::{Rendus, armure, rendu};
use crate::sauvegarde::Sauvegarde;
use crate::textures::Textures;

/// Tours par seconde.
const VITESSE: f32 = 0.08;

/// Héros affiché : ce qu'il porte, ses rendus une fois les textures arrivées.
#[derive(Resource, Default)]
pub struct Apercu {
    cle: Option<(Apparence, [Option<Monde>; 7])>,
    apparence: Option<&'static Apparence>,
    rendus: Option<Rendus>,
    angle: f32,
}

/// Caméra 3D du menu (sous la caméra de l'interface).
pub fn camera(commandes: &mut Commands, fond: Color) {
    commandes.spawn((
        Name::new("caméra de l'aperçu"),
        Camera3d::default(),
        Camera {
            order: 0,
            clear_color: ClearColorConfig::Custom(fond),
            ..default()
        },
        Tonemapping::None,
        Projection::Perspective(PerspectiveProjection {
            fov: 30f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(0.0, 1.4, 8.0).looking_at(Vec3::new(0.0, 1.25, 0.0), Vec3::Y),
        ElementMenu,
    ));
}

/// Retire le héros affiché.
pub fn vider(commandes: &mut Commands, apercu: &mut Apercu) {
    if let Some(r) = apercu.rendus.take() {
        for e in r.racines() {
            commandes.entity(e).despawn();
        }
    }
    apercu.cle = None;
    apercu.apparence = None;
}

/// Suit la page : demande les textures du héros voulu, le construit quand elles sont
/// là, puis le fait tourner.
#[allow(clippy::too_many_arguments)]
pub fn animer(
    mut commandes: Commands,
    menu: Res<Menu>,
    dj: Res<DonneesJeu>,
    sauvegarde: Res<Sauvegarde>,
    mut apercu: ResMut<Apercu>,
    (serveur, mut textures, temps): (Res<AssetServer>, ResMut<Textures>, Res<Time>),
    (mut images, mut maillages, mut materiaux): (
        ResMut<Assets<Image>>,
        ResMut<Assets<Mesh>>,
        ResMut<Assets<MateriauEntite>>,
    ),
    mut q: rendu::Composants,
) {
    let d = dj.d;
    if menu.is_changed() {
        if menu.page != Page::Profil {
            vider(&mut commandes, &mut apercu);
            return;
        }
        let personnage = menu.choix.personnage;
        let apparence = sauvegarde
            .depart(d, personnage)
            .apparence
            .unwrap_or(&d.personnages.personnages[personnage].apparence);
        let portees = sauvegarde.collection.portees();
        let cle = (apparence.clone(), portees);
        if apercu.cle.as_ref() != Some(&cle) {
            vider(&mut commandes, &mut apercu);
            textures.demander(&serveur, &apparence.skin);
            if let Some(o) = &apparence.objet_en_main {
                textures.demander(&serveur, o);
            }
            armure::demander_textures(d, &portees, &serveur, &mut textures);
            apercu.cle = Some(cle);
            apercu.apparence = Some(apparence);
        }
    }
    let Some(apparence) = apercu.apparence else {
        return;
    };
    if apercu.rendus.is_none() {
        let a = textures.avancement(&serveur);
        if a.faites < a.total {
            return;
        }
        let portees = sauvegarde.collection.portees();
        let ambiance = ParametresBloc::de_l_arene(dj.arenes[menu.choix.arene].def);
        apercu.rendus = Some(Rendus::heros(
            d,
            apparence,
            &portees,
            ambiance,
            &serveur,
            &textures,
            &mut images,
            &mut maillages,
            &mut materiaux,
        ));
    }
    let Apercu { rendus, angle, .. } = &mut *apercu;
    let Some(rendus) = rendus.as_mut() else {
        return;
    };
    *angle =
        (*angle + temps.delta_secs() * VITESSE * std::f32::consts::TAU) % std::f32::consts::TAU;
    for r in &mut rendus.liste {
        r.commencer();
    }
    let j = rendus.joueur();
    j.pose.position = Vec3::ZERO;
    j.pose.angle = *angle;
    j.ajouter(&mut commandes, &mut q);
    rendus.poser_armure(&mut commandes, &mut q);
    for r in &mut rendus.liste {
        r.terminer(&mut q);
    }
}
