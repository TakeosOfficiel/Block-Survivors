//! Caméra de jeu (src/moteur/camera.ts), deux vues (touche F5, option « vue ») :
//! - de dessus : inclinée, suit le joueur avec un lissage, zoom à la molette ;
//! - à la 3e personne : en hauteur derrière le joueur, la souris capturée (un clic dans
//!   le jeu) la fait tourner, la molette la rapproche ; les touches de déplacement
//!   suivent son orientation et la simulation fait apparaître les ennemis hors de son
//!   champ.
//!
//! Secousses demandées par la simulation (explosions, impacts), plus douces à la 3e
//! personne. Bords des ombres du soleil adoucis (filtre gaussien).

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::{AccumulatedMouseMotion, MouseWheel};
use bevy::light::ShadowFilteringMethod;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bs_commun::maths::facteur_lissage;
use bs_regles::vue::Troisieme;

use crate::PartieEnCours;
use crate::ecran_cartes;
use crate::ecrans::Ecrans;
use crate::hud::police::TextePixel;
use crate::options::{Options, PanneauOptions};

/// Réglages de src/config.ts (`camera`).
const INCLINAISON_DEG: f32 = 55.0;
const CHAMP_DEG: f32 = 45.0;
const HAUTEUR_VISEE: f32 = 1.0;
const DISTANCE: f32 = 24.0;
const DISTANCE_MIN: f32 = 10.0;
const DISTANCE_MAX: f32 = 40.0;
/// Facteur exponentiel par pixel de molette.
const SENSIBILITE_ZOOM: f32 = 0.0012;
/// Raideurs du suivi du joueur et du zoom (1/s).
const SUIVI: f32 = 10.0;
const LISSAGE_ZOOM: f32 = 12.0;
/// Vue à la 3e personne (config.ts `camera.troisieme`) : champ, point visé au-dessus des
/// pieds, recul, plongée de départ et bornes (degrés), raideur du suivi.
const CHAMP_TROISIEME_DEG: f32 = 60.0;
const HAUTEUR_VISEE_TROISIEME: f32 = 1.8;
const DISTANCE_TROISIEME: f32 = 12.0;
const DISTANCE_TROISIEME_MIN: f32 = 2.5;
const DISTANCE_TROISIEME_MAX: f32 = 28.0;
const TANGAGE_DEG: f32 = 32.0;
const TANGAGE_MIN_DEG: f32 = 0.0;
const TANGAGE_MAX_DEG: f32 = 80.0;
const SUIVI_TROISIEME: f32 = 30.0;
/// Rad par pixel de souris.
const SENSIBILITE_SOURIS: f32 = 0.0024;
/// Part de la secousse gardée à la 3e personne.
const SECOUSSE_TROISIEME: f32 = 0.35;
/// Pixels par cran de molette (les souris à crans comptent en lignes).
const PIXELS_PAR_LIGNE: f32 = 100.0;
/// Secousse (config.ts `camera.secousse`) : décalage max (blocs), extinction (1/s),
/// fréquence du bruit.
const SECOUSSE_AMPLITUDE: f32 = 0.55;
const SECOUSSE_DECROISSANCE: f32 = 2.2;
const SECOUSSE_FREQUENCE: f32 = 31.0;

/// Intensité de la secousse (0 à 1) et son horloge.
#[derive(Resource, Default)]
pub struct Secousse {
    force: f32,
    temps: f32,
}

impl Secousse {
    pub fn ajouter(&mut self, force: f32) {
        self.force = (self.force + force).min(1.0);
    }

    /// S'éteint au fil du temps ; renvoie le décalage (x, z) de la caméra : bruit
    /// continu, somme de sinus de fréquences premières entre elles.
    fn avancer(&mut self, dt: f32) -> Vec2 {
        if self.force <= 0.0 {
            return Vec2::ZERO;
        }
        self.force = (self.force - SECOUSSE_DECROISSANCE * dt).max(0.0);
        self.temps += dt;
        let a = self.force * self.force * SECOUSSE_AMPLITUDE;
        let t = self.temps * SECOUSSE_FREQUENCE;
        Vec2::new(
            a * (t.sin() + (t * 2.3 + 1.7).sin()) * 0.5,
            a * ((t * 1.3 + 4.1).sin() + (t * 3.1 + 0.6).sin()) * 0.5,
        )
    }
}

#[derive(Component)]
pub struct CameraJeu {
    /// Point suivi (lissé).
    x: f32,
    z: f32,
    distance: f32,
    /// Zoom voulu de chaque vue (gardé d'un changement de vue à l'autre).
    distance_cible: f32,
    distance_troisieme: f32,
    /// Vue à la 3e personne, et son orientation (rad) : lacet autour de la verticale,
    /// tangage vers le bas.
    troisieme: bool,
    lacet: f32,
    tangage: f32,
}

/// Rappel « cliquez pour tourner la caméra » (vue à la 3e personne, souris libre).
#[derive(Component)]
pub struct AideSouris;

/// Point que la caméra doit suivre (le joueur, interpolé).
#[derive(Resource, Default)]
pub struct Cible(pub Vec2);

pub fn creer_camera(commandes: &mut Commands, depart: Vec2, ciel: Color) {
    commandes.spawn((
        CameraJeu {
            x: depart.x,
            z: depart.y,
            distance: DISTANCE,
            distance_cible: DISTANCE,
            distance_troisieme: DISTANCE_TROISIEME,
            troisieme: false,
            lacet: 0.0,
            tangage: TANGAGE_DEG.to_radians(),
        },
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(ciel),
            ..default()
        },
        // Couleurs telles quelles, comme le rendu three.js (pas de tonemapping)
        Tonemapping::None,
        ShadowFilteringMethod::Gaussian,
        Projection::Perspective(PerspectiveProjection {
            fov: CHAMP_DEG.to_radians(),
            near: 0.1,
            far: 500.0,
            ..default()
        }),
        Transform::default(),
    ));
    commandes.spawn((
        AideSouris,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            top: Val::Percent(30.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Visibility::Hidden,
        children![(TextePixel::nouveau(
            "Cliquez pour tourner la caméra avec la souris (F5 : changer de vue)",
            Color::WHITE,
            2.0,
        )
        .contour(),)],
    ));
}

/// Aucun écran (cartes, pause, options, fin) ne demande le curseur.
fn interactive(jeu: &PartieEnCours, ecrans: &Ecrans, panneau: &PanneauOptions) -> bool {
    let p = &jeu.partie;
    !ecrans.ouvert() && !panneau.ouvert() && !ecran_cartes::en_attente(p) && p.etat.issue.is_none()
}

/// F5 change de vue ; à la 3e personne, un clic dans le jeu capture la souris, rendue
/// aux écrans et à la vue de dessus.
#[allow(clippy::too_many_arguments)]
pub fn capturer_souris(
    touches: Res<ButtonInput<KeyCode>>,
    boutons: Res<ButtonInput<MouseButton>>,
    jeu: Res<PartieEnCours>,
    ecrans: Res<Ecrans>,
    panneau: Res<PanneauOptions>,
    mut options: ResMut<Options>,
    mut curseur: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut aide: Query<&mut Visibility, With<AideSouris>>,
) {
    let libre = interactive(&jeu, &ecrans, &panneau);
    if touches.just_pressed(KeyCode::F5) && libre {
        options.troisieme_personne = !options.troisieme_personne;
    }
    let Ok(mut c) = curseur.single_mut() else {
        return;
    };
    let capturee = c.grab_mode != CursorGrabMode::None;
    let vue = options.troisieme_personne && libre;
    let capturer = vue
        && (capturee
            || boutons.just_pressed(MouseButton::Left)
            || touches.just_pressed(KeyCode::F5));
    if capturer != capturee {
        c.grab_mode = if capturer {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        c.visible = !capturer;
    }
    for mut v in &mut aide {
        v.set_if_neq(if vue && !capturer {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

/// Rend la souris en quittant la partie.
pub fn liberer_souris(mut curseur: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    if let Ok(mut c) = curseur.single_mut() {
        c.grab_mode = CursorGrabMode::None;
        c.visible = true;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn suivre(
    temps: Res<Time>,
    reel: Res<Time<Real>>,
    cible: Res<Cible>,
    options: Res<Options>,
    souris: Res<AccumulatedMouseMotion>,
    curseur: Query<&CursorOptions, With<PrimaryWindow>>,
    mut jeu: ResMut<PartieEnCours>,
    mut secousse: ResMut<Secousse>,
    mut molette: MessageReader<MouseWheel>,
    ecrans: Res<Ecrans>,
    mut cameras: Query<(&mut CameraJeu, &mut Transform, &mut Projection)>,
) {
    let dt = temps.delta_secs();
    let dt_reel = reel.delta_secs();
    let mut crans = 0.0;
    for m in molette.read() {
        crans += match m.unit {
            bevy::input::mouse::MouseScrollUnit::Line => m.y * PIXELS_PAR_LIGNE,
            bevy::input::mouse::MouseScrollUnit::Pixel => m.y,
        };
    }
    // Sous un écran, la molette fait défiler le bilan
    if ecrans.ouvert() {
        crans = 0.0;
    }
    let capturee = curseur
        .single()
        .is_ok_and(|c| c.grab_mode != CursorGrabMode::None);
    let (sin_i, cos_i) = INCLINAISON_DEG.to_radians().sin_cos();
    let mut decalage = secousse.avancer(dt);
    let Ok((mut c, mut t, mut projection)) = cameras.single_mut() else {
        return;
    };
    if c.troisieme != options.troisieme_personne {
        // En quittant la vue de dessus : on regarde dans le sens où marche le joueur
        c.troisieme = options.troisieme_personne;
        if c.troisieme {
            c.lacet = jeu.partie.etat.joueur.angle + std::f32::consts::PI;
            c.tangage = TANGAGE_DEG.to_radians();
        }
        c.distance = if c.troisieme {
            c.distance_troisieme
        } else {
            c.distance_cible
        };
        c.x = cible.0.x;
        c.z = cible.0.y;
        if let Projection::Perspective(p) = &mut *projection {
            p.fov = if c.troisieme {
                CHAMP_TROISIEME_DEG
            } else {
                CHAMP_DEG
            }
            .to_radians();
        }
    }
    let zoom = (-crans * SENSIBILITE_ZOOM).exp();
    let kz = facteur_lissage(LISSAGE_ZOOM, dt_reel.max(dt));
    let (suivi, cible_distance) = if c.troisieme {
        // La souris tourne la caméra même en pause : on peut regarder autour de soi
        if capturee {
            let d = souris.delta * SENSIBILITE_SOURIS;
            c.lacet -= d.x;
            c.tangage =
                (c.tangage + d.y).clamp(TANGAGE_MIN_DEG.to_radians(), TANGAGE_MAX_DEG.to_radians());
        }
        c.distance_troisieme =
            (c.distance_troisieme * zoom).clamp(DISTANCE_TROISIEME_MIN, DISTANCE_TROISIEME_MAX);
        decalage *= SECOUSSE_TROISIEME;
        (SUIVI_TROISIEME, c.distance_troisieme)
    } else {
        // Molette vers soi (y < 0) : on s'éloigne
        c.distance_cible = (c.distance_cible * zoom).clamp(DISTANCE_MIN, DISTANCE_MAX);
        (SUIVI, c.distance_cible)
    };
    let k = facteur_lissage(suivi, dt);
    c.x += (cible.0.x - c.x) * k;
    c.z += (cible.0.y - c.z) * k;
    c.distance += (cible_distance - c.distance) * kz;

    if let Some(s) = crate::scene::scene() {
        // Capture de scène : caméra fixe autour du joueur
        c.troisieme = true;
        c.lacet = s.lacet;
        c.tangage = s.tangage;
        c.distance = s.distance;
        c.x = cible.0.x;
        c.z = cible.0.y;
        decalage = Vec2::ZERO;
        if let Projection::Perspective(p) = &mut *projection {
            p.fov = s.champ;
        }
    }
    if c.troisieme {
        // Derrière le point visé, à l'opposé du regard
        let (sl, cl) = c.lacet.sin_cos();
        let (st, ct) = c.tangage.sin_cos();
        let horizontal = ct * c.distance;
        t.translation = Vec3::new(
            c.x + sl * horizontal + decalage.x,
            (HAUTEUR_VISEE_TROISIEME + st * c.distance).max(0.3),
            c.z + cl * horizontal + decalage.y,
        );
        t.rotation = Quat::from_euler(EulerRot::YXZ, c.lacet, -c.tangage, 0.0);
    } else {
        t.translation = Vec3::new(
            c.x + decalage.x,
            HAUTEUR_VISEE + c.distance * sin_i,
            c.z + decalage.y + c.distance * cos_i,
        );
        t.rotation = Quat::from_rotation_x(-sin_i.asin());
    }

    // La simulation : touches orientées par la caméra, apparitions hors de son champ
    let troisieme = c.troisieme.then_some(Troisieme {
        lacet: c.lacet,
        tangage: c.tangage,
        distance: c.distance,
    });
    let lacet = if c.troisieme { c.lacet } else { 0.0 };
    let p = &mut jeu.bypass_change_detection().partie;
    p.joueur.commandes.lacet = lacet;
    p.vue.troisieme = troisieme;
}
