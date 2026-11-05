//! Map glissée sur la fenêtre (à l'accueil) : copiée dans le dossier des maps du joueur,
//! ajoutée aux arènes et choisie, volet des arènes ouvert avec un message. Une map que
//! l'import refuse au chargement ramène au menu avec la raison (`ImportRefuse`).

use bevy::prelude::*;
use bs_monde::minecraft::dossier::{ajouter_carte, est_une_map};

use super::{Menu, Page, Volet};
use crate::lancement::DonneesJeu;
use crate::sauvegarde::Sauvegarde;

/// Raison du refus d'une map au chargement, à montrer au retour au menu.
#[derive(Resource)]
pub struct ImportRefuse(pub String);

pub fn deposer_carte(
    mut deposes: MessageReader<FileDragAndDrop>,
    mut menu: ResMut<Menu>,
    mut dj: ResMut<DonneesJeu>,
    sauvegarde: Res<Sauvegarde>,
) {
    for e in deposes.read() {
        let FileDragAndDrop::DroppedFile { path_buf, .. } = e else {
            continue;
        };
        if menu.page != Page::Accueil || !est_une_map(path_buf) {
            continue;
        }
        let resultat = ajouter_carte(&sauvegarde.dossier_cartes(), path_buf)
            .and_then(|carte| dj.ajouter(carte));
        menu.message = Some(match resultat {
            Ok(k) => {
                menu.choix.arene = k;
                crate::lancement::memoriser(&dj, &sauvegarde, menu.choix);
                format!("Map ajoutée : {}.", dj.arenes[k].def.nom)
            }
            Err(e) => format!("Map refusée : {e}"),
        });
        menu.volet = Volet::Arenes;
        menu.a_refaire = true;
    }
}

/// Au retour au menu après un import refusé : la raison, dans le volet des arènes.
pub fn signaler_refus(
    mut commandes: Commands,
    refus: Option<Res<ImportRefuse>>,
    mut menu: ResMut<Menu>,
) {
    let Some(r) = refus else {
        return;
    };
    menu.message = Some(r.0.clone());
    menu.volet = Volet::Arenes;
    menu.a_refaire = true;
    commandes.remove_resource::<ImportRefuse>();
}
