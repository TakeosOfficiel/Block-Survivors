//! Menu principal (phase `Menu`, src/ui/ecranMenu.ts) : choix de la classe et de
//! l'arène (gardé dans le profil), « Jouer » (ou Entrée) prépare la partie choisie et
//! passe au chargement, « Quitter » ferme le jeu. Ses pages (Talents, Collection, Profil, Profils, Nouveautés, Multijoueur) remplacent
//! l'accueil ; Échap ou « Retour » y ramène. Le menu a sa caméra 2D, et une caméra 3D dessous pour l'aperçu du héros ; tout ce
//! qu'il crée porte `ElementMenu` et disparaît à sa sortie. Chaque changement refait la
//! page affichée en entier (hors de la boucle de jeu).
//!
//! Essais : `BS_JOUER=n` lance la partie choisie aux n premières entrées au menu ;
//! `BS_MENU=talents` (ou `collection`, `profil`, `profils`, `nouveautes`, `multijoueur`) ouvre cette page à l'entrée.

mod air;
pub mod apercu;
mod collection;
pub mod fond;
pub mod multijoueur;
mod nouveautes;
pub mod profil;
mod profils;
mod talents;
mod vue;
pub use talents::survoler_cases;
pub use vue::survoler_liens;

use bevy::app::AppExit;
use bevy::prelude::*;

use crate::Phase;
use crate::chargement::{Photos, photo};
use crate::coop::{Coop, MessageCoop};
use crate::dessins::Dessins;
use crate::lancement::{self, ChoixPartie, DonneesJeu, PartiePreparee};
use crate::polices::PolicesMenu;
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
    /// Tire un pseudo au hasard (le dé de l'accueil).
    PseudoAleatoire,
    Profil(profil::ActionProfil),
    /// Ouvre la page des Profils.
    OuvrirProfils,
    Profils(profils::ActionProfils),
    /// Revient à l'accueil.
    Fermer,
    /// Ouvre le panneau des options.
    Options,
    /// Ouvre les Nouveautés.
    Nouveautes,
    /// Déplie une version des Nouveautés.
    Version(usize),
    Talent(ActionTalent),
    /// Ouvre la page Multijoueur.
    OuvrirMulti,
    Multi(multijoueur::ActionMulti),
    /// Ouvre ce volet de l'accueil (le referme s'il l'est déjà).
    Volet(Volet),
}

/// Volet ouvert sur l'accueil, au-dessus du choix de partie.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Volet {
    #[default]
    Aucun,
    Classes,
    Arenes,
}

/// Page affichée.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Page {
    Accueil,
    Talents(EtatTalents),
    /// Collection, sur un monde.
    Collection(usize),
    Profil,
    /// Profils ; celui dont la suppression attend confirmation.
    Profils(Option<usize>),
    /// Nouveautés ; la version dépliée.
    Nouveautes(usize),
    Multijoueur,
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
    /// Saisie validée par Entrée, à traiter selon la page.
    validee: Option<String>,
    /// Fichier glissé sur la page Profils.
    depose: Option<std::path::PathBuf>,
    /// Pseudo proposé pour un nouveau profil.
    suggestion: String,
    /// Volet ouvert de l'accueil.
    volet: Volet,
}

/// Icônes des pages (chargées à la demande).
pub struct Icones<'a> {
    pub serveur: &'a AssetServer,
    pub textures: &'a mut Textures,
    pub dessins: &'a Dessins,
    pub images: &'a Assets<Image>,
    pub polices: &'a PolicesMenu,
    /// Photos des arènes, décodées à l'entrée au menu.
    pub photos: &'a Photos,
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
    multi: &multijoueur::VueMulti,
) -> Entity {
    let (choix, page) = (menu.choix, menu.page);
    let racine = match page {
        Page::Accueil => vue::construire(commandes, dj, sauvegarde, ic, (choix, menu.volet)),
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
        Page::Profils(_) => profils::construire(commandes, (dj, sauvegarde), ic, menu),
        Page::Nouveautes(k) => nouveautes::construire(commandes, ic, k),
        Page::Multijoueur => multijoueur::construire(commandes, dj, ic, menu, multi),
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
        confirmation: false,
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
    (mut images, mut photos, polices): (ResMut<Assets<Image>>, ResMut<Photos>, Res<PolicesMenu>),
    (message, mut annuaire): (Res<MessageCoop>, ResMut<multijoueur::Annuaire>),
) {
    // Les vignettes des arènes ont besoin de leurs photos dès la construction
    for a in &dj.arenes {
        photo(&mut images, &mut photos, &a.def.id);
    }
    let choix = lancement::choix_memorise(&dj, &sauvegarde);
    let page = match std::env::var("BS_MENU").as_deref() {
        Ok("talents") => page_talents(&dj, &sauvegarde, choix.personnage),
        Ok("collection") => Page::Collection(collection::monde_de_l_arene(&dj, choix.arene)),
        Ok("profil") => Page::Profil,
        Ok("profils") => Page::Profils(None),
        Ok("nouveautes") => Page::Nouveautes(0),
        Ok("multijoueur") => Page::Multijoueur,
        _ if message.0.is_some() => Page::Multijoueur,
        _ => Page::Accueil,
    };
    fond::creer(&mut commandes, vue::FOND);
    apercu::camera(&mut commandes);
    commandes.spawn((
        Name::new("caméra du menu"),
        Camera2d,
        IsDefaultUiCamera,
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
        validee: None,
        depose: None,
        suggestion: profils::suggestion(),
        volet: match std::env::var("BS_VOLET").as_deref() {
            Ok("classes") => Volet::Classes,
            Ok("arenes") => Volet::Arenes,
            _ => Volet::Aucun,
        },
    };
    let mut ic = Icones {
        serveur: &serveur,
        textures: &mut textures,
        dessins: &dessins,
        images: &images,
        polices: &polices,
        photos: &photos,
    };
    if page == Page::Multijoueur {
        annuaire.actualiser();
    }
    let multi = multijoueur::VueMulti {
        coop: None,
        message: message.0.as_deref(),
        annuaire: &annuaire,
        amis: None,
        liste: &sauvegarde.amis.amis,
        progression: &sauvegarde.progression,
        pseudo: &sauvegarde.profil.pseudo,
    };
    menu.racine = construire(&mut commandes, &dj, &sauvegarde, &mut ic, &menu, &multi);
    commandes.insert_resource(menu);
}

/// Clics et touches du menu.
#[allow(clippy::too_many_arguments)]
pub fn gerer(
    mut commandes: Commands,
    mut menu: ResMut<Menu>,
    dj: Res<DonneesJeu>,
    mut sauvegarde: ResMut<Sauvegarde>,
    (serveur, mut textures, dessins, mut images): (
        Res<AssetServer>,
        ResMut<Textures>,
        Res<Dessins>,
        ResMut<Assets<Image>>,
    ),
    (polices, photos): (Res<PolicesMenu>, Res<Photos>),
    touches: Res<ButtonInput<KeyCode>>,
    boutons: Query<(&Interaction, &ActionMenu), Changed<Interaction>>,
    mut phase: ResMut<NextState<Phase>>,
    mut sortie: MessageWriter<AppExit>,
    (mut essais, mut options): (Local<u32>, ResMut<crate::options::PanneauOptions>),
    (mut coop, mut message, mut annuaire, mut amis): (
        Option<ResMut<Coop>>,
        ResMut<MessageCoop>,
        ResMut<multijoueur::Annuaire>,
        Option<ResMut<crate::amis::ReseauAmis>>,
    ),
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
    let clavier = !menu.clavier_pris && menu.saisie.is_none() && !options.ouvert();
    let entree = touches.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]);
    let echap = clavier && touches.just_pressed(KeyCode::Escape);
    if auto || (accueil && entree && clavier) {
        action = Some(ActionMenu::Jouer);
    } else if echap && !accueil {
        action = Some(ActionMenu::Fermer);
    } else if echap && menu.volet != Volet::Aucun {
        action = Some(ActionMenu::Volet(Volet::Aucun));
    }
    let mut suite = None;
    // Quitter la page Multijoueur, c'est quitter son salon
    let sur_multi = menu.page == Page::Multijoueur;
    let multi = match action {
        Some(ActionMenu::Multi(a)) => Some(a),
        Some(ActionMenu::Fermer) if sur_multi => Some(multijoueur::ActionMulti::Quitter),
        _ => None,
    };
    let code = menu.validee.take_if(|_| sur_multi);
    if multi.is_some() || code.is_some() {
        let mut c = multijoueur::Contexte {
            commandes: &mut commandes,
            dj: &dj,
            sauvegarde: &mut sauvegarde,
            coop: coop.as_deref_mut(),
            amis: amis.as_deref_mut(),
            message: &mut message,
            annuaire: &mut annuaire,
            phase: &mut phase,
        };
        if multijoueur::traiter(&mut c, &mut menu, code.as_deref(), multi) {
            coop = None;
        }
    }
    if let Some(s) = menu.validee.take() {
        match menu.page {
            Page::Profil => profil::valider(&mut sauvegarde.profil, &s),
            Page::Profils(_) => suite = Some(profils::creer(&mut sauvegarde, &mut menu, &s)),
            _ => {}
        }
    }
    if let Some(f) = menu.depose.take() {
        suite = Some(profils::importer(&mut sauvegarde, &mut menu, &f));
    }
    if let Some(ActionMenu::Profils(a)) = action {
        suite = Some(profils::agir(&mut sauvegarde, &mut menu, a));
    }
    if suite == Some(profils::Suite::Recharger) {
        // Autre profil : ses données, sa skin, son dernier choix ; retour à l'accueil
        sauvegarde.recharger(dj.d);
        if let Some(image) = sauvegarde.skin_importee() {
            let chemin = bs_sauvegarde::profil_joueur::CHEMIN_SKIN_IMPORTE;
            textures.inserer(chemin, images.add(image));
        }
        menu.choix = lancement::choix_memorise(&dj, &sauvegarde);
        menu.page = Page::Accueil;
        menu.a_refaire = true;
    }
    let (mut choix, mut page) = (menu.choix, menu.page);
    match action {
        _ if suite.is_some() => {}
        None if menu.a_refaire => {}
        None => return,
        Some(ActionMenu::Classe(k)) => {
            choix.personnage = k;
            menu.volet = Volet::Aucun;
        }
        Some(ActionMenu::Arene(k)) => {
            choix.arene = k;
            menu.volet = Volet::Aucun;
        }
        Some(ActionMenu::Volet(v)) => {
            menu.volet = if menu.volet == v { Volet::Aucun } else { v };
        }
        Some(ActionMenu::Talents) => page = page_talents(&dj, &sauvegarde, choix.personnage),
        Some(ActionMenu::Collection) => {
            page = Page::Collection(collection::monde_de_l_arene(&dj, choix.arene));
        }
        Some(ActionMenu::Monde(k)) => page = Page::Collection(k),
        Some(ActionMenu::OuvrirProfil) => page = Page::Profil,
        Some(ActionMenu::PseudoAleatoire) => {
            profil::valider(&mut sauvegarde.profil, &profils::suggestion());
        }
        Some(ActionMenu::OuvrirProfils) => {
            page = Page::Profils(None);
            menu.suggestion = profils::suggestion();
        }
        Some(ActionMenu::Profils(_)) => {}
        Some(ActionMenu::Profil(a)) => {
            profil::agir(&mut sauvegarde, &mut menu, a);
        }
        Some(ActionMenu::Fermer) => page = Page::Accueil,
        Some(ActionMenu::OuvrirMulti) => {
            annuaire.actualiser();
            page = Page::Multijoueur;
        }
        Some(ActionMenu::Multi(_)) if menu.page != Page::Multijoueur => return,
        Some(ActionMenu::Multi(_)) => {}
        Some(ActionMenu::Nouveautes) => {
            nouveautes::marquer_lues(sauvegarde.profils.stockage());
            page = Page::Nouveautes(0);
        }
        Some(ActionMenu::Version(k)) => page = Page::Nouveautes(k),
        Some(ActionMenu::Options) => {
            options.ouvrir();
            return;
        }
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
        // Une page quittée oublie sa saisie, son message et son volet
        menu.saisie = None;
        menu.message = None;
        menu.volet = Volet::Aucun;
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
        polices: &polices,
        photos: &photos,
    };
    let multi = multijoueur::VueMulti {
        coop: coop.as_deref(),
        message: message.0.as_deref(),
        annuaire: &annuaire,
        amis: amis.as_deref(),
        liste: &sauvegarde.amis.amis,
        progression: &sauvegarde.progression,
        pseudo: &sauvegarde.profil.pseudo,
    };
    menu.racine = construire(&mut commandes, &dj, &sauvegarde, &mut ic, &menu, &multi);
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
