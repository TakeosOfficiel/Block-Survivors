//! Fiche du talent consulté (35 % de la largeur) : en tête l'icône, le nom, le type, le
//! rang et la branche ; l'effet (ou les deux options d'un talent à choix) ; les
//! conditions (prérequis et palier avec leur état, coût) ; en bas l'action : « Apprendre
//! · 1 point » en vert si possible, sinon un bouton éteint qui dit pourquoi, plus les
//! réglages (talent final actif, arme de départ).

use bevy::prelude::*;
use bs_donnees::progression::{Noeud, TypeNoeud};
use bs_sauvegarde::progression::RefusAchat;

use super::{ActionTalent, Contexte, bouton, ecrire, rangee, schema};
use crate::ecrans::charte::{
    self, AIDE, BORD, IVOIRE, OR_VIF, ROUGE, Relief, T_AIDE, T_CORPS, T_SECTION, VERT_GAIN,
};
use crate::menu::{ActionMenu, Icones};

const ORANGE: Color = Color::srgb_u8(0xe8, 0xa8, 0x4a);

/// Fiche du talent consulté.
pub fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, parent: Entity) {
    let prog = &c.sauvegarde.progression;
    let (p, a, n) = (c.id(), c.etat.arbre, c.noeud());
    let def = c.arbre();
    let rang = prog.rang(p, a, n);
    let f = charte::panneau(
        commandes,
        parent,
        Node {
            flex_grow: 35.0,
            flex_basis: Val::ZERO,
            min_width: Val::Px(340.0),
            row_gap: Val::Px(10.0),
            ..default()
        },
    );
    // En tête : icône, nom, type · rang, branche
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
                row_gap: Val::Px(2.0),
                ..default()
            },
            ChildOf(tete),
        ))
        .id();
    ecrire(commandes, ic, noms, &n.nom, (true, T_SECTION, IVOIRE));
    let genre = match n.genre {
        TypeNoeud::Bonus => "Bonus",
        TypeNoeud::Capacite => "Capacité",
        TypeNoeud::Choix => "Talent à choix",
        TypeNoeud::Final => "Talent final",
    };
    let meta = rangee(commandes, noms, 6.0);
    let t = format!("{genre} · Rang {rang}/{} ·", n.rang_max);
    ecrire(commandes, ic, meta, &t, (false, T_AIDE, AIDE));
    let branche = def.branches.get(&n.branche).map_or("", String::as_str);
    let couleur = schema::couleur_branche(n.branche);
    ecrire(commandes, ic, meta, branche, (true, T_AIDE, couleur));

    effet(commandes, c, ic, f, n);
    conditions(commandes, c, ic, f, n);
    actions(commandes, c, ic, f, n);
}

/// Vrai si la mécanique du talent n'est pas encore jouée.
fn en_preparation(c: &Contexte, m: &Option<String>) -> bool {
    let pretes = &c.dj.d.talents.mecaniques_pretes;
    m.as_ref().is_some_and(|m| !pretes.iter().any(|x| x == m))
}

/// Effet du talent, ou ses deux options (choisissables une fois acheté).
fn effet(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, f: Entity, n: &Noeud) {
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let rang = prog.rang(p, a, n);
    let Some(options) = &n.options else {
        let e = ecrire(commandes, ic, f, &n.effet, (false, T_CORPS + 1.0, IVOIRE));
        commandes.entity(e).insert(Node {
            margin: UiRect::top(Val::Px(6.0)),
            ..default()
        });
        if en_preparation(c, &n.gains.mecanique) {
            let t = "Effet en préparation : il arrive dans une prochaine version.";
            ecrire(commandes, ic, f, t, (false, T_AIDE, ORANGE));
        }
        return;
    };
    let choisie = prog.option(p, a, n);
    for (i, o) in options.iter().enumerate() {
        let relief = if rang > 0 && i == choisie {
            Relief::CHOISI
        } else {
            Relief::SECONDAIRE
        };
        let b = charte::bouton_relief(commandes, f, relief, 0.0, ());
        commandes.entity(b).entry::<Node>().and_modify(|mut n| {
            n.height = Val::Auto;
            n.justify_content = JustifyContent::FlexStart;
            n.align_items = AlignItems::FlexStart;
            n.column_gap = Val::Px(10.0);
            n.padding = UiRect::all(Val::Px(8.0));
        });
        if rang > 0 {
            commandes
                .entity(b)
                .insert(ActionMenu::Talent(ActionTalent::Option(i)));
        } else {
            commandes.entity(b).remove::<Button>();
        }
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
                    flex_shrink: 1.0,
                    row_gap: Val::Px(3.0),
                    ..default()
                },
                ChildOf(b),
            ))
            .id();
        let nom = if rang > 0 && i == choisie {
            format!("✓ {}", o.nom)
        } else {
            o.nom.clone()
        };
        ecrire(commandes, ic, textes, &nom, (true, T_CORPS, IVOIRE));
        ecrire(commandes, ic, textes, &o.effet, (false, T_AIDE, AIDE));
        if en_preparation(c, &o.gains.mecanique) {
            ecrire(
                commandes,
                ic,
                textes,
                "Effet en préparation",
                (false, T_AIDE, ORANGE),
            );
        }
    }
}

/// Ligne de condition : coche verte ou croix rouge, libellé, détail.
fn condition(
    commandes: &mut Commands,
    ic: &Icones,
    f: Entity,
    ok: Option<bool>,
    libelle: &str,
    detail: &str,
) {
    let l = rangee(commandes, f, 8.0);
    commandes.entity(l).entry::<Node>().and_modify(|mut n| {
        n.align_items = AlignItems::FlexStart;
    });
    let (marque, couleur) = match ok {
        Some(true) => ("✓", VERT_GAIN),
        Some(false) => ("×", ROUGE),
        None => ("·", AIDE),
    };
    ecrire(commandes, ic, l, marque, (true, T_CORPS, couleur));
    let textes = rangee(commandes, l, 6.0);
    commandes
        .entity(textes)
        .entry::<Node>()
        .and_modify(|mut n| {
            n.flex_wrap = FlexWrap::Wrap;
            n.flex_shrink = 1.0;
        });
    ecrire(commandes, ic, textes, libelle, (true, T_AIDE, IVOIRE));
    let c = if ok == Some(false) { ROUGE } else { AIDE };
    ecrire(commandes, ic, textes, detail, (false, T_AIDE, c));
}

/// Prérequis, palier et coût, avec leur état.
fn conditions(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, f: Entity, n: &Noeud) {
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let def = c.arbre();
    let bloc = commandes
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                margin: UiRect::top(Val::Px(4.0)),
                padding: UiRect::top(Val::Px(10.0)),
                border: UiRect::top(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(BORD),
            ChildOf(f),
        ))
        .id();
    let rempli = |id: &String| {
        def.noeuds
            .iter()
            .find(|x| &x.id == id)
            .is_some_and(|x| prog.complet(p, a, x))
    };
    let (ok, detail) = if n.libre {
        (None, "talent libre".to_owned())
    } else if n.prerequis.is_empty() {
        (None, "racine de l'arbre".to_owned())
    } else {
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
        let ok = n.prerequis.iter().any(rempli);
        (Some(ok), format!("{} complet", noms.join(" ou ")))
    };
    condition(commandes, ic, bloc, ok, "Prérequis", &detail);
    let palier =
        c.dj.d
            .talents
            .paliers
            .iter()
            .rev()
            .find(|x| n.ligne >= x.ligne);
    if let Some(palier) = palier {
        let ok = prog.palier_atteint(p, a, palier.ligne);
        let achetes = palier.rangs.min(prog.points(p, a, Some(palier.ligne)));
        let t = format!("{achetes}/{} rangs achetés au-dessus", palier.rangs);
        condition(commandes, ic, bloc, Some(ok), "Palier", &t);
    }
    condition(
        commandes,
        ic,
        bloc,
        None,
        "Coût",
        "1 point de talent par rang",
    );
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
        ecrire(commandes, ic, bloc, t, (false, T_AIDE, AIDE));
    }
}

/// Réglages puis action principale, collés au bas de la fiche.
fn actions(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, f: Entity, n: &Noeud) {
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let rang = prog.rang(p, a, n);
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
    if rang > 0 && !actif {
        ecrire(
            commandes,
            ic,
            bas,
            "Acheté mais inactif dans cet arbre.",
            (false, T_AIDE, ORANGE),
        );
    }
    if rang > 0 && n.genre == TypeNoeud::Final {
        let (t, action) = if actif {
            ("✓ Talent final actif", None)
        } else {
            ("Activer ce talent final", Some(ActionTalent::ActiverFinal))
        };
        bouton(commandes, ic, bas, t, Relief::SECONDAIRE, action);
    }
    if rang > 0 && n.gains.arme_depart.is_some() {
        let t = if actif {
            "Désactiver l'arme de départ"
        } else {
            "Activer l'arme de départ"
        };
        let action = Some(ActionTalent::BasculerArme);
        bouton(commandes, ic, bas, t, Relief::SECONDAIRE, action);
    }
    let perso = &c.dj.d.personnages.personnages[c.etat.personnage];
    let (t, aide) = match prog.refus_achat(p, a, n) {
        None if rang == 0 => ("Apprendre · 1 point".to_owned(), None),
        None => ("Rang suivant · 1 point".to_owned(), None),
        Some(RefusAchat::Max) => ("✓ Appris".to_owned(), None),
        Some(RefusAchat::Verrou) => ("Prérequis non remplis".to_owned(), None),
        Some(RefusAchat::Points) => (
            "Aucun point disponible".to_owned(),
            Some(format!("Joue {} pour gagner de la maîtrise.", perso.nom)),
        ),
    };
    let achat = prog
        .refus_achat(p, a, n)
        .is_none()
        .then_some(ActionTalent::Acheter);
    let b = bouton(commandes, ic, bas, &t, Relief::PRINCIPAL, achat);
    commandes
        .entity(b)
        .entry::<Node>()
        .and_modify(|mut n| n.height = Val::Px(charte::H_PRINCIPAL));
    if let Some(aide) = aide {
        let e = ecrire(commandes, ic, bas, &aide, (false, T_AIDE, OR_VIF));
        commandes
            .entity(e)
            .insert(TextLayout::new_with_justify(Justify::Center));
    }
}
