//! Page Collection du menu, en « menu de console » comme le Multijoueur (`console`) : à
//! gauche les mondes en grands intitulés (avancée en %, pièces et objets trouvés) et
//! « Tous les mondes » ; à droite, sans cadre ni case, l'armure du monde ouvert (icônes
//! posées sur un halo rond, nom et effet, ou grisées tant qu'elles manquent ; provenance
//! et bonus d'ensemble), puis ses bagues et amulettes (« ??? » tant qu'elles ne sont pas
//! trouvées) ; « Tous les mondes » montre les objets communs. Ouverte sur le monde de
//! l'arène choisie.

use bevy::prelude::*;
use bs_donnees::arenes::Monde;
use bs_donnees::equipement::{Piece, Rarete};
use bs_sauvegarde::collection::{MONDES, PIECES};

use super::console::{self, Police, TERNE, aide, ecrit, intitule, surtitre};
use super::{ActionMenu, Icones, air};
use crate::dessins::Dessins;
use crate::ecrans::charte::{AIDE, IVOIRE, OR_VIF, VERT_GAIN};
use crate::icones_armures::{self, IconePiece};
use crate::lancement::DonneesJeu;
use crate::sauvegarde::Sauvegarde;

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

/// Grille à deux colonnes, sans cadre.
fn grille(c: &mut Commands, parent: Entity) -> Entity {
    c.spawn((
        Node {
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::flex(2, 1.0),
            width: Val::Percent(100.0),
            column_gap: Val::Px(24.0),
            row_gap: Val::Px(12.0),
            margin: UiRect::vertical(Val::Px(6.0)),
            flex_shrink: 0.0,
            ..default()
        },
        ChildOf(parent),
    ))
    .id()
}

/// Entrée : icône sur un halo rond (éteinte si non obtenue), nom et description.
fn entree(
    c: &mut Commands,
    ic: &Icones,
    parent: Entity,
    icone: IconePiece,
    (nom, couleur_nom, description): (&str, Color, &str),
    obtenu: bool,
) {
    let t = air::rangee(c, parent, 14.0);
    let halo = if obtenu {
        couleur_nom.with_alpha(0.14)
    } else {
        Color::srgba(1.0, 1.0, 1.0, 0.04)
    };
    let rond = air::point(c, t, 46.0, halo);
    c.entity(rond).entry::<Node>().and_modify(|mut n| {
        n.justify_content = JustifyContent::Center;
        n.align_items = AlignItems::Center;
    });
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
    let taille = Node {
        width: Val::Px(32.0),
        height: Val::Px(32.0),
        ..default()
    };
    let image = c
        .spawn((
            ImageNode::new(icone.image).with_color(teinte.into()),
            taille.clone(),
            ChildOf(rond),
        ))
        .id();
    if let Some(calque) = icone.calque {
        c.spawn((
            ImageNode::new(calque).with_color(eteint),
            Node {
                position_type: PositionType::Absolute,
                ..taille
            },
            ChildOf(image),
        ));
    }
    let textes = air::colonne(c, t, 2.0);
    c.entity(textes)
        .entry::<Node>()
        .and_modify(|mut n| n.flex_shrink = 1.0);
    let couleur = if obtenu { couleur_nom } else { TERNE };
    ecrit(c, ic, textes, nom, (Police::Grasse, 16.0, couleur));
    let couleur = if obtenu { AIDE } else { AIDE.with_alpha(0.55) };
    ecrit(c, ic, textes, description, (Police::Normale, 13.0, couleur));
}

/// Titre de section : surtitre doré à gauche, compte à droite.
fn section(c: &mut Commands, ic: &Icones, parent: Entity, (titre, compte): (&str, &str)) {
    let r = air::rangee(c, parent, 14.0);
    c.entity(r)
        .entry::<Node>()
        .and_modify(|mut n| n.margin = UiRect::top(Val::Px(10.0)));
    surtitre(c, ic, r, titre);
    ecrit(c, ic, r, compte, (Police::Logo, 18.0, OR_VIF));
}

/// Fine jauge arrondie.
fn jauge(c: &mut Commands, parent: Entity, part: f32) {
    let fond = air::point(c, parent, 0.0, Color::srgba(1.0, 1.0, 1.0, 0.08));
    c.entity(fond).entry::<Node>().and_modify(|mut n| {
        n.width = Val::Percent(100.0);
        n.height = Val::Px(4.0);
        n.margin = UiRect::vertical(Val::Px(4.0));
    });
    let plein = air::point(c, fond, 0.0, OR_VIF);
    c.entity(plein).entry::<Node>().and_modify(move |mut n| {
        n.width = Val::Percent(part * 100.0);
        n.height = Val::Percent(100.0);
    });
}

/// Règle commune des pièces d'armure du monde : quel boss lâche quoi.
fn regle_armure(d: &bs_donnees::Donnees) -> String {
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
    let mut t = "Lâchées parfois par les boss".to_owned();
    if !premier.is_empty() {
        t += &format!(" · 1er boss : {}", premier.join(", "));
    }
    if !second.is_empty() {
        t += &format!(" · 2e boss : {}", second.join(", "));
    }
    t + "."
}

/// Objets d'un monde (ou communs si `None`) dans une grille.
fn objets(
    c: &mut Commands,
    (dj, sauvegarde, dessins): (&DonneesJeu, &Sauvegarde, &Dessins),
    ic: &Icones,
    parent: Entity,
    monde: Option<Monde>,
) {
    let d = dj.d;
    let g = grille(c, parent);
    for (i, (id, o)) in d.equipement.objets.iter().enumerate() {
        if o.monde != monde {
            continue;
        }
        let trouve = sauvegarde.collection.decouvert(id);
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
        let premiere = o.description.first().map_or("", |s| s.as_str());
        let (nom, description) = if trouve {
            (o.nom.as_str(), premiere)
        } else {
            ("???", rarete)
        };
        let texte = (nom, couleur_rarete(o.rarete), description);
        entree(c, ic, g, icone, texte, trouve);
    }
}

/// Compte des objets découverts d'un monde (ou communs) : (trouvés, total).
fn compte_objets(dj: &DonneesJeu, sauvegarde: &Sauvegarde, monde: Option<Monde>) -> (usize, usize) {
    let objets =
        dj.d.equipement
            .objets
            .iter()
            .filter(|(_, o)| o.monde == monde);
    objets.fold((0, 0), |(n, t), (id, _)| {
        (n + usize::from(sauvegarde.collection.decouvert(id)), t + 1)
    })
}

/// Scène d'un monde : son armure puis ses objets.
fn scene_monde(
    c: &mut Commands,
    (dj, sauvegarde, dessins): (&DonneesJeu, &Sauvegarde, &Dessins),
    ic: &mut Icones,
    d_scene: Entity,
    monde: Monde,
) {
    let d = dj.d;
    let col = &sauvegarde.collection;
    let armure = &d.armures.armures[&monde];
    let pieces = PIECES.iter().filter(|&&p| col.possede(monde, p)).count();
    surtitre(c, ic, d_scene, "ARMURE DU MONDE");
    let r = air::rangee(c, d_scene, 16.0);
    ecrit(c, ic, r, &armure.nom, (Police::Logo, 40.0, IVOIRE));
    let t = format!("{pieces}/{}", PIECES.len());
    ecrit(c, ic, r, &t, (Police::Logo, 26.0, OR_VIF));
    jauge(c, d_scene, pieces as f32 / PIECES.len() as f32);
    aide(c, ic, d_scene, &regle_armure(d));
    let g = grille(c, d_scene);
    for piece in PIECES {
        let def = &armure.pieces[&piece];
        let possedee = col.possede(monde, piece);
        let icone = icones_armures::icone(d, dessins, monde, piece, |x| ic.icone(x));
        let nom_piece = d.armures.noms_pieces.get(&piece).map_or("", |n| n.as_str());
        let texte = if possedee {
            (def.nom.as_str(), IVOIRE, def.description.as_str())
        } else {
            (nom_piece, IVOIRE, "Pas encore trouvée")
        };
        entree(c, ic, g, icone, texte, possedee);
    }
    let complet = pieces == PIECES.len();
    let (puce, couleur) = if complet {
        ("★ ", VERT_GAIN)
    } else {
        ("", TERNE)
    };
    let t = format!("{puce}{}", armure.ensemble.description);
    ecrit(c, ic, d_scene, &t, (Police::Grasse, 14.0, couleur));

    let (n, total) = compte_objets(dj, sauvegarde, Some(monde));
    section(
        c,
        ic,
        d_scene,
        ("BAGUES ET AMULETTES", &format!("{n}/{total}")),
    );
    objets(c, (dj, sauvegarde, dessins), ic, d_scene, Some(monde));
}

/// Construit la page sur le monde `k` (`MONDES.len()` : tous les mondes) ; renvoie sa
/// racine.
pub fn construire(
    commandes: &mut Commands,
    (dj, sauvegarde, dessins): (&DonneesJeu, &Sauvegarde, &Dessins),
    ic: &mut Icones,
    k: usize,
) -> Entity {
    let c = commandes;
    let d = dj.d;
    let col = &sauvegarde.collection;
    let k = k.min(MONDES.len());
    let p = console::page(c, ic, ("collection", "COLLECTION"), (620.0, 760.0));
    let action = |i: usize| (52.0, (i != k).then_some(ActionMenu::Monde(i)));
    let mut total = 0.0;
    for (i, &m) in MONDES.iter().enumerate() {
        let a = col.avancee(d, m);
        total += a.part;
        let nom = d.armures.noms_mondes.get(&m).map_or("", |n| n.as_str());
        let t = format!(
            "{} % · {}/{} pièces · {}/{} objets",
            (a.part * 100.0).round(),
            a.pieces,
            PIECES.len(),
            a.objets,
            a.total_objets
        );
        intitule(c, ic, p.gauche, (nom, &t), action(i));
    }
    let (n, t) = compte_objets(dj, sauvegarde, None);
    let aide_tous = format!("{n}/{t} objets communs");
    let ouvrir = action(MONDES.len());
    intitule(c, ic, p.gauche, ("Tous les mondes", &aide_tous), ouvrir);

    let ressources = (dj, sauvegarde, dessins);
    match MONDES.get(k) {
        Some(&m) => scene_monde(c, ressources, ic, p.scene, m),
        None => {
            surtitre(c, ic, p.scene, "OBJETS COMMUNS");
            console::accroche(c, ic, p.scene, "Trouvables dans tous les mondes.");
            aide(
                c,
                ic,
                p.scene,
                "Dans les coffres ; « ??? » tant qu'ils ne sont pas découverts.",
            );
            jauge(c, p.scene, n as f32 / t.max(1) as f32);
            objets(c, ressources, ic, p.scene, None);
        }
    }

    let pied = console::pied(c, ic, p.racine);
    let r = air::rangee(c, pied, 10.0);
    ecrit(c, ic, r, "Collection", (Police::Normale, 14.0, AIDE));
    let part = (total / MONDES.len() as f32 * 100.0).round();
    let t = format!("{part} %");
    ecrit(c, ic, r, &t, (Police::Logo, 20.0, OR_VIF));
    p.racine
}
