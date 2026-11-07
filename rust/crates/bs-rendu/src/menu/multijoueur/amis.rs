//! Actions de la colonne des amis de la page Multijoueur : ajouter un ami par son code,
//! rejoindre son salon, l'inviter (un salon privé s'ouvre s'il le faut : l'invitation part
//! dès qu'il a son code), lui demander de jouer, le retirer ; répondre aux messages reçus.

use bs_reseau::amis::Lettre;

use super::{ActionMulti, Contexte, entrer_salon};
use crate::amis::Note;
use crate::menu::Menu;

/// Ajoute l'ami du code saisi (le refus s'affiche).
pub(super) fn ajouter(c: &mut Contexte, saisie: &str) {
    let Some(amis) = c.amis.as_deref_mut() else {
        c.message.0 = Some("Amis indisponibles (relais injoignable)".into());
        return;
    };
    c.message.0 = amis
        .ajouter(c.sauvegarde, saisie, "?")
        .err()
        .map(|r| r.message().to_owned());
}

/// Invite `code` : tout de suite si l'on héberge un salon, sinon à l'ouverture du sien.
fn inviter(c: &mut Contexte, menu: &Menu, code: String) {
    let salon = c
        .coop
        .as_deref()
        .filter(|coop| coop.est_hote())
        .and_then(|coop| coop.code())
        .and_then(|s| bs_reseau::code_saisi(&s));
    let invite_ailleurs = c.coop.as_deref().is_some_and(|coop| !coop.est_hote());
    let Some(amis) = c.amis.as_deref_mut() else {
        return;
    };
    match salon {
        Some(s) => amis.service.poster(&code, &Lettre::Invitation(s)),
        None if invite_ailleurs => {
            c.message.0 = Some("Seul l'hôte du salon peut inviter".into());
        }
        None => {
            amis.a_inviter = Some(code);
            entrer_salon(c, menu.choix, None, false);
        }
    }
}

pub(super) fn agir(c: &mut Contexte, menu: &Menu, action: ActionMulti) {
    let code_ami = |c: &Contexte, k: usize| c.sauvegarde.amis.amis.get(k).map(|a| a.code.clone());
    match action {
        ActionMulti::RejoindreAmi(k) if c.coop.is_none() => {
            let salon = code_ami(c, k).and_then(|code| {
                let a = c.amis.as_deref()?.service.ami(&code)?;
                a.presence.as_ref()?.salon
            });
            if let Some(s) = salon {
                entrer_salon(c, menu.choix, Some(s), false);
            }
        }
        ActionMulti::Inviter(k) => {
            if let Some(code) = code_ami(c, k) {
                inviter(c, menu, code);
            }
        }
        ActionMulti::Demander(k) => {
            if let (Some(code), Some(amis)) = (code_ami(c, k), c.amis.as_deref_mut()) {
                amis.service.poster(&code, &Lettre::Demande);
            }
        }
        ActionMulti::Retirer(k) => {
            if let (Some(code), Some(amis)) = (code_ami(c, k), c.amis.as_deref_mut()) {
                amis.retirer(c.sauvegarde, &code);
            }
        }
        ActionMulti::Accepter(k) => {
            let Some(note) = c.amis.as_deref_mut().and_then(|a| a.effacer(k)) else {
                return;
            };
            match note {
                Note::Invitation { salon, .. } if c.coop.is_none() => {
                    entrer_salon(c, menu.choix, Some(salon), false);
                }
                Note::Invitation { .. } => {
                    c.message.0 = Some("Quittez d'abord votre salon".into());
                }
                Note::Demande { code } => inviter(c, menu, code),
                Note::Ajout { code, pseudo } => {
                    if let Some(amis) = c.amis.as_deref_mut() {
                        c.message.0 = amis
                            .ajouter(c.sauvegarde, &code, &pseudo)
                            .err()
                            .map(|r| r.message().to_owned());
                    }
                }
            }
        }
        ActionMulti::Ignorer(k) => {
            if let Some(amis) = c.amis.as_deref_mut() {
                amis.effacer(k);
            }
        }
        _ => {}
    }
}
