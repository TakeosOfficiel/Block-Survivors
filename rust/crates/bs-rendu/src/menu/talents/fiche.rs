//! Fiche du talent consulté, flottant à droite sur un voile sombre, sans cadre : une
//! grande icône, le nom, le type et la branche, le rang en crans ; l'effet (ou les deux
//! options d'un talent à choix, la jouée marquée d'un trait d'or) ; les conditions
//! (prérequis, palier, coût) cochées ou barrées ; en bas un seul bouton vert
//! « Apprendre · 1 point » quand l'achat est possible, sinon l'état en texte, plus les
//! réglages en liens (talent final actif, arme de départ).

use bevy::prelude::*;
use bs_donnees::progression::{Noeud, TypeNoeud};
use bs_sauvegarde::progression::RefusAchat;

use super::{ActionTalent, Contexte, bouton, ecrire, lien, rangee, schema};
use crate::ecrans::charte::{
    AIDE, IVOIRE, MARGE, OR_VIF, ROUGE, T_AIDE, T_CORPS, T_SECTION, VERT_GAIN,
};
use crate::menu::{ActionMenu, Icones};

const ORANGE: Color = Color::srgb_u8(0xe8, 0xa8, 0x4a);

/// Fiche du talent consulté.
pub fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, parent: Entity) {
    let prog = &c.sauvegarde.progression;
    let (p, a, n) = (c.id(), c.etat.arbre, c.noeud());
    let def = c.arbre();
    let rang = prog.rang(p, a, n);
    let voile = LinearGradient::to_right(vec![
        ColorStop::new(Color::NONE, Val::Percent(0.0)),
        ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.45), Val::Percent(30.0)),
        ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.55), Val::Percent(100.0)),
    ]);
    let f = commandes
        .spawn((
            Node {
                width: Val::Px(430.0),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                padding: UiRect::new(Val::Px(70.0), Val::Px(MARGE), Val::Px(24.0), Val::Px(8.0)),
                ..default()
            },
            BackgroundGradient::from(voile),
            ChildOf(parent),
        ))
        .id();
    // En tête : icône, nom, type · branche, crans du rang
    let tete = rangee(commandes, f, 14.0);
    let icone = n
        .options
        .as_ref()
        .map_or(&n.icone, |o| &o[prog.option(p, a, n)].icone);
    commandes.spawn((
        ImageNode::new(ic.icone(icone)),
        Node {
            width: Val::Px(56.0),
            height: Val::Px(56.0),
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
    ecrire(commandes, ic, noms, &n.nom, (true, T_SECTION + 4.0, IVOIRE));
    let genre = match n.genre {
        TypeNoeud::Bonus => "Bonus",
        TypeNoeud::Capacite => "Capacité",
        TypeNoeud::Choix => "Talent à choix",
        TypeNoeud::Final => "Talent final",
    };
    let meta = rangee(commandes, noms, 6.0);
    ecrire(
        commandes,
        ic,
        meta,
        &format!("{genre} ·"),
        (false, T_AIDE, AIDE),
    );
    let branche = def.branches.get(&n.branche).map_or("", String::as_str);
    let couleur = schema::couleur_branche(n.branche);
    ecrire(commandes, ic, meta, branche, (true, T_AIDE, couleur));
    crans(commandes, ic, f, rang, n.rang_max);

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
        let jouee = rang > 0 && i == choisie;
        let b = commandes
            .spawn((
                Node {
                    column_gap: Val::Px(12.0),
                    padding: UiRect::new(Val::Px(12.0), Val::Px(8.0), Val::Px(8.0), Val::Px(8.0)),
                    border: UiRect::left(Val::Px(3.0)),
                    ..default()
                },
                BorderColor::all(if jouee { OR_VIF } else { Color::NONE }),
                BackgroundColor(if jouee {
                    Color::srgba(0.95, 0.78, 0.36, 0.10)
                } else {
                    Color::srgba(1.0, 1.0, 1.0, 0.03)
                }),
                ChildOf(f),
            ))
            .id();
        if rang > 0 && !jouee {
            commandes
                .entity(b)
                .insert((Button, ActionMenu::Talent(ActionTalent::Option(i))));
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
        let nom = if jouee {
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
                margin: UiRect::top(Val::Px(10.0)),
                ..default()
            },
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
        if actif {
            ecrire(
                commandes,
                ic,
                bas,
                "✓ Talent final actif",
                (true, T_CORPS, VERT_GAIN),
            );
        } else {
            let action = Some(ActionMenu::Talent(ActionTalent::ActiverFinal));
            lien(
                commandes,
                ic,
                bas,
                "Activer ce talent final ›",
                (T_CORPS, IVOIRE),
                action,
            );
        }
    }
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
    match prog.refus_achat(p, a, n) {
        None => {
            let t = if rang == 0 {
                "Apprendre · 1 point"
            } else {
                "Rang suivant · 1 point"
            };
            bouton(commandes, ic, bas, t, ActionTalent::Acheter);
        }
        Some(RefusAchat::Max) => {
            ecrire(commandes, ic, bas, "✓ Appris", (true, T_SECTION, OR_VIF));
        }
        Some(RefusAchat::Verrou) => {
            ecrire(
                commandes,
                ic,
                bas,
                "Prérequis non remplis",
                (true, T_CORPS, ROUGE),
            );
        }
        Some(RefusAchat::Points) => {
            ecrire(
                commandes,
                ic,
                bas,
                "Aucun point disponible",
                (true, T_CORPS, AIDE),
            );
            let t = format!("Joue {} pour gagner de la maîtrise.", perso.nom);
            ecrire(commandes, ic, bas, &t, (false, T_AIDE, OR_VIF));
        }
    }
}

/// Rang en crans : un trait par rang, d'or s'il est acquis.
fn crans(commandes: &mut Commands, ic: &Icones, f: Entity, rang: u32, max: u32) {
    let l = rangee(commandes, f, 4.0);
    for i in 0..max {
        let couleur = if i < rang {
            OR_VIF
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.12)
        };
        commandes.spawn((
            Node {
                width: Val::Px(26.0),
                height: Val::Px(5.0),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(couleur),
            ChildOf(l),
        ));
    }
    let t = format!("Rang {rang}/{max}");
    let e = ecrire(commandes, ic, l, &t, (false, T_AIDE, AIDE));
    commandes.entity(e).insert(Node {
        margin: UiRect::left(Val::Px(6.0)),
        ..default()
    });
}
