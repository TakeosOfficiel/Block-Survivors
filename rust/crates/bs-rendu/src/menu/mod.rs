//! Menu principal (phase `Menu`, src/ui/ecranMenu.ts) : choix de la classe et de
//! l'arène (gardé dans le profil), « Jouer » (ou Entrée) prépare la partie choisie et
//! passe au chargement, « Quitter » ferme le jeu. Ses pages (Talents, Collection, Profil…) remplacent
//! l'accueil ; Échap ou « Retour » y ramène. Le menu a sa caméra 2D, et une caméra 3D dessous pour l'aperçu du héros ; tout ce
//! qu'il crée porte `ElementMenu` et disparaît à sa sortie. Chaque changement refait la
//! page affichée en entier (hors de la boucle de jeu).
//!
//! Essais : `BS_JOUER=n` lance la partie choisie aux n premières entrées au menu ;
//! `BS_MENU=talents` (ou `collection`, `profil`) ouvre cette page à l'entrée.

pub mod apercu;
mod collection;
pub mod profil;
mod talents;
mod vue;

use bevy::app::AppExit;
use bevy::prelude::*;

use crate::Phase;
use crate::dessins::Dessins;
use crate::lancement::{self, ChoixPartie, DonneesJeu, PartiePreparee};
use crate::sauvegarde::Sauvegarde;
use crate::textures::Textures;
use talents::{ActionTalent, EtatTalents};

/// Bouton du menu.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionMenu {
    Classe(usize),
    Arene(usize),
    Jouer,
    Quitter,
    /// Ouvre la page des Talents.
    Talents,
    /// Ouvre la Collection.
    Collection,
    /// Onglet de la Collection.
    Monde(usize),
    /// Ouvre le Profil.
    OuvrirProfil,
    Profil(profil::ActionProfil),
    /// Revient à l'accueil.
    Fermer,
    Talent(ActionTalent),
}

/// Page affichée.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Page {
    Accueil,
    Talents(EtatTalents),
    /// Collection, sur un monde.
    Collection(usize),
    Profil,
}

/// Marque de tout ce que le menu crée.
#[derive(Component)]
pub struct ElementMenu;

/// Choix, page et racine de l'interface ; saisie du pseudo en cours, message de la
/// page Profil.
#[derive(Resource)]
pub struct Menu {
    choix: ChoixPartie,
    page: Page,
    racine: Entity,
    saisie: Option<String>,
    message: Option<String>,
    /// La page est à refaire (changée hors d'un bouton).
    a_refaire: bool,
    /// Le clavier a servi à la saisie à cette image (Entrée, Échap ignorés).
    clavier_pris: bool,
}

/// Icônes des pages (chargées à la demande).
pub struct Icones<'a> {
    pub serveur: &'a AssetServer,
    pub textures: &'a mut Textures,
    pub dessins: &'a Dessins,
    pub images: &'a Assets<Image>,
}

impl Icones<'_> {
    pub fn icone(&mut self, chemin: &str) -> Handle<Image> {
        self.textures.poignee(self.serveur, chemin)
    }
}

fn construire(
    commandes: &mut Commands,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    ic: &mut Icones,
    menu: &Menu,
) -> Entity {
    let (choix, page) = (menu.choix, menu.page);
    let racine = match page {
        Page::Accueil => vue::construire(commandes, dj, sauvegarde, choix),
        Page::Talents(etat) => {
            let c = talents::Contexte {
                dj,
                sauvegarde,
                etat,
            };
            talents::construire(commandes, &c, ic)
        }
        Page::Collection(k) => {
            let dessins = ic.dessins;
            collection::construire(commandes, (dj, sauvegarde, dessins), ic, k)
        }
        Page::Profil => {
            let dessins = ic.dessins;
            let etat = (
                choix.personnage,
                menu.saisie.as_deref(),
                menu.message.as_deref(),
            );
            profil::construire(commandes, (dj, sauvegarde, dessins), ic, etat)
        }
    };
    commandes.entity(racine).insert(ElementMenu);
    racine
}

/// Page des Talents, sur la classe choisie et son arbre actif.
fn page_talents(dj: &DonneesJeu, sauvegarde: &Sauvegarde, personnage: usize) -> Page {
    let id =
        dj.d.personnages
            .personnages
            .get_index(personnage)
            .map_or("", |(id, _)| id.as_str());
    Page::Talents(EtatTalents {
        personnage,
        arbre: sauvegarde.progression.arbre_actif(id),
        noeud: 0,
    })
}

/// Entrée au menu : caméras et interface, sur le dernier choix du profil.
#[allow(clippy::too_many_arguments)]
pub fn entrer(
    mut commandes: Commands,
    dj: Res<DonneesJeu>,
    sauvegarde: Res<Sauvegarde>,
    serveur: Res<AssetServer>,
    mut textures: ResMut<Textures>,
    dessins: Res<Dessins>,
    images: Res<Assets<Image>>,
) {
    let choix = lancement::choix_memorise(&dj, &sauvegarde);
    let page = match std::env::var("BS_MENU").as_deref() {
        Ok("talents") => page_talents(&dj, &sauvegarde, choix.personnage),
        Ok("collection") => Page::Collection(collection::monde_de_l_arene(&dj, choix.arene)),
        Ok("profil") => Page::Profil,
        _ => Page::Accueil,
    };
    apercu::camera(&mut commandes, vue::FOND);
    commandes.spawn((
        Name::new("caméra du menu"),
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        ElementMenu,
    ));
    let mut menu = Menu {
        choix,
        page,
        racine: Entity::PLACEHOLDER,
        saisie: None,
        message: None,
        a_refaire: false,
        clavier_pris: false,
    };
    let mut ic = Icones {
        serveur: &serveur,
        textures: &mut textures,
        dessins: &dessins,
        images: &images,
    };
    menu.racine = construire(&mut commandes, &dj, &sauvegarde, &mut ic, &menu);
    commandes.insert_resource(menu);
}

/// Clics et touches du menu.
#[allow(clippy::too_many_arguments)]
pub fn gerer(
    mut commandes: Commands,
    mut menu: ResMut<Menu>,
    dj: Res<DonneesJeu>,
    mut sauvegarde: ResMut<Sauvegarde>,
    (serveur, mut textures, dessins, images): (
        Res<AssetServer>,
        ResMut<Textures>,
        Res<Dessins>,
        Res<Assets<Image>>,
    ),
    touches: Res<ButtonInput<KeyCode>>,
    boutons: Query<(&Interaction, &ActionMenu), Changed<Interaction>>,
    mut phase: ResMut<NextState<Phase>>,
    mut sortie: MessageWriter<AppExit>,
    mut essais: Local<u32>,
) {
    let mut action = boutons
        .iter()
        .find(|(i, _)| **i == Interaction::Pressed)
        .map(|(_, a)| *a);
    let mut auto = false;
    if menu.is_added() {
        let n: u32 = std::env::var("BS_JOUER").map_or(0, |v| v.parse().unwrap_or(1));
        auto = *essais < n;
        *essais += 1;
    }
    let accueil = menu.page == Page::Accueil;
    let clavier = !menu.clavier_pris && menu.saisie.is_none();
    let entree = touches.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]);
    if auto || (accueil && entree && clavier) {
        action = Some(ActionMenu::Jouer);
    } else if !accueil && clavier && touches.just_pressed(KeyCode::Escape) {
        action = Some(ActionMenu::Fermer);
    }
    let (mut choix, mut page) = (menu.choix, menu.page);
    match action {
        None if menu.a_refaire => {}
        None => return,
        Some(ActionMenu::Classe(k)) => choix.personnage = k,
        Some(ActionMenu::Arene(k)) => choix.arene = k,
        Some(ActionMenu::Talents) => page = page_talents(&dj, &sauvegarde, choix.personnage),
        Some(ActionMenu::Collection) => {
            page = Page::Collection(collection::monde_de_l_arene(&dj, choix.arene));
        }
        Some(ActionMenu::Monde(k)) => page = Page::Collection(k),
        Some(ActionMenu::OuvrirProfil) => page = Page::Profil,
        Some(ActionMenu::Profil(a)) => {
            profil::agir(&mut sauvegarde, &mut menu, a);
        }
        Some(ActionMenu::Fermer) => page = Page::Accueil,
        Some(ActionMenu::Talent(a)) => {
            let Page::Talents(mut etat) = page else {
                return;
            };
            if !talents::agir(&dj, &mut sauvegarde, &mut etat, a) {
                return;
            }
            page = Page::Talents(etat);
        }
        Some(ActionMenu::Jouer) => {
            let graine = lancement::graine_au_hasard();
            match PartiePreparee::nouvelle(&dj, &sauvegarde, choix, graine) {
                Ok(p) => {
                    info!("partie lancée du menu (graine {graine})");
                    p.inserer(&mut commandes);
                    phase.set(Phase::Chargement);
                }
                Err(e) => error!("partie impossible : {e}"),
            }
            return;
        }
        Some(ActionMenu::Quitter) => {
            sortie.write(AppExit::Success);
            return;
        }
    }
    if choix != menu.choix {
        lancement::memoriser(&dj, &sauvegarde, choix);
    }
    if page != menu.page {
        // Une page quittée oublie sa saisie et son message
        menu.saisie = None;
        menu.message = None;
    }
    commandes.entity(menu.racine).despawn();
    menu.choix = choix;
    menu.page = page;
    menu.a_refaire = false;
    let mut ic = Icones {
        serveur: &serveur,
        textures: &mut textures,
        dessins: &dessins,
        images: &images,
    };
    menu.racine = construire(&mut commandes, &dj, &sauvegarde, &mut ic, &menu);
}

/// Sortie du menu : tout ce qu'il a créé disparaît.
pub fn sortir(
    mut commandes: Commands,
    elements: Query<Entity, With<ElementMenu>>,
    mut ap: ResMut<apercu::Apercu>,
) {
    apercu::vider(&mut commandes, &mut ap);
    for e in &elements {
        commandes.entity(e).despawn();
    }
    commandes.remove_resource::<Menu>();
}
