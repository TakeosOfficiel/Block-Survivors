//! Fiche du talent choisi, dans le panneau de droite : nom, rang, genre ; l'effet (ou
//! les deux options d'un talent à choix, à choisir avant l'achat) ; en rouge ce qui
//! manque (points dans l'arbre, prérequis, autre talent final, points restants) ; puis,
//! collé en bas, le grand « Apprendre » vert de l'accueil (ou un second clic sur la
//! case) et le réglage de l'arme de départ.

use bevy::prelude::*;
use bs_donnees::progression::{Noeud, TypeNoeud};
use bs_sauvegarde::progression::RefusAchat;

use super::{ActionTalent, Contexte, bouton, ecrire, lien, rangee};
use crate::ecrans::charte::{AIDE, IVOIRE, OR_VIF, ROUGE, T_AIDE, T_CORPS};
use crate::menu::{ActionMenu, Icones};

/// Texte d'effet, doré.
const DORE: Color = Color::srgb_u8(0xff, 0xd8, 0x6a);
const ORANGE: Color = Color::srgb_u8(0xff, 0x9a, 0x3c);

pub fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, f: Entity) {
    let prog = &c.sauvegarde.progression;
    let (p, a, n) = (c.id(), c.etat.arbre, c.noeud());
    let rang = prog.rang(p, a, n);
    let tete = rangee(commandes, f, 12.0);
    let icone = n
        .options
        .as_ref()
        .map_or(&n.icone, |o| &o[prog.option(p, a, n)].icone);
    commandes.spawn((
        ImageNode::new(ic.icone(icone)),
        Node {
            width: Val::Px(40.0),
            height: Val::Px(40.0),
            flex_shrink: 0.0,
            ..default()
        },
        ChildOf(tete),
    ));
    let noms = commandes
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                flex_shrink: 1.0,
                ..default()
            },
            ChildOf(tete),
        ))
        .id();
    ecrire(commandes, ic, noms, &n.nom, (true, 20.0, IVOIRE));
    let genre = match n.genre {
        TypeNoeud::Bonus => "Bonus",
        TypeNoeud::Capacite => "Capacité",
        TypeNoeud::Choix => "Talent à choix",
        TypeNoeud::Final => "Talent final",
    };
    let t = format!("Rang {rang}/{} · {genre}", n.rang_max);
    ecrire(commandes, ic, noms, &t, (false, T_AIDE, AIDE));
    effet(commandes, c, ic, f, n, rang);
    manques(commandes, c, ic, f, n);
    actions(commandes, c, ic, f, n, rang);
}

fn en_preparation(c: &Contexte, m: &Option<String>) -> bool {
    let pretes = &c.dj.d.talents.mecaniques_pretes;
    m.as_ref().is_some_and(|m| !pretes.iter().any(|x| x == m))
}

/// Effet, ou les deux options (cliquables tant que le talent n'est pas appris).
fn effet(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, f: Entity, n: &Noeud, rang: u32) {
    let Some(options) = &n.options else {
        ecrire(commandes, ic, f, &n.effet, (false, T_CORPS - 1.0, DORE));
        if en_preparation(c, &n.gains.mecanique) {
            let t = "Effet en préparation : il arrive dans une prochaine version.";
            ecrire(commandes, ic, f, t, (false, T_AIDE, ORANGE));
        }
        return;
    };
    let t = if rang > 0 {
        "Option retenue (réinitialiser pour changer) :"
    } else {
        "Choisis une option, définitive une fois apprise :"
    };
    ecrire(commandes, ic, f, t, (false, T_AIDE, AIDE));
    let choisie = c.sauvegarde.progression.option(c.id(), c.etat.arbre, n);
    for (i, o) in options.iter().enumerate() {
        let retenue = i == choisie;
        let b = commandes
            .spawn((
                Node {
                    column_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BorderColor::all(if retenue {
                    DORE.with_alpha(0.8)
                } else {
                    Color::srgba(1.0, 1.0, 1.0, 0.12)
                }),
                BackgroundColor(if retenue {
                    Color::srgba(1.0, 0.82, 0.29, 0.08)
                } else {
                    Color::NONE
                }),
                ChildOf(f),
            ))
            .id();
        if rang == 0 && !retenue {
            commandes
                .entity(b)
                .insert((Button, ActionMenu::Talent(ActionTalent::Option(i))));
        }
        let teinte = if retenue || rang == 0 {
            Color::WHITE
        } else {
            Color::srgb(0.4, 0.4, 0.4)
        };
        commandes.spawn((
            ImageNode::new(ic.icone(&o.icone)).with_color(teinte),
            Node {
                width: Val::Px(28.0),
                height: Val::Px(28.0),
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(b),
        ));
        let textes = commandes
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_shrink: 1.0,
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                ChildOf(b),
            ))
            .id();
        let couleur = if retenue { IVOIRE } else { AIDE };
        ecrire(commandes, ic, textes, &o.nom, (true, T_AIDE, couleur));
        let couleur = if retenue { DORE } else { AIDE };
        ecrire(
            commandes,
            ic,
            textes,
            &o.effet,
            (false, T_AIDE - 1.0, couleur),
        );
        if en_preparation(c, &o.gains.mecanique) {
            let t = "Effet en préparation";
            ecrire(commandes, ic, textes, t, (false, T_AIDE - 1.0, ORANGE));
        }
    }
}

/// Ce qui manque pour apprendre, en rouge (rien si le talent est ouvert).
fn manques(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, f: Entity, n: &Noeud) {
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let def = c.arbre();
    if prog.complet(p, a, n) {
        return;
    }
    let palier =
        c.dj.d
            .talents
            .paliers
            .iter()
            .rev()
            .find(|x| n.ligne >= x.ligne);
    if let Some(x) = palier.filter(|x| !prog.palier_atteint(p, a, x.ligne)) {
        let mis = prog.points(p, a, Some(x.ligne));
        let t = format!(
            "Nécessite {} points en {} ({mis}/{})",
            x.rangs, def.nom, x.rangs
        );
        ecrire(commandes, ic, f, &t, (false, T_AIDE, ROUGE));
    }
    if !n.libre {
        for id in &n.prerequis {
            let Some(x) = def.noeuds.iter().find(|x| &x.id == id) else {
                continue;
            };
            if !prog.complet(p, a, x) {
                let t = format!(
                    "Nécessite {} ({}/{})",
                    x.nom,
                    prog.rang(p, a, x),
                    x.rang_max
                );
                ecrire(commandes, ic, f, &t, (false, T_AIDE, ROUGE));
            }
        }
    }
    let note = match n.genre {
        TypeNoeud::Final => Some("Un seul talent final par arbre : il ferme les deux autres."),
        _ if n.gains.arme_depart.is_some() => {
            Some("Arme de départ en plus, désactivable sans perdre l'achat.")
        }
        _ => None,
    };
    if let Some(t) = note {
        ecrire(commandes, ic, f, t, (false, T_AIDE, AIDE));
    }
}

/// Action en bas : apprendre, ou l'état ; réglage de l'arme de départ.
fn actions(
    commandes: &mut Commands,
    c: &Contexte,
    ic: &mut Icones,
    f: Entity,
    n: &Noeud,
    rang: u32,
) {
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let actif = prog.actif(p, a, n);
    let bas = commandes
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                margin: UiRect::top(Val::Auto),
                ..default()
            },
            ChildOf(f),
        ))
        .id();
    if rang > 0 && n.gains.arme_depart.is_some() {
        let t = if actif {
            "Désactiver l'arme de départ ›"
        } else {
            "Activer l'arme de départ ›"
        };
        let action = Some(ActionMenu::Talent(ActionTalent::BasculerArme));
        lien(commandes, ic, bas, t, (T_CORPS, IVOIRE), action);
    }
    let perso = &c.dj.d.personnages.personnages[c.etat.personnage];
    let texte = |t: &str, couleur| (t.to_owned(), couleur);
    let etat = match prog.refus_achat(p, a, n) {
        None => {
            let t = if rang == 0 {
                "APPRENDRE"
            } else {
                "RANG SUIVANT"
            };
            bouton(commandes, ic, bas, t, ActionTalent::Acheter);
            let t = "ou clique encore sur la case";
            ecrire(commandes, ic, bas, t, (false, T_AIDE - 1.0, AIDE));
            return;
        }
        Some(RefusAchat::Max) => texte("Appris", OR_VIF),
        Some(RefusAchat::Verrou) => return,
        Some(RefusAchat::Exclusif) => texte("Un autre talent final est appris", ROUGE),
        Some(RefusAchat::Points) if prog.maitrise(p).au_max() => {
            texte("Tous les points sont dépensés", AIDE)
        }
        Some(RefusAchat::Points) => (
            format!("Plus de point : joue {} pour en gagner", perso.nom),
            AIDE,
        ),
    };
    ecrire(commandes, ic, bas, &etat.0, (true, T_CORPS, etat.1));
}
