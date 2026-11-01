//! Menu principal (phase `Menu`, src/ui/ecranMenu.ts) : choix de la classe et de
//! l'arène (gardé dans le profil), « Jouer » (ou Entrée) prépare la partie choisie et
//! passe au chargement, « Quitter » ferme le jeu. Le menu a sa propre caméra 2D ; tout
//! ce qu'il crée porte `ElementMenu` et disparaît à sa sortie.
//!
//! Essai : `BS_JOUER=n` lance la partie choisie aux n premières entrées au menu.

mod vue;

use bevy::app::AppExit;
use bevy::prelude::*;

use crate::Phase;
use crate::lancement::{self, ChoixPartie, DonneesJeu, PartiePreparee};
use crate::sauvegarde::Sauvegarde;

/// Bouton du menu.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionMenu {
    Classe(usize),
    Arene(usize),
    Jouer,
    Quitter,
}

/// Marque de tout ce que le menu crée.
#[derive(Component)]
pub struct ElementMenu;

/// Choix affiché et racine de l'interface.
#[derive(Resource)]
pub struct Menu {
    choix: ChoixPartie,
    racine: Entity,
}

/// Entrée au menu : caméra et interface, sur le dernier choix du profil.
pub fn entrer(mut commandes: Commands, dj: Res<DonneesJeu>, sauvegarde: Res<Sauvegarde>) {
    let choix = lancement::choix_memorise(&dj, &sauvegarde);
    commandes.spawn((Name::new("caméra du menu"), Camera2d, ElementMenu));
    let racine = vue::construire(&mut commandes, &dj, &sauvegarde, choix);
    commandes.entity(racine).insert(ElementMenu);
    commandes.insert_resource(Menu { choix, racine });
}

/// Clics et touches du menu.
pub fn gerer(
    mut commandes: Commands,
    mut menu: ResMut<Menu>,
    dj: Res<DonneesJeu>,
    sauvegarde: Res<Sauvegarde>,
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
    if auto || touches.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]) {
        action = Some(ActionMenu::Jouer);
    }
    let mut choix = menu.choix;
    match action {
        None => return,
        Some(ActionMenu::Classe(k)) => choix.personnage = k,
        Some(ActionMenu::Arene(k)) => choix.arene = k,
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
    if choix == menu.choix {
        return;
    }
    lancement::memoriser(&dj, &sauvegarde, choix);
    commandes.entity(menu.racine).despawn();
    let racine = vue::construire(&mut commandes, &dj, &sauvegarde, choix);
    commandes.entity(racine).insert(ElementMenu);
    *menu = Menu { choix, racine };
}

/// Sortie du menu : tout ce qu'il a créé disparaît.
pub fn sortir(mut commandes: Commands, elements: Query<Entity, With<ElementMenu>>) {
    for e in &elements {
        commandes.entity(e).despawn();
    }
    commandes.remove_resource::<Menu>();
}
