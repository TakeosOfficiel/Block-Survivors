//! Rendu Bevy de la partie (`bs-regles`). La simulation avance dans `FixedUpdate`
//! (60 pas/s) ; le rendu place chaque entité entre ses deux dernières positions
//! (`overstep_fraction`), donc fluide à toute fréquence d'écran. Le rendu ne modifie
//! jamais la simulation.
//!
//! Déroulé : chargement des textures de l'arène (écran de chargement à venir), puis
//! construction de l'arène et partie. Tant que l'interface n'existe pas (étape 3), le
//! robot choisit les cartes et conduit le joueur si personne ne touche au clavier.

mod arene;
mod camera;
mod entites;
mod materiau;
mod mesure;
mod modeles;
mod textures;

use bevy::prelude::*;
use bs_donnees::Donnees;
use bs_monde::{MondeGenere, generer_monde};
use bs_regles::partie::Partie;
use bs_robot::Robot;

pub use arene::{GroupeFaces, MaillageBloc, maillage_exterieur, maillages_arene};
pub use mesure::Mesure;
pub use textures::PluginSourceTextures;

use crate::materiau::{AnimationTexture, MateriauBloc, ParametresBloc};
use crate::textures::Textures;

/// Pas fixe de la simulation (s).
const PAS: f32 = 1.0 / 60.0;
/// Attente après la fin d'une partie avant d'en relancer une (s).
const ATTENTE_RELANCE: f32 = 3.0;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum Phase {
    #[default]
    Chargement,
    Partie,
}

/// La partie en cours, ressource Bevy.
#[derive(Resource)]
pub struct PartieEnCours {
    pub partie: Partie,
    robot: Robot,
    graine: u32,
    arene: usize,
    /// Temps écoulé depuis la fin de la partie.
    fin: f32,
}

/// Monde généré de l'arène (blocs à afficher).
#[derive(Resource)]
struct Monde(MondeGenere);

/// Plugin du jeu : données, partie, arène, entités, caméra.
pub struct PluginJeu {
    pub graine: u32,
    pub arene: usize,
    pub personnage: usize,
}

impl Plugin for PluginJeu {
    fn build(&self, app: &mut App) {
        let d: &'static Donnees =
            Box::leak(Box::new(Donnees::embarquees().expect("données du jeu")));
        let partie = Partie::nouvelle(d, self.arene, self.personnage, self.graine, true);
        let monde = generer_monde(d, partie.etat.arene);
        app.insert_resource(PartieEnCours {
            partie,
            robot: Robot::default(),
            graine: self.graine,
            arene: self.arene,
            fin: 0.0,
        })
        .insert_resource(Monde(monde))
        .insert_resource(Time::<Fixed>::from_seconds(f64::from(PAS)))
        .init_resource::<Textures>()
        .init_resource::<camera::Cible>()
        .init_state::<Phase>()
        .add_plugins((materiau::PluginMateriaux, mesure::PluginMesure))
        .add_systems(Startup, demander_textures)
        .add_systems(
            Update,
            attendre_textures.run_if(in_state(Phase::Chargement)),
        )
        .add_systems(
            OnEnter(Phase::Partie),
            (construire_arene, entites::creer_entites),
        )
        .add_systems(
            FixedUpdate,
            (entites::noter_positions, avancer)
                .chain()
                .run_if(in_state(Phase::Partie)),
        )
        .add_systems(
            Update,
            (entites::placer_entites, camera::suivre)
                .chain()
                .run_if(in_state(Phase::Partie)),
        );
    }
}

/// Toutes les textures des blocs de l'arène (et de son terrain extérieur).
fn demander_textures(
    serveur: Res<AssetServer>,
    mut textures: ResMut<Textures>,
    jeu: Res<PartieEnCours>,
    monde: Res<Monde>,
) {
    let d = jeu.partie.d;
    let mut blocs: Vec<&str> = monde.0.blocs.iter().map(|(id, _)| id.as_str()).collect();
    if let Some(ext) = &jeu.partie.etat.arene.exterieur {
        blocs.push(&ext.bloc);
    }
    for id in blocs {
        let f = &d.blocs[id].faces;
        for chemin in [&f.haut, &f.cotes, &f.bas] {
            textures.demander(&serveur, chemin);
        }
    }
}

fn attendre_textures(
    serveur: Res<AssetServer>,
    textures: Res<Textures>,
    mut phase: ResMut<NextState<Phase>>,
) {
    let a = textures.avancement(&serveur);
    if a.faites == a.total {
        if a.manquantes > 0 {
            warn!(
                "{} texture(s) introuvable(s) : couleurs de secours",
                a.manquantes
            );
        }
        phase.set(Phase::Partie);
    }
}

fn construire_arene(
    mut commandes: Commands,
    serveur: Res<AssetServer>,
    textures: Res<Textures>,
    jeu: Res<PartieEnCours>,
    monde: Res<Monde>,
    mut maillages: ResMut<Assets<Mesh>>,
    mut materiaux: ResMut<Assets<MateriauBloc>>,
) {
    let d = jeu.partie.d;
    let def = jeu.partie.etat.arene;
    let ambiance = ParametresBloc::de_l_arene(def);
    let a_texture = |chemin: &str| textures.obtenir(&serveur, chemin).is_some();
    let mut liste = maillages_arene(d, &monde.0, a_texture);
    let exterieur = maillage_exterieur(d, def, a_texture);
    let indice_exterieur = exterieur.as_ref().map(|_| liste.len());
    liste.extend(exterieur);
    for (k, m) in liste.iter().enumerate() {
        let b = &d.blocs[m.bloc];
        let groupe = m.groupe.unwrap_or(GroupeFaces::Cotes);
        let chemin = groupe.texture(b);
        let mut parametres = ambiance;
        parametres.brouillard.z = if b.decoupe { 0.5 } else { 0.0 };
        parametres.brouillard.w = if b.lumineux { 1.0 } else { 0.0 };
        // Terrain extérieur : texture répétée (une par bloc)
        parametres.animation.z = if Some(k) == indice_exterieur {
            1.0
        } else {
            0.0
        };
        let croix = b.forme == Some(bs_donnees::blocs::FormeBloc::Croix);
        let materiau = materiaux.add(MateriauBloc {
            parametres,
            texture: textures.obtenir(&serveur, chemin),
            double_face: croix || b.decoupe,
        });
        let mut entite = commandes.spawn((
            Name::new(format!(
                "{}:{groupe:?}",
                d.blocs.get_index(m.bloc).map_or("", |(id, _)| id)
            )),
            Mesh3d(maillages.add(m.en_mesh())),
            MeshMaterial3d(materiau.clone()),
        ));
        if let Some(a) = &b.animation {
            entite.insert(AnimationTexture {
                materiau,
                images_par_seconde: a.images_par_seconde,
                aller_retour: a.aller_retour,
            });
        }
    }
    let ciel = Color::LinearRgba(LinearRgba::from_vec4(ambiance.ciel));
    let j = &jeu.partie.etat.joueur;
    camera::creer_camera(&mut commandes, Vec2::new(j.x, j.z), ciel);
    // Lumière des entités provisoires (StandardMaterial), en attendant leurs modèles
    commandes.spawn((
        DirectionalLight {
            illuminance: 6000.0,
            ..default()
        },
        Transform::from_xyz(3.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commandes.insert_resource(GlobalAmbientLight {
        brightness: 600.0,
        ..default()
    });
}

/// Un pas : le clavier (ZQSD, WASD, flèches) conduit le joueur ; sans touche, le robot.
/// Les cartes sont choisies par le robot tant que l'interface n'existe pas.
fn avancer(
    mut jeu: ResMut<PartieEnCours>,
    touches: Res<ButtonInput<KeyCode>>,
    mut mesure: ResMut<Mesure>,
) {
    let debut = bevy::platform::time::Instant::now();
    if jeu.partie.etat.issue.is_some() {
        jeu.fin += PAS;
        if jeu.fin >= ATTENTE_RELANCE {
            let graine = jeu.graine.wrapping_add(1);
            let (d, arene, perso) = (jeu.partie.d, jeu.arene, jeu.partie.etat.index_personnage);
            *jeu = PartieEnCours {
                partie: Partie::nouvelle(d, arene, perso, graine, true),
                robot: Robot::default(),
                graine,
                arene,
                fin: 0.0,
            };
        }
        return;
    }
    let PartieEnCours { partie, robot, .. } = &mut *jeu;
    let appui = |codes: &[KeyCode]| codes.iter().any(|c| touches.pressed(*c));
    let c = &mut partie.joueur.commandes;
    c.haut = appui(&[KeyCode::KeyW, KeyCode::ArrowUp]);
    c.bas = appui(&[KeyCode::KeyS, KeyCode::ArrowDown]);
    c.gauche = appui(&[KeyCode::KeyA, KeyCode::ArrowLeft]);
    c.droite = appui(&[KeyCode::KeyD, KeyCode::ArrowRight]);
    if c.haut || c.bas || c.gauche || c.droite {
        robot.choisir_cartes(partie);
        partie.pas(PAS);
    } else {
        robot.avancer(partie);
    }
    partie.ev.vider();
    mesure.ajouter_pas(debut.elapsed());
}
