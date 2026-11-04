//! Haut et bas de la page des Talents, sans cadres : le titre, les classes en liens
//! (icône d'arme et nom, soulignés d'or si choisis, cadenas si fermées) et, à droite,
//! les points restants de la classe en grand et la maîtrise ; en bas, les liens
//! « Retour » et « Réinitialiser » (les deux arbres, confirmé en deux temps).

use bevy::prelude::*;
use bs_donnees::progression::TypeDeblocage;

use super::{ActionTalent, Contexte, ETEINT, ecrire, lien, rangee, souligner};
use crate::ecrans::charte::{self, AIDE, IVOIRE, MARGE, OR_VIF, ROUGE, T_AIDE, T_CORPS, T_SECTION};
use crate::menu::{ActionMenu, Icones};

fn icone(commandes: &mut Commands, ic: &mut Icones, parent: Entity, i: &str, cote: f32) {
    let e = commandes
        .spawn((
            ImageNode::new(ic.icone(i)),
            Node {
                width: Val::Px(cote),
                height: Val::Px(cote),
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .id();
    commandes.entity(parent).insert_children(0, &[e]);
}

/// Titre, classes, points disponibles.
pub(super) fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, racine: Entity) {
    let d = c.dj.d;
    let prog = &c.sauvegarde.progression;
    let p = c.id();
    let e = rangee(commandes, racine, 40.0);
    commandes.entity(e).entry::<Node>().and_modify(|mut n| {
        n.padding = UiRect::new(Val::Px(MARGE), Val::Px(MARGE), Val::Px(18.0), Val::Px(6.0));
    });
    let titre = ecrire(commandes, ic, e, "Talents", (true, 36.0, IVOIRE));
    commandes.entity(titre).insert(Node {
        margin: UiRect::right(Val::Px(36.0)),
        ..default()
    });
    // Classes
    let r = rangee(commandes, e, 30.0);
    for (k, (id, perso)) in d.personnages.personnages.iter().enumerate() {
        let libre = prog.verrou(TypeDeblocage::Personnage, id).is_none();
        let choisie = k == c.etat.personnage;
        let couleur = if choisie { IVOIRE } else { ETEINT };
        let action = (libre && !choisie).then_some(ActionMenu::Talent(ActionTalent::Classe(k)));
        let b = lien(
            commandes,
            ic,
            r,
            &perso.nom,
            (T_SECTION - 2.0, couleur),
            action,
        );
        let arme = perso.apparence.objet_en_main.as_deref().or_else(|| {
            let arme = perso.armes_depart.first().and_then(|a| d.armes.get(a));
            arme.map(|a| a.icone.as_str())
        });
        if let Some(i) = arme {
            icone(commandes, ic, b, i, 26.0);
        }
        if !libre {
            charte::cadenas(commandes, b, 11.0, AIDE);
        }
        if choisie {
            souligner(commandes, b);
        }
    }
    // Points disponibles, poussés à droite
    let droite = commandes
        .spawn((
            Node {
                margin: UiRect::left(Val::Auto),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            ChildOf(e),
        ))
        .id();
    let libres = prog.points_libres(p);
    let s = if libres > 1 { "s" } else { "" };
    let couleur = if libres > 0 { OR_VIF } else { AIDE };
    let t = format!("{libres} point{s} restant{s}");
    let points = ecrire(commandes, ic, droite, &t, (true, T_SECTION + 2.0, couleur));
    if libres > 0 {
        commandes.entity(points).insert(TextShadow {
            offset: Vec2::ZERO,
            color: OR_VIF.with_alpha(0.35),
        });
    }
    let m = prog.maitrise(p);
    let max = d.talents.maitrise.niveau_max;
    let t = format!("Maîtrise {} / {max} · un point par niveau", m.niveau);
    let l = rangee(commandes, droite, 6.0);
    ecrire(commandes, ic, l, &t, (false, T_AIDE, AIDE));
    icone(commandes, ic, l, &d.talents.maitrise.icone, 18.0);
}

/// « ‹ Retour » à gauche ; « Réinitialiser » à droite, en deux temps.
pub(super) fn pied(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, racine: Entity) {
    let prog = &c.sauvegarde.progression;
    let p = c.id();
    let r = rangee(commandes, racine, 8.0);
    commandes.entity(r).entry::<Node>().and_modify(|mut n| {
        n.justify_content = JustifyContent::SpaceBetween;
        n.padding = UiRect::new(Val::Px(MARGE), Val::Px(MARGE), Val::Px(4.0), Val::Px(16.0));
    });
    let retour = lien(
        commandes,
        ic,
        r,
        "‹ Retour",
        (T_CORPS + 1.0, IVOIRE),
        Some(ActionMenu::Fermer),
    );
    ecrire(commandes, ic, retour, "Échap", (false, T_AIDE, AIDE));
    let droite = rangee(commandes, r, 24.0);
    let valeur = prog.points_classe(p);
    let s = if valeur > 1 { "s" } else { "" };
    let reinit = Some(ActionMenu::Talent(ActionTalent::Reinitialiser));
    if c.etat.confirmation {
        let t = format!("Rendre les {valeur} point{s} des deux arbres ?");
        ecrire(commandes, ic, droite, &t, (false, T_CORPS, ROUGE));
        let annuler = Some(ActionMenu::Talent(ActionTalent::Annuler));
        lien(commandes, ic, droite, "Annuler", (T_CORPS, IVOIRE), annuler);
        lien(commandes, ic, droite, "Confirmer", (T_CORPS, ROUGE), reinit);
    } else if valeur > 0 {
        let t = format!("Réinitialiser (+{valeur} point{s})");
        lien(commandes, ic, droite, &t, (T_CORPS, ETEINT), reinit);
    }
}
