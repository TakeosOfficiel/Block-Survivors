//! L'arbre choisi, à plat, sans cadre, sans anneau ni trait de liaison : le tronc en
//! bas, au centre (le talent de base, puis sa ligne) ; au-dessus, une colonne par
//! branche, rangées de gauche à droite selon les colonnes des données et séparées par
//! un large vide, où ses talents montent ligne par ligne jusqu'au final ; son nom, à sa
//! couleur, et ses points sont écrits en haut. Les talents d'une même ligne d'une
//! branche s'y étalent autour du milieu. Le seuil de points d'une ligne est écrit à
//! gauche, à sa hauteur (cadenas s'il manque). Les pastilles sont dans `case`.

use bevy::prelude::*;
use bs_donnees::progression::{Arbre, Branche, Noeud};

use super::{Contexte, case, ecrire};
use crate::ecrans::charte::{self, AIDE, IVOIRE, OR_VIF};
use crate::menu::Icones;

/// Zone de l'arbre, axe du tronc, hauteur de la ligne 0 (px).
const LARGEUR: f32 = 830.0;
const HAUTEUR: f32 = 640.0;
const CX: f32 = LARGEUR / 2.0 + 20.0;
const BAS: f32 = 590.0;
/// Pas entre lignes ; vide en plus entre le tronc (lignes 0 et 1) et les branches.
const PAS_LIGNE: f32 = 74.0;
const FOURCHE: f32 = 26.0;
/// Demi-largeur des branches, vide entre deux branches, pas entre deux talents d'une
/// ligne (au plus), écart des talents du tronc.
const DEMI_LARGEUR: f32 = 360.0;
const VIDE: f32 = 56.0;
const PAS_MAX: f32 = 76.0;
const ECART_TRONC: f32 = 66.0;
const MAX_BRANCHES: usize = 4;

/// Couleur propre à chaque branche.
pub(super) fn teinte(b: Branche) -> Color {
    match b {
        Branche::Gauche => Color::srgb(0.92, 0.48, 0.40),
        Branche::Centre => Color::srgb(0.48, 0.68, 0.98),
        Branche::Droite => Color::srgb(0.40, 0.84, 0.80),
        Branche::Libre => Color::srgb(0.78, 0.58, 0.98),
        Branche::Tronc => IVOIRE,
    }
}

/// Hauteur de la ligne `ligne`.
fn y(ligne: u32) -> f32 {
    let fourche = if ligne > 1 { FOURCHE } else { 0.0 };
    BAS - ligne as f32 * PAS_LIGNE - fourche
}

/// Point de la ligne `ligne`, à `x` de l'axe du tronc.
fn point(x: f32, ligne: u32) -> Vec2 {
    Vec2::new(CX + x, y(ligne))
}

/// Colonne d'une branche : sa largeur (le plus de talents qu'elle a sur une ligne),
/// son bord gauche et son pas (px, depuis l'axe du tronc).
#[derive(Clone, Copy)]
struct Secteur {
    branche: Branche,
    largeur: usize,
    debut: f32,
    pas: f32,
}

impl Secteur {
    fn milieu(&self) -> f32 {
        self.debut + self.largeur as f32 * self.pas / 2.0
    }
}

/// Colonnes des branches, de gauche à droite (moyenne des colonnes).
fn secteurs(def: &Arbre) -> [Option<Secteur>; MAX_BRANCHES] {
    let mut lus = [(Branche::Tronc, 0.0_f32, 0); MAX_BRANCHES];
    let mut n = 0;
    for &b in def.branches.keys().filter(|b| **b != Branche::Tronc) {
        let noeuds = || def.noeuds.iter().filter(move |x| x.branche == b);
        let (somme, nb) = noeuds().fold((0, 0), |(s, k), x| (s + x.colonne, k + 1));
        let largeur = noeuds()
            .map(|x| noeuds().filter(|y| y.ligne == x.ligne).count())
            .max()
            .unwrap_or(0);
        if nb > 0 && n < MAX_BRANCHES {
            lus[n] = (b, somme as f32 / nb as f32, largeur);
            n += 1;
        }
    }
    lus[..n].sort_by(|a, b| a.1.total_cmp(&b.1));
    let total: usize = lus[..n].iter().map(|x| x.2).sum();
    let vides = VIDE * n.saturating_sub(1) as f32;
    let pas = ((2.0 * DEMI_LARGEUR - vides) / total.max(1) as f32).min(PAS_MAX);
    let mut debut = -(total as f32 * pas + vides) / 2.0;
    let mut s = [None; MAX_BRANCHES];
    for (i, &(branche, _, largeur)) in lus[..n].iter().enumerate() {
        s[i] = Some(Secteur {
            branche,
            largeur,
            debut,
            pas,
        });
        debut += largeur as f32 * pas + VIDE;
    }
    s
}

/// Centre de la pastille de `n`, et la couleur de sa branche.
fn place(def: &Arbre, s: &[Option<Secteur>], n: &Noeud) -> Option<(Vec2, Color)> {
    if n.branche == Branche::Tronc {
        let meme = def
            .noeuds
            .iter()
            .filter(|m| m.branche == Branche::Tronc && m.ligne == n.ligne);
        let (total, avant) = meme.fold((0, 0), |(t, a), m| {
            (t + 1, a + usize::from(m.colonne < n.colonne))
        });
        let x = (avant as f32 - (total - 1) as f32 / 2.0) * ECART_TRONC;
        return Some((point(x, n.ligne), IVOIRE));
    }
    // Seul sur sa ligne dans sa branche : au milieu de la colonne ; sinon étalés
    let sec = s.iter().flatten().find(|x| x.branche == n.branche)?;
    let meme = def
        .noeuds
        .iter()
        .filter(|m| m.branche == n.branche && m.ligne == n.ligne);
    let (total, avant) = meme.fold((0, 0), |(t, a), m| {
        (t + 1, a + usize::from(m.colonne < n.colonne))
    });
    let x = sec.milieu() + (avant as f32 - (total - 1) as f32 / 2.0) * sec.pas;
    Some((point(x, n.ligne), teinte(n.branche)))
}

pub fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, parent: Entity) {
    let def = c.arbre();
    let s = secteurs(def);
    let zone = commandes
        .spawn((
            Node {
                width: Val::Px(LARGEUR),
                height: Val::Px(HAUTEUR),
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    let lignes = def.noeuds.iter().map(|n| n.ligne).max().unwrap_or(0);
    for l in 0..=lignes {
        seuil(commandes, c, ic, zone, l);
    }
    for sec in s.iter().flatten() {
        bout(commandes, c, ic, zone, sec);
    }
    for (k, n) in def.noeuds.iter().enumerate() {
        if let Some(place) = place(def, &s, n) {
            case::construire(commandes, c, ic, zone, (k, n), place);
        }
    }
}

/// Seuil de points qui ouvre la ligne `l`, écrit à gauche, à sa hauteur.
fn seuil(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, zone: Entity, l: u32) {
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
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                top: Val::Px(y(l) - 10.0),
                height: Val::Px(20.0),
                align_items: AlignItems::Center,
                column_gap: Val::Px(5.0),
                ..default()
            },
            ChildOf(zone),
        ))
        .id();
    let couleur = if ouverte {
        AIDE.with_alpha(0.5)
    } else {
        OR_VIF
    };
    if !ouverte {
        charte::cadenas(commandes, marque, 8.0, couleur);
    }
    let s = if palier.rangs > 1 { "s" } else { "" };
    let t = format!("{} pt{s}", palier.rangs);
    ecrire(commandes, ic, marque, &t, (true, 13.0, couleur));
}

/// Nom de la branche et points qui y sont, au-dessus de son talent le plus haut.
fn bout(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, zone: Entity, sec: &Secteur) {
    let def = c.arbre();
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let noeuds = def.noeuds.iter().filter(|n| n.branche == sec.branche);
    let (haut, points) = noeuds.fold((0, 0), |(h, s), n| (h.max(n.ligne), s + prog.rang(p, a, n)));
    let o = point(sec.milieu(), haut) - Vec2::new(0.0, case::CASE / 2.0 + 24.0);
    let x = o.x.clamp(70.0, LARGEUR - 70.0);
    let boite = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(x - 80.0),
                top: Val::Px((o.y - 20.0).max(0.0)),
                width: Val::Px(160.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            ChildOf(zone),
        ))
        .id();
    let nom = def.branches.get(&sec.branche).map_or("", String::as_str);
    let e = ecrire(commandes, ic, boite, nom, (true, 18.0, teinte(sec.branche)));
    commandes.entity(e).insert(TextShadow::default());
    let s = match points {
        0 => "aucun point".to_owned(),
        1 => "1 point".to_owned(),
        n => format!("{n} points"),
    };
    let couleur = if points > 0 { OR_VIF } else { AIDE };
    ecrire(
        commandes,
        ic,
        boite,
        &s,
        (false, charte::T_AIDE - 2.0, couleur),
    );
}
