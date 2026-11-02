//! Bas de l'accueil (styles.css `#menu-jouer`, `.menu-liens`, `.menu-pied`) : le grand
//! bouton « Jouer » vert, les liens (Options, Multijoueur, Nouveautés avec sa pastille,
//! Quitter) et la version ; plus le dé du pseudo.

use bevy::prelude::*;

use super::{BLANC, MARGE, ORE_TRANSPARENT, Pinceau, place};
use crate::ecrans::style::{BoutonPage, OR, OR_CLAIR, Teinte};
use crate::hud::police::TextePixel;
use crate::lancement::{ChoixPartie, DonneesJeu};
use crate::menu::ActionMenu;
use crate::sauvegarde::Sauvegarde;

/// Dé du pseudo au hasard (`.menu-de`) : cinq points dessinés.
pub(super) fn de(p: &mut Pinceau, parent: Entity) {
    let b = p.bouton_nu(parent, ActionMenu::PseudoAleatoire);
    p.c.entity(b).entry::<Node>().and_modify(|mut n| {
        n.min_width = Val::Px(30.0);
        n.width = Val::Px(30.0);
        n.height = Val::Px(30.0);
        n.padding = UiRect::ZERO;
        n.border_radius = BorderRadius::all(Val::Px(6.0));
    });
    let face = p.noeud(
        b,
        Node {
            width: Val::Px(16.0),
            height: Val::Px(16.0),
            border_radius: BorderRadius::all(Val::Px(3.0)),
            ..default()
        },
    );
    p.c.entity(face)
        .insert(BackgroundColor(Color::srgb_u8(0xdd, 0xdd, 0xdd)));
    for (x, y) in [
        (3.0, 3.0),
        (10.0, 3.0),
        (6.5, 6.5),
        (3.0, 10.0),
        (10.0, 10.0),
    ] {
        let point = p.noeud(
            face,
            Node {
                width: Val::Px(3.0),
                height: Val::Px(3.0),
                border_radius: BorderRadius::MAX,
                ..place(Some(Val::Px(x)), None, Some(Val::Px(y)), None)
            },
        );
        p.c.entity(point)
            .insert(BackgroundColor(Color::srgb_u8(0x14, 0x17, 0x1f)));
    }
}

/// Largeur du bouton « Multijoueur », écart avec « Jouer ».
const LARGEUR_MULTI: f32 = 190.0;
const ECART: f32 = 12.0;

/// « Multijoueur », à droite de « Jouer », de la même hauteur.
fn multijoueur(p: &mut Pinceau, racine: Entity) {
    let b = p.bouton_nu(racine, ActionMenu::OuvrirMulti);
    let gauche = (420.0 + ECART + LARGEUR_MULTI) / 2.0 - LARGEUR_MULTI;
    p.c.entity(b).entry::<Node>().and_modify(move |mut n| {
        *n = Node {
            width: Val::Px(LARGEUR_MULTI),
            height: Val::Px(66.0),
            margin: UiRect::left(Val::Px(gauche)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(2.0),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(10.0)),
            ..place(Some(Val::Percent(50.0)), None, None, Some(Val::Px(26.0)))
        };
    });
    p.texte(b, "Multijoueur", true, 18.0, BLANC);
    p.texte(
        b,
        "Coop en ligne",
        false,
        12.0,
        Color::srgb_u8(0xc8, 0xc8, 0xc8),
    );
}

/// « Jouer », « Multijoueur », liens (Options, Nouveautés, Quitter) et pied de page.
pub(super) fn construire(
    p: &mut Pinceau,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    choix: ChoixPartie,
    racine: Entity,
) {
    let vert = |hex: u32| Color::srgb_u8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8);
    let jouer =
        p.c.spawn((
            Button,
            BoutonPage,
            Teinte {
                bord: OR,
                fond: ORE_TRANSPARENT,
            },
            ActionMenu::Jouer,
            Node {
                width: Val::Px(420.0),
                height: Val::Px(66.0),
                // « Jouer » et « Multijoueur » centrés ensemble
                margin: UiRect::left(Val::Px(-(420.0 + ECART + LARGEUR_MULTI) / 2.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..place(Some(Val::Percent(50.0)), None, None, Some(Val::Px(26.0)))
            },
            BorderColor::all(OR),
            BackgroundGradient::from(LinearGradient::to_bottom(vec![
                ColorStop::new(vert(0x34a046), Val::Percent(0.0)),
                ColorStop::new(vert(0x1d6b2a), Val::Percent(100.0)),
            ])),
            BoxShadow::new(
                Color::srgba(1.0, 0.78, 0.31, 0.35),
                Val::ZERO,
                Val::ZERO,
                Val::Px(2.0),
                Val::Px(18.0),
            ),
            ChildOf(racine),
        ))
        .id();
    p.c.spawn((TextePixel::nouveau("JOUER", BLANC, 4.0), ChildOf(jouer)));
    let nom = &dj.d.personnages.personnages[choix.personnage].nom;
    let sous_titre = format!("{nom} · {}", dj.arenes[choix.arene].def.nom);
    p.texte(
        jouer,
        &sous_titre,
        true,
        14.0,
        Color::srgb_u8(0xd8, 0xff, 0xd0),
    );
    multijoueur(p, racine);

    let liens = p.noeud(
        racine,
        Node {
            column_gap: Val::Px(8.0),
            ..place(Some(Val::Px(MARGE)), None, None, Some(Val::Px(30.0)))
        },
    );
    p.bouton(liens, "Options", ActionMenu::Options);
    let nouveautes = p.bouton(liens, "Nouveautés", ActionMenu::Nouveautes);
    if crate::menu::nouveautes::a_lire(sauvegarde.profils.stockage()) {
        // Pastille : nouveautés de cette version pas encore lues
        let pastille = p.noeud(
            nouveautes,
            Node {
                padding: UiRect::axes(Val::Px(7.0), Val::Px(1.0)),
                border_radius: BorderRadius::MAX,
                ..place(None, Some(Val::Px(-8.0)), Some(Val::Px(-11.0)), None)
            },
        );
        p.c.entity(pastille).insert(BackgroundColor(OR_CLAIR));
        let t =
            p.ic.polices
                .texte("NOUVEAU !", true, 11.0, Color::srgb_u8(0x2a, 0x1d, 0x02));
        p.c.spawn((t, ChildOf(pastille)));
    }
    p.bouton(liens, "Quitter", ActionMenu::Quitter);

    let pied = p.noeud(
        racine,
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            ..place(None, Some(Val::Px(MARGE)), None, Some(Val::Px(30.0)))
        },
    );
    let pale = Color::srgba(1.0, 1.0, 1.0, 0.55);
    p.texte(pied, &format!("v{}", bs_commun::VERSION), false, 12.0, pale);
    p.texte(
        pied,
        "Projet de fan, non affilié à Mojang",
        false,
        12.0,
        pale,
    );
}
