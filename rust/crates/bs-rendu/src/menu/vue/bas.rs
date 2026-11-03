//! Bas de l'accueil : au centre le choix de partie (tuile de la classe, vignette de
//! l'arène, qui ouvrent leur volet, puis le grand « JOUER » vert en relief) ; dans les
//! coins Options et Nouveautés (avec sa pastille) à gauche, la version et « Quitter » à
//! droite, à l'écart du lancement. Plus le dé du pseudo, posé en haut à droite.

use bevy::prelude::*;

use super::{Pinceau, arenes, place};
use crate::ecrans::charte::{
    self, AIDE, IVOIRE, MARGE, OR_VIF, Relief, T_AIDE, T_CORPS, T_SECTION,
};
use crate::hud::police::TextePixel;
use crate::lancement::{ChoixPartie, DonneesJeu};
use crate::menu::{ActionMenu, Volet};
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

/// Marge du bas de l'écran.
const BAS: f32 = 24.0;
/// Largeur du choix de partie (tuiles et « JOUER »).
pub(super) const LARGEUR_CHOIX: f32 = 560.0;
const H_TUILE: f32 = 64.0;
const H_JOUER: f32 = 68.0;
/// Hauteur du choix de partie, du bas de l'écran au haut des tuiles.
pub(super) const HAUTEUR_CHOIX: f32 = BAS + H_JOUER + 10.0 + H_TUILE;

/// Tuile du choix de partie : vignette, nom, rubrique et flèche ; dorée si son volet est
/// ouvert.
fn tuile(
    p: &mut Pinceau,
    parent: Entity,
    (nom, rubrique): (&str, &str),
    (volet, ouvert): (Volet, bool),
) -> (Entity, Entity) {
    let relief = if ouvert {
        Relief::CHOISI
    } else {
        Relief::SECONDAIRE
    };
    let b = charte::bouton_relief(p.c, parent, relief, H_TUILE, ActionMenu::Volet(volet));
    p.c.entity(b).entry::<Node>().and_modify(|mut n| {
        n.flex_grow = 1.0;
        n.flex_basis = Val::ZERO;
        n.justify_content = JustifyContent::FlexStart;
        n.padding = UiRect::new(Val::Px(6.0), Val::Px(12.0), Val::ZERO, Val::ZERO);
        n.column_gap = Val::Px(10.0);
    });
    if ouvert {
        p.c.entity(b)
            .insert(Outline::new(Val::Px(2.0), Val::ZERO, OR_VIF));
    }
    let vignette = p.noeud(
        b,
        Node {
            width: Val::Px(64.0),
            height: Val::Px(44.0),
            flex_shrink: 0.0,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            ..default()
        },
    );
    p.c.entity(vignette).insert(BackgroundColor(charte::CREUX));
    let textes = p.noeud(
        b,
        Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            row_gap: Val::Px(1.0),
            ..default()
        },
    );
    p.texte(textes, rubrique, false, T_AIDE - 2.0, AIDE);
    p.texte(textes, nom, true, T_CORPS, IVOIRE);
    // Chevron : vers le haut (le volet s'ouvre au-dessus), vers le bas une fois ouvert
    let chevron = p.texte(b, "⌄", true, T_SECTION, if ouvert { OR_VIF } else { AIDE });
    if !ouvert {
        p.c.entity(chevron)
            .insert(UiTransform::from_rotation(Rot2::degrees(180.0)));
    }
    (b, vignette)
}

/// Choix de partie au centre, puis les coins.
pub(super) fn construire(
    p: &mut Pinceau,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    (choix, ouvert): (ChoixPartie, Volet),
    racine: Entity,
) {
    let d = dj.d;
    let centre = p.noeud(
        racine,
        Node {
            width: Val::Px(LARGEUR_CHOIX),
            margin: UiRect::left(Val::Px(-LARGEUR_CHOIX / 2.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..place(Some(Val::Percent(50.0)), None, None, Some(Val::Px(BAS)))
        },
    );
    let tuiles = p.noeud(
        centre,
        Node {
            column_gap: Val::Px(10.0),
            ..default()
        },
    );
    let perso = &d.personnages.personnages[choix.personnage];
    let classes = (Volet::Classes, ouvert == Volet::Classes);
    let (_, v) = tuile(p, tuiles, (&perso.nom, "Classe"), classes);
    let arme = perso.apparence.objet_en_main.as_deref().or_else(|| {
        perso
            .armes_depart
            .first()
            .and_then(|a| d.armes.get(a))
            .map(|a| a.icone.as_str())
    });
    if let Some(i) = arme {
        p.icone(v, i, 40.0);
    }
    let arene = dj.arenes[choix.arene].def;
    let arenes = (Volet::Arenes, ouvert == Volet::Arenes);
    let (_, v) = tuile(p, tuiles, (&arene.nom, "Arène"), arenes);
    arenes::photo(p, arene, false, v);

    let jouer = charte::bouton_relief(p.c, centre, Relief::PRINCIPAL, H_JOUER, ActionMenu::Jouer);
    p.c.entity(jouer).insert(BoxShadow::new(
        Color::srgba(0.0, 0.0, 0.0, 0.55),
        Val::ZERO,
        Val::Px(6.0),
        Val::ZERO,
        Val::Px(14.0),
    ));
    p.c.entity(jouer).entry::<Node>().and_modify(|mut n| {
        n.border = UiRect::new(Val::Px(3.0), Val::Px(3.0), Val::Px(3.0), Val::Px(6.0));
    });
    p.c.spawn((TextePixel::nouveau("JOUER", IVOIRE, 4.5), ChildOf(jouer)));

    coins(p, sauvegarde, racine);
}

/// Coins du bas : Options et Nouveautés à gauche ; version et « Quitter » à droite.
fn coins(p: &mut Pinceau, sauvegarde: &Sauvegarde, racine: Entity) {
    let bas = Some(Val::Px(BAS));
    let gauche = p.noeud(
        racine,
        Node {
            column_gap: Val::Px(8.0),
            ..place(Some(Val::Px(MARGE)), None, None, bas)
        },
    );
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

    let droite = p.noeud(
        racine,
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(16.0),
            ..place(None, Some(Val::Px(MARGE)), None, bas)
        },
    );
    let version = p.noeud(
        droite,
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            ..default()
        },
    );
    let pale = Color::srgba(1.0, 1.0, 1.0, 0.6);
    let v = format!("v{}", bs_commun::VERSION);
    p.texte(version, &v, false, 12.0, pale);
    p.texte(
        version,
        "Projet de fan, non affilié à Mojang",
        false,
        12.0,
        pale,
    );
    p.bouton(droite, "Quitter", Relief::SECONDAIRE, ActionMenu::Quitter);
}
