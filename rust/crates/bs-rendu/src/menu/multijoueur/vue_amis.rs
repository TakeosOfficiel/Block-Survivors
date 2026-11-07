//! Scène « Amis » de la page Multijoueur (maquette C) : son code ami en or, l'ajout d'un
//! ami par le sien (champ en pilule et bouton vert), les messages reçus (invitation,
//! demande, ajout) puis la liste des amis, chacun avec un avatar rond, une pastille de
//! présence et des liens d'action (rejoindre, inviter, demander ; « × » pour retirer).

use bevy::prelude::*;
use bs_reseau::amis::{Etat, EtatPresence};
use bs_sauvegarde::amis::Ami;

use super::ActionMulti;
use super::scene::{lien, ligne};
use super::vue::{Police, VueMulti, accroche, aide, ecrit, m, surtitre};
use crate::amis::Note;
use crate::ecrans::charte::{AIDE, OR_VIF};
use crate::menu::air;
use crate::menu::{ActionMenu, Icones, Menu};

const VERT: Color = crate::ecrans::charte::VERT_GAIN;
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
        return ("Hors ligne".into(), AIDE);
    };
    let t = match (p.etat, p.salon) {
        (EtatPresence::Menu, _) => return ("En ligne".into(), VERT),
        (EtatPresence::Salon, Some(_)) => format!("Dans un salon ({}/5)", p.joueurs),
        (EtatPresence::Salon, None) => "Dans un salon complet".into(),
        (EtatPresence::Partie, _) => format!("En partie ({} joueurs)", p.joueurs),
    };
    (t, BLEU)
}

/// Ligne d'un ami ou d'un message : avatar, nom, pastille et statut ; renvoie la ligne.
fn entete(
    c: &mut Commands,
    ic: &Icones,
    col: Entity,
    (qui, bas): (&str, &str),
    couleur: Color,
) -> Entity {
    let l = ligne(c, ic, col, qui);
    let s = air::rangee(c, l, 8.0);
    c.entity(s).entry::<Node>().and_modify(|mut n| {
        n.min_width = Val::Px(200.0);
    });
    air::point(c, s, 8.0, couleur);
    ecrit(c, ic, s, bas, (Police::Normale, 14.0, couleur));
    l
}

fn croix(c: &mut Commands, ic: &Icones, l: Entity, a: ActionMulti) {
    air::lien(c, ic, l, "×", (24.0, AIDE.with_alpha(0.7)), m(a));
}

/// Construit la scène des amis.
pub fn construire(c: &mut Commands, ic: &Icones, d: Entity, menu: &Menu, vue: &VueMulti) {
    surtitre(c, ic, d, "TON CODE AMI");
    let Some(amis) = vue.amis else {
        accroche(c, ic, d, "Amis indisponibles pour l'instant.");
        return;
    };
    ecrit(c, ic, d, amis.service.code(), (Police::Logo, 40.0, OR_VIF));
    let t = if amis.service.connecte() {
        "Donne-le à tes amis, ou ajoute le leur."
    } else {
        "Relais injoignable, nouvel essai…"
    };
    aide(c, ic, d, t);

    let (t, actif) = match &menu.saisie {
        Some(s) if vue.annuaire.champ_ami => (format!("{s}|"), true),
        _ => ("Code d'un ami".to_owned(), false),
    };
    let r = air::rangee(c, d, 14.0);
    c.entity(r)
        .entry::<Node>()
        .and_modify(|mut n| n.margin = UiRect::vertical(Val::Px(10.0)));
    let f = air::champ(c, ic, r, &t, actif, ActionMenu::Multi(ActionMulti::CodeAmi));
    c.entity(f).entry::<Node>().and_modify(|mut n| {
        n.width = Val::Px(300.0);
        n.height = Val::Px(52.0);
    });
    let b = air::bouton_vert(c, ic, r, "AJOUTER", m(ActionMulti::AjouterAmi));
    c.entity(b)
        .entry::<Node>()
        .and_modify(|mut n| n.height = Val::Px(50.0));
    if actif {
        aide(c, ic, d, "Entrée : ajouter · Échap : annuler");
    }

    let col = air::colonne(c, d, 14.0);
    c.entity(col).entry::<Node>().and_modify(|mut n| {
        n.width = Val::Percent(100.0);
        n.margin = UiRect::top(Val::Px(10.0));
    });
    for (k, n) in amis.notes.iter().enumerate() {
        let qui = nom(vue.liste, n.code());
        let (qui, bas, oui) = match n {
            Note::Invitation { .. } => (qui, "t'invite dans son salon", "Rejoindre"),
            Note::Demande { .. } => (qui, "voudrait jouer avec toi", "Inviter"),
            Note::Ajout { pseudo, .. } => (pseudo.as_str(), "t'a ajouté", "Ajouter"),
        };
        let l = entete(c, ic, col, (qui, bas), OR_VIF);
        lien(c, ic, l, oui, m(ActionMulti::Accepter(k)));
        croix(c, ic, l, ActionMulti::Ignorer(k));
    }
    if vue.liste.is_empty() {
        aide(c, ic, col, "Aucun ami pour l'instant.");
    }
    let hote = vue.coop.is_some_and(|c| c.est_hote());
    let libre = vue.coop.is_none();
    for (k, a) in vue.liste.iter().enumerate() {
        let p = amis.service.ami(&a.code).and_then(|s| s.presence.as_ref());
        let (t, couleur) = statut(p);
        let l = entete(c, ic, col, (nom(vue.liste, &a.code), &t), couleur);
        let action = p.and_then(|p| {
            if p.salon.is_some() && libre {
                Some(("Rejoindre", ActionMulti::RejoindreAmi(k)))
            } else if p.etat == EtatPresence::Menu && (libre || hote) {
                Some(("Inviter", ActionMulti::Inviter(k)))
            } else {
                libre.then_some(("Jouer ?", ActionMulti::Demander(k)))
            }
        });
        if let Some((t, a)) = action {
            lien(c, ic, l, t, m(a));
        }
        croix(c, ic, l, ActionMulti::Retirer(k));
    }
}
