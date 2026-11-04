//! Volet des amis de la page Multijoueur, à droite : replié en un lien « ‹ Amis » (avec
//! qui est en ligne et les messages reçus), ouvert sur un voile sombre en dégradé, sans
//! cadre : son code ami en or, l'ajout d'un
//! ami par le sien (champ en pilule), les messages reçus (invitation, demande, ajout) et
//! la liste des amis, chacun avec une pastille de présence et des liens d'action
//! (rejoindre, inviter, demander ; « × » pour retirer).

use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bs_reseau::amis::{Etat, EtatPresence};
use bs_sauvegarde::amis::Ami;

use super::ActionMulti;
use super::vue::{VueMulti, aide, etire, ligne, m, titre, valeur};
use crate::amis::Note;
use crate::ecrans::charte::{AIDE, IVOIRE, OR_VIF, T_AIDE, T_CORPS};
use crate::menu::air::{self, ecrire};
use crate::menu::{ActionMenu, Icones, Menu};

pub const LARGEUR: f32 = 420.0;
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

/// Pastille, pseudo et statut l'un sous l'autre ; renvoie la ligne pour ses liens.
fn entete(
    commandes: &mut Commands,
    ic: &Icones,
    parent: Entity,
    (haut, bas): (&str, &str),
    couleur: Color,
) -> Entity {
    let l = ligne(commandes, parent);
    air::point(commandes, l, 10.0, couleur);
    let c = etire(commandes, l, 2.0);
    ecrire(commandes, ic, c, haut, (true, T_CORPS, IVOIRE));
    ecrire(commandes, ic, c, bas, (false, T_AIDE, couleur));
    l
}

/// Lien d'action d'une ligne (or) ; « × » en pâle.
fn action(c: &mut Commands, ic: &Icones, l: Entity, t: &str, a: ActionMulti) {
    air::lien(c, ic, l, t, (T_CORPS, OR_VIF), m(a));
}
fn croix(c: &mut Commands, ic: &Icones, l: Entity, a: ActionMulti) {
    air::lien(c, ic, l, "×", (26.0, AIDE), m(a));
}

fn messages(commandes: &mut Commands, ic: &Icones, col: Entity, vue: &VueMulti, notes: &[Note]) {
    for (k, n) in notes.iter().enumerate() {
        let qui = nom(vue.liste, n.code());
        let (haut, bas, oui) = match n {
            Note::Invitation { .. } => (qui.to_owned(), "vous invite dans son salon", "Rejoindre"),
            Note::Demande { .. } => (qui.to_owned(), "voudrait jouer avec vous", "Inviter"),
            Note::Ajout { pseudo, .. } => (format!("{pseudo} vous a ajouté"), n.code(), "Ajouter"),
        };
        let l = entete(commandes, ic, col, (&haut, bas), OR_VIF);
        action(commandes, ic, l, oui, ActionMulti::Accepter(k));
        croix(commandes, ic, l, ActionMulti::Ignorer(k));
    }
}

/// Colonne des amis.
fn construire(commandes: &mut Commands, ic: &Icones, col: Entity, menu: &Menu, vue: &VueMulti) {
    let Some(amis) = vue.amis else {
        aide(commandes, ic, col, "Amis indisponibles pour l'instant.");
        return;
    };
    let l = air::rangee(commandes, col, 12.0);
    ecrire(commandes, ic, l, "Votre code", (false, T_AIDE, AIDE));
    ecrire(commandes, ic, l, amis.service.code(), (true, 26.0, OR_VIF));
    if !amis.service.connecte() {
        aide(commandes, ic, col, "Relais injoignable, nouvel essai…");
    }

    let (t, actif) = valeur(menu, vue.annuaire.champ_ami, "Code d'un ami");
    let l = air::rangee(commandes, col, 18.0);
    let champ = ActionMenu::Multi(ActionMulti::CodeAmi);
    air::champ(commandes, ic, l, &t, actif, champ);
    action(commandes, ic, l, "Ajouter ›", ActionMulti::AjouterAmi);
    if actif {
        aide(commandes, ic, col, "Entrée : ajouter · Échap : annuler");
    }
    air::trait_fin(commandes, col);

    messages(commandes, ic, col, vue, &amis.notes);
    if vue.liste.is_empty() {
        let t = "Aucun ami pour l'instant : donnez-leur votre code, ou ajoutez le leur.";
        aide(commandes, ic, col, t);
    }
    let hote = vue.coop.is_some_and(|c| c.est_hote());
    let libre = vue.coop.is_none();
    for (k, a) in vue.liste.iter().enumerate() {
        let p = amis.service.ami(&a.code).and_then(|s| s.presence.as_ref());
        let (t, couleur) = statut(p);
        let l = entete(commandes, ic, col, (nom(vue.liste, &a.code), &t), couleur);
        if let Some(p) = p {
            if p.salon.is_some() && libre {
                action(commandes, ic, l, "Rejoindre", ActionMulti::RejoindreAmi(k));
            } else if p.etat == EtatPresence::Menu && (libre || hote) {
                action(commandes, ic, l, "Inviter", ActionMulti::Inviter(k));
            } else if libre {
                action(commandes, ic, l, "Jouer ?", ActionMulti::Demander(k));
            }
        }
        croix(commandes, ic, l, ActionMulti::Retirer(k));
    }
}

/// Volet des amis, posé à droite de la page : replié en lien, ou ouvert.
pub fn volet(commandes: &mut Commands, ic: &Icones, racine: Entity, menu: &Menu, vue: &VueMulti) {
    let marge = Val::Px(crate::ecrans::charte::MARGE);
    if !vue.annuaire.amis_ouverts {
        let o = commandes
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: marge,
                    right: marge,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                ChildOf(racine),
            ))
            .id();
        air::lien(
            commandes,
            ic,
            o,
            "‹ Amis",
            (22.0, IVOIRE),
            m(ActionMulti::Amis),
        );
        let Some(amis) = vue.amis else { return };
        let en_ligne = vue
            .liste
            .iter()
            .filter(|a| {
                amis.service
                    .ami(&a.code)
                    .is_some_and(|s| s.presence.is_some())
            })
            .count();
        let l = air::rangee(commandes, o, 6.0);
        let couleur = if en_ligne > 0 { VERT } else { AIDE };
        air::point(commandes, l, 8.0, couleur);
        let t = format!("{en_ligne} en ligne");
        ecrire(commandes, ic, l, &t, (false, T_AIDE, couleur));
        let n = amis.notes.len();
        if n > 0 {
            let t = format!("{n} message{}", if n > 1 { "s" } else { "" });
            ecrire(commandes, ic, o, &t, (false, T_AIDE, OR_VIF));
        }
        return;
    }
    let v = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::ZERO,
                right: Val::ZERO,
                width: Val::Px(LARGEUR + 140.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                padding: UiRect::all(marge),
                ..default()
            },
            // Voile : transparent à gauche, nuit à droite (pas de bord)
            BackgroundGradient::from(LinearGradient::to_right(vec![
                ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.0), Val::Percent(0.0)),
                ColorStop::new(Color::srgba(0.01, 0.02, 0.03, 0.85), Val::Percent(30.0)),
                ColorStop::new(Color::srgba(0.01, 0.02, 0.03, 0.95), Val::Percent(100.0)),
            ])),
            FocusPolicy::Block,
            ChildOf(racine),
        ))
        .id();
    let col = air::colonne(commandes, v, 14.0);
    commandes
        .entity(col)
        .entry::<Node>()
        .and_modify(|mut n| n.width = Val::Px(LARGEUR));
    let haut = ligne(commandes, col);
    let g = etire(commandes, haut, 0.0);
    titre(commandes, ic, g, "Amis");
    air::lien(
        commandes,
        ic,
        haut,
        "Fermer ›",
        (T_AIDE, AIDE),
        m(ActionMulti::Amis),
    );
    construire(commandes, ic, col, menu, vue);
}
