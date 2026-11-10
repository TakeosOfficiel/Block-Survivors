//! Côté droit, à l'air libre (aucun panneau ni case) : le titre, les classes en
//! liens (la choisie soulignée d'or), les deux arbres en pilules arrondies (nom et
//! points mis), les points restants et la maîtrise, puis, sous un trait fin, la fiche
//! du talent choisi (`fiche`). En bas à gauche, « ‹ Retour » et « Réinitialiser »
//! (les deux arbres, confirmé en deux temps), en liens.

use bevy::prelude::*;
use bs_donnees::progression::TypeDeblocage;

use super::{ActionTalent, Contexte, ecrire, fiche, lien, rangee};
use crate::ecrans::charte::{self, AIDE, IVOIRE, MARGE, OR_VIF, ROUGE, T_AIDE, T_CORPS};
use crate::hud::police::TextePixel;
use crate::menu::vue::Lien;
use crate::menu::{ActionMenu, Icones};

pub(super) const LARGEUR: f32 = 380.0;

/// Trait fin, sans cadre.
fn trait_fin(commandes: &mut Commands, parent: Entity) {
    commandes.spawn((
        Node {
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.1)),
        ChildOf(parent),
    ));
}

/// Le côté, de haut en bas.
pub(super) fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, parent: Entity) {
    let d = c.dj.d;
    let prog = &c.sauvegarde.progression;
    let p = c.id();
    let cadre = commandes
        .spawn((
            Node {
                width: Val::Px(LARGEUR),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(14.0),
                padding: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(16.0), Val::Px(8.0)),
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    commandes.spawn((TextePixel::nouveau("Talents", IVOIRE, 3.0), ChildOf(cadre)));
    // Classes : des liens, la choisie soulignée ; passent à la ligne au-delà de quatre
    let r = commandes
        .spawn((
            Node {
                align_items: AlignItems::Center,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(20.0),
                row_gap: Val::Px(8.0),
                ..default()
            },
            ChildOf(cadre),
        ))
        .id();
    for (k, (id, perso)) in d.personnages.personnages.iter().enumerate() {
        let libre = prog.verrou(TypeDeblocage::Personnage, id).is_none();
        let choisie = k == c.etat.personnage;
        let action = (libre && !choisie).then_some(ActionMenu::Talent(ActionTalent::Classe(k)));
        let couleur = if choisie {
            IVOIRE
        } else if libre {
            AIDE
        } else {
            AIDE.with_alpha(0.45)
        };
        let b = lien(commandes, ic, r, &perso.nom, (T_CORPS, couleur), action);
        if !libre {
            charte::cadenas(commandes, b, 9.0, couleur);
        }
        if choisie {
            commandes.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::ZERO,
                    right: Val::ZERO,
                    bottom: Val::ZERO,
                    height: Val::Px(2.0),
                    ..default()
                },
                BackgroundColor(OR_VIF),
                ChildOf(b),
            ));
        }
    }
    // Arbres : des pilules arrondies, nom et points mis
    let r = rangee(commandes, cadre, 10.0);
    for (a, def) in d.talents.arbres[p].iter().enumerate() {
        let choisi = a == c.etat.arbre;
        let couleur = if choisi { OR_VIF } else { IVOIRE };
        let b = commandes
            .spawn((
                Node {
                    flex_grow: 1.0,
                    flex_basis: Val::ZERO,
                    height: Val::Px(42.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    border: UiRect::all(Val::Px(1.5)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BorderColor::all(if choisi {
                    OR_VIF
                } else {
                    Color::srgba(1.0, 1.0, 1.0, 0.2)
                }),
                ChildOf(r),
            ))
            .id();
        if !choisi {
            let action = ActionMenu::Talent(ActionTalent::Arbre(a));
            commandes.entity(b).insert((Button, Lien(couleur), action));
        }
        let t = format!("{} · {}", def.nom, prog.points(p, a, None));
        ecrire(commandes, ic, b, &t, (true, T_CORPS, couleur));
    }
    // Points restants et maîtrise
    let r = rangee(commandes, cadre, 12.0);
    let libres = prog.points_libres(p);
    let couleur = if libres > 0 { OR_VIF } else { AIDE };
    ecrire(commandes, ic, r, &libres.to_string(), (true, 44.0, couleur));
    let s = if libres > 1 { "s" } else { "" };
    let m = prog.maitrise(p);
    let max = d.talents.maitrise.niveau_max;
    let t = format!("point{s} restant{s} · maîtrise {}/{max}", m.niveau);
    ecrire(commandes, ic, r, &t, (false, T_CORPS, AIDE));
    trait_fin(commandes, cadre);
    let f = commandes
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                row_gap: Val::Px(8.0),
                ..default()
            },
            ChildOf(cadre),
        ))
        .id();
    fiche::construire(commandes, c, ic, f);
}

/// « ‹ Retour » et « Réinitialiser » en bas à gauche, en liens.
pub(super) fn pied(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, racine: Entity) {
    let prog = &c.sauvegarde.progression;
    let r = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(MARGE),
                bottom: Val::Px(20.0),
                align_items: AlignItems::Center,
                column_gap: Val::Px(28.0),
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    let retour = Some(ActionMenu::Fermer);
    lien(commandes, ic, r, "‹ Retour", (T_CORPS, IVOIRE), retour);
    let valeur = prog.points_classe(c.id());
    let s = if valeur > 1 { "s" } else { "" };
    let reinit = Some(ActionMenu::Talent(ActionTalent::Reinitialiser));
    if c.etat.confirmation {
        let t = format!("Rendre les {valeur} point{s} des deux arbres ?");
        ecrire(commandes, ic, r, &t, (true, T_CORPS, ROUGE));
        let annuler = Some(ActionMenu::Talent(ActionTalent::Annuler));
        lien(commandes, ic, r, "Annuler", (T_CORPS, IVOIRE), annuler);
        lien(commandes, ic, r, "Confirmer", (T_CORPS, ROUGE), reinit);
    } else if valeur > 0 {
        let t = format!("Réinitialiser (+{valeur} point{s})");
        lien(commandes, ic, r, &t, (T_AIDE + 1.0, AIDE), reinit);
    }
}
