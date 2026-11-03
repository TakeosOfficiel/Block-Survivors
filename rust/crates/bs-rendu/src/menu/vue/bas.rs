//! Pied de l'accueil, en trois groupes : à gauche Options et Nouveautés (avec sa
//! pastille), au centre « Jouer » (vert en relief, résumé de la partie dessous) et
//! « Multijoueur », à droite la version et « Quitter », à l'écart des lancements. Plus
//! le dé du pseudo, posé dans le bandeau du haut.

use bevy::prelude::*;

use super::{PIED, Pinceau, place};
use crate::ecrans::charte::{self, AIDE, IVOIRE, MARGE, OR_VIF, Relief, T_AIDE, T_CORPS};
use crate::hud::police::TextePixel;
use crate::lancement::{ChoixPartie, DonneesJeu};
use crate::menu::ActionMenu;
use crate::sauvegarde::Sauvegarde;

/// Dé du pseudo au hasard : cinq points dessinés.
pub(super) fn de(p: &mut Pinceau, parent: Entity) {
    let b = charte::bouton_relief(
        p.c,
        parent,
        Relief::SECONDAIRE,
        charte::H_SECONDAIRE,
        ActionMenu::PseudoAleatoire,
    );
    p.c.entity(b).entry::<Node>().and_modify(|mut n| {
        n.width = Val::Px(charte::H_SECONDAIRE);
        n.padding = UiRect::ZERO;
    });
    let face = p.noeud(
        b,
        Node {
            width: Val::Px(18.0),
            height: Val::Px(18.0),
            border_radius: BorderRadius::all(Val::Px(2.0)),
            ..default()
        },
    );
    p.c.entity(face).insert(BackgroundColor(IVOIRE));
    for (x, y) in [
        (3.0, 3.0),
        (12.0, 3.0),
        (7.5, 7.5),
        (3.0, 12.0),
        (12.0, 12.0),
    ] {
        let point = p.noeud(
            face,
            Node {
                width: Val::Px(3.0),
                height: Val::Px(3.0),
                ..place(Some(Val::Px(x)), None, Some(Val::Px(y)), None)
            },
        );
        p.c.entity(point).insert(BackgroundColor(charte::FOND));
    }
}

/// Groupe du pied (un tiers de la largeur), aligné selon `alignement`.
fn groupe(p: &mut Pinceau, parent: Entity, alignement: JustifyContent) -> Entity {
    p.noeud(
        parent,
        Node {
            flex_grow: 1.0,
            flex_basis: Val::ZERO,
            justify_content: alignement,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        },
    )
}

/// Grand bouton à deux lignes (titre, sous-titre), de largeur `largeur`.
fn bouton_double(
    p: &mut Pinceau,
    parent: Entity,
    relief: Relief,
    largeur: f32,
    action: ActionMenu,
) -> Entity {
    let b = charte::bouton_relief(p.c, parent, relief, 60.0, action);
    p.c.entity(b).entry::<Node>().and_modify(move |mut n| {
        n.width = Val::Px(largeur);
        n.flex_direction = FlexDirection::Column;
        n.row_gap = Val::Px(3.0);
    });
    b
}

/// Pied : liens, « Jouer », « Multijoueur », version et « Quitter ».
pub(super) fn construire(
    p: &mut Pinceau,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    choix: ChoixPartie,
    racine: Entity,
) {
    let pied = p.noeud(
        racine,
        Node {
            height: Val::Px(PIED),
            flex_shrink: 0.0,
            padding: UiRect::horizontal(Val::Px(MARGE)),
            align_items: AlignItems::Center,
            ..default()
        },
    );
    let gauche = groupe(p, pied, JustifyContent::FlexStart);
    p.bouton(gauche, "Options", Relief::SECONDAIRE, ActionMenu::Options);
    let nouveautes = p.bouton(
        gauche,
        "Nouveautés",
        Relief::SECONDAIRE,
        ActionMenu::Nouveautes,
    );
    if crate::menu::nouveautes::a_lire(sauvegarde.profils.stockage()) {
        // Pastille : nouveautés de cette version pas encore lues
        let pastille = p.noeud(
            nouveautes,
            Node {
                padding: UiRect::axes(Val::Px(6.0), Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..place(None, Some(Val::Px(-8.0)), Some(Val::Px(-12.0)), None)
            },
        );
        p.c.entity(pastille).insert(BackgroundColor(OR_VIF));
        let t =
            p.ic.polices
                .texte("NOUVEAU", true, 11.0, Color::srgb_u8(0x2a, 0x1d, 0x02));
        p.c.spawn((t, ChildOf(pastille)));
    }

    let centre = groupe(p, pied, JustifyContent::Center);
    let jouer = bouton_double(p, centre, Relief::PRINCIPAL, 300.0, ActionMenu::Jouer);
    p.c.entity(jouer).insert(BoxShadow::new(
        Color::srgba(0.0, 0.0, 0.0, 0.5),
        Val::ZERO,
        Val::Px(4.0),
        Val::ZERO,
        Val::Px(10.0),
    ));
    p.c.spawn((TextePixel::nouveau("JOUER", IVOIRE, 3.0), ChildOf(jouer)));
    let nom = &dj.d.personnages.personnages[choix.personnage].nom;
    let resume = format!("{nom} · {}", dj.arenes[choix.arene].def.nom);
    p.texte(
        jouer,
        &resume,
        true,
        T_AIDE - 1.0,
        Color::srgb_u8(0xd6, 0xf0, 0xd8),
    );
    let multi = bouton_double(
        p,
        centre,
        Relief::SECONDAIRE,
        180.0,
        ActionMenu::OuvrirMulti,
    );
    p.texte(multi, "Multijoueur", true, T_CORPS, IVOIRE);
    p.texte(multi, "Coop en ligne", false, T_AIDE - 2.0, AIDE);

    let droite = groupe(p, pied, JustifyContent::FlexEnd);
    let version = p.noeud(
        droite,
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            margin: UiRect::right(Val::Px(16.0)),
            ..default()
        },
    );
    let pale = Color::srgba(1.0, 1.0, 1.0, 0.55);
    p.texte(
        version,
        &format!("v{}", bs_commun::VERSION),
        false,
        12.0,
        pale,
    );
    p.texte(
        version,
        "Projet de fan, non affilié à Mojang",
        false,
        12.0,
        pale,
    );
    p.bouton(droite, "Quitter", Relief::SECONDAIRE, ActionMenu::Quitter);
}
