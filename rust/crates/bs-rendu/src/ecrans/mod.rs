//! Écrans en jeu hors cartes (src/ui/ecranPause.ts, ecranFin.ts) : pause (Échap, ou
//! fenêtre qui perd le focus quand le joueur joue), statistiques de la pause, écran de fin
//! 1,6 s après la mort ou la victoire, mode sans fin après une victoire. Pas de pause
//! pendant les cartes ni après la fin. Le menu principal et les options ne sont pas
//! encore portés. Le HUD est masqué sous l'écran de fin.
//!
//! Essais : `BS_PAUSE=1` (ou `stats`) ouvre la pause (ou ses statistiques) au début.

mod bilan;
pub mod style;
mod vues;

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::WindowFocused;

use crate::PartieEnCours;
use crate::dessins::Dessins;
use crate::ecran_cartes::EcranCartes;
use crate::hud::police::TextePixel;
use crate::hud::{self, Hud};
use crate::textures::Textures;

pub use bilan::demander_textures;

/// Délai entre la fin de la partie et son écran (s).
pub const DELAI_FIN: f32 = 1.6;
/// Défilement par cran de molette (px).
const PIXELS_PAR_LIGNE: f32 = 40.0;
const ALERTE_SANS_FIN: &str = "Mode sans fin : tenez le plus longtemps possible !";

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Vue {
    #[default]
    Aucune,
    Pause,
    Statistiques,
    Fin,
}

/// Écran affiché et sa racine.
#[derive(Resource, Default)]
pub struct Ecrans {
    vue: Vue,
    racine: Option<Entity>,
}

impl Ecrans {
    /// La partie est en pause (pause ou ses statistiques).
    pub fn fige(&self) -> bool {
        matches!(self.vue, Vue::Pause | Vue::Statistiques)
    }

    /// Un écran est affiché (la molette ne zoome plus).
    pub fn ouvert(&self) -> bool {
        self.vue != Vue::Aucune
    }
}

/// Bouton d'un écran.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Reprendre,
    Recommencer,
    Statistiques,
    Retour,
    Rejouer,
    Continuer,
}

/// Ouvre, ferme et change d'écran selon les touches, les clics, le focus et l'issue.
#[allow(clippy::too_many_arguments)]
pub fn gerer(
    mut commandes: Commands,
    mut ecrans: ResMut<Ecrans>,
    mut jeu: ResMut<PartieEnCours>,
    cartes: Res<EcranCartes>,
    (serveur, textures, dessins): (Res<AssetServer>, Res<Textures>, Res<Dessins>),
    touches: Res<ButtonInput<KeyCode>>,
    mut focus: MessageReader<WindowFocused>,
    boutons: Query<(&Interaction, &Action), Changed<Interaction>>,
    (mut hud, mut textes, mut visibilites): (
        ResMut<Hud>,
        Query<&mut TextePixel>,
        Query<&mut Visibility>,
    ),
    mut debut: Local<bool>,
) {
    let perte_focus = focus.read().any(|f| !f.focused);
    let mut action = boutons
        .iter()
        .find(|(i, _)| **i == Interaction::Pressed)
        .map(|(_, a)| *a);
    let echap = touches.just_pressed(KeyCode::Escape);
    let entree = touches.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]);
    let finie = jeu.partie.etat.issue.is_some();
    let mut vue = ecrans.vue;
    match vue {
        Vue::Aucune => {
            let essai = if std::mem::replace(&mut *debut, true) {
                None
            } else {
                std::env::var("BS_PAUSE").ok()
            };
            if cartes.humain && finie && jeu.fin >= DELAI_FIN {
                vue = Vue::Fin;
            } else if !finie && !cartes.ouvert() {
                if echap || (perte_focus && cartes.humain) || essai.as_deref() == Some("1") {
                    vue = Vue::Pause;
                } else if essai.as_deref() == Some("stats") {
                    vue = Vue::Statistiques;
                }
            }
        }
        Vue::Pause if echap => action = Some(Action::Reprendre),
        Vue::Statistiques if echap => action = Some(Action::Retour),
        Vue::Fin if entree => action = Some(Action::Rejouer),
        _ => {}
    }
    match action {
        Some(Action::Reprendre) => vue = Vue::Aucune,
        Some(Action::Statistiques) => vue = Vue::Statistiques,
        Some(Action::Retour) => vue = Vue::Pause,
        Some(Action::Recommencer | Action::Rejouer) => {
            jeu.relancer();
            vue = Vue::Aucune;
        }
        Some(Action::Continuer) => {
            let p = &mut jeu.partie;
            p.etat.issue = None;
            p.etat.sans_fin = true;
            p.reprendre_directeurs_boss();
            jeu.fin = 0.0;
            hud::alerter(&mut hud, &mut textes, ALERTE_SANS_FIN, 4.0);
            vue = Vue::Aucune;
        }
        None => {}
    }
    if vue == ecrans.vue {
        return;
    }
    if let Some(r) = ecrans.racine.take() {
        commandes.entity(r).despawn();
    }
    ecrans.vue = vue;
    let c = bilan::Contexte {
        serveur: &serveur,
        textures: &textures,
        dessins: &dessins,
    };
    let p = &jeu.partie;
    ecrans.racine = match vue {
        Vue::Aucune => None,
        Vue::Pause => Some(vues::pause(&mut commandes, &c, p)),
        Vue::Statistiques => Some(vues::statistiques(&mut commandes, &c, p)),
        Vue::Fin => Some(vues::fin(&mut commandes, &c, p)),
    };
    if let Ok(mut v) = visibilites.get_mut(hud.racine) {
        *v = if vue == Vue::Fin {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

/// Molette : fait défiler les bilans affichés.
pub fn defiler(
    mut molette: MessageReader<MouseWheel>,
    mut zones: Query<(&mut ScrollPosition, &ComputedNode), With<vues::Defilant>>,
) {
    let dy: f32 = molette
        .read()
        .map(|m| match m.unit {
            MouseScrollUnit::Line => m.y * PIXELS_PAR_LIGNE,
            MouseScrollUnit::Pixel => m.y,
        })
        .sum();
    if dy == 0.0 {
        return;
    }
    for (mut s, n) in &mut zones {
        let max = (n.content_size.y - n.size.y).max(0.0) * n.inverse_scale_factor;
        s.0.y = (s.0.y - dy).clamp(0.0, max);
    }
}
