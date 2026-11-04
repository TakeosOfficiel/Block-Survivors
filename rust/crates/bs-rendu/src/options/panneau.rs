//! Panneau des options (src/ui/panneauOptions.ts), par-dessus le menu ou la pause (vue :
//! `vue`) : un rail pour le volume général et un pour chaque famille de sons (clic ou
//! glissé), des bascules pour le reste, « Retour » ou Échap pour fermer. Construit une fois à
//! l'ouverture ; les libellés et les curseurs suivent les options sans le refaire (un
//! curseur reste saisi pendant qu'on le glisse).

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use super::Options;
use super::vue::{HALO, Survol, TERNE};
use crate::ecrans::charte::OR_VIF;
use crate::polices::PolicesMenu;

const LIBELLES_CATEGORIES: [&str; 4] = ["Joueur", "Armes", "Créatures", "Interface"];
const LIBELLES_OMBRES: [&str; 3] = ["Non", "Oui", "Fines"];

/// Panneau ouvert (sa racine) ; `ouvrir` le demande pour l'image suivante.
#[derive(Resource, Default)]
pub struct PanneauOptions {
    racine: Option<Entity>,
    demande: bool,
}

impl PanneauOptions {
    pub fn ouvrir(&mut self) {
        self.demande = true;
    }

    /// Ouvert (ou sur le point de l'être) : Échap et Entrée sont pour lui.
    pub fn ouvert(&self) -> bool {
        self.racine.is_some() || self.demande
    }
}

/// Réglage d'une ligne du panneau.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Reglage {
    /// 0 : général, puis les familles de sons.
    Volume(usize),
    Fps,
    Secousses,
    Ombres,
    Chiffres,
    Compteur,
    Vue,
    Beta,
    /// Effets des autres joueurs (coop).
    Allies,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum ActionOptions {
    Basculer(Reglage),
    Fermer,
}

/// Curseur de volume (sa barre suit la valeur).
#[derive(Component)]
pub(super) struct Curseur(pub(super) usize);

#[derive(Component)]
pub(super) struct Barre(pub(super) usize);

/// Texte d'un réglage.
#[derive(Component)]
pub(super) struct Libelle(pub(super) Reglage);

/// Nom d'un réglage.
pub(super) fn nom(r: Reglage) -> &'static str {
    match r {
        Reglage::Volume(0) => "Volume général",
        Reglage::Volume(k) => LIBELLES_CATEGORIES[k - 1],
        Reglage::Fps => "Compteur de FPS",
        Reglage::Secousses => "Secousses de l'écran",
        Reglage::Ombres => "Ombres et vent",
        Reglage::Chiffres => "Chiffres des dégâts",
        Reglage::Compteur => "Compteur de dégâts (Tab)",
        Reglage::Vue => "Vue (F5)",
        Reglage::Beta => "Versions bêta",
        Reglage::Allies => "Effets des autres joueurs",
    }
}

/// Valeur affichée d'un réglage, et s'il est actif (en or).
pub(super) fn valeur(o: &Options, r: Reglage) -> (String, bool) {
    let oui = |b: bool| (if b { "Oui" } else { "Non" }.to_owned(), b);
    match r {
        Reglage::Volume(k) => match (volume(o, k) * 100.0).round() as u32 {
            0 => ("Muet".to_owned(), false),
            p => (format!("{p} %"), true),
        },
        Reglage::Fps => oui(o.afficher_fps),
        Reglage::Secousses => oui(o.secousses),
        Reglage::Ombres => (LIBELLES_OMBRES[o.ombres].to_owned(), o.ombres > 0),
        Reglage::Chiffres => oui(o.chiffres_degats),
        Reglage::Compteur => oui(o.compteur_degats),
        Reglage::Vue => {
            let t = if o.troisieme_personne {
                "3e personne"
            } else {
                "De dessus"
            };
            (t.to_owned(), true)
        }
        Reglage::Beta => oui(o.beta),
        Reglage::Allies => oui(o.effets_allies),
    }
}

pub(super) fn volume(o: &Options, k: usize) -> f32 {
    if k == 0 { o.volume } else { o.volumes[k - 1] }
}

/// Ouvre (à la demande), ferme (Terminé, Échap) et applique les clics.
#[allow(clippy::type_complexity)]
pub(super) fn gerer(
    mut commandes: Commands,
    mut panneau: ResMut<PanneauOptions>,
    mut options: ResMut<Options>,
    touches: Res<ButtonInput<KeyCode>>,
    boutons: Query<(&Interaction, &ActionOptions), Changed<Interaction>>,
    curseurs: Query<(&Interaction, &Curseur, &RelativeCursorPosition)>,
    (polices, mut survols, mut essai): (
        Res<PolicesMenu>,
        Query<(&Interaction, &mut BackgroundColor), (With<Survol>, Changed<Interaction>)>,
        Local<bool>,
    ),
) {
    // Capture d'écran : BS_MENU=options ouvre le panneau d'emblée
    if !std::mem::replace(&mut *essai, true) && std::env::var("BS_MENU").as_deref() == Ok("options")
    {
        panneau.demande = true;
    }
    for (i, mut fond) in &mut survols {
        fond.0 = match i {
            Interaction::None => Color::NONE,
            _ => HALO,
        };
    }
    if std::mem::take(&mut panneau.demande) {
        if let Some(r) = panneau.racine.take() {
            commandes.entity(r).despawn();
        }
        panneau.racine = Some(super::vue::construire(&mut commandes, &polices, &options));
        return;
    }
    let Some(racine) = panneau.racine else {
        return;
    };
    let clic = boutons
        .iter()
        .find(|(i, _)| **i == Interaction::Pressed)
        .map(|(_, a)| *a);
    let echap = touches.just_pressed(KeyCode::Escape);
    match clic {
        _ if echap => {}
        Some(ActionOptions::Fermer) => {}
        Some(ActionOptions::Basculer(r)) => {
            let o = &mut *options;
            match r {
                Reglage::Fps => o.afficher_fps = !o.afficher_fps,
                Reglage::Secousses => o.secousses = !o.secousses,
                Reglage::Ombres => o.ombres = (o.ombres + 1) % LIBELLES_OMBRES.len(),
                Reglage::Chiffres => o.chiffres_degats = !o.chiffres_degats,
                Reglage::Compteur => o.compteur_degats = !o.compteur_degats,
                Reglage::Vue => o.troisieme_personne = !o.troisieme_personne,
                Reglage::Beta => o.beta = !o.beta,
                Reglage::Allies => o.effets_allies = !o.effets_allies,
                Reglage::Volume(_) => {}
            }
            return;
        }
        None => {
            // Curseurs : la valeur suit le pointeur tant qu'il est appuyé (pas de 5 %)
            for (i, c, position) in &curseurs {
                let Some(n) = position.normalized.filter(|_| *i == Interaction::Pressed) else {
                    continue;
                };
                let v = ((n.x + 0.5).clamp(0.0, 1.0) * 20.0).round() / 20.0;
                if volume(&options, c.0) != v {
                    match c.0 {
                        0 => options.volume = v,
                        k => options.volumes[k - 1] = v,
                    }
                }
            }
            return;
        }
    }
    commandes.entity(racine).despawn();
    panneau.racine = None;
}

/// Libellés et barres à jour des options.
pub(super) fn rafraichir(
    options: Res<Options>,
    mut libelles: Query<(&Libelle, &mut Text, &mut TextColor)>,
    mut barres: Query<(&Barre, &mut Node)>,
) {
    if !options.is_changed() {
        return;
    }
    for (l, mut t, mut couleur) in &mut libelles {
        let (v, actif) = valeur(&options, l.0);
        if t.0 != v {
            t.0 = v;
        }
        couleur.0 = if actif { OR_VIF } else { TERNE };
    }
    for (b, mut n) in &mut barres {
        let largeur = Val::Percent(volume(&options, b.0) * 100.0);
        if n.width != largeur {
            n.width = largeur;
        }
    }
}

/// Ferme le panneau (changement de phase).
pub(super) fn fermer(mut commandes: Commands, mut panneau: ResMut<PanneauOptions>) {
    if let Some(r) = panneau.racine.take() {
        commandes.entity(r).despawn();
    }
    panneau.demande = false;
}
