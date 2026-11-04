//! L'arbre choisi en radar, sans cadre ni trait de liaison : le talent de base en bas,
//! au centre ; chaque ligne de l'arbre est un anneau pâle plus grand ; le tronc tient
//! le premier anneau, puis chaque branche a son secteur (rangés de gauche à droite
//! selon les colonnes des données, séparés par un vide) où ses talents montent en
//! rayon jusqu'au final ; son nom, à sa couleur, et ses points sont écrits au bout.
//! Les talents d'une même ligne d'une branche s'y étalent autour du milieu.
//! Le seuil de points d'une ligne est écrit sous l'anneau, à gauche (cadenas s'il
//! manque). Les pastilles des talents sont dans `case`.

use bevy::prelude::*;
use bs_donnees::progression::{Arbre, Branche, Noeud};

use super::{Contexte, case, ecrire};
use crate::ecrans::charte::{self, AIDE, IVOIRE, OR_VIF};
use crate::menu::Icones;

/// Zone du radar et centre des anneaux (px).
const LARGEUR: f32 = 830.0;
const HAUTEUR: f32 = 640.0;
const CENTRE: Vec2 = Vec2::new(LARGEUR / 2.0, 585.0);
/// Rayon du premier anneau, pas entre anneaux.
const RAYON_1: f32 = 90.0;
const PAS_RAYON: f32 = 62.0;
/// Demi-ouverture du radar, vide entre secteurs, écart des talents du tronc (degrés).
const DEMI_OUVERTURE: f32 = 66.0;
const VIDE: f32 = 8.0;
const ECART_TRONC: f32 = 36.0;
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

fn rayon(ligne: u32) -> f32 {
    if ligne == 0 {
        0.0
    } else {
        RAYON_1 + (ligne - 1) as f32 * PAS_RAYON
    }
}

/// Point du radar à l'angle `degres` (0 = vers le haut, positif vers la droite).
fn point(degres: f32, r: f32) -> Vec2 {
    let a = degres.to_radians();
    CENTRE + r * Vec2::new(a.sin(), -a.cos())
}

/// Secteur d'une branche : sa largeur (le plus de talents qu'elle a sur une ligne)
/// et ses angles.
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

/// Secteurs des branches, de gauche à droite (moyenne des colonnes).
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
    let libre = 2.0 * DEMI_OUVERTURE - VIDE * n.saturating_sub(1) as f32;
    let pas = libre / total.max(1) as f32;
    let mut debut = -DEMI_OUVERTURE;
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
        let angle = (avant as f32 - (total - 1) as f32 / 2.0) * ECART_TRONC;
        return Some((point(angle, rayon(n.ligne)), IVOIRE));
    }
    // Seul sur sa ligne dans sa branche : au milieu du secteur ; sinon étalés
    let sec = s.iter().flatten().find(|x| x.branche == n.branche)?;
    let meme = def
        .noeuds
        .iter()
        .filter(|m| m.branche == n.branche && m.ligne == n.ligne);
    let (total, avant) = meme.fold((0, 0), |(t, a), m| {
        (t + 1, a + usize::from(m.colonne < n.colonne))
    });
    let angle = sec.milieu() + (avant as f32 - (total - 1) as f32 / 2.0) * sec.pas;
    Some((point(angle, rayon(n.ligne)), teinte(n.branche)))
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
                overflow: Overflow::clip(),
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    let lignes = def.noeuds.iter().map(|n| n.ligne).max().unwrap_or(0);
    for l in 1..=lignes {
        anneau(commandes, c, ic, zone, l);
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

/// Anneau pâle de la ligne `l`, et le seuil de points qui l'ouvre, sous son bord gauche.
fn anneau(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, zone: Entity, l: u32) {
    let prog = &c.sauvegarde.progression;
    let ouverte = prog.palier_atteint(c.id(), c.etat.arbre, l);
    let r = rayon(l);
    commandes.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(CENTRE.x - r),
            top: Val::Px(CENTRE.y - r),
            width: Val::Px(2.0 * r),
            height: Val::Px(2.0 * r),
            border: UiRect::all(Val::Px(1.5)),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..default()
        },
        BorderColor::all(Color::srgba(
            0.7,
            0.8,
            1.0,
            if ouverte { 0.13 } else { 0.06 },
        )),
        ChildOf(zone),
    ));
    let palier = c.dj.d.talents.paliers.iter().find(|x| x.ligne == l);
    let Some(palier) = palier.filter(|x| x.rangs > 0) else {
        return;
    };
    let marque = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(CENTRE.x - r - 30.0),
                top: Val::Px(CENTRE.y + 12.0),
                width: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(4.0),
                ..default()
            },
            ChildOf(zone),
        ))
        .id();
    let couleur = if ouverte {
        AIDE.with_alpha(0.55)
    } else {
        OR_VIF
    };
    if !ouverte {
        charte::cadenas(commandes, marque, 8.0, couleur);
    }
    let t = palier.rangs.to_string();
    ecrire(commandes, ic, marque, &t, (true, 13.0, couleur));
}

/// Nom de la branche et points qui y sont, au bout de son secteur.
fn bout(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, zone: Entity, sec: &Secteur) {
    let def = c.arbre();
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let noeuds = def.noeuds.iter().filter(|n| n.branche == sec.branche);
    let (haut, points) = noeuds.fold((0, 0), |(h, s), n| (h.max(n.ligne), s + prog.rang(p, a, n)));
    let o = point(sec.milieu(), rayon(haut)) - Vec2::new(0.0, case::CASE / 2.0 + 24.0);
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
