//! Rendu Bevy de la partie (`bs-regles`). La simulation avance dans `FixedUpdate`
//! (60 pas/s) ; le rendu place chaque entité entre ses deux dernières positions
//! (`overstep_fraction`), donc fluide à toute fréquence d'écran. Le rendu ne modifie
//! jamais la simulation.
//!
//! Déroulé : menu principal (`menu`), chargement des textures de l'arène (écran de
//! chargement à venir ; pour une map importée, d'abord ses états et modèles de blocs),
//! puis construction de l'arène et partie ; « Menu principal » ramène au menu
//! (`lancement::nettoyer`). Une partie lancée directement (`BS_ARENE`) saute le menu :
//! tant que personne n'a touché au clavier (ou sans `BS_CARTES=1`), le robot y choisit
//! les cartes et conduit le joueur (démonstration, captures).

mod amis;
mod arene;
mod armes;
mod camera;
mod carte_mc;
mod chargement;
mod coequipiers;
mod coop;
mod creatures;
mod dessins;
mod ecran_cartes;
mod ecrans;
mod effets;
mod entites;
mod fichiers;
mod hud;
mod icones_armures;
mod lancement;
mod lumiere;
mod maj;
mod materiau;
mod materiau_entite;
mod menu;
mod mesure;
mod modeles;
mod options;
mod polices;
mod retours;
mod sauvegarde;
mod scene;
mod simulation;
mod skins;
mod soleil;
mod sons;
mod sources;
mod textures;

use bevy::prelude::*;
use bs_donnees::Donnees;
use bs_donnees::arenes::Arene;
use bs_monde::CarteMonde;
use bs_monde::MondeGenere;
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

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Phase {
    Menu,
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
    /// Talents et Collection du profil, appliqués à chaque partie.
    depart: sauvegarde::Depart,
    /// Temps écoulé depuis la fin de la partie.
    fin: f32,
}

impl PartieEnCours {
    fn nouvelle(
        mut partie: Partie,
        graine: u32,
        source: SourceArene,
        depart: sauvegarde::Depart,
    ) -> Self {
        depart.appliquer(&mut partie);
        let talents = &depart.talents;
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
            depart,
            fin: 0.0,
        }
    }

    /// Apparence du personnage joué : celle du profil, ou celle de la classe.
    fn apparence_joueur(&self) -> &'static bs_donnees::Apparence {
        let d = self.partie.d;
        self.depart.apparence.unwrap_or_else(|| {
            &d.personnages.personnages[self.partie.etat.index_personnage].apparence
        })
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
        self.depart.appliquer(&mut self.partie);
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
        let dj = lancement::DonneesJeu::nouvelles(d);
        // Partie lancée directement (sinon : menu principal)
        let directe = self.arene.as_deref().map(|id| {
            let arene = dj.arene(id).unwrap_or(0);
            let choix = lancement::ChoixPartie {
                personnage: self.personnage,
                arene,
            };
            lancement::PartiePreparee::nouvelle(&dj, &sauvegarde, choix, self.graine)
                .unwrap_or_else(|e| panic!("arène {id} : {e}"))
        });
        let phase = if directe.is_some() {
            Phase::Chargement
        } else {
            Phase::Menu
        };
        if let Some(p) = directe {
            p.inserer(&mut app.world_mut().commands());
        }
        app.insert_resource(sauvegarde).insert_resource(dj);
        app.insert_resource(Time::<Fixed>::from_seconds(f64::from(PAS)))
            .init_resource::<Textures>()
            .init_resource::<camera::Cible>()
            .init_resource::<effets::Ombres>()
            .init_resource::<retours::Barres>()
            .init_resource::<camera::Secousse>()
            .init_resource::<ecrans::Ecrans>()
            .init_resource::<menu::apercu::Apercu>()
            .init_resource::<menu::multijoueur::Annuaire>()
            .init_resource::<chargement::Photos>()
            .init_resource::<coop::MessageCoop>()
            .insert_resource(ecran_cartes::EcranCartes::nouveau(
                std::env::var_os("BS_CARTES").is_some(),
            ))
            .insert_state(phase)
            .add_plugins((
                materiau::PluginMateriaux,
                materiau_entite::PluginMateriauEntite,
                armes::PluginLueur,
                mesure::PluginMesure,
                fichiers::PluginFichiers,
                soleil::PluginSoleil,
                sons::PluginSons,
                hud::PluginHud,
                options::PluginOptions,
                polices::PluginPolices,
            ))
            .add_systems(
                OnEnter(Phase::Menu),
                (
                    lancement::nettoyer,
                    dessins::creer.run_if(not(resource_exists::<dessins::Dessins>)),
                    sauvegarde::inserer_skin,
                    menu::entrer,
                    maj::creer_encart,
                    amis::creer,
                )
                    .chain(),
            )
            .add_systems(Startup, maj::demarrer)
            .add_systems(Update, amis::suivre)
            .add_systems(
                Update,
                (
                    maj::suivre_voie,
                    maj::noter_reussite,
                    (maj::rafraichir, maj::cliquer).run_if(in_state(Phase::Menu)),
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    (
                        menu::profil::saisir,
                        menu::multijoueur::suivre,
                        menu::gerer,
                        menu::apercu::animer,
                        menu::apercu::cadrer,
                        ecrans::defiler,
                        ecrans::vues::indiquer,
                    )
                        .chain(),
                    (
                        ecrans::style::survoler,
                        ecrans::charte::survoler_relief,
                        menu::survoler_liens,
                        menu::survoler_cases,
                    ),
                    (menu::fond::suivre, menu::fond::couvrir),
                )
                    .run_if(in_state(Phase::Menu)),
            )
            .add_systems(OnExit(Phase::Menu), menu::sortir)
            .add_systems(
                OnEnter(Phase::Chargement),
                (
                    sauvegarde::inserer_skin,
                    dessins::creer.run_if(not(resource_exists::<dessins::Dessins>)),
                    demander_textures.run_if(resource_exists::<Monde>),
                    coequipiers::preparer,
                    chargement::entrer,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    carte_mc::avancer_import,
                    hud::demander_textures.run_if(resource_exists::<PartieEnCours>),
                    demander_armure.run_if(resource_exists::<PartieEnCours>),
                    attendre_textures.run_if(resource_exists::<PartieEnCours>),
                    chargement::avancer,
                )
                    .chain()
                    .run_if(in_state(Phase::Chargement)),
            )
            .add_systems(OnExit(Phase::Chargement), coop::demarrer)
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
                    hud::construire,
                    hud::etiquette::creer,
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
                    sauvegarde::noter_collection,
                )
                    .chain()
                    .run_if(in_state(Phase::Partie)),
            )
            .add_systems(
                Update,
                (
                    entites::placer_entites,
                    camera::capturer_souris,
                    camera::suivre,
                    scene::masquer_interface,
                    hud::etiquette::placer,
                    effets::placer,
                    retours::dessiner,
                    armes::dessiner,
                    armes::placer_solides,
                    ecran_cartes::gerer,
                    coop::envoyer_cartes,
                    ecran_cartes::survoler,
                    ecrans::gerer,
                    ecrans::defiler,
                    ecrans::vues::indiquer,
                    (ecrans::style::survoler, ecrans::charte::survoler_relief),
                    hud::afficher,
                    hud::compteur::afficher,
                    soleil::suivre,
                )
                    .chain()
                    .run_if(in_state(Phase::Partie)),
            )
            .add_systems(OnExit(Phase::Partie), camera::liberer_souris)
            .add_systems(PostUpdate, lumiere::distribuer);
    }
}

/// Textures de l'armure de la Collection portée (elle peut changer entre deux parties).
fn demander_armure(
    serveur: Res<AssetServer>,
    mut textures: ResMut<Textures>,
    jeu: Res<PartieEnCours>,
    sauvegarde: Res<sauvegarde::Sauvegarde>,
) {
    // Une fois par partie chargée
    if jeu.is_added() {
        let portees = sauvegarde.collection.portees();
        modeles::armure::demander_textures(jeu.partie.d, &portees, &serveur, &mut textures);
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
    modeles::demander_textures(d, jeu.apparence_joueur(), &serveur, &mut textures);
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
    sauvegarde: Res<sauvegarde::Sauvegarde>,
    mut images: ResMut<Assets<Image>>,
    mut maillages: ResMut<Assets<Mesh>>,
    mut materiaux: ResMut<Assets<materiau_entite::MateriauEntite>>,
    coequipiers: Res<coequipiers::Coequipiers>,
) {
    let p = &jeu.partie;
    let autres: Vec<_> = coequipiers.0.iter().map(|c| c.apparence).collect();
    commandes.insert_resource(modeles::Rendus::construire(
        p.d,
        (jeu.apparence_joueur(), &autres),
        &sauvegarde.collection.portees(),
        ParametresBloc::pour_entites(p.etat.arene),
        &serveur,
        &textures,
        &mut images,
        &mut maillages,
        &mut materiaux,
    ));
}

/// Textures chargées (et, en coop, tout le monde prêt) : la partie commence.
#[allow(clippy::too_many_arguments)]
fn attendre_textures(
    serveur: Res<AssetServer>,
    textures: Res<Textures>,
    mut phase: ResMut<NextState<Phase>>,
    mut rester: Local<Option<bool>>,
    coop: Option<ResMut<coop::Coop>>,
    mut message: ResMut<coop::MessageCoop>,
    temps: Res<Time>,
) {
    // Essai : `BS_CHARGEMENT=1` garde l'écran de chargement
    if *rester.get_or_insert_with(|| std::env::var_os("BS_CHARGEMENT").is_some()) {
        return;
    }
    let a = textures.avancement(&serveur);
    let chargees = a.faites == a.total;
    if let Some(mut coop) = coop {
        match coop::partie_prete(&mut coop, chargees, temps.delta_secs()) {
            Ok(true) => {}
            Ok(false) => return,
            Err(raison) => {
                message.0 = Some(raison);
                phase.set(Phase::Menu);
                return;
            }
        }
    }
    if chargees {
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
