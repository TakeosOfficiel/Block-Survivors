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
use crate::menu::vue::Lien;
use crate::menu::{ActionMenu, Icones, Menu};

/// Intitulés non choisis, libellés éteints.
pub(super) const TERNE: Color = Color::srgba(0.95, 0.92, 0.83, 0.38);
const OMBRE_OR: Color = Color::srgb_u8(0x8a, 0x5f, 0x0b);
/// Largeur de la scène de droite.
pub(super) const SCENE: f32 = 660.0;

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

/// Police d'un texte de la page.
#[derive(Clone, Copy)]
pub(super) enum Police {
    Normale,
    Grasse,
    Logo,
}

/// Texte dans la police, la taille (px) et la couleur données.
pub(super) fn ecrit(
    c: &mut Commands,
    ic: &Icones,
    parent: Entity,
    t: &str,
    (police, taille, couleur): (Police, f32, Color),
) -> Entity {
    let p = ic.polices;
    let font = match police {
        Police::Normale => &p.normale,
        Police::Grasse => &p.grasse,
        Police::Logo => &p.logo,
    };
    let e = c
        .spawn((
            Text::new(t),
            TextFont {
                font: font.clone(),
                font_size: taille,
                ..default()
            },
            TextColor(couleur),
            ChildOf(parent),
        ))
        .id();
    if let Police::Logo = police {
        c.entity(e).insert(TextShadow {
            offset: Vec2::new(0.0, taille / 16.0),
            color: Color::srgba(0.0, 0.0, 0.0, 0.5),
        });
    }
    e
}

/// Action de la page, pour les liens.
pub(super) fn m(a: ActionMulti) -> Option<ActionMenu> {
    Some(ActionMenu::Multi(a))
}

/// Petit surtitre doré en capitales.
pub(super) fn surtitre(c: &mut Commands, ic: &Icones, parent: Entity, t: &str) {
    ecrit(
        c,
        ic,
        parent,
        t,
        (Police::Grasse, 13.0, OR_VIF.with_alpha(0.85)),
    );
}

/// Phrase d'accroche de la scène.
pub(super) fn accroche(c: &mut Commands, ic: &Icones, parent: Entity, t: &str) {
    let e = ecrit(c, ic, parent, t, (Police::Grasse, 30.0, IVOIRE));
    c.entity(e).insert(Node {
        margin: UiRect::top(Val::Px(6.0)),
        ..default()
    });
}

/// Texte d'aide gris, replié sur la scène.
pub(super) fn aide(c: &mut Commands, ic: &Icones, parent: Entity, t: &str) -> Entity {
    let e = ecrit(c, ic, parent, t, (Police::Normale, 15.0, AIDE));
    c.entity(e).insert(Node {
        max_width: Val::Px(SCENE),
        ..default()
    });
    e
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

/// Un intitulé de la liste de gauche : grand titre et sa ligne d'aide ; `choisi` vaut
/// `None` pour l'intitulé ouvert, sinon l'onglet qu'il ouvre au clic.
fn intitule(
    c: &mut Commands,
    ic: &Icones,
    parent: Entity,
    (t, aide): (&str, &str),
    choisi: Option<Onglet>,
) {
    let ouvert = choisi.is_none();
    let l = air::rangee(c, parent, 22.0);
    // Losange doré devant l'intitulé ouvert, place gardée sinon
    c.spawn((
        Node {
            width: Val::Px(16.0),
            height: Val::Px(16.0),
            flex_shrink: 0.0,
            ..default()
        },
        UiTransform::from_rotation(Rot2::degrees(45.0)),
        BackgroundColor(if ouvert { OR_VIF } else { Color::NONE }),
        ChildOf(l),
    ));
    let b = air::colonne(c, l, if ouvert { 10.0 } else { 4.0 });
    if let Some(o) = choisi {
        c.entity(b).insert((
            Button,
            Lien(TERNE),
            ActionMenu::Multi(ActionMulti::Onglet(o)),
        ));
    }
    let (taille, couleur) = if ouvert {
        (68.0, OR_VIF)
    } else {
        (44.0, TERNE)
    };
    let e = ecrit(c, ic, b, t, (Police::Logo, taille, couleur));
    if ouvert {
        c.entity(e).insert(TextShadow {
            offset: Vec2::new(0.0, 4.0),
            color: OMBRE_OR,
        });
    }
    let (taille, couleur) = if ouvert {
        (16.0, IVOIRE.with_alpha(0.75))
    } else {
        (14.0, AIDE.with_alpha(0.7))
    };
    ecrit(c, ic, b, aide, (Police::Normale, taille, couleur));
}

/// Liste de gauche.
fn liste(c: &mut Commands, ic: &Icones, parent: Entity, vue: &VueMulti) {
    let ouvert = vue.annuaire.onglet;
    let choix = |o: Onglet| (o != ouvert).then_some(o);
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
    let p = air::rangee(c, racine, 40.0);
    c.entity(p)
        .entry::<Node>()
        .and_modify(|mut n| n.width = Val::Percent(100.0));
    // Retour : ferme la page (et quitte le salon, comme Échap)
    let r = air::rangee(c, p, 12.0);
    c.entity(r).insert((Button, Lien(AIDE), ActionMenu::Fermer));
    let k = air::point(c, r, 30.0, Color::NONE);
    c.entity(k)
        .insert(BorderColor::all(AIDE))
        .entry::<Node>()
        .and_modify(|mut n| {
            n.border = UiRect::all(Val::Px(1.5));
            n.justify_content = JustifyContent::Center;
            n.align_items = AlignItems::Center;
        });
    ecrit(c, ic, k, "‹", (Police::Grasse, 16.0, AIDE));
    ecrit(c, ic, r, "Retour", (Police::Grasse, 16.0, AIDE));
    let vide = air::rangee(c, p, 0.0);
    c.entity(vide)
        .entry::<Node>()
        .and_modify(|mut n| n.flex_grow = 1.0);
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
    let nuit = |r, g, b, p| ColorStop::new(Color::srgb_u8(r, g, b), Val::Percent(p));
    let racine = c
        .spawn((
            Name::new("multijoueur"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(Val::Px(112.0), Val::Px(64.0)),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(0x02, 0x03, 0x08)),
            BackgroundGradient::from(LinearGradient::to_bottom(vec![
                nuit(0x0b, 0x14, 0x30, 0.0),
                nuit(0x07, 0x0c, 0x1d, 45.0),
                nuit(0x02, 0x03, 0x08, 100.0),
            ])),
        ))
        .id();
    // En-tête : nom du jeu · Multijoueur
    let h = air::rangee(c, racine, 22.0);
    let pale = AIDE.with_alpha(0.8);
    ecrit(c, ic, h, "BLOCK SURVIVORS", (Police::Logo, 18.0, pale));
    air::point(c, h, 6.0, OR_VIF);
    let t = IVOIRE.with_alpha(0.9);
    ecrit(c, ic, h, "MULTIJOUEUR", (Police::Logo, 30.0, t));

    let milieu = air::rangee(c, racine, 0.0);
    c.entity(milieu).entry::<Node>().and_modify(|mut n| {
        n.flex_grow = 1.0;
        n.min_height = Val::ZERO;
    });
    let g = air::colonne(c, milieu, 26.0);
    c.entity(g).entry::<Node>().and_modify(|mut n| {
        n.width = Val::Px(680.0);
        n.flex_shrink = 1.0;
    });
    liste(c, ic, g, vue);
    // Filet doré, fondu aux deux bouts
    c.spawn((
        Node {
            width: Val::Px(1.0),
            height: Val::Px(500.0),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::new(OR_VIF.with_alpha(0.0), Val::Percent(0.0)),
            ColorStop::new(OR_VIF.with_alpha(0.35), Val::Percent(50.0)),
            ColorStop::new(OR_VIF.with_alpha(0.0), Val::Percent(100.0)),
        ])),
        ChildOf(milieu),
    ));
    let d = air::colonne(c, milieu, 14.0);
    c.entity(d).entry::<Node>().and_modify(|mut n| {
        n.width = Val::Px(SCENE);
        n.flex_shrink = 1.0;
        n.margin = UiRect::left(Val::Px(70.0));
    });
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
