//! Aperçu 3D du héros de l'accueil et de la page Profil (src/ui/apercuHeros.ts) : le personnage choisi
//! avec la skin du profil et l'armure portée, de trois quarts, qui respire doucement ;
//! glisser à la souris (bouton gauche, hors des boutons) le fait tourner. Une
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
use crate::modeles::animation::{EtatAnimation, calculer_rotations};
use crate::modeles::{Rendus, armure, rendu};
use crate::sauvegarde::Sauvegarde;
use crate::textures::Textures;

/// Orientation de départ : de trois quarts, comme sur un écran de personnage.
const ANGLE_INITIAL: f32 = 0.45;
/// Radians par pixel glissé.
const SENSIBILITE: f32 = 0.012;
/// Place du héros sur l'accueil : au premier plan, à gauche du choix de partie, les
/// pieds sur le chemin (sur la page Profil : l'origine).
const POSITION_ACCUEIL: Vec3 = Vec3::new(-2.3, -0.45, -2.2);
/// Sur l'accueil, le héros se tourne vers la droite (vers les mobs).
const TOUR_ACCUEIL: f32 = 0.5;

/// Hauteur visée par la caméra de scène (`camera::HAUTEUR_VISEE_TROISIEME`).
const HAUTEUR_VISEE: f32 = 1.8;
/// Format des photos de scène.
const FORMAT_PHOTO: f32 = 16.0 / 9.0;
/// Orientation du héros au centre de la scène (de trois quarts vers la caméra).
const ANGLE_SCENE: f32 = -0.35;

/// Marque la caméra de l'aperçu.
#[derive(Component)]
pub struct CameraApercu;

/// Héros affiché : ce qu'il porte, ses rendus une fois les textures arrivées.
#[derive(Resource)]
pub struct Apercu {
    cle: Option<(Apparence, [Option<Monde>; 7])>,
    apparence: Option<&'static Apparence>,
    rendus: Option<Rendus>,
    angle: f32,
    animation: EtatAnimation,
}

impl Default for Apercu {
    fn default() -> Self {
        Self {
            cle: None,
            apparence: None,
            rendus: None,
            angle: ANGLE_INITIAL,
            animation: EtatAnimation::default(),
        }
    }
}

/// Caméra 3D du menu (sur le fond, sous la caméra de l'interface).
pub fn camera(commandes: &mut Commands) {
    commandes.spawn((
        Name::new("caméra de l'aperçu"),
        CameraApercu,
        Camera3d::default(),
        Camera {
            order: 0,
            clear_color: ClearColorConfig::None,
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

/// Place la caméra de l'aperçu : sur l'accueil d'une arène photographiée de haut,
/// celle de la photo (champ resserré si la fenêtre, plus large, la recadre en
/// hauteur) ; sinon la vue de face.
pub fn cadrer(
    menu: Res<Menu>,
    dj: Res<DonneesJeu>,
    fenetre: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut camera: Query<(&mut Transform, &mut Projection), With<CameraApercu>>,
) {
    let Ok((mut t, mut projection)) = camera.single_mut() else {
        return;
    };
    let Projection::Perspective(p) = &mut *projection else {
        return;
    };
    if !scene_cadree(&menu, &dj) {
        *t = Transform::from_xyz(0.0, 1.4, 8.0).looking_at(Vec3::new(0.0, 1.25, 0.0), Vec3::Y);
        p.fov = 30f32.to_radians();
        return;
    }
    let c = &super::fond::CADRAGE;
    let (l, tg) = (c.lacet.to_radians(), c.tangage.to_radians());
    let (sl, cl) = l.sin_cos();
    let (st, ct) = tg.sin_cos();
    let visee = Vec3::new(-cl * c.decalage, HAUTEUR_VISEE, sl * c.decalage);
    t.translation = visee + Vec3::new(sl * ct, st, cl * ct) * c.distance;
    t.rotation = Quat::from_euler(EulerRot::YXZ, l, -tg, 0.0);
    let format = fenetre
        .single()
        .map(|f| f.width() / f.height().max(1.0))
        .unwrap_or(FORMAT_PHOTO);
    let demi = (c.champ.to_radians() / 2.0).tan() * (FORMAT_PHOTO / format).min(1.0);
    p.fov = 2.0 * demi.atan();
}

/// Vrai sur l'accueil quand la photo de l'arène choisie est vue de haut.
fn scene_cadree(menu: &Menu, dj: &DonneesJeu) -> bool {
    menu.page == Page::Accueil && super::fond::cadree(&dj.arenes[menu.choix.arene].def.id)
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
    (souris, mut glisse, interactions): (
        Res<ButtonInput<MouseButton>>,
        MessageReader<bevy::input::mouse::MouseMotion>,
        Query<&Interaction>,
    ),
    (mut images, mut maillages, mut materiaux): (
        ResMut<Assets<Image>>,
        ResMut<Assets<Mesh>>,
        ResMut<Assets<MateriauEntite>>,
    ),
    mut q: rendu::Composants,
) {
    let d = dj.d;
    if menu.is_changed() {
        if !matches!(menu.page, Page::Profil | Page::Accueil) {
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
        let ambiance = ParametresBloc::pour_entites(dj.arenes[menu.choix.arene].def);
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
    let Apercu {
        rendus,
        angle,
        animation,
        ..
    } = &mut *apercu;
    let Some(rendus) = rendus.as_mut() else {
        return;
    };
    // Glisser hors des boutons fait tourner le héros
    let sur_bouton = interactions.iter().any(|i| *i != Interaction::None);
    let dx: f32 = glisse.read().map(|m| m.delta.x).sum();
    if souris.pressed(MouseButton::Left) && !sur_bouton {
        *angle += dx * SENSIBILITE;
    }
    for r in &mut rendus.liste {
        r.commencer();
    }
    let j = rendus.joueur();
    animation.avancer(&j.modele, temps.delta_secs().min(0.1), 0.0);
    calculer_rotations(&j.modele, animation, &mut j.pose.rotations);
    // Accueil : au centre de la scène vue de haut, ou au premier plan à gauche
    let accueil = menu.page == Page::Accueil;
    let (position, tour) = if scene_cadree(&menu, &dj) {
        (Vec3::ZERO, ANGLE_SCENE)
    } else if accueil {
        (POSITION_ACCUEIL, TOUR_ACCUEIL)
    } else {
        (Vec3::ZERO, 0.0)
    };
    j.pose.position = position;
    j.pose.angle = *angle + tour;
    j.ajouter(&mut commandes, &mut q);
    rendus.poser_armure(&mut commandes, &mut q);
    for r in &mut rendus.liste {
        r.terminer(&mut q);
    }
}
