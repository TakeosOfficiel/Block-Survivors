//! Rendu Bevy de la simulation (`bs-regles`) : la simulation avance dans `FixedUpdate`
//! (60 pas/s), le rendu place chaque entité entre ses deux dernières positions
//! (`overstep_fraction`), donc fluide à toute fréquence d'écran. Les entités visibles sont
//! créées une fois (une par emplacement des tableaux de la simulation) et seulement
//! déplacées ou cachées ensuite : aucune création pendant la partie.
//!
//! Prototype (étape 0 bis, pari 1) : arène, joueur, 300 zombies, une arme ; mesure des
//! images/s dans la console.

mod mesure;
mod modeles;

use bevy::prelude::*;
use bs_commun::PAS_SIMULATION;
use bs_regles::{Commande, Parametres, Simulation};

pub use mesure::Mesure;

/// La simulation, ressource Bevy.
#[derive(Resource)]
pub struct Partie(pub Simulation);

/// Emplacement d'un ennemi ou d'un projectile dans les tableaux de la simulation.
#[derive(Component)]
struct Zombie(usize);
#[derive(Component)]
struct Projectile(usize);
#[derive(Component)]
struct JoueurVisible;

/// Plugin du prototype : simulation, scène et caméra.
pub struct PluginPrototype {
    pub graine: u64,
}

impl Plugin for PluginPrototype {
    fn build(&self, app: &mut App) {
        app.insert_resource(Partie(Simulation::nouvelle(
            self.graine,
            Parametres::default(),
        )))
        .insert_resource(Time::<Fixed>::from_seconds(PAS_SIMULATION as f64))
        .insert_resource(ClearColor(Color::srgb(0.47, 0.65, 1.0)))
        .add_plugins(mesure::PluginMesure)
        .add_systems(Startup, installer_scene)
        .add_systems(FixedUpdate, avancer)
        .add_systems(
            Update,
            (
                placer_joueur,
                placer_zombies,
                placer_projectiles,
                suivre_camera,
            )
                .chain(),
        );
    }
}

/// Clavier (ZQSD, WASD ou flèches) ; sans touche, un pilote automatique tourne en rond
/// (mesures sans personne au clavier).
fn commande(touches: &ButtonInput<KeyCode>, pas: u64) -> Commande {
    let appui = |codes: &[KeyCode]| codes.iter().any(|c| touches.pressed(*c)) as i32 as f32;
    let dx =
        appui(&[KeyCode::KeyD, KeyCode::ArrowRight]) - appui(&[KeyCode::KeyA, KeyCode::ArrowLeft]);
    let dz =
        appui(&[KeyCode::KeyS, KeyCode::ArrowDown]) - appui(&[KeyCode::KeyW, KeyCode::ArrowUp]);
    if dx != 0.0 || dz != 0.0 {
        return Commande { dx, dz };
    }
    let a = pas as f32 * 0.004;
    Commande {
        dx: a.cos(),
        dz: a.sin(),
    }
}

fn avancer(
    mut partie: ResMut<Partie>,
    touches: Res<ButtonInput<KeyCode>>,
    mut mesure: ResMut<Mesure>,
) {
    let debut = bevy::platform::time::Instant::now();
    let c = commande(&touches, partie.0.pas);
    partie.0.avancer(c);
    mesure.ajouter_pas(debut.elapsed());
}

fn installer_scene(
    mut commandes: Commands,
    mut maillages: ResMut<Assets<Mesh>>,
    mut materiaux: ResMut<Assets<StandardMaterial>>,
    partie: Res<Partie>,
) {
    let s = &partie.0;
    let (largeur, profondeur) = (s.largeur(), s.profondeur());
    // Sol en damier de deux herbes (deux maillages, pas un bloc par case)
    let (sol_a, sol_b) = modeles::sol(largeur, profondeur);
    for (m, couleur) in [
        (sol_a, Color::srgb(0.36, 0.62, 0.25)),
        (sol_b, Color::srgb(0.33, 0.58, 0.23)),
    ] {
        commandes.spawn((
            Mesh3d(maillages.add(m)),
            MeshMaterial3d(materiaux.add(couleur)),
        ));
    }
    commandes.spawn((
        JoueurVisible,
        Mesh3d(maillages.add(modeles::personnage())),
        MeshMaterial3d(materiaux.add(Color::srgb(0.2, 0.45, 0.85))),
        Transform::default(),
    ));
    // Tous les zombies partagent maillage et matériau : Bevy les dessine en lots
    let zombie = maillages.add(modeles::personnage());
    let peau = materiaux.add(Color::srgb(0.33, 0.55, 0.3));
    for i in 0..s.ennemis.x.len() {
        commandes.spawn((
            Zombie(i),
            Mesh3d(zombie.clone()),
            MeshMaterial3d(peau.clone()),
            Transform::default(),
        ));
    }
    let boule = maillages.add(Sphere::new(0.25).mesh().ico(1).expect("sphère"));
    let feu = materiaux.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.55, 0.1),
        emissive: LinearRgba::rgb(4.0, 1.5, 0.2),
        ..default()
    });
    for k in 0..s.projectiles.x.len() {
        commandes.spawn((
            Projectile(k),
            Mesh3d(boule.clone()),
            MeshMaterial3d(feu.clone()),
            Transform::default(),
            Visibility::Hidden,
        ));
    }
    commandes.spawn((
        DirectionalLight {
            illuminance: 9000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(20.0, 40.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commandes.insert_resource(GlobalAmbientLight {
        brightness: 400.0,
        ..default()
    });
    commandes.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 18.0, 14.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn entre(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn placer_joueur(
    partie: Res<Partie>,
    temps: Res<Time<Fixed>>,
    mut q: Query<&mut Transform, With<JoueurVisible>>,
) {
    let t = temps.overstep_fraction();
    let j = &partie.0.joueur;
    for mut tr in &mut q {
        tr.translation = Vec3::new(entre(j.x_prec, j.x, t), 0.0, entre(j.z_prec, j.z, t));
        tr.rotation = Quat::from_rotation_y(j.angle);
    }
}

fn placer_zombies(
    partie: Res<Partie>,
    temps: Res<Time<Fixed>>,
    mut q: Query<(&Zombie, &mut Transform, &mut Visibility)>,
) {
    let t = temps.overstep_fraction();
    let e = &partie.0.ennemis;
    for (Zombie(i), mut tr, mut vis) in &mut q {
        let i = *i;
        let voulu = if e.vivant[i] {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        vis.set_if_neq(voulu);
        if e.vivant[i] {
            tr.translation = Vec3::new(
                entre(e.x_prec[i], e.x[i], t),
                0.0,
                entre(e.z_prec[i], e.z[i], t),
            );
            tr.rotation = Quat::from_rotation_y(e.angle[i]);
        }
    }
}

fn placer_projectiles(
    partie: Res<Partie>,
    temps: Res<Time<Fixed>>,
    mut q: Query<(&Projectile, &mut Transform, &mut Visibility)>,
) {
    let t = temps.overstep_fraction();
    let p = &partie.0.projectiles;
    for (Projectile(k), mut tr, mut vis) in &mut q {
        let k = *k;
        let actif = k < p.nombre;
        vis.set_if_neq(if actif {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        if actif {
            tr.translation = Vec3::new(
                entre(p.x_prec[k], p.x[k], t),
                0.9,
                entre(p.z_prec[k], p.z[k], t),
            );
        }
    }
}

/// Caméra au-dessus du joueur, légèrement en arrière (comme la vue « dessus » du jeu TS).
fn suivre_camera(
    joueur: Query<&Transform, (With<JoueurVisible>, Without<Camera3d>)>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    let (Ok(j), Ok(mut c)) = (joueur.single(), camera.single_mut()) else {
        return;
    };
    let cible = j.translation;
    *c = Transform::from_translation(cible + Vec3::new(0.0, 18.0, 12.0)).looking_at(cible, Vec3::Y);
}
