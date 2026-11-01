//! Page Collection du menu (src/ui/ecranCollection.ts) : un onglet par monde (avancée en
//! %), sa barre de progression, son armure (pièces gagnées, ou grisées avec leur
//! provenance ; bonus de pièce et d'ensemble), ses bagues et amulettes (« ??? » tant
//! qu'elles ne sont pas trouvées), puis les objets de tous les mondes. Ouverte sur le
//! monde de l'arène choisie.

use bevy::prelude::*;
use bs_donnees::arenes::Monde;
use bs_donnees::equipement::{Piece, Rarete};
use bs_sauvegarde::collection::{MONDES, PIECES};

use super::{ActionMenu, Icones};
use crate::dessins::Dessins;
use crate::ecrans::style::{self, GRIS, OR, OR_CLAIR, TEXTE};
use crate::hud::police::TextePixel;
use crate::icones_armures::{self, IconePiece};
use crate::lancement::DonneesJeu;
use crate::sauvegarde::Sauvegarde;

const LARGEUR_PANNEAU: f32 = 400.0;
const VERT: Color = Color::srgb_u8(0x55, 0xff, 0x55);

/// Monde ouvert d'abord : celui de l'arène choisie.
pub fn monde_de_l_arene(dj: &DonneesJeu, arene: usize) -> usize {
    let monde = dj.arenes[arene].def.monde;
    MONDES.iter().position(|&m| m == monde).unwrap_or(0)
}

fn couleur_rarete(r: Rarete) -> Color {
    match r {
        Rarete::Commune => Color::srgb_u8(0xc8, 0xcc, 0xd4),
        Rarete::Rare => Color::srgb_u8(0x6f, 0xa4, 0xff),
        Rarete::Epique => Color::srgb_u8(0xc0, 0x8a, 0xff),
        Rarete::Legendaire => Color::srgb_u8(0xff, 0xaa, 0x3a),
    }
}

fn texte(commandes: &mut Commands, parent: Entity, t: &str, couleur: Color, taille: f32) {
    commandes.spawn((TextePixel::nouveau(t, couleur, taille), ChildOf(parent)));
}

fn panneau(commandes: &mut Commands, parent: Entity, titre: &str) -> Entity {
    let p = commandes
        .spawn((
            Node {
                width: Val::Px(LARGEUR_PANNEAU),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            BorderColor::all(Color::srgb_u8(0x1a, 0x1a, 0x1a)),
            ChildOf(parent),
        ))
        .id();
    texte(commandes, p, titre, OR, 2.0);
    p
}

/// Tuile : icône dans sa case, nom et description ; grisée tant qu'elle n'est pas
/// obtenue.
fn tuile(
    commandes: &mut Commands,
    parent: Entity,
    icone: IconePiece,
    (nom, couleur_nom): (&str, Color),
    description: &str,
    obtenu: bool,
) {
    let t = commandes
        .spawn((
            Node {
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    let case = commandes
        .spawn((
            Node {
                width: Val::Px(40.0),
                height: Val::Px(40.0),
                flex_shrink: 0.0,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(0x8b, 0x8b, 0x8b)),
            BorderColor::all(Color::srgb_u8(0x37, 0x37, 0x37)),
            ChildOf(t),
        ))
        .id();
    let eteint = if obtenu {
        Color::WHITE
    } else {
        Color::srgba(0.15, 0.15, 0.15, 0.8)
    };
    let (ti, e) = (icone.teinte.to_srgba(), eteint.to_srgba());
    let teinte = Srgba::new(
        ti.red * e.red,
        ti.green * e.green,
        ti.blue * e.blue,
        e.alpha,
    );
    let image = commandes
        .spawn((
            ImageNode::new(icone.image).with_color(teinte.into()),
            Node {
                width: Val::Px(32.0),
                height: Val::Px(32.0),
                ..default()
            },
            ChildOf(case),
        ))
        .id();
    if let Some(calque) = icone.calque {
        commandes.spawn((
            ImageNode::new(calque).with_color(eteint),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(32.0),
                height: Val::Px(32.0),
                ..default()
            },
            ChildOf(image),
        ));
    }
    let textes = commandes
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                ..default()
            },
            ChildOf(t),
        ))
        .id();
    let couleur = if obtenu { couleur_nom } else { GRIS };
    texte(commandes, textes, nom, couleur, 1.5);
    commandes.spawn((
        TextePixel::nouveau(description, if obtenu { TEXTE } else { GRIS }, 1.5)
            .bloc(LARGEUR_PANNEAU - 72.0)
            .a_gauche(),
        ChildOf(textes),
    ));
}

/// « Lâchée parfois par le 1er boss de « Prairie ». »
fn provenance(d: &bs_donnees::Donnees, monde: Monde, piece: Piece) -> String {
    let poids = |rang: usize| {
        d.armures
            .butin_boss
            .get(rang)
            .and_then(|b| b.poids.get(&piece))
            .copied()
            .unwrap_or(0.0)
    };
    let (premier, second) = (poids(0), poids(1));
    let boss = if premier >= second {
        "le 1er boss"
    } else if premier == 0.0 {
        "le 2e boss (ou les suivants, en mode sans fin)"
    } else {
        "le 2e boss"
    };
    let nom = d.armures.noms_mondes.get(&monde).map_or("", |n| n.as_str());
    format!("Lâchée parfois par {boss} de « {nom} ».")
}

/// Construit la page sur le monde `k` ; renvoie sa racine.
pub fn construire(
    commandes: &mut Commands,
    (dj, sauvegarde, dessins): (&DonneesJeu, &Sauvegarde, &Dessins),
    ic: &mut Icones,
    k: usize,
) -> Entity {
    let d = dj.d;
    let col = &sauvegarde.collection;
    let monde = MONDES[k.min(MONDES.len() - 1)];
    let nom_monde = |m: Monde| d.armures.noms_mondes.get(&m).map_or("", |n| n.as_str());
    let racine = commandes
        .spawn((
            Name::new("collection"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(super::vue::FOND),
        ))
        .id();
    texte(commandes, racine, "Collection", OR_CLAIR, 3.0);
    // Onglets des mondes
    let onglets = commandes
        .spawn((
            Node {
                column_gap: Val::Px(6.0),
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    for (i, &m) in MONDES.iter().enumerate() {
        let part = (col.avancee(d, m).part * 100.0).round();
        let t = format!("{} · {part} %", nom_monde(m));
        let b = style::bouton(commandes, onglets, &t, 0.0, false, ActionMenu::Monde(i));
        let mut e = commandes.entity(b);
        e.entry::<Node>().and_modify(|mut n| {
            n.width = Val::Auto;
            n.height = Val::Px(36.0);
            n.padding = UiRect::horizontal(Val::Px(12.0));
        });
        if i == k {
            e.insert((
                super::talents::CHOISI,
                BorderColor::all(OR),
                BackgroundColor(super::talents::CHOISI.fond),
            ));
        }
    }
    // Avancée du monde
    let a = col.avancee(d, monde);
    let t = format!(
        "{} : {} / {} pièces · {} / {} objets",
        nom_monde(monde),
        a.pieces,
        PIECES.len(),
        a.objets,
        a.total_objets
    );
    texte(commandes, racine, &t, TEXTE, 2.0);
    let barre = commandes
        .spawn((
            Node {
                width: Val::Px(LARGEUR_PANNEAU),
                height: Val::Px(10.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(0x22, 0x22, 0x22)),
            BorderColor::all(Color::BLACK),
            ChildOf(racine),
        ))
        .id();
    commandes.spawn((
        Node {
            width: Val::Percent(a.part * 100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(VERT),
        ChildOf(barre),
    ));

    let corps = commandes
        .spawn((
            Node {
                column_gap: Val::Px(12.0),
                align_items: AlignItems::FlexStart,
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    // Armure du monde
    let armure = &d.armures.armures[&monde];
    let p = panneau(commandes, corps, &armure.nom);
    for piece in PIECES {
        let def = &armure.pieces[&piece];
        let possedee = col.possede(monde, piece);
        let icone = icones_armures::icone(d, dessins, monde, piece, |c| ic.icone(c));
        let nom_piece = d.armures.noms_pieces.get(&piece).map_or("", |n| n.as_str());
        let (nom, description) = if possedee {
            (def.nom.clone(), def.description.clone())
        } else {
            (
                format!("{nom_piece} : à gagner"),
                provenance(d, monde, piece),
            )
        };
        tuile(commandes, p, icone, (&nom, TEXTE), &description, possedee);
    }
    let complet = PIECES.iter().all(|&piece| col.possede(monde, piece));
    commandes.spawn((
        TextePixel::nouveau(
            &armure.ensemble.description,
            if complet { VERT } else { GRIS },
            1.5,
        )
        .bloc(LARGEUR_PANNEAU - 20.0)
        .a_gauche(),
        ChildOf(p),
    ));
    // Objets du monde, puis de tous les mondes
    let du_monde = panneau(commandes, corps, "Bagues et amulettes du monde");
    let communs = panneau(commandes, corps, "De tous les mondes");
    for (i, (id, o)) in d.equipement.objets.iter().enumerate() {
        let parent = match o.monde {
            Some(m) if m == monde => du_monde,
            Some(_) => continue,
            None => communs,
        };
        let trouve = col.decouvert(id);
        let icone = IconePiece {
            image: dessins.objets[i].clone(),
            calque: None,
            teinte: Color::WHITE,
        };
        let rarete = d
            .equipement
            .raretes
            .get(&o.rarete)
            .map_or("", |r| r.nom.as_str());
        let (nom, description) = if trouve {
            (
                o.nom.clone(),
                o.description.first().cloned().unwrap_or_default(),
            )
        } else {
            (
                "???".to_owned(),
                format!("{rarete} · à trouver dans un coffre"),
            )
        };
        tuile(
            commandes,
            parent,
            icone,
            (&nom, couleur_rarete(o.rarete)),
            &description,
            trouve,
        );
    }
    let retour = style::bouton(
        commandes,
        racine,
        "Retour",
        200.0,
        false,
        ActionMenu::Fermer,
    );
    commandes
        .entity(retour)
        .entry::<Node>()
        .and_modify(|mut n| {
            n.margin = UiRect::top(Val::Auto);
        });
    racine
}
