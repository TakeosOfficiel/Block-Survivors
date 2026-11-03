//! Page Collection du menu (src/ui/ecranCollection.ts) : un onglet par monde (avancée en
//! %), sa barre de progression, son armure (pièces gagnées, ou grisées avec leur
//! provenance ; bonus de pièce et d'ensemble), ses bagues et amulettes (« ??? » tant
//! qu'elles ne sont pas trouvées), puis les objets de tous les mondes. Ouverte sur le
//! monde de l'arène choisie. Charte commune (docs/CHARTE-UI.md) : onglets soulignés,
//! compteurs par panneau, une règle de provenance par panneau plutôt que sur chaque tuile.

use bevy::prelude::*;
use bs_donnees::arenes::Monde;
use bs_donnees::equipement::{Piece, Rarete};
use bs_sauvegarde::collection::{MONDES, PIECES};

use super::{ActionMenu, Icones};
use crate::dessins::Dessins;
use crate::ecrans::charte::{self, AIDE, IVOIRE, OR_VIF, Relief};
use crate::hud::police::TextePixel;
use crate::icones_armures::{self, IconePiece};
use crate::lancement::DonneesJeu;
use crate::sauvegarde::Sauvegarde;

const LARGEUR_PANNEAU: f32 = 400.0;
const VERT: Color = charte::VERT_GAIN;
const GRIS: Color = AIDE;
const TEXTE: Color = IVOIRE;

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

/// Panneau de la charte : titre et compteur (« 2/4 »), règle commune dessous.
fn panneau(
    commandes: &mut Commands,
    parent: Entity,
    (titre, compte): (&str, String),
    regle: &str,
) -> Entity {
    let p = charte::panneau(
        commandes,
        parent,
        Node {
            width: Val::Px(LARGEUR_PANNEAU),
            row_gap: Val::Px(8.0),
            ..default()
        },
    );
    let entete = commandes
        .spawn((
            Node {
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::bottom(Val::Px(6.0)),
                border: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(charte::BORD),
            ChildOf(p),
        ))
        .id();
    texte(commandes, entete, titre, IVOIRE, 2.0);
    texte(commandes, entete, &compte, OR_VIF, 2.0);
    commandes.spawn((
        TextePixel::nouveau(regle, GRIS, 1.5)
            .bloc(LARGEUR_PANNEAU - 36.0)
            .a_gauche(),
        ChildOf(p),
    ));
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
            BackgroundColor(charte::CREUX),
            BorderColor::all(charte::BORD),
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

/// Règle commune des pièces d'armure du monde : quel boss lâche quoi.
fn regle_armure(d: &bs_donnees::Donnees, monde: Monde) -> String {
    let poids = |rang: usize, piece: Piece| {
        d.armures
            .butin_boss
            .get(rang)
            .and_then(|b| b.poids.get(&piece))
            .copied()
            .unwrap_or(0.0)
    };
    let nom_piece = |p: Piece| d.armures.noms_pieces.get(&p).map_or("", |n| n.as_str());
    let (mut premier, mut second) = (Vec::new(), Vec::new());
    for piece in PIECES {
        if poids(0, piece) >= poids(1, piece) && poids(0, piece) > 0.0 {
            premier.push(nom_piece(piece).to_lowercase());
        } else {
            second.push(nom_piece(piece).to_lowercase());
        }
    }
    let nom = d.armures.noms_mondes.get(&monde).map_or("", |n| n.as_str());
    let mut t = format!("Lâchées parfois par les boss de « {nom} »");
    if !premier.is_empty() {
        t += &format!(" · 1er boss : {}", premier.join(", "));
    }
    if !second.is_empty() {
        t += &format!(" · 2e boss : {}", second.join(", "));
    }
    t + "."
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
                padding: UiRect::all(Val::Px(charte::MARGE)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(charte::FOND),
        ))
        .id();
    texte(commandes, racine, "Collection", IVOIRE, 4.0);
    // Onglets des mondes, avancée de chacun ; l'ouvert souligné d'or
    let onglets = commandes
        .spawn((
            Node {
                column_gap: Val::Px(8.0),
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    for (i, &m) in MONDES.iter().enumerate() {
        let part = (col.avancee(d, m).part * 100.0).round();
        let relief = if i == k {
            Relief::CHOISI
        } else {
            Relief::SECONDAIRE
        };
        let b = charte::bouton_relief(commandes, onglets, relief, 40.0, ActionMenu::Monde(i));
        commandes
            .entity(b)
            .entry::<Node>()
            .and_modify(|mut n| n.min_width = Val::Px(150.0));
        let couleur = if i == k { OR_VIF } else { IVOIRE };
        texte(commandes, b, nom_monde(m), couleur, 2.0);
        texte(commandes, b, &format!("{part} %"), GRIS, 1.5);
        if i == k {
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
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(charte::CREUX),
            BorderColor::all(charte::BORD),
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
    let pieces = PIECES.iter().filter(|&&p| col.possede(monde, p)).count();
    let compte = format!("{pieces}/{}", PIECES.len());
    let p = panneau(
        commandes,
        corps,
        (&armure.nom, compte),
        &regle_armure(d, monde),
    );
    for piece in PIECES {
        let def = &armure.pieces[&piece];
        let possedee = col.possede(monde, piece);
        let icone = icones_armures::icone(d, dessins, monde, piece, |c| ic.icone(c));
        let nom_piece = d.armures.noms_pieces.get(&piece).map_or("", |n| n.as_str());
        let (nom, description) = if possedee {
            (def.nom.clone(), def.description.clone())
        } else {
            (nom_piece.to_owned(), "???".to_owned())
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
    let compter = |dans_monde: bool| {
        let objets = d.equipement.objets.iter().filter(|(_, o)| match o.monde {
            Some(m) => dans_monde && m == monde,
            None => !dans_monde,
        });
        let (n, t) = objets.fold((0, 0), |(n, t), (id, _)| {
            (n + usize::from(col.decouvert(id)), t + 1)
        });
        format!("{n}/{t}")
    };
    let regle = "Objets à trouver dans les coffres ; « ??? » tant qu'ils ne sont pas découverts.";
    let du_monde = panneau(
        commandes,
        corps,
        ("Bagues et amulettes", compter(true)),
        regle,
    );
    let communs = panneau(
        commandes,
        corps,
        ("De tous les mondes", compter(false)),
        regle,
    );
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
            ("???".to_owned(), rarete.to_owned())
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
    let retour = charte::bouton_relief(
        commandes,
        racine,
        Relief::SECONDAIRE,
        charte::H_SECONDAIRE,
        ActionMenu::Fermer,
    );
    commandes
        .entity(retour)
        .entry::<Node>()
        .and_modify(|mut n| {
            n.width = Val::Px(200.0);
            n.margin = UiRect::top(Val::Auto);
        });
    texte(commandes, retour, "Retour", IVOIRE, 2.0);
    racine
}
