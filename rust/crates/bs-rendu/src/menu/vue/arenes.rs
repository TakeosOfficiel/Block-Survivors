//! Volet des arènes de l'accueil : à gauche les arènes dans l'ordre, en lignes de 52
//! (miniature, nom, ou cadenas et condition de déblocage), la choisie marquée d'un
//! liseré d'or ; à droite la choisie dépliée (grande vignette, description, mobs, boss à
//! venir, records) et le bouton de la Collection (classée par monde). Sous la liste :
//! comment ajouter une map, et le résultat du dernier dépôt.

use bevy::prelude::*;
use bs_donnees::arenes::Arene;
use bs_donnees::progression::TypeDeblocage;

use super::{Pinceau, place};
use crate::ecrans::charte::{
    self, AIDE, IVOIRE, OR_VIF, ROUGE, Relief, T_AIDE, T_CORPS, VERT_GAIN,
};
use crate::hud::formater_temps;
use crate::lancement::{ChoixPartie, DonneesJeu};
use crate::menu::ActionMenu;
use crate::menu::fond::Couvrir;
use crate::sauvegarde::Sauvegarde;

const TEXTE_BOSS: Color = Color::srgb_u8(0xe8, 0xd8, 0xff);
const FOND_BOSS: Color = Color::srgba(110.0 / 255.0, 60.0 / 255.0, 160.0 / 255.0, 0.55);

pub(super) fn construire(
    p: &mut Pinceau,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    (choix, message): (ChoixPartie, Option<&str>),
    (parent, detail): (Entity, Entity),
) {
    p.section(parent, "Arène");
    let liste = p.noeud(
        parent,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        },
    );
    for (k, a) in dj.arenes.iter().enumerate() {
        ligne(p, sauvegarde, a.def, (k, k == choix.arene), liste);
    }
    let aide = "Ajouter une map : glisse un fichier .schem ou .nbt sur la fenêtre.";
    p.texte(parent, aide, false, T_AIDE - 1.0, AIDE);
    if let Some(m) = message {
        let couleur = if m.starts_with("Map refusée") {
            ROUGE
        } else {
            VERT_GAIN
        };
        p.texte(parent, m, true, T_AIDE, couleur);
    }
    choisie(p, dj, sauvegarde, dj.arenes[choix.arene].def, detail);
    let b = p.bouton(
        detail,
        "Collection",
        Relief::SECONDAIRE,
        ActionMenu::Collection,
    );
    p.c.entity(b).entry::<Node>().and_modify(|mut n| {
        n.margin = UiRect::top(Val::Px(16.0));
    });
}

/// Photo de l'arène couvrant son parent (grisée si `gris`).
pub(super) fn photo(p: &mut Pinceau, a: &Arene, gris: bool, parent: Entity) {
    if let Some(image) = p.ic.photos.decodee(&a.id) {
        let g = if gris { 0.3 } else { 1.0 };
        p.c.spawn((
            ImageNode::new(image).with_color(Color::srgb(g, g, g)),
            Couvrir,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..place(Some(Val::ZERO), None, Some(Val::ZERO), None)
            },
            ChildOf(parent),
        ));
    }
}

/// Arène choisie : grande vignette (photo, voile, nom) et ses détails.
fn choisie(p: &mut Pinceau, dj: &DonneesJeu, sauvegarde: &Sauvegarde, a: &Arene, parent: Entity) {
    let bloc = p.noeud(
        parent,
        Node {
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(2.0)),
            ..default()
        },
    );
    p.c.entity(bloc)
        .insert((BackgroundColor(charte::CHOISI), BorderColor::all(OR_VIF)));
    let v = p.noeud(
        bloc,
        Node {
            height: Val::Px(150.0),
            flex_shrink: 0.0,
            overflow: Overflow::clip(),
            ..default()
        },
    );
    photo(p, a, false, v);
    let voile = LinearGradient::to_bottom(vec![
        ColorStop::new(Color::NONE, Val::Percent(35.0)),
        ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.85), Val::Percent(100.0)),
    ]);
    let tout = Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..place(Some(Val::ZERO), None, Some(Val::ZERO), None)
    };
    p.c.spawn((tout, BackgroundGradient::from(voile), ChildOf(v)));
    let bas = Some(Val::Px(8.0));
    let n = p.noeud(
        v,
        place(Some(Val::Px(12.0)), Some(Val::Px(12.0)), None, bas),
    );
    p.texte(n, &a.nom, true, charte::T_SECTION, IVOIRE);
    details(p, dj, sauvegarde, a, bloc);
}

/// Ligne d'une arène : miniature, nom, ou cadenas et condition si verrouillée ; la
/// choisie avec liseré et coche.
fn ligne(
    p: &mut Pinceau,
    sauvegarde: &Sauvegarde,
    a: &Arene,
    (k, choisie): (usize, bool),
    parent: Entity,
) {
    let relief = if choisie {
        Relief::CHOISI
    } else {
        Relief::SECONDAIRE
    };
    let verrou = sauvegarde.progression.verrou(TypeDeblocage::Arene, &a.id);
    let mut l = p.c.spawn((
        Node {
            min_height: Val::Px(52.0),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
            border: charte::bord_relief(),
            border_radius: BorderRadius::all(Val::Px(2.0)),
            ..default()
        },
        relief.composants(),
        ChildOf(parent),
    ));
    if verrou.is_none() {
        l.insert((Button, ActionMenu::Arene(k)));
    }
    let l = l.id();
    if choisie {
        super::classes::lisere(p, l);
    }
    let mini = p.noeud(
        l,
        Node {
            width: Val::Px(64.0),
            height: Val::Px(36.0),
            flex_shrink: 0.0,
            overflow: Overflow::clip(),
            ..default()
        },
    );
    photo(p, a, verrou.is_some(), mini);
    let textes = p.noeud(
        l,
        Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            flex_shrink: 1.0,
            row_gap: Val::Px(1.0),
            ..default()
        },
    );
    p.texte(textes, &a.nom, true, T_CORPS, IVOIRE);
    if let Some(v) = verrou {
        p.texte(textes, &v.texte, false, T_AIDE - 1.0, AIDE);
        charte::cadenas(p.c, l, 14.0, AIDE);
    } else if choisie {
        p.texte(l, "✓", true, 20.0, OR_VIF);
    }
}

/// Description, mobs, boss à venir et records de l'arène choisie.
fn details(p: &mut Pinceau, dj: &DonneesJeu, sauvegarde: &Sauvegarde, a: &Arene, parent: Entity) {
    let d = dj.d;
    let col = p.noeud(
        parent,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
    );
    p.texte(col, &a.description, false, T_AIDE, AIDE);
    let mobs = p.noeud(
        col,
        Node {
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(6.0),
            ..default()
        },
    );
    p.texte(mobs, "Mobs", true, T_AIDE, IVOIRE);
    p.texte(mobs, &noms_mobs(d, a), false, T_AIDE, VERT_GAIN);
    let boss = p.noeud(
        col,
        Node {
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(4.0),
            row_gap: Val::Px(4.0),
            ..default()
        },
    );
    for b in prochains_boss(d, a) {
        let pastille = p.noeud(
            boss,
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
        );
        p.c.entity(pastille).insert(BackgroundColor(FOND_BOSS));
        p.texte(pastille, &b, false, T_AIDE - 1.0, TEXTE_BOSS);
    }
    let prog = &sauvegarde.progression;
    let record = prog.record(&a.id);
    if record > 0.0 {
        let t = format!("Record : {}", formater_temps(record as i64));
        p.texte(col, &t, true, T_AIDE, OR_VIF);
    }
    let sans_fin = prog.record_sans_fin(&a.id);
    if sans_fin > 0.0 {
        let t = format!("Record sans fin : {}", formater_temps(sans_fin as i64));
        p.texte(col, &t, true, T_AIDE, OR_VIF);
    }
}

/// Noms des mobs de l'arène (toutes zones), dans l'ordre d'arrivée.
fn noms_mobs(d: &bs_donnees::Donnees, a: &Arene) -> String {
    let mut tous: Vec<_> = a
        .ennemis
        .iter()
        .chain(a.zones.iter().flat_map(|z| &z.ennemis))
        .collect();
    tous.sort_by(|x, y| x.depuis.total_cmp(&y.depuis));
    let mut noms: Vec<String> = Vec::new();
    for e in tous {
        let nom = d
            .ennemis
            .get(&e.ennemi)
            .map_or(e.ennemi.clone(), |x| x.nom.to_lowercase());
        if !noms.contains(&nom) {
            noms.push(nom);
        }
    }
    noms.join(", ")
}

/// « 5:00 · Géant », « 10:00 · Seigneur wither »… avant la fin de la partie.
fn prochains_boss(d: &bs_donnees::Donnees, a: &Arene) -> Vec<String> {
    let c = &d.regles.boss;
    let duree = d.regles.progression.duree_partie;
    let mut liste = Vec::new();
    if a.boss.is_empty() || c.intervalle <= 0.0 {
        return liste;
    }
    let mut t = c.premier_passage;
    let mut i = 0;
    while t < duree {
        let id = &a.boss[i % a.boss.len()];
        let nom = d.ennemis.get(id).map_or(id.as_str(), |e| e.nom.as_str());
        liste.push(format!("{} · {nom}", formater_temps(t as i64)));
        t += c.intervalle;
        i += 1;
    }
    liste
}
