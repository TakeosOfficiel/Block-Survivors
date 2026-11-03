//! Raccourcis de l'accueil, sous le logo : Talents (fiole de maîtrise, points libres en
//! pastille d'or), Collection (plastron), Multijoueur (deux visages). Une icône
//! reconnaissable, un libellé, une ligne d'aide.

use bevy::prelude::*;

use super::{Pinceau, place};
use crate::ecrans::charte::{self, AIDE, IVOIRE, OR_VIF, Relief, T_AIDE, T_CORPS};
use crate::lancement::{ChoixPartie, DonneesJeu};
use crate::menu::ActionMenu;
use crate::sauvegarde::Sauvegarde;

const LARGEUR: f32 = 236.0;
const HAUTEUR: f32 = 56.0;
const PLASTRON: &str = "item/iron_chestplate";

/// Bouton de raccourci ; renvoie (bouton, case de l'icône).
fn raccourci(
    p: &mut Pinceau,
    parent: Entity,
    (titre, aide): (&str, &str),
    action: ActionMenu,
) -> (Entity, Entity) {
    let b = charte::bouton_relief(p.c, parent, Relief::SECONDAIRE, HAUTEUR, action);
    p.c.entity(b).entry::<Node>().and_modify(|mut n| {
        n.width = Val::Px(LARGEUR);
        n.justify_content = JustifyContent::FlexStart;
        n.padding = UiRect::horizontal(Val::Px(8.0));
        n.column_gap = Val::Px(10.0);
    });
    let case = p.noeud(
        b,
        Node {
            width: Val::Px(40.0),
            height: Val::Px(40.0),
            flex_shrink: 0.0,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(2.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
    );
    p.c.entity(case).insert((
        BackgroundColor(charte::FOND),
        BorderColor::all(charte::BORD),
    ));
    let textes = p.noeud(
        b,
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
    );
    p.texte(textes, titre, true, T_CORPS, IVOIRE);
    p.texte(textes, aide, false, T_AIDE - 2.0, AIDE);
    (b, case)
}

pub(super) fn construire(
    p: &mut Pinceau,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    choix: ChoixPartie,
    parent: Entity,
) {
    let d = dj.d;
    let col = p.noeud(
        parent,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        },
    );
    // Talents de la classe choisie, points libres en pastille
    let (id, perso) = d
        .personnages
        .personnages
        .get_index(choix.personnage)
        .expect("classe choisie");
    let prog = &sauvegarde.progression;
    let arbre = prog.arbre_actif(id);
    let libres = prog.points_libres(id, arbre);
    let aide = format!("{} · Maîtrise {}", perso.nom, prog.maitrise(id).niveau);
    let (b, case) = raccourci(p, col, ("Talents", &aide), ActionMenu::Talents);
    p.icone(case, &d.talents.maitrise.icone, 28.0);
    if libres > 0 {
        let pastille = p.noeud(
            b,
            Node {
                min_width: Val::Px(22.0),
                height: Val::Px(22.0),
                padding: UiRect::horizontal(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(11.0)),
                ..place(None, Some(Val::Px(-8.0)), Some(Val::Px(-8.0)), None)
            },
        );
        p.c.entity(pastille).insert(BackgroundColor(OR_VIF));
        let t = p.ic.polices.texte(
            libres.to_string(),
            true,
            13.0,
            Color::srgb_u8(0x2a, 0x1d, 0x02),
        );
        p.c.spawn((t, ChildOf(pastille)));
    }
    let (_, case) = raccourci(
        p,
        col,
        ("Collection", "Armures et butin des boss"),
        ActionMenu::Collection,
    );
    p.icone(case, PLASTRON, 28.0);
    let (_, case) = raccourci(
        p,
        col,
        ("Multijoueur", "Coop en ligne, amis"),
        ActionMenu::OuvrirMulti,
    );
    // Deux visages : la skin du joueur et celle d'une autre classe
    let skin = &sauvegarde
        .depart(d, choix.personnage)
        .apparence
        .unwrap_or(&perso.apparence)
        .skin;
    let autre = d
        .personnages
        .personnages
        .values()
        .map(|p| &p.apparence.skin)
        .find(|s| *s != skin)
        .unwrap_or(skin);
    crate::menu::profil::visage(p.c, p.ic, case, skin, 16.0);
    crate::menu::profil::visage(p.c, p.ic, case, autre, 16.0);
}
