//! L'arbre choisi, en branches : le tronc en bas, au centre (talent de base puis sa
//! ligne), d'où part une fourche vers une colonne par branche (Puissance,
//! Enchaînements…). Chaque colonne a sa couleur, son nom et les points qu'on y a mis ;
//! ses talents montent ligne par ligne jusqu'au final. Les seuils de points des lignes
//! sont écrits à gauche, cadenas tant qu'ils manquent.
//! Les cases des talents sont dans `case`.

use bevy::prelude::*;
use bs_donnees::progression::Branche;

use super::case::CASE;
use super::{Contexte, case, ecrire};
use crate::ecrans::charte::{self, AIDE, BORD, IVOIRE, OR_SOURD, OR_VIF};
use crate::menu::Icones;

/// Pas entre lignes et entre sous-colonnes (px).
const PAS_LIGNE: f32 = 80.0;
const PAS_COLONNE: f32 = 80.0;
/// Écart entre colonnes de branche, bandeau du nom, marge gauche des seuils.
const ECART: f32 = 16.0;
/// Largeur minimale d'une colonne de branche (place du nom).
const MIN_COLONNE: f32 = 140.0;
const ENTETE: f32 = 40.0;
const SEUILS: f32 = 56.0;
/// Hauteur de la fourche entre le tronc et les branches.
const FOURCHE: f32 = 34.0;
/// Branches au plus (gauche, centre, droite, libre) et sous-colonnes par branche.
const MAX_BRANCHES: usize = 4;
const MAX_SOUS: usize = 4;

/// Couleur propre à chaque branche.
fn teinte(b: Branche) -> Color {
    match b {
        Branche::Gauche => Color::srgb(0.88, 0.42, 0.36),
        Branche::Centre => Color::srgb(0.42, 0.64, 0.96),
        Branche::Droite => Color::srgb(0.36, 0.80, 0.80),
        Branche::Libre => Color::srgb(0.74, 0.52, 0.95),
        Branche::Tronc => IVOIRE,
    }
}

fn absolu(x: f32, y: f32, l: f32, h: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(x),
        top: Val::Px(y),
        width: Val::Px(l),
        height: Val::Px(h),
        ..default()
    }
}

/// Trait plein (ligne de liaison).
fn trait_(commandes: &mut Commands, zone: Entity, (x, y, l, h): (f32, f32, f32, f32), c: Color) {
    commandes.spawn((absolu(x, y, l, h), BackgroundColor(c), ChildOf(zone)));
}

/// Une colonne de branche : sa branche, ses colonnes de données (triées), sa place.
#[derive(Clone, Copy)]
struct Colonne {
    branche: Branche,
    sous: [u32; MAX_SOUS],
    n_sous: usize,
    x: f32,
    largeur: f32,
}

impl Colonne {
    fn centre_sous(&self, colonne: u32) -> f32 {
        let i = self.sous[..self.n_sous]
            .iter()
            .position(|&c| c == colonne)
            .unwrap_or(0);
        let contenu = (self.n_sous - 1) as f32 * PAS_COLONNE;
        self.x + (self.largeur - contenu) / 2.0 + i as f32 * PAS_COLONNE
    }
}

pub fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, parent: Entity) {
    let def = c.arbre();
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let lignes = def.noeuds.iter().map(|n| n.ligne).max().unwrap_or(0);
    let haut_tronc = def
        .noeuds
        .iter()
        .filter(|n| n.branche == Branche::Tronc)
        .map(|n| n.ligne)
        .max()
        .unwrap_or(0);
    // Colonnes de branche, dans l'ordre des données
    let mut cols = [None::<Colonne>; MAX_BRANCHES];
    let mut n_cols = 0;
    let mut x = SEUILS;
    for &b in def.branches.keys().filter(|b| **b != Branche::Tronc) {
        let mut sous = [0; MAX_SOUS];
        let mut n_sous = 0;
        for n in def.noeuds.iter().filter(|n| n.branche == b) {
            if n_sous < MAX_SOUS && !sous[..n_sous].contains(&n.colonne) {
                sous[n_sous] = n.colonne;
                n_sous += 1;
            }
        }
        if n_sous == 0 || n_cols == MAX_BRANCHES {
            continue;
        }
        sous[..n_sous].sort_unstable();
        let largeur = (2.0 * ECART + CASE + (n_sous - 1) as f32 * PAS_COLONNE).max(MIN_COLONNE);
        cols[n_cols] = Some(Colonne {
            branche: b,
            sous,
            n_sous,
            x,
            largeur,
        });
        n_cols += 1;
        x += largeur + ECART;
    }
    let largeur = x.max(SEUILS + 3.0 * PAS_COLONNE);
    // Lignes des branches de haut en bas, puis la fourche, puis le tronc
    let rangs_branches = lignes.saturating_sub(haut_tronc) as f32;
    let bas_branches = ENTETE + rangs_branches * PAS_LIGNE + 8.0;
    let hauteur = bas_branches + FOURCHE + (haut_tronc + 1) as f32 * PAS_LIGNE;
    let y_ligne = |l: u32| {
        if l > haut_tronc {
            ENTETE + (lignes - l) as f32 * PAS_LIGNE + PAS_LIGNE / 2.0
        } else {
            bas_branches + FOURCHE + (haut_tronc - l) as f32 * PAS_LIGNE + PAS_LIGNE / 2.0
        }
    };
    let zone = commandes
        .spawn((
            Node {
                width: Val::Px(largeur),
                height: Val::Px(hauteur),
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    let milieu = SEUILS + (largeur - SEUILS - ECART) / 2.0;
    // Fourche : du haut du tronc à chaque colonne
    let y_fourche = bas_branches + FOURCHE / 2.0;
    let pale = Color::srgba(1.0, 1.0, 1.0, 0.22);
    trait_(
        commandes,
        zone,
        (milieu - 1.0, y_fourche, 2.0, FOURCHE / 2.0),
        pale,
    );
    let centres: Vec<f32> = cols[..n_cols]
        .iter()
        .flatten()
        .map(|k| k.x + k.largeur / 2.0)
        .collect();
    if let (Some(g), Some(d)) = (centres.first(), centres.last()) {
        let (g, d) = (g.min(milieu), d.max(milieu));
        trait_(
            commandes,
            zone,
            (g - 1.0, y_fourche - 1.0, d - g + 2.0, 2.0),
            pale,
        );
    }
    for k in cols[..n_cols].iter().flatten() {
        let points: u32 = def
            .noeuds
            .iter()
            .filter(|n| n.branche == k.branche)
            .map(|n| prog.rang(p, a, n))
            .sum();
        colonne(commandes, c, ic, zone, k, points, bas_branches);
        let cx = k.x + k.largeur / 2.0;
        let t = teinte(k.branche).with_alpha(if points > 0 { 0.7 } else { 0.3 });
        trait_(
            commandes,
            zone,
            (cx - 1.0, bas_branches, 2.0, y_fourche - bas_branches),
            t,
        );
    }
    // Liaisons verticales dans chaque sous-colonne (et dans le tronc)
    for k in cols[..n_cols].iter().flatten() {
        for &sc in &k.sous[..k.n_sous] {
            let ls = def
                .noeuds
                .iter()
                .filter(|n| n.branche == k.branche && n.colonne == sc)
                .map(|n| n.ligne);
            let (min, max) = ls.fold((u32::MAX, 0), |(a, b), l| (a.min(l), b.max(l)));
            if min < max {
                let x = k.centre_sous(sc);
                let (y0, y1) = (y_ligne(max), y_ligne(min));
                let t = teinte(k.branche).with_alpha(0.45);
                trait_(commandes, zone, (x - 1.0, y0, 2.0, y1 - y0), t);
            }
        }
    }
    trait_(
        commandes,
        zone,
        (
            milieu - 1.0,
            y_ligne(haut_tronc),
            2.0,
            y_ligne(0) - y_ligne(haut_tronc),
        ),
        pale,
    );
    if haut_tronc > 0 {
        let y = y_ligne(haut_tronc);
        trait_(
            commandes,
            zone,
            (milieu - PAS_COLONNE / 2.0, y - 1.0, PAS_COLONNE, 2.0),
            pale,
        );
    }
    // Seuils de points, à gauche
    for l in 0..=lignes {
        seuil(commandes, c, ic, zone, l, y_ligne(l));
    }
    // Cases
    for (k, n) in def.noeuds.iter().enumerate() {
        let x = if n.branche == Branche::Tronc {
            let meme: Vec<u32> = def
                .noeuds
                .iter()
                .filter(|m| m.branche == Branche::Tronc && m.ligne == n.ligne)
                .map(|m| m.colonne)
                .collect();
            let i = meme.iter().filter(|&&m| m < n.colonne).count() as f32;
            milieu + (i - (meme.len() - 1) as f32 / 2.0) * PAS_COLONNE
        } else {
            match cols[..n_cols]
                .iter()
                .flatten()
                .find(|k| k.branche == n.branche)
            {
                Some(k) => k.centre_sous(n.colonne),
                None => continue,
            }
        };
        case::construire(
            commandes,
            c,
            ic,
            zone,
            (k, n),
            Vec2::new(x, y_ligne(n.ligne)),
        );
    }
}

/// Fond, bandeau coloré et nom d'une colonne de branche, avec les points qui y sont.
fn colonne(
    commandes: &mut Commands,
    c: &Contexte,
    ic: &mut Icones,
    zone: Entity,
    k: &Colonne,
    points: u32,
    bas: f32,
) {
    let t = teinte(k.branche);
    let nom = c
        .arbre()
        .branches
        .get(&k.branche)
        .map_or("", String::as_str);
    let fond = commandes
        .spawn((
            Node {
                border: UiRect {
                    top: Val::Px(3.0),
                    ..UiRect::all(Val::Px(1.0))
                },
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..absolu(k.x, 0.0, k.largeur, bas)
            },
            BackgroundColor(t.with_alpha(if points > 0 { 0.09 } else { 0.05 })),
            BorderColor {
                top: t,
                ..BorderColor::all(t.with_alpha(0.3))
            },
            ChildOf(zone),
        ))
        .id();
    let entete = commandes
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(ENTETE - 4.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },
            ChildOf(fond),
        ))
        .id();
    ecrire(commandes, ic, entete, nom, (true, 18.0, t));
    let s = points.to_string();
    ecrire(
        commandes,
        ic,
        entete,
        &s,
        (true, 18.0, if points > 0 { OR_VIF } else { AIDE }),
    );
}

/// Seuil de points de la ligne `l`, en pastille à gauche (cadenas s'il manque).
fn seuil(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, zone: Entity, l: u32, y: f32) {
    let palier = c.dj.d.talents.paliers.iter().find(|x| x.ligne == l);
    let Some(palier) = palier.filter(|x| x.rangs > 0) else {
        return;
    };
    let ouverte = c
        .sauvegarde
        .progression
        .palier_atteint(c.id(), c.etat.arbre, l);
    let marque = commandes
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(4.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..absolu(0.0, y - 12.0, SEUILS - 12.0, 24.0)
            },
            BackgroundColor(charte::PANNEAU),
            BorderColor::all(if ouverte { BORD } else { OR_SOURD }),
            ChildOf(zone),
        ))
        .id();
    let couleur = if ouverte { AIDE } else { OR_VIF };
    if !ouverte {
        charte::cadenas(commandes, marque, 9.0, couleur);
    }
    let t = palier.rangs.to_string();
    ecrire(commandes, ic, marque, &t, (true, 13.0, couleur));
}
