//! Rendu Bevy de la partie (`bs-regles`). La simulation avance dans `FixedUpdate`
//! (60 pas/s) ; le rendu place chaque entité entre ses deux dernières positions
//! (`overstep_fraction`), donc fluide à toute fréquence d'écran. Le rendu ne modifie
//! jamais la simulation.
//!
//! Déroulé : chargement des textures de l'arène (écran de chargement à venir ; pour une
//! map importée, d'abord ses états et modèles de blocs), puis construction de l'arène
//! et partie. Tant que personne n'a touché au clavier (ou sans `BS_CARTES=1`), le robot
//! choisit les cartes et conduit le joueur (démonstration, captures).

mod arene;
mod armes;
mod camera;
mod carte_mc;
mod dessins;
mod ecran_cartes;
mod ecrans;
mod effets;
mod entites;
mod fichiers;
mod hud;
mod lumiere;
mod materiau;
mod materiau_entite;
mod mesure;
mod modeles;
mod retours;
mod sauvegarde;
mod simulation;
mod soleil;
mod sons;
mod textures;

use bevy::prelude::*;
use bs_donnees::Donnees;
use bs_donnees::arenes::Arene;
use bs_donnees::progression::BonusTalents;
use bs_monde::CarteMonde;
use bs_monde::{MondeGenere, generer_monde};
use bs_regles::partie::Partie;
use bs_robot::Robot;

pub use arene::{GroupeFaces, MaillageBloc, maillage_exterieur, maillages_arene};
pub use mesure::Mesure;
pub use textures::PluginSourceTextures;

use crate::materiau::{AnimationTexture, MateriauBloc, ParametresBloc};
use crate::textures::Textures;

/// Pas fixe de la simulation (s).
pub(crate) const PAS: f32 = 1.0 / 60.0;
/// Attente après la fin d'une partie avant d'en relancer une (s).
const ATTENTE_RELANCE: f32 = 3.0;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum Phase {
    #[default]
    Chargement,
    Partie,
}

/// D'où vient l'arène (pour relancer une partie au même endroit).
enum SourceArene {
    /// Arène générée : index dans les données.
    Generee(usize),
    /// Map Minecraft importée : définition et carte de collision.
    Importee(&'static Arene, CarteMonde),
}

/// La partie en cours, ressource Bevy.
#[derive(Resource)]
pub struct PartieEnCours {
    pub partie: Partie,
    robot: Robot,
    graine: u32,
    source: SourceArene,
    /// Effets de l'arbre de talents actif du profil, appliqués à chaque partie.
    talents: BonusTalents,
    /// Temps écoulé depuis la fin de la partie.
    fin: f32,
}

impl PartieEnCours {
    fn nouvelle(
        mut partie: Partie,
        graine: u32,
        source: SourceArene,
        talents: BonusTalents,
    ) -> Self {
        partie.appliquer_talents(&talents);
        let mecaniques: Vec<_> = talents
            .mecaniques
            .iter()
            .map(|(m, r)| format!("{m} {r}"))
            .collect();
        info!(
            "talents : armes offertes {:?}, relances +{}, mécaniques [{}]",
            talents.armes,
            talents.relances,
            mecaniques.join(", ")
        );
        Self {
            partie,
            robot: Robot::default(),
            graine,
            source,
            talents,
            fin: 0.0,
        }
    }

    /// Partie suivante, au même endroit avec le même personnage.
    fn relancer(&mut self) {
        let graine = self.graine.wrapping_add(1);
        let (d, perso) = (self.partie.d, self.partie.etat.index_personnage);
        self.partie = match &self.source {
            SourceArene::Generee(index) => Partie::nouvelle(d, *index, perso, graine, true),
            SourceArene::Importee(arene, carte) => {
                Partie::avec_arene(d, arene, carte.clone(), perso, graine, true)
            }
        };
        self.partie.appliquer_talents(&self.talents);
        self.robot = Robot::default();
        self.graine = graine;
        self.fin = 0.0;
    }
}

/// Monde généré de l'arène (blocs à afficher).
#[derive(Resource)]
struct Monde(MondeGenere);

/// Plugin du jeu : données, partie, arène, entités, caméra.
pub struct PluginJeu {
    pub graine: u32,
    /// Identifiant d'une arène générée ou d'une map importée (`cartes/`) ; inconnu ou
    /// absent : la première arène.
    pub arene: Option<String>,
    pub personnage: usize,
}

impl Plugin for PluginJeu {
    fn build(&self, app: &mut App) {
        let d: &'static Donnees =
            Box::leak(Box::new(Donnees::embarquees().expect("données du jeu")));
        app.insert_non_send_resource(sons::Sons::nouveaux(d));
        let sauvegarde = sauvegarde::Sauvegarde::charger(d);
        let talents = sauvegarde.talents(d, self.personnage);
        app.insert_resource(sauvegarde);
        let id = self.arene.as_deref().unwrap_or("");
        let carte = bs_monde::minecraft::cartes::cartes()
            .iter()
            .find(|c| c.id == id);
        if let Some(carte) = carte {
            let import = carte_mc::Import::nouvel(d, carte, self.personnage, self.graine, talents)
                .unwrap_or_else(|e| panic!("map {id} : {e}"));
            app.insert_resource(import);
        } else {
            let index = d.arenes.arenes.iter().position(|a| a.id == id).unwrap_or(0);
            let partie = Partie::nouvelle(d, index, self.personnage, self.graine, true);
            let monde = generer_monde(d, partie.etat.arene);
            let mut jeu =
                PartieEnCours::nouvelle(partie, self.graine, SourceArene::Generee(index), talents);
            simulation::avancer_debut(&mut jeu);
            app.insert_resource(jeu).insert_resource(Monde(monde));
        }
        app.insert_resource(Time::<Fixed>::from_seconds(f64::from(PAS)))
            .init_resource::<Textures>()
            .init_resource::<camera::Cible>()
            .init_resource::<effets::Ombres>()
            .init_resource::<retours::Barres>()
            .init_resource::<camera::Secousse>()
            .init_resource::<ecrans::Ecrans>()
            .insert_resource(ecran_cartes::EcranCartes::nouveau(
                std::env::var_os("BS_CARTES").is_some(),
            ))
            .init_state::<Phase>()
            .add_plugins((
                materiau::PluginMateriaux,
                materiau_entite::PluginMateriauEntite,
                armes::PluginLueur,
                mesure::PluginMesure,
                fichiers::PluginFichiers,
                soleil::PluginSoleil,
                sons::PluginSons,
                hud::PluginHud,
            ))
            .add_systems(Startup, demander_textures.run_if(resource_exists::<Monde>))
            .add_systems(
                Update,
                (
                    carte_mc::avancer_import,
                    hud::demander_textures.run_if(resource_exists::<PartieEnCours>),
                    attendre_textures.run_if(resource_exists::<PartieEnCours>),
                )
                    .chain()
                    .run_if(in_state(Phase::Chargement)),
            )
            .add_systems(
                OnEnter(Phase::Partie),
                (
                    construire_arene.run_if(resource_exists::<Monde>),
                    carte_mc::construire_carte.run_if(resource_exists::<carte_mc::CarteImportee>),
                    creer_camera,
                    soleil::creer,
                    entites::creer_entites,
                    construire_modeles,
                    effets::construire,
                    retours::construire,
                    armes::construire,
                    dessins::creer,
                    hud::construire,
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    entites::noter_positions,
                    simulation::avancer,
                    entites::animer,
                    retours::recevoir,
                    armes::recevoir,
                    sons::jouer,
                    hud::recevoir,
                )
                    .chain()
                    .run_if(in_state(Phase::Partie)),
            )
            .add_systems(
                Update,
                (
                    entites::placer_entites,
                    camera::suivre,
                    effets::placer,
                    retours::dessiner,
                    armes::dessiner,
                    armes::placer_solides,
                    ecran_cartes::gerer,
                    ecran_cartes::survoler,
                    ecrans::gerer,
                    ecrans::defiler,
                    ecrans::style::survoler,
                    hud::afficher,
                )
                    .chain()
                    .run_if(in_state(Phase::Partie)),
            )
            .add_systems(PostUpdate, lumiere::distribuer);
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
    let perso = jeu.partie.etat.index_personnage;
    modeles::demander_textures(d, perso, &serveur, &mut textures);
    effets::demander_textures(d, &serveur, &mut textures);
    retours::demander_textures(&jeu, &serveur, &mut textures);
    armes::demander_textures(d, &serveur, &mut textures);
}

/// Modèles des entités (textures déjà chargées).
fn construire_modeles(
    mut commandes: Commands,
    serveur: Res<AssetServer>,
    textures: Res<Textures>,
    jeu: Res<PartieEnCours>,
    images: Res<Assets<Image>>,
    mut maillages: ResMut<Assets<Mesh>>,
    mut materiaux: ResMut<Assets<materiau_entite::MateriauEntite>>,
) {
    let p = &jeu.partie;
    commandes.insert_resource(modeles::Rendus::construire(
        p.d,
        p.etat.index_personnage,
        ParametresBloc::de_l_arene(p.etat.arene),
        &serveur,
        &textures,
        &images,
        &mut maillages,
        &mut materiaux,
    ));
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
    mut images: ResMut<Assets<Image>>,
) {
    let d = jeu.partie.d;
    let def = jeu.partie.etat.arene;
    let lumiere = bs_monde::lumiere::volume_du_monde(d, &monde.0);
    lumiere::inserer(&mut commandes, &mut images, &lumiere, Vec3::ZERO);
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
            lumiere: None,
            double_face: croix || b.decoupe,
            translucide: false,
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
}

/// Caméra au-dessus du joueur, fond couleur du ciel de l'arène.
fn creer_camera(mut commandes: Commands, jeu: Res<PartieEnCours>) {
    let ciel = ParametresBloc::de_l_arene(jeu.partie.etat.arene).ciel;
    let j = &jeu.partie.etat.joueur;
    camera::creer_camera(
        &mut commandes,
        Vec2::new(j.x, j.z),
        Color::LinearRgba(LinearRgba::from_vec4(ciel)),
    );
}
