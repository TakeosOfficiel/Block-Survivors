//! Fiche du talent choisi (src/ui/ecranTalents.ts `dessinerFiche`) : nom, type, rang et
//! branche, effet (ou les deux options d'un talent à choix), prérequis, coût, puis les
//! boutons d'achat et de réglage (talent final actif, arme de départ).

use bevy::prelude::*;
use bs_donnees::progression::{Noeud, TypeNoeud};
use bs_sauvegarde::progression::RefusAchat;

use super::{
    ActionTalent, Contexte, JAUNE, LARGEUR_FICHE, ORANGE, onglet, paragraphe, rangee, schema, texte,
};
use crate::ecrans::style::{GRIS, TEXTE};
use crate::hud::police::TextePixel;
use crate::menu::Icones;

/// Fiche du talent choisi : effet, prérequis, coût, achat et réglages.
pub fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, parent: Entity) {
    let prog = &c.sauvegarde.progression;
    let (p, a, n) = (c.id(), c.etat.arbre, c.noeud());
    let def = c.arbre();
    let pretes = &c.dj.d.talents.mecaniques_pretes;
    let en_preparation =
        |m: &Option<String>| m.as_ref().is_some_and(|m| !pretes.iter().any(|x| x == m));
    let rang = prog.rang(p, a, n);
    let f = commandes
        .spawn((
            Node {
                width: Val::Px(LARGEUR_FICHE),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(16, 0, 16, 235)),
            BorderColor::all(Color::srgb_u8(0x28, 0x00, 0x6e)),
            ChildOf(parent),
        ))
        .id();
    texte(commandes, f, &n.nom, JAUNE, 2.5);
    let genre = match n.genre {
        TypeNoeud::Bonus => "Bonus",
        TypeNoeud::Capacite => "Capacité",
        TypeNoeud::Choix => "Talent à choix",
        TypeNoeud::Final => "Talent final",
    };
    let meta = rangee(commandes, f, 0.0);
    let t = format!("{genre} · rang {rang}/{} · ", n.rang_max);
    texte(commandes, meta, &t, GRIS, 1.5);
    let branche = def.branches.get(&n.branche).map_or("", String::as_str);
    texte(
        commandes,
        meta,
        branche,
        schema::couleur_branche(n.branche),
        1.5,
    );

    match &n.options {
        Some(options) => {
            let choisie = prog.option(p, a, n);
            for (i, o) in options.iter().enumerate() {
                let action = (rang > 0).then_some(ActionTalent::Option(i));
                let b = onglet(commandes, f, "", rang > 0 && i == choisie, action);
                commandes.entity(b).entry::<Node>().and_modify(|mut n| {
                    n.height = Val::Auto;
                    n.justify_content = JustifyContent::FlexStart;
                    n.column_gap = Val::Px(8.0);
                    n.padding = UiRect::all(Val::Px(6.0));
                });
                commandes.spawn((
                    ImageNode::new(ic.icone(&o.icone)),
                    Node {
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    ChildOf(b),
                ));
                let textes = commandes
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        ChildOf(b),
                    ))
                    .id();
                texte(commandes, textes, &o.nom, JAUNE, 2.0);
                commandes.spawn((
                    TextePixel::nouveau(&o.effet, TEXTE, 1.5)
                        .bloc(LARGEUR_FICHE - 80.0)
                        .a_gauche(),
                    ChildOf(textes),
                ));
                if en_preparation(&o.gains.mecanique) {
                    texte(commandes, textes, "Effet en préparation", ORANGE, 1.5);
                }
            }
        }
        None => {
            paragraphe(commandes, f, &n.effet, TEXTE);
            if en_preparation(&n.gains.mecanique) {
                let t = "Effet en préparation : il arrive dans une prochaine version.";
                paragraphe(commandes, f, t, ORANGE);
            }
        }
    }

    let couleur = if prog.ouvert(p, a, n) {
        TEXTE
    } else {
        Color::srgb_u8(0xff, 0x55, 0x55)
    };
    paragraphe(
        commandes,
        f,
        &format!("Prérequis : {}", prerequis(c, n)),
        couleur,
    );
    paragraphe(commandes, f, "Coût : 1 point de talent par rang", TEXTE);
    let activation = match n.genre {
        TypeNoeud::Final => {
            Some("Un seul talent final actif par arbre, changeable avant chaque partie.")
        }
        _ if n.gains.arme_depart.is_some() => {
            Some("Une seule arme de départ en plus par arbre ; désactivable sans perdre l'achat.")
        }
        _ if n.options.is_some() => {
            Some("L'achat débloque les deux options ; une seule joue à la fois.")
        }
        _ => None,
    };
    if let Some(t) = activation {
        paragraphe(commandes, f, &format!("Activation : {t}"), GRIS);
    }

    let boutons = rangee(commandes, f, 6.0);
    commandes
        .entity(boutons)
        .entry::<Node>()
        .and_modify(|mut n| {
            n.flex_wrap = FlexWrap::Wrap;
            n.row_gap = Val::Px(6.0);
        });
    let refus = prog.refus_achat(p, a, n);
    let t = match refus {
        Some(RefusAchat::Max) => "Complet",
        _ if rang == 0 => "Acheter (1 point)",
        _ => "Rang suivant (1 point)",
    };
    let achat = refus.is_none().then_some(ActionTalent::Acheter);
    onglet(commandes, boutons, t, false, achat);
    let actif = prog.actif(p, a, n);
    if rang > 0 && n.genre == TypeNoeud::Final {
        let (t, action) = if actif {
            ("Talent final actif", None)
        } else {
            ("Activer ce talent final", Some(ActionTalent::ActiverFinal))
        };
        onglet(commandes, boutons, t, false, action);
    }
    if rang > 0 && n.gains.arme_depart.is_some() {
        let t = if actif {
            "Désactiver l'arme de départ"
        } else {
            "Activer l'arme de départ"
        };
        onglet(
            commandes,
            boutons,
            t,
            false,
            Some(ActionTalent::BasculerArme),
        );
    }
    match refus {
        Some(RefusAchat::Verrou) => paragraphe(commandes, f, "Prérequis non remplis", ORANGE),
        Some(RefusAchat::Points) => {
            let t = "Plus de point libre : jouez cette classe pour monter sa maîtrise";
            paragraphe(commandes, f, t, ORANGE);
        }
        _ => {}
    }
    if rang > 0 && !actif {
        paragraphe(commandes, f, "Acheté mais inactif dans cet arbre.", ORANGE);
    }
}

/// Conditions d'achat du talent.
fn prerequis(c: &Contexte, n: &Noeud) -> String {
    let def = c.arbre();
    let mut parties = Vec::new();
    if n.libre {
        parties.push("Talent libre".to_owned());
    } else if !n.prerequis.is_empty() {
        let noms: Vec<&str> = n
            .prerequis
            .iter()
            .map(|id| {
                def.noeuds
                    .iter()
                    .find(|x| &x.id == id)
                    .map_or(id.as_str(), |x| &x.nom)
            })
            .collect();
        parties.push(format!("{} complet", noms.join(" ou ")));
    } else {
        parties.push("Racine de l'arbre".to_owned());
    }
    let palier =
        c.dj.d
            .talents
            .paliers
            .iter()
            .rev()
            .find(|x| n.ligne >= x.ligne);
    if let Some(palier) = palier {
        parties.push(format!("{} rangs achetés au-dessus", palier.rangs));
    }
    parties.join(" · ")
}
