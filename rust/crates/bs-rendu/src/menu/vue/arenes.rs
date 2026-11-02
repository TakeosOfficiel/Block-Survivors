//! Colonne des arènes de l'accueil (src/ui/ecranMenu.ts `afficherArenes`) : une vignette
//! par arène (sa photo recadrée, son nom, voilée et grisée si elle est verrouillée),
//! puis les détails de la choisie : description, mobs, boss à venir et records.

use bevy::prelude::*;
use bs_donnees::arenes::Arene;
use bs_donnees::progression::TypeDeblocage;

use super::{BLANC, Pinceau, place};
use crate::ecrans::style::{BoutonPage, OR, OR_CLAIR, Teinte};
use crate::hud::formater_temps;
use crate::lancement::{ChoixPartie, DonneesJeu};
use crate::menu::ActionMenu;
use crate::menu::fond::Couvrir;
use crate::sauvegarde::Sauvegarde;

const BORD: Color = Color::srgba(1.0, 1.0, 1.0, 0.12);
const VERROU: Color = Color::srgb_u8(0xff, 0xb0, 0xa0);
const DESCRIPTION: Color = Color::srgb_u8(0xd0, 0xd0, 0xd0);
const MOBS: Color = Color::srgb_u8(0x8f, 0xe0, 0x8f);
const TEXTE_BOSS: Color = Color::srgb_u8(0xe8, 0xd8, 0xff);
const FOND_BOSS: Color = Color::srgba(110.0 / 255.0, 60.0 / 255.0, 160.0 / 255.0, 0.55);

pub(super) fn construire(
    p: &mut Pinceau,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    choix: ChoixPartie,
    parent: Entity,
) {
    p.section(parent, "Arène");
    let liste = p.noeud(
        parent,
        Node {
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            row_gap: Val::Px(10.0),
            ..default()
        },
    );
    for (k, a) in dj.arenes.iter().enumerate() {
        vignette(p, sauvegarde, a.def, k == choix.arene, k, liste);
    }
    details(p, dj, sauvegarde, dj.arenes[choix.arene].def, parent);
}

/// Vignette d'une arène : photo, voile vers le bas, nom (et verrou).
fn vignette(
    p: &mut Pinceau,
    sauvegarde: &Sauvegarde,
    a: &Arene,
    choisie: bool,
    k: usize,
    parent: Entity,
) {
    let verrou = sauvegarde.progression.verrou(TypeDeblocage::Arene, &a.id);
    let teinte = Teinte {
        bord: if choisie { OR } else { BORD },
        fond: Color::NONE,
    };
    let mut v = p.c.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(84.0),
            min_height: Val::Px(84.0),
            flex_shrink: 0.0,
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(10.0)),
            overflow: Overflow::clip(),
            ..default()
        },
        teinte,
        BorderColor::all(teinte.bord),
        BackgroundColor(Color::srgb_u8(0x1a, 0x1d, 0x26)),
        ChildOf(parent),
    ));
    if verrou.is_none() {
        v.insert((Button, BoutonPage, ActionMenu::Arene(k)));
    }
    let v = v.id();
    let tout = || Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..place(Some(Val::ZERO), None, Some(Val::ZERO), None)
    };
    if let Some(image) = p.ic.photos.decodee(&a.id) {
        let gris = if verrou.is_some() { 0.3 } else { 1.0 };
        p.c.spawn((
            ImageNode::new(image).with_color(Color::srgb(gris, gris, gris)),
            Couvrir,
            tout(),
            ChildOf(v),
        ));
    }
    let voile = LinearGradient::to_bottom(vec![
        ColorStop::new(Color::NONE, Val::Percent(30.0)),
        ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.85), Val::Percent(100.0)),
    ]);
    p.c.spawn((tout(), BackgroundGradient::from(voile), ChildOf(v)));
    let bas = Some(Val::Px(10.0));
    let gauche = Some(Val::Px(12.0));
    match verrou {
        None => {
            let n = p.noeud(v, place(gauche, None, None, bas));
            p.texte(n, &a.nom, true, 16.0, BLANC);
        }
        Some(verrou) => {
            let n = p.noeud(v, place(gauche, None, Some(Val::Px(8.0)), None));
            p.texte(n, &a.nom, true, 16.0, Color::srgb_u8(0xa0, 0xa0, 0xa0));
            let n = p.noeud(v, place(gauche, Some(Val::Px(12.0)), None, bas));
            p.texte(
                n,
                &format!("Verrouillée · {}", verrou.texte),
                false,
                13.0,
                VERROU,
            );
        }
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
            margin: UiRect::top(Val::Px(12.0)),
            ..default()
        },
    );
    p.texte(col, &a.description, false, 13.0, DESCRIPTION);
    p.texte(
        col,
        &format!("Mobs : {}", noms_mobs(d, a)),
        false,
        13.0,
        MOBS,
    );
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
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
        );
        p.c.entity(pastille).insert(BackgroundColor(FOND_BOSS));
        p.texte(pastille, &b, false, 12.0, TEXTE_BOSS);
    }
    let prog = &sauvegarde.progression;
    let record = prog.record(&a.id);
    if record > 0.0 {
        let t = format!("Record : {}", formater_temps(record as i64));
        p.texte(col, &t, true, 13.0, OR_CLAIR);
    }
    let sans_fin = prog.record_sans_fin(&a.id);
    if sans_fin > 0.0 {
        let t = format!("Record sans fin : {}", formater_temps(sans_fin as i64));
        p.texte(col, &t, true, 13.0, OR_CLAIR);
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
