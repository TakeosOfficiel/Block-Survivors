//! Vue de la page Multijoueur, en « menu de console » (maquette C) : sur un fond de nuit,
//! à gauche une liste de grands intitulés en police du logo (Créer un salon, Rejoindre,
//! Salons publics, Amis ; dans un salon : Ton salon, Amis), le choisi énorme et doré ; à
//! droite, après un filet doré, seulement le contenu de ce choix (`scene`, `vue_amis`).
//! En bas : « Retour » à gauche, le code ami et les succès de coop à droite. Aucun cadre.

use bevy::prelude::*;

use bs_sauvegarde::amis::Ami;
use bs_sauvegarde::progression::Progression;

use super::{ActionMulti, Annuaire, Onglet};
use crate::amis::ReseauAmis;
use crate::coop::Coop;
use crate::ecrans::charte::{AIDE, IVOIRE, OR_VIF};
use crate::lancement::DonneesJeu;
use crate::menu::air;
use crate::menu::console::{self, intitule};
pub(super) use crate::menu::console::{Police, TERNE, accroche, aide, ecrit, surtitre};
use crate::menu::vue::Lien;
use crate::menu::{ActionMenu, Icones, Menu};

/// Ce que la page montre de la coop.
pub struct VueMulti<'a> {
    pub coop: Option<&'a Coop>,
    pub message: Option<&'a str>,
    pub annuaire: &'a Annuaire,
    pub amis: Option<&'a ReseauAmis>,
    pub liste: &'a [Ami],
    pub progression: &'a Progression,
    pub pseudo: &'a str,
}

/// Action de la page, pour les liens.
pub(super) fn m(a: ActionMulti) -> Option<ActionMenu> {
    Some(ActionMenu::Multi(a))
}

/// Avatar rond : initiale du pseudo sur une couleur tirée du pseudo.
pub(super) fn avatar(c: &mut Commands, ic: &Icones, parent: Entity, pseudo: &str, cote: f32) {
    const TEINTES: [Color; 5] = [
        Color::srgb_u8(0xf5, 0xc5, 0x42),
        Color::srgb_u8(0x7f, 0xb3, 0xff),
        Color::srgb_u8(0xe5, 0x8f, 0xb0),
        Color::srgb_u8(0x8f, 0xe0, 0xa0),
        Color::srgb_u8(0xc9, 0xa2, 0xff),
    ];
    let k = pseudo.bytes().map(usize::from).sum::<usize>() % TEINTES.len();
    rond(c, ic, parent, (initiale(pseudo), cote), TEINTES[k]);
}

/// Initiale en capitale d'un pseudo.
pub(super) fn initiale(pseudo: &str) -> char {
    pseudo
        .chars()
        .next()
        .map_or('?', |l| l.to_uppercase().next().unwrap_or(l))
}

/// Disque plein portant une lettre sombre.
pub(super) fn rond(
    c: &mut Commands,
    ic: &Icones,
    parent: Entity,
    (l, cote): (char, f32),
    fond: Color,
) {
    let r = air::point(c, parent, cote, fond);
    c.entity(r).entry::<Node>().and_modify(|mut n| {
        n.justify_content = JustifyContent::Center;
        n.align_items = AlignItems::Center;
    });
    let mut t = [0u8; 4];
    let sombre = Color::srgb_u8(0x0b, 0x14, 0x30);
    let l = l.encode_utf8(&mut t);
    ecrit(c, ic, r, l, (Police::Logo, cote * 0.42, sombre));
}

/// Gros bouton vert en pilule, police du logo ; éteint sans action.
pub(super) fn appel(c: &mut Commands, ic: &Icones, parent: Entity, t: &str, a: Option<ActionMenu>) {
    let b = air::bouton_vert(c, ic, parent, t, a);
    c.entity(b).entry::<Node>().and_modify(|mut n| {
        n.height = Val::Px(80.0);
        n.min_width = Val::Px(280.0);
        n.padding = UiRect::horizontal(Val::Px(56.0));
        n.align_self = AlignSelf::FlexStart;
        n.margin = UiRect::top(Val::Px(12.0));
    });
}

/// Liste de gauche.
fn liste(c: &mut Commands, ic: &Icones, parent: Entity, vue: &VueMulti) {
    let ouvert = vue.annuaire.onglet;
    let choix = |o: Onglet| {
        (
            68.0,
            (o != ouvert).then_some(ActionMenu::Multi(ActionMulti::Onglet(o))),
        )
    };
    let amis = match vue.amis {
        Some(a) => {
            let n = vue
                .liste
                .iter()
                .filter(|x| a.service.ami(&x.code).is_some_and(|s| s.presence.is_some()))
                .count();
            match (n, a.notes.len()) {
                (_, 1) => "1 message reçu".to_owned(),
                (_, k @ 2..) => format!("{k} messages reçus"),
                (0, _) => "Aucun ami connecté".to_owned(),
                (1, _) => "1 ami connecté".to_owned(),
                (n, _) => format!("{n} amis connectés"),
            }
        }
        None => "Indisponibles".to_owned(),
    };
    if let Some(coop) = vue.coop {
        let t = format!("{}/5 survivants", coop.joueurs().len());
        intitule(c, ic, parent, ("Ton salon", &t), choix(Onglet::Creer));
    } else {
        let creer = ("Créer un salon", "Lance ta partie, jusqu'à 5 survivants");
        intitule(c, ic, parent, creer, choix(Onglet::Creer));
        let rejoindre = ("Rejoindre", "Avec un code de salon");
        intitule(c, ic, parent, rejoindre, choix(Onglet::Rejoindre));
        let t = match vue.annuaire.salons.len() {
            0 => "Aucun salon ouvert".to_owned(),
            1 => "1 salon ouvert".to_owned(),
            n => format!("{n} salons ouverts"),
        };
        intitule(
            c,
            ic,
            parent,
            ("Salons publics", &t),
            choix(Onglet::Publics),
        );
    }
    intitule(c, ic, parent, ("Amis", &amis), choix(Onglet::Amis));
}

/// Bas de page : retour, code ami, succès de coop (dépliés au clic, au-dessus).
fn pied(c: &mut Commands, ic: &Icones, racine: Entity, dj: &DonneesJeu, vue: &VueMulti) {
    let p = console::pied(c, ic, racine);
    if let Some(a) = vue.amis {
        let l = air::rangee(c, p, 8.0);
        ecrit(c, ic, l, "Code ami", (Police::Normale, 14.0, AIDE));
        ecrit(c, ic, l, a.service.code(), (Police::Grasse, 15.0, IVOIRE));
    }
    let (liste, prog) = (&dj.d.succes, vue.progression);
    if liste.is_empty() {
        return;
    }
    let s = air::rangee(c, p, 8.0);
    c.entity(s)
        .insert((Button, Lien(AIDE), ActionMenu::Multi(ActionMulti::Succes)));
    ecrit(c, ic, s, "Succès coop", (Police::Normale, 14.0, AIDE));
    let gagnes = liste.iter().filter(|x| prog.succes(&x.id)).count();
    for x in liste {
        let couleur = if prog.succes(&x.id) {
            OR_VIF
        } else {
            IVOIRE.with_alpha(0.15)
        };
        air::point(c, s, 10.0, couleur);
    }
    let t = format!("{gagnes}/{}", liste.len());
    ecrit(c, ic, s, &t, (Police::Logo, 20.0, OR_VIF));
    if !vue.annuaire.succes_ouverts {
        return;
    }
    let d = air::colonne(c, racine, 6.0);
    c.entity(d).entry::<Node>().and_modify(|mut n| {
        n.position_type = PositionType::Absolute;
        n.right = Val::Px(112.0);
        n.bottom = Val::Px(120.0);
        n.align_items = AlignItems::FlexEnd;
    });
    for x in liste {
        let (t, couleur) = if prog.succes(&x.id) {
            (format!("★ {} · {}", x.nom, x.texte), OR_VIF)
        } else {
            (format!("{} · {}", x.nom, x.texte), AIDE)
        };
        ecrit(c, ic, d, &t, (Police::Normale, 14.0, couleur));
    }
}

/// Construit la page ; renvoie sa racine.
pub fn construire(
    commandes: &mut Commands,
    dj: &DonneesJeu,
    ic: &Icones,
    menu: &Menu,
    vue: &VueMulti,
) -> Entity {
    let c = commandes;
    let console::Page {
        racine,
        gauche,
        scene: d,
    } = console::page(c, ic, ("multijoueur", "MULTIJOUEUR"), (680.0, 660.0));
    liste(c, ic, gauche, vue);
    if let Some(t) = vue.message {
        let rouge = crate::ecrans::charte::ROUGE;
        ecrit(c, ic, d, t, (Police::Grasse, 15.0, rouge));
    }
    match (vue.annuaire.onglet, vue.coop) {
        (Onglet::Amis, _) => super::vue_amis::construire(c, ic, d, menu, vue),
        (_, Some(coop)) => super::scene::salon(c, ic, d, (dj, menu), coop),
        (Onglet::Creer, None) => super::scene::creer(c, ic, d, vue),
        (Onglet::Rejoindre, None) => super::scene::rejoindre(c, ic, d, menu, vue),
        (Onglet::Publics, None) => super::scene::publics(c, ic, d, vue),
    }
    pied(c, ic, racine, dj, vue);
    racine
}
