//! Écrans en jeu hors cartes (src/ui/ecranPause.ts, ecranFin.ts) : pause (Échap, ou
//! fenêtre qui perd le focus quand le joueur joue), statistiques de la pause, écran de fin
//! 1,6 s après la mort ou la victoire, mode sans fin après une victoire. Pas de pause
//! pendant les cartes ni après la fin. « Menu principal » abandonne la partie (une
//! partie finie est déjà comptée). Le HUD est masqué sous l'écran de fin.
//!
//! Recommencer et Menu principal demandent confirmation depuis la pause.
//!
//! Essais : `BS_PAUSE=1` (ou `stats`) ouvre la pause (ou ses statistiques) au début ;
//! `BS_PAUSE=menu` revient aussitôt au menu principal.

mod bilan;
pub mod charte;
mod fin;
mod nuit;
pub mod style;
pub(crate) mod vues;

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::WindowFocused;

use crate::PartieEnCours;
use crate::coop::Coop;
use crate::dessins::Dessins;
use crate::ecran_cartes::EcranCartes;
use crate::hud::police::TextePixel;
use crate::hud::{self, Hud};
use crate::sauvegarde::Sauvegarde;
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
    /// Confirmation de Recommencer ou Menu principal depuis la pause.
    Confirmer(Action),
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
        matches!(self.vue, Vue::Pause | Vue::Statistiques | Vue::Confirmer(_))
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
    /// Retour au menu principal (la partie est abandonnée).
    Menu,
    /// Ouvre le panneau des options (par-dessus la pause).
    Options,
}

/// Ouvre, ferme et change d'écran selon les touches, les clics, le focus et l'issue.
#[allow(clippy::too_many_arguments)]
pub fn gerer(
    mut commandes: Commands,
    mut ecrans: ResMut<Ecrans>,
    mut jeu: ResMut<PartieEnCours>,
    cartes: Res<EcranCartes>,
    mut sauvegarde: ResMut<Sauvegarde>,
    (serveur, textures, dessins, polices): (
        Res<AssetServer>,
        Res<Textures>,
        Res<Dessins>,
        Res<crate::polices::PolicesMenu>,
    ),
    touches: Res<ButtonInput<KeyCode>>,
    mut focus: MessageReader<WindowFocused>,
    boutons: Query<(&Interaction, &Action), Changed<Interaction>>,
    (mut hud, mut textes, mut visibilites): (
        ResMut<Hud>,
        Query<&mut TextePixel>,
        Query<&mut Visibility>,
    ),
    (mut debut, mut phase, mut options): (
        Local<bool>,
        ResMut<NextState<crate::Phase>>,
        ResMut<crate::options::PanneauOptions>,
    ),
    mut coop: Option<ResMut<Coop>>,
) {
    let perte_focus = focus.read().any(|f| !f.focused);
    let mut action = boutons
        .iter()
        .find(|(i, _)| **i == Interaction::Pressed)
        .map(|(_, a)| *a);
    // Échap ferme d'abord les options
    let echap = touches.just_pressed(KeyCode::Escape) && !options.ouvert();
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
                } else if essai.as_deref() == Some("confirmer") {
                    vue = Vue::Confirmer(Action::Menu);
                } else if essai.as_deref() == Some("menu") {
                    action = Some(Action::Menu);
                }
            }
        }
        Vue::Pause if echap || entree => action = Some(Action::Reprendre),
        Vue::Confirmer(_) if echap || entree => action = Some(Action::Retour),
        // Recommencer et Menu principal, depuis la pause : confirmer d'abord
        Vue::Pause
            if matches!(action, Some(Action::Recommencer | Action::Menu))
                && !(action == Some(Action::Recommencer) && coop.is_some()) =>
        {
            vue = Vue::Confirmer(action.take().expect("action"));
        }
        Vue::Statistiques if echap => action = Some(Action::Retour),
        // Coop : l'hôte a repris la partie (« Continuer sans fin »)
        Vue::Fin if !finie => {
            sauvegarde.continuer_sans_fin(&jeu.partie);
            jeu.fin = 0.0;
            hud::alerter(&mut hud, &mut textes, ALERTE_SANS_FIN, 4.0);
            vue = Vue::Aucune;
        }
        Vue::Fin if entree && coop.is_none() => action = Some(Action::Rejouer),
        _ => {}
    }
    match action {
        // En coop, on ne recommence pas seul
        Some(Action::Recommencer | Action::Rejouer) if coop.is_some() => {}
        Some(Action::Continuer) if coop.is_some() => {
            let coop = coop.as_deref_mut().expect("coop");
            if crate::coop::continuer(coop, &mut jeu.partie) {
                sauvegarde.continuer_sans_fin(&jeu.partie);
                jeu.fin = 0.0;
                hud::alerter(&mut hud, &mut textes, ALERTE_SANS_FIN, 4.0);
                vue = Vue::Aucune;
            }
        }
        Some(Action::Reprendre) => vue = Vue::Aucune,
        Some(Action::Statistiques) => vue = Vue::Statistiques,
        Some(Action::Retour) => vue = Vue::Pause,
        Some(Action::Recommencer | Action::Rejouer) => {
            sauvegarde.recompense = None;
            jeu.relancer();
            vue = Vue::Aucune;
        }
        Some(Action::Continuer) => {
            sauvegarde.continuer_sans_fin(&jeu.partie);
            jeu.partie.continuer_sans_fin();
            jeu.fin = 0.0;
            hud::alerter(&mut hud, &mut textes, ALERTE_SANS_FIN, 4.0);
            vue = Vue::Aucune;
        }
        Some(Action::Options) => options.ouvrir(),
        Some(Action::Menu) => {
            sauvegarde.recompense = None;
            phase.set(crate::Phase::Menu);
            return;
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
    if vue == Vue::Fin {
        let joueurs = coop.as_deref().map(Coop::joueurs).unwrap_or_default();
        let classes = joueurs.iter().map(|j| j.presentation.personnage.as_str());
        sauvegarde.terminer(&jeu.partie, classes);
    }
    let c = bilan::Contexte {
        serveur: &serveur,
        textures: &textures,
        dessins: &dessins,
        polices: &polices,
    };
    let p = &jeu.partie;
    ecrans.racine = match vue {
        Vue::Aucune => None,
        Vue::Pause => Some(vues::pause(&mut commandes, &c, p, coop.is_some())),
        Vue::Statistiques => Some(vues::statistiques(&mut commandes, &c, p)),
        Vue::Confirmer(a) => Some(vues::confirmer(&mut commandes, &c, a)),
        Vue::Fin => Some(fin::fin(
            &mut commandes,
            &c,
            p,
            sauvegarde.recompense.as_ref(),
            coop.is_some(),
        )),
    };
    if let Ok(mut v) = visibilites.get_mut(hud.racine) {
        // Masqué sous la fin et les statistiques (rien ne dépasse du panneau)
        *v = if matches!(vue, Vue::Fin | Vue::Statistiques) {
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
