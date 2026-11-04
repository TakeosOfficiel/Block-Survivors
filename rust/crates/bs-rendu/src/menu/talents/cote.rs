//! Panneau charbon de droite, comme les volets de l'accueil : le titre, les classes
//! (une case par arme, la choisie en or), les deux arbres (nom et points mis), les
//! points restants et la maîtrise, puis la fiche du talent choisi (`fiche`). En bas à
//! gauche, « ‹ Retour » et « Réinitialiser » (les deux arbres, confirmé en deux temps),
//! en boutons à relief comme Options et Nouveautés.

use bevy::prelude::*;
use bs_donnees::progression::TypeDeblocage;

use super::{ActionTalent, Contexte, ecrire, fiche, rangee};
use crate::ecrans::charte::{
    self, AIDE, BORD, IVOIRE, MARGE, OR_VIF, ROUGE, Relief, T_AIDE, T_CORPS,
};
use crate::hud::police::TextePixel;
use crate::menu::{ActionMenu, Icones};

pub(super) const LARGEUR: f32 = 400.0;

fn icone(commandes: &mut Commands, ic: &mut Icones, parent: Entity, i: &str, cote: f32) -> Entity {
    commandes
        .spawn((
            ImageNode::new(ic.icone(i)),
            Node {
                width: Val::Px(cote),
                height: Val::Px(cote),
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(parent),
        ))
        .id()
}

/// Liseré d'or sous un bouton choisi.
fn lisere(commandes: &mut Commands, b: Entity) {
    commandes.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(-2.0),
            right: Val::Px(-2.0),
            bottom: Val::Px(-4.0),
            height: Val::Px(3.0),
            ..default()
        },
        BackgroundColor(OR_VIF),
        ChildOf(b),
    ));
}

fn bouton(
    commandes: &mut Commands,
    parent: Entity,
    choisi: bool,
    action: Option<ActionTalent>,
) -> Entity {
    let relief = if choisi {
        Relief::CHOISI
    } else {
        Relief::SECONDAIRE
    };
    let b = charte::bouton_relief(commandes, parent, relief, 50.0, ());
    if let Some(a) = action {
        commandes.entity(b).insert(ActionMenu::Talent(a));
    } else {
        commandes.entity(b).remove::<Button>();
    }
    commandes.entity(b).entry::<Node>().and_modify(|mut n| {
        n.flex_grow = 1.0;
        n.flex_basis = Val::ZERO;
        n.padding = UiRect::horizontal(Val::Px(8.0));
    });
    if choisi {
        lisere(commandes, b);
    }
    b
}

/// Le panneau, de haut en bas.
pub(super) fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, parent: Entity) {
    let d = c.dj.d;
    let prog = &c.sauvegarde.progression;
    let p = c.id();
    let cadre = charte::panneau(
        commandes,
        parent,
        Node {
            width: Val::Px(LARGEUR),
            flex_shrink: 0.0,
            row_gap: Val::Px(12.0),
            align_self: AlignSelf::Stretch,
            ..default()
        },
    );
    commandes.spawn((TextePixel::nouveau("Talents", IVOIRE, 2.5), ChildOf(cadre)));
    // Classes : une case par arme
    let r = rangee(commandes, cadre, 8.0);
    for (k, (id, perso)) in d.personnages.personnages.iter().enumerate() {
        let libre = prog.verrou(TypeDeblocage::Personnage, id).is_none();
        let choisie = k == c.etat.personnage;
        let action = (libre && !choisie).then_some(ActionTalent::Classe(k));
        let b = bouton(commandes, r, choisie, action);
        let arme = perso.apparence.objet_en_main.as_deref().or_else(|| {
            let arme = perso.armes_depart.first().and_then(|a| d.armes.get(a));
            arme.map(|a| a.icone.as_str())
        });
        if let Some(i) = arme {
            let e = icone(commandes, ic, b, i, 30.0);
            if !libre {
                commandes
                    .entity(e)
                    .insert(ImageNode::new(ic.icone(i)).with_color(Color::srgb(0.35, 0.35, 0.35)));
            }
        }
        if !libre {
            charte::cadenas(commandes, b, 11.0, AIDE);
        }
    }
    let perso = &d.personnages.personnages[c.etat.personnage];
    let l = ecrire(
        commandes,
        ic,
        cadre,
        &perso.nom,
        (true, T_CORPS + 3.0, IVOIRE),
    );
    commandes.entity(l).insert(Node {
        margin: UiRect::top(Val::Px(2.0)),
        ..default()
    });
    // Arbres : nom et points mis
    let r = rangee(commandes, cadre, 8.0);
    for (a, def) in d.talents.arbres[p].iter().enumerate() {
        let choisi = a == c.etat.arbre;
        let action = (!choisi).then_some(ActionTalent::Arbre(a));
        let b = bouton(commandes, r, choisi, action);
        commandes.entity(b).entry::<Node>().and_modify(|mut n| {
            n.justify_content = JustifyContent::FlexStart;
        });
        icone(commandes, ic, b, &def.icone, 24.0);
        let couleur = if choisi { IVOIRE } else { AIDE };
        ecrire(commandes, ic, b, &def.nom, (true, T_CORPS, couleur));
        let mis = prog.points(p, a, None).to_string();
        let e = ecrire(commandes, ic, b, &mis, (true, T_CORPS, OR_VIF));
        commandes.entity(e).insert(Node {
            margin: UiRect::left(Val::Auto),
            ..default()
        });
    }
    // Points restants et maîtrise
    let r = rangee(commandes, cadre, 10.0);
    let libres = prog.points_libres(p);
    let couleur = if libres > 0 { OR_VIF } else { AIDE };
    ecrire(commandes, ic, r, &libres.to_string(), (true, 30.0, couleur));
    let s = if libres > 1 { "s" } else { "" };
    let textes = commandes
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(r),
        ))
        .id();
    let t = format!("point{s} restant{s}");
    ecrire(commandes, ic, textes, &t, (true, T_CORPS, couleur));
    let m = prog.maitrise(p);
    let max = d.talents.maitrise.niveau_max;
    let t = format!("Maîtrise {} / {max} · un point par niveau", m.niveau);
    ecrire(commandes, ic, textes, &t, (false, T_AIDE - 1.0, AIDE));
    // Fiche, sous un trait
    let f = commandes
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                row_gap: Val::Px(8.0),
                padding: UiRect::top(Val::Px(12.0)),
                border: UiRect::top(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(BORD),
            ChildOf(cadre),
        ))
        .id();
    fiche::construire(commandes, c, ic, f);
}

/// « ‹ Retour » et « Réinitialiser » en bas à gauche.
pub(super) fn pied(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, racine: Entity) {
    let prog = &c.sauvegarde.progression;
    let r = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(MARGE),
                bottom: Val::Px(24.0),
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    let h = charte::H_SECONDAIRE;
    let b = charte::bouton_relief(commandes, r, Relief::SECONDAIRE, h, ActionMenu::Fermer);
    ecrire(commandes, ic, b, "‹ Retour", (true, T_CORPS, IVOIRE));
    let valeur = prog.points_classe(c.id());
    let s = if valeur > 1 { "s" } else { "" };
    let reinit = ActionMenu::Talent(ActionTalent::Reinitialiser);
    if c.etat.confirmation {
        let t = format!("Rendre les {valeur} point{s} des deux arbres ?");
        let e = ecrire(commandes, ic, r, &t, (true, T_CORPS, ROUGE));
        commandes.entity(e).insert(TextShadow::default());
        let annuler = ActionMenu::Talent(ActionTalent::Annuler);
        let b = charte::bouton_relief(commandes, r, Relief::SECONDAIRE, h, annuler);
        ecrire(commandes, ic, b, "Annuler", (true, T_CORPS, IVOIRE));
        let b = charte::bouton_relief(commandes, r, Relief::SECONDAIRE, h, reinit);
        ecrire(commandes, ic, b, "Confirmer", (true, T_CORPS, ROUGE));
    } else if valeur > 0 {
        let b = charte::bouton_relief(commandes, r, Relief::SECONDAIRE, h, reinit);
        let t = format!("Réinitialiser (+{valeur} point{s})");
        ecrire(commandes, ic, b, &t, (true, T_CORPS, IVOIRE));
    }
}
