//! Colonne des amis de la page Multijoueur : son code ami, l'ajout d'un ami par le sien,
//! les messages reçus (invitation, demande, ajout) et la liste des amis avec leur présence
//! et ce qu'on peut faire avec eux (rejoindre, inviter, demander, retirer).

use bevy::prelude::*;
use bs_reseau::amis::{Etat, EtatPresence};
use bs_sauvegarde::amis::Ami;

use super::ActionMulti;
use super::vue::{VueMulti, bouton, ligne, texte};
use crate::amis::Note;
use crate::ecrans::style::{GRIS, OR, OR_CLAIR, TEXTE};
use crate::menu::Menu;

pub const LARGEUR: f32 = 420.0;
const VERT: Color = Color::srgb_u8(0x6d, 0xe0, 0x6d);
const BLEU: Color = Color::srgb_u8(0x8a, 0xc8, 0xff);

/// Pseudo connu d'un code (sinon le code lui-même).
fn nom<'a>(liste: &'a [Ami], code: &'a str) -> &'a str {
    liste
        .iter()
        .find(|a| a.code == code && a.pseudo != "?")
        .map_or(code, |a| a.pseudo.as_str())
}

/// Où en est un ami, en clair.
fn statut(p: Option<&Etat>) -> (String, Color) {
    let Some(p) = p else {
        return ("Hors ligne".into(), GRIS);
    };
    let t = match (p.etat, p.salon) {
        (EtatPresence::Menu, _) => return ("En ligne".into(), VERT),
        (EtatPresence::Salon, Some(_)) => format!("Dans un salon ({}/5)", p.joueurs),
        (EtatPresence::Salon, None) => "Dans un salon complet".into(),
        (EtatPresence::Partie, _) => format!("En partie ({} joueurs)", p.joueurs),
    };
    (t, BLEU)
}

/// Pseudo et statut l'un sous l'autre, prenant la place libre de la ligne.
fn deux_lignes(commandes: &mut Commands, l: Entity, haut: &str, bas: &str, couleur: Color) {
    let c = commandes
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                row_gap: Val::Px(2.0),
                ..default()
            },
            ChildOf(l),
        ))
        .id();
    texte(commandes, c, haut, TEXTE, 1.75);
    texte(commandes, c, bas, couleur, 1.5);
}

fn messages(commandes: &mut Commands, racine: Entity, vue: &VueMulti, notes: &[Note]) {
    for (k, n) in notes.iter().enumerate() {
        let l = ligne(commandes, racine, LARGEUR);
        let qui = nom(vue.liste, n.code());
        let (t, oui) = match n {
            Note::Invitation { .. } => ("vous invite dans son salon", "Rejoindre"),
            Note::Demande { .. } => ("voudrait jouer avec vous", "Inviter"),
            Note::Ajout { pseudo, .. } => {
                let t = format!("{pseudo} vous a ajouté");
                deux_lignes(commandes, l, &t, n.code(), OR);
                bouton(commandes, l, "Ajouter", Some(ActionMulti::Accepter(k)));
                bouton(commandes, l, "x", Some(ActionMulti::Ignorer(k)));
                continue;
            }
        };
        deux_lignes(commandes, l, qui, t, OR);
        bouton(commandes, l, oui, Some(ActionMulti::Accepter(k)));
        bouton(commandes, l, "x", Some(ActionMulti::Ignorer(k)));
    }
}

pub fn construire(commandes: &mut Commands, racine: Entity, menu: &Menu, vue: &VueMulti) {
    texte(commandes, racine, "Amis", OR, 2.0);
    let Some(amis) = vue.amis else {
        texte(commandes, racine, "Amis indisponibles.", GRIS, 1.5);
        return;
    };
    let code = amis.service.code();
    texte(
        commandes,
        racine,
        &format!("Votre code : {code}"),
        OR_CLAIR,
        2.0,
    );
    if !amis.service.connecte() {
        texte(
            commandes,
            racine,
            "Relais injoignable, nouvel essai…",
            GRIS,
            1.5,
        );
    }

    let l = ligne(commandes, racine, LARGEUR);
    let saisie = vue.annuaire.champ_ami;
    let champ = match &menu.saisie {
        Some(s) if saisie => format!("{s}_"),
        _ => "Code d'un ami".into(),
    };
    bouton(commandes, l, &champ, Some(ActionMulti::CodeAmi));
    bouton(commandes, l, "Ajouter", Some(ActionMulti::AjouterAmi));
    if menu.saisie.is_some() && saisie {
        texte(
            commandes,
            racine,
            "Entrée : ajouter · Échap : annuler",
            GRIS,
            1.5,
        );
    }

    messages(commandes, racine, vue, &amis.notes);

    if vue.liste.is_empty() {
        let t = "Aucun ami : donnez-leur votre code, ou ajoutez le leur.";
        commandes.spawn((
            crate::hud::police::TextePixel::nouveau(t, GRIS, 1.5).bloc(LARGEUR),
            ChildOf(racine),
        ));
    }
    let hote = vue.coop.is_some_and(|c| c.est_hote());
    let libre = vue.coop.is_none();
    for (k, a) in vue.liste.iter().enumerate() {
        let l = ligne(commandes, racine, LARGEUR);
        let p = amis.service.ami(&a.code).and_then(|s| s.presence.as_ref());
        let (t, couleur) = statut(p);
        deux_lignes(commandes, l, nom(vue.liste, &a.code), &t, couleur);
        if let Some(p) = p {
            if p.salon.is_some() && libre {
                bouton(
                    commandes,
                    l,
                    "Rejoindre",
                    Some(ActionMulti::RejoindreAmi(k)),
                );
            } else if p.etat == EtatPresence::Menu && (libre || hote) {
                bouton(commandes, l, "Inviter", Some(ActionMulti::Inviter(k)));
            } else if libre {
                bouton(commandes, l, "Jouer ?", Some(ActionMulti::Demander(k)));
            }
        }
        bouton(commandes, l, "x", Some(ActionMulti::Retirer(k)));
    }
}
