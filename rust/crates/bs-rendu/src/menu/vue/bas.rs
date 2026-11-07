//! Bas de l'accueil : en bas à gauche, sous la colonne, les liens Options, Nouveautés
//! (avec sa pastille) et Quitter ; en bas à droite, la version. Plus le dé du pseudo,
//! posé en haut à droite.

use bevy::prelude::*;

use super::colonne::lien;
use super::{Pinceau, place};
use crate::ecrans::charte::{self, IVOIRE, MARGE, OR_VIF, Relief, T_CORPS};
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

/// Marge du bas de l'écran.
pub(super) const BAS: f32 = 24.0;

/// Liens du bas à gauche, version à droite.
pub(super) fn construire(p: &mut Pinceau, sauvegarde: &Sauvegarde, racine: Entity) {
    let bas = Some(Val::Px(BAS));
    let gauche = p.noeud(
        racine,
        Node {
            column_gap: Val::Px(28.0),
            align_items: AlignItems::Center,
            ..place(Some(Val::Px(MARGE)), None, None, bas)
        },
    );
    let (_, icone) = lien(p, gauche, "Options", T_CORPS, ActionMenu::Options);
    p.icone(icone, "item/comparator", 28.0);
    let (b, icone) = lien(p, gauche, "Nouveautés", T_CORPS, ActionMenu::Nouveautes);
    p.icone(icone, "item/writable_book", 28.0);
    if crate::menu::nouveautes::a_lire(sauvegarde.profils.stockage()) {
        // Pastille : nouveautés de cette version pas encore lues
        let pastille = p.noeud(
            b,
            Node {
                padding: UiRect::axes(Val::Px(6.0), Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
        );
        p.c.entity(pastille).insert(BackgroundColor(OR_VIF));
        let t =
            p.ic.polices
                .texte("NOUVEAU", true, 11.0, Color::srgb_u8(0x2a, 0x1d, 0x02));
        p.c.spawn((t, ChildOf(pastille)));
    }
    let (_, icone) = lien(p, gauche, "Quitter", T_CORPS, ActionMenu::Quitter);
    p.icone(icone, "item/oak_door", 28.0);

    let version = p.noeud(
        racine,
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            ..place(None, Some(Val::Px(MARGE)), None, bas)
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
}
