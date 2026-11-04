//! Fiche du talent consulté, écrite à l'encre sur une page du livre de Minecraft posée
//! à droite : une grande icône, le nom, le type et la branche, le rang en crans ; l'effet (ou les deux
//! options d'un talent à choix, la retenue marquée d'un trait d'or) ; les conditions
//! (prérequis, palier, coût) cochées ou barrées ; en bas un seul bouton vert
//! « Apprendre · 1 point » quand l'achat est possible, sinon l'état en texte, plus les
//! réglage de l'arme de départ en lien. Un talent final écarté par un autre ou des
//! points épuisés à la maîtrise maximale le disent.

use bevy::prelude::*;
use bs_donnees::progression::{Noeud, TypeNoeud};
use bs_sauvegarde::progression::RefusAchat;

use super::{ActionTalent, Contexte, bouton, ecrire, lien, rangee, schema};
use crate::ecrans::charte::{MARGE, T_AIDE, T_CORPS, T_SECTION};
use crate::menu::{ActionMenu, Icones};

const ORANGE: Color = Color::srgb_u8(0xa8, 0x5a, 0x00);
/// Encres de la page.
const ENCRE: Color = Color::srgb_u8(0x3b, 0x26, 0x14);
const ENCRE_PALE: Color = Color::srgb_u8(0x7d, 0x65, 0x4b);
const VERT_GAIN: Color = Color::srgb_u8(0x2b, 0x7a, 0x33);
const ROUGE: Color = Color::srgb_u8(0xb0, 0x2e, 0x26);
const OR_VIF: Color = Color::srgb_u8(0xa3, 0x6e, 0x00);
/// Page du livre dans `gui/book` (px) et son agrandissement.
const PAGE: Rect = Rect {
    min: Vec2::new(20.0, 1.0),
    max: Vec2::new(166.0, 181.0),
};
const ECHELLE: f32 = 2.6;

/// Fiche du talent consulté.
pub fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, parent: Entity) {
    let prog = &c.sauvegarde.progression;
    let (p, a, n) = (c.id(), c.etat.arbre, c.noeud());
    let def = c.arbre();
    let rang = prog.rang(p, a, n);
    // Page posée à droite, centrée en hauteur
    let colonne = commandes
        .spawn((
            Node {
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                padding: UiRect::new(Val::Px(8.0), Val::Px(MARGE), Val::ZERO, Val::Px(8.0)),
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    let f = commandes
        .spawn((
            ImageNode {
                image: ic.icone("gui/book"),
                rect: Some(PAGE),
                ..default()
            },
            Node {
                width: Val::Px(PAGE.width() * ECHELLE),
                height: Val::Px(PAGE.height() * ECHELLE),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                padding: UiRect::new(Val::Px(40.0), Val::Px(26.0), Val::Px(26.0), Val::Px(24.0)),
                ..default()
            },
            BoxShadow::new(
                Color::srgba(0.0, 0.0, 0.0, 0.6),
                Val::Px(6.0),
                Val::Px(10.0),
                Val::ZERO,
                Val::Px(20.0),
            ),
            ChildOf(colonne),
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
    ecrire(commandes, ic, noms, &n.nom, (true, T_SECTION + 4.0, ENCRE));
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
        (false, T_AIDE, ENCRE_PALE),
    );
    let branche = def.branches.get(&n.branche).map_or("", String::as_str);
    let couleur = schema::couleur_branche(n.branche).darker(0.3);
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

/// Effet du talent, ou ses deux options (à choisir avant l'achat, fixées ensuite).
fn effet(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, f: Entity, n: &Noeud) {
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let rang = prog.rang(p, a, n);
    let Some(options) = &n.options else {
        let e = ecrire(commandes, ic, f, &n.effet, (false, T_CORPS + 1.0, ENCRE));
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
    // Choisie avant l'achat, fixée ensuite (réinitialiser pour changer)
    let choisie = prog.option(p, a, n);
    for (i, o) in options.iter().enumerate() {
        let jouee = i == choisie;
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
                    Color::srgba(0.64, 0.43, 0.0, 0.14)
                } else {
                    Color::srgba(0.23, 0.15, 0.08, 0.06)
                }),
                ChildOf(f),
            ))
            .id();
        if rang == 0 && !jouee {
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
        let nom = if jouee && rang > 0 {
            format!("✓ {}", o.nom)
        } else if jouee {
            format!("› {}", o.nom)
        } else {
            o.nom.clone()
        };
        ecrire(commandes, ic, textes, &nom, (true, T_CORPS, ENCRE));
        ecrire(commandes, ic, textes, &o.effet, (false, T_AIDE, ENCRE_PALE));
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
        None => ("·", ENCRE_PALE),
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
    ecrire(commandes, ic, textes, libelle, (true, T_AIDE, ENCRE));
    let c = if ok == Some(false) { ROUGE } else { ENCRE_PALE };
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
            Some("Un seul talent final par arbre : l'apprendre ferme les deux autres.")
        }
        _ if n.gains.arme_depart.is_some() => {
            Some("Une seule arme de départ en plus par arbre ; désactivable sans perdre l'achat.")
        }
        _ if n.options.is_some() => {
            Some("Choisis l'option avant d'apprendre : elle est définitive.")
        }
        _ => None,
    };
    if let Some(t) = activation {
        ecrire(commandes, ic, bloc, t, (false, T_AIDE, ENCRE_PALE));
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
                row_gap: Val::Px(10.0),
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
    if rang > 0 && n.gains.arme_depart.is_some() {
        let t = if actif {
            "Désactiver l'arme de départ ›"
        } else {
            "Activer l'arme de départ ›"
        };
        let action = Some(ActionMenu::Talent(ActionTalent::BasculerArme));
        lien(commandes, ic, bas, t, (T_CORPS, ENCRE), action);
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
        Some(RefusAchat::Exclusif) => {
            let t = "Un autre talent final est appris";
            ecrire(commandes, ic, bas, t, (true, T_CORPS, ROUGE));
            let t = "Réinitialise l'arbre pour changer de voie.";
            ecrire(commandes, ic, bas, t, (false, T_AIDE, ENCRE_PALE));
        }
        Some(RefusAchat::Points) if prog.maitrise(p).au_max() => {
            let t = "Tous les points sont dépensés";
            ecrire(commandes, ic, bas, t, (true, T_CORPS, ENCRE_PALE));
            let t = "Maîtrise au maximum : réinitialise l'arbre pour changer de voie.";
            ecrire(commandes, ic, bas, t, (false, T_AIDE, ENCRE_PALE));
        }
        Some(RefusAchat::Points) => {
            ecrire(
                commandes,
                ic,
                bas,
                "Aucun point disponible",
                (true, T_CORPS, ENCRE_PALE),
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
            Color::srgba(0.23, 0.15, 0.08, 0.18)
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
    let e = ecrire(commandes, ic, l, &t, (false, T_AIDE, ENCRE_PALE));
    commandes.entity(e).insert(Node {
        margin: UiRect::left(Val::Px(6.0)),
        ..default()
    });
}
