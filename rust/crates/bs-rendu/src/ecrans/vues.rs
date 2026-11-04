//! Mise en page des écrans en jeu, au style des menus (`nuit`) : pause sur un voile de
//! nuit (« PAUSE » en grand, le résumé en ligne, « REPRENDRE » en pilule verte avec le
//! focus, le reste en liens), confirmation avant de recommencer ou d'abandonner,
//! statistiques. Le bilan défile à la molette (`Defilant`), sans panneau ; un indicateur
//! dit qu'il reste de quoi lire dessous. L'écran de fin est dans `fin`.

use bevy::prelude::*;
use bs_regles::partie::Partie;

use super::Action;
use super::bilan::{self, Contexte};
use super::charte::{AIDE, IVOIRE, OR_VIF, Relief};
use super::nuit::{self, Style, liens, nuit, pilule, voile};

/// Largeur du bilan.
const LARGEUR: f32 = 860.0;

/// Zone qui défile à la molette.
#[derive(Component)]
pub struct Defilant;

/// Indicateur « encore du contenu dessous » d'une zone défilante.
#[derive(Component)]
pub struct Indicateur(Entity);

/// Bilan défilant sans cadre, fondu en bas, indicateur dessous.
pub(super) fn panneau(commandes: &mut Commands, c: &Contexte, parent: Entity, p: &Partie) {
    let zone = commandes
        .spawn((
            Defilant,
            Node {
                width: Val::Px(LARGEUR),
                max_width: Val::Percent(96.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                min_height: Val::ZERO,
                flex_shrink: 1.0,
                padding: UiRect::new(Val::Px(8.0), Val::Px(20.0), Val::ZERO, Val::Px(8.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition::default(),
            ChildOf(parent),
        ))
        .id();
    bilan::bilan(commandes, c, zone, p);
    let indicateur = commandes
        .spawn((
            Indicateur(zone),
            Node {
                flex_shrink: 0.0,
                ..default()
            },
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id();
    let t = "⌄  Molette : la suite plus bas  ⌄";
    c.ecrit(commandes, indicateur, t, (Style::Grasse, 13.0, OR_VIF));
}

/// Montre l'indicateur tant que sa zone n'est pas défilée jusqu'en bas.
pub fn indiquer(
    mut indicateurs: Query<(&Indicateur, &mut Visibility)>,
    zones: Query<(&ScrollPosition, &ComputedNode), With<Defilant>>,
) {
    for (ind, mut v) in &mut indicateurs {
        let Ok((s, n)) = zones.get(ind.0) else {
            continue;
        };
        let reste = (n.content_size.y - n.size.y) * n.inverse_scale_factor - s.0.y;
        let voulu = if reste > 4.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        v.set_if_neq(voulu);
    }
}

/// Pause : titre, résumé, « Reprendre » (vert, focus), puis les liens Statistiques,
/// Options, Recommencer et Menu principal (confirmés). `coop` : pas de « Recommencer ».
pub fn pause(commandes: &mut Commands, c: &Contexte, p: &Partie, coop: bool) -> Entity {
    let r = voile(commandes, (nuit(0.45), nuit(0.85)));
    nuit::titre(commandes, c.polices, r, "PAUSE", OR_VIF);
    bilan::pastilles(commandes, c, r, p);
    let b = pilule(
        commandes,
        c.polices,
        r,
        "REPRENDRE",
        (Relief::PRINCIPAL, true),
        Action::Reprendre,
    );
    commandes
        .entity(b)
        .entry::<Node>()
        .and_modify(|mut n| n.margin = UiRect::top(Val::Px(14.0)));
    let mut liste = vec![
        ("Statistiques", Action::Statistiques),
        ("Options", Action::Options),
    ];
    if !coop {
        liste.push(("Recommencer", Action::Recommencer));
    }
    liste.push(("Menu principal", Action::Menu));
    liens(commandes, c.polices, r, liste);
    nuit::aide(commandes, c.polices, r, "Échap ou Entrée : reprendre");
    r
}

/// Confirmation de `action` (Recommencer ou Menu) : la partie en cours sera perdue.
pub fn confirmer(commandes: &mut Commands, c: &Contexte, action: Action) -> Entity {
    let r = voile(commandes, (nuit(0.6), nuit(0.92)));
    let (titre, detail, oui) = if action == Action::Menu {
        (
            "Abandonner la partie ?",
            "Retour au menu principal : la partie en cours est perdue.",
            "ABANDONNER",
        )
    } else {
        (
            "Recommencer la partie ?",
            "Une nouvelle partie démarre : celle-ci est perdue.",
            "RECOMMENCER",
        )
    };
    c.ecrit(commandes, r, titre, (Style::Logo, 52.0, IVOIRE));
    c.ecrit(commandes, r, detail, (Style::Normale, 18.0, AIDE));
    let l = nuit::rangee(commandes, r, 28.0);
    commandes
        .entity(l)
        .entry::<Node>()
        .and_modify(|mut n| n.margin = UiRect::top(Val::Px(12.0)));
    let rouge = (nuit::ROUGE, false);
    pilule(commandes, c.polices, l, oui, rouge, action);
    nuit::lien(
        commandes,
        c.polices,
        l,
        "Annuler",
        (20.0, IVOIRE),
        Action::Retour,
    );
    nuit::aide(commandes, c.polices, r, "Échap ou Entrée : annuler");
    r
}

/// Statistiques de la pause : le bilan, Retour.
pub fn statistiques(commandes: &mut Commands, c: &Contexte, p: &Partie) -> Entity {
    let r = voile(commandes, (nuit(0.82), nuit(0.95)));
    let t = "STATISTIQUES";
    let e = c.ecrit(commandes, r, t, (Style::Logo, 52.0, OR_VIF));
    commandes.entity(e).insert(Node {
        flex_shrink: 0.0,
        ..default()
    });
    panneau(commandes, c, r, p);
    let retour = ("‹  Retour", Action::Retour);
    liens(commandes, c.polices, r, [retour]);
    r
}
