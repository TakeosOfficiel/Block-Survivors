//! Page Profil du menu (src/ui/ecranProfil.ts) : skin (celle de la classe, une des
//! skins par défaut du jeu, ou une skin importée en glissant un PNG sur la fenêtre),
//! gabarit des bras, pseudo (clic, saisie au clavier, Entrée), pièces d'armure de la
//! Collection portées ; le héros tourne en 3D au centre (`apercu`). Chaque changement
//! est enregistré aussitôt.

mod page;

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bs_sauvegarde::collection::{MONDES, PIECES};
use bs_sauvegarde::profil_joueur::{
    CHEMIN_SKIN_IMPORTE, ChoixSkin, LONGUEUR_PSEUDO, nettoyer_pseudo,
};

use super::apercu::{self, Apercu};
use super::{Menu, Page};
use crate::sauvegarde::Sauvegarde;

/// Skin d'un bouton (copiable).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Skin {
    Classe,
    Importe,
    Vanilla(usize),
}

impl Skin {
    fn choix(self) -> ChoixSkin {
        match self {
            Self::Classe => ChoixSkin::Classe,
            Self::Importe => ChoixSkin::Importe,
            Self::Vanilla(k) => ChoixSkin::Vanilla(k),
        }
    }
}

/// Bouton de la page.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionProfil {
    Skin(Skin),
    Bras,
    /// Commence la saisie du pseudo (ou la valide).
    Pseudo,
    /// Emplacement (ordre de `PIECES`), monde (ordre de `MONDES`) ou rien.
    Porter(usize, Option<usize>),
}

/// Applique une action ; vrai si la page est à refaire.
pub fn agir(sauvegarde: &mut Sauvegarde, menu: &mut Menu, action: ActionProfil) -> bool {
    let profil = &mut sauvegarde.profil;
    match action {
        ActionProfil::Skin(s) => profil.skin = s.choix(),
        ActionProfil::Bras => profil.bras_fins = !profil.bras_fins,
        ActionProfil::Pseudo => {
            match menu.saisie.take() {
                Some(s) => valider(profil, &s),
                None => menu.saisie = Some(profil.pseudo.clone()),
            }
            return true;
        }
        ActionProfil::Porter(p, m) => {
            let monde = m.map(|k| MONDES[k]);
            sauvegarde.collection.porter(PIECES[p], monde);
            return true;
        }
    }
    profil.enregistrer();
    true
}

pub fn valider(profil: &mut bs_sauvegarde::profil_joueur::ProfilJoueur, saisie: &str) {
    let pseudo = nettoyer_pseudo(saisie);
    if !pseudo.is_empty() {
        profil.pseudo = pseudo;
        profil.enregistrer();
    }
}

/// Saisie au clavier (pseudo, nouveau profil : Entrée la passe à `gerer`), fichier
/// glissé sur la fenêtre (skin sur la page Profil, `.bssave` sur la page Profils).
#[allow(clippy::too_many_arguments)]
pub fn saisir(
    mut menu: ResMut<Menu>,
    mut sauvegarde: ResMut<Sauvegarde>,
    mut apercu: ResMut<Apercu>,
    mut commandes: Commands,
    mut clavier: MessageReader<KeyboardInput>,
    mut deposes: MessageReader<FileDragAndDrop>,
    mut textures: ResMut<crate::textures::Textures>,
    mut images: ResMut<Assets<Image>>,
) {
    menu.bypass_change_detection().clavier_pris = false;
    let profil_ouvert = menu.page == Page::Profil;
    for e in deposes.read() {
        let FileDragAndDrop::DroppedFile { path_buf, .. } = e else {
            continue;
        };
        if matches!(menu.page, Page::Profils(_)) {
            menu.depose = Some(path_buf.clone());
            menu.a_refaire = true;
        }
        if !profil_ouvert {
            continue;
        }
        let lu = std::fs::read(path_buf).map_err(|_| "Skin refusée : fichier illisible.".into());
        menu.message = Some(match lu.and_then(|o| crate::skins::normaliser(&o)) {
            Ok(s) => {
                let message = if s.reduite {
                    format!(
                        "Skin réduite à {0}×{0} pour tenir dans la sauvegarde.",
                        s.taille
                    )
                } else {
                    "Skin importée.".to_owned()
                };
                let profil = &mut sauvegarde.profil;
                profil.skin_importe = Some(s.adresse);
                profil.skin = ChoixSkin::Importe;
                profil.enregistrer();
                if let Some(image) = sauvegarde.skin_importee() {
                    textures.inserer(CHEMIN_SKIN_IMPORTE, images.add(image));
                }
                apercu::vider(&mut commandes, &mut apercu);
                message
            }
            Err(e) => e,
        });
        menu.a_refaire = true;
    }
    if menu.saisie.is_none() {
        clavier.clear();
        return;
    }
    for e in clavier.read() {
        if e.state != ButtonState::Pressed {
            continue;
        }
        let Some(saisie) = menu.saisie.as_mut() else {
            break;
        };
        match &e.logical_key {
            Key::Character(c) => {
                if saisie.chars().count() < LONGUEUR_PSEUDO {
                    saisie.extend(c.chars().filter(|c| !c.is_control()));
                }
            }
            Key::Space if saisie.chars().count() < LONGUEUR_PSEUDO => saisie.push(' '),
            Key::Backspace => {
                saisie.pop();
            }
            Key::Enter => menu.validee = Some(menu.saisie.take().unwrap_or_default()),
            Key::Escape => menu.saisie = None,
            _ => continue,
        }
        menu.clavier_pris = true;
        menu.a_refaire = true;
    }
}

pub use page::construire;
