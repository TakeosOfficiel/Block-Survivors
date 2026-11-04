//! L'arbre choisi, à l'air libre (aucun cadre) : le tronc en bas, au centre, puis les
//! branches qui s'en écartent en éventail, chacune dans sa direction, sa couleur et
//! reliée par ses propres traits jusqu'à son talent final ; son nom et les points qui
//! y sont écrits au bout. Les branches sont rangées de gauche à droite selon les
//! colonnes des données. Les seuils de points des lignes sont écrits à gauche, cadenas
//! tant qu'ils manquent. Les cases des talents sont dans `case`.

use bevy::prelude::*;
use bs_donnees::progression::{Arbre, Branche, Noeud};

use super::{Contexte, case, ecrire};
use crate::ecrans::charte::{self, AIDE, IVOIRE, OR_VIF};
use crate::menu::Icones;

/// Zone de l'arbre, centre du tronc, bas de la ligne 0 (px).
const LARGEUR: f32 = 790.0;
const HAUTEUR: f32 = 650.0;
const CX: f32 = 430.0;
const BAS: f32 = 600.0;
/// Pas entre lignes ; écart en plus entre le tronc et les branches.
const PAS_LIGNE: f32 = 80.0;
const FOURCHE: f32 = 24.0;
/// Écart des branches au centre : à leur première ligne, puis en plus à chaque ligne.
const ECART_BASE: f32 = 130.0;
const ECART_LIGNE: f32 = 20.0;
/// Pas entre deux talents d'une même ligne (tronc, ou branche à plusieurs colonnes).
const PAS_COLONNE: f32 = 72.0;
const MAX_BRANCHES: usize = 4;
const MAX_SOUS: usize = 4;

/// Couleur propre à chaque branche.
fn teinte(b: Branche) -> Color {
    match b {
        Branche::Gauche => Color::srgb(0.90, 0.45, 0.38),
        Branche::Centre => Color::srgb(0.45, 0.66, 0.98),
        Branche::Droite => Color::srgb(0.38, 0.82, 0.80),
        Branche::Libre => Color::srgb(0.76, 0.55, 0.97),
        Branche::Tronc => IVOIRE,
    }
}

/// Une branche : sa direction (−1,5 à 1,5 de la gauche à la droite) et ses colonnes
/// de données distinctes, triées.
#[derive(Clone, Copy)]
struct Voie {
    branche: Branche,
    direction: f32,
    sous: [u32; MAX_SOUS],
    n_sous: usize,
}

/// Plan de l'arbre : les branches et la première ligne au-dessus du tronc.
struct Plan {
    voies: [Option<Voie>; MAX_BRANCHES],
    debut: u32,
}

impl Plan {
    fn nouveau(def: &Arbre) -> Self {
        let haut_tronc = def
            .noeuds
            .iter()
            .filter(|n| n.branche == Branche::Tronc)
            .map(|n| n.ligne)
            .max()
            .unwrap_or(0);
        // (branche, moyenne des colonnes) puis tri de gauche à droite
        let mut lues = [(Branche::Tronc, 0.0_f32); MAX_BRANCHES];
        let mut n = 0;
        for &b in def.branches.keys().filter(|b| **b != Branche::Tronc) {
            let (somme, nb) = def
                .noeuds
                .iter()
                .filter(|x| x.branche == b)
                .fold((0, 0), |(s, k), x| (s + x.colonne, k + 1));
            if nb > 0 && n < MAX_BRANCHES {
                lues[n] = (b, somme as f32 / nb as f32);
                n += 1;
            }
        }
        lues[..n].sort_by(|a, b| a.1.total_cmp(&b.1));
        let mut voies = [None; MAX_BRANCHES];
        for (i, &(b, _)) in lues[..n].iter().enumerate() {
            let mut sous = [0; MAX_SOUS];
            let mut n_sous = 0;
            for x in def.noeuds.iter().filter(|x| x.branche == b) {
                if n_sous < MAX_SOUS && !sous[..n_sous].contains(&x.colonne) {
                    sous[n_sous] = x.colonne;
                    n_sous += 1;
                }
            }
            sous[..n_sous].sort_unstable();
            let direction = i as f32 - (n - 1) as f32 / 2.0;
            voies[i] = Some(Voie {
                branche: b,
                direction,
                sous,
                n_sous,
            });
        }
        Self {
            voies,
            debut: haut_tronc + 1,
        }
    }

    fn voie(&self, b: Branche) -> Option<&Voie> {
        self.voies.iter().flatten().find(|v| v.branche == b)
    }

    fn y(&self, ligne: u32) -> f32 {
        let fourche = if ligne >= self.debut { FOURCHE } else { 0.0 };
        BAS - ligne as f32 * PAS_LIGNE - fourche
    }

    /// Axe de la branche `v` à la ligne `ligne`.
    fn axe(&self, v: &Voie, ligne: u32) -> f32 {
        let pas = ECART_BASE + ligne.saturating_sub(self.debut) as f32 * ECART_LIGNE;
        CX + v.direction * pas
    }

    /// Centre de la case de `n` (dont la ligne du tronc compte `def` pour s'étaler).
    fn place(&self, def: &Arbre, n: &Noeud) -> Option<Vec2> {
        let x = if n.branche == Branche::Tronc {
            let meme = def
                .noeuds
                .iter()
                .filter(|m| m.branche == Branche::Tronc && m.ligne == n.ligne);
            let (total, avant) = meme.fold((0, 0), |(t, a), m| {
                (t + 1, a + usize::from(m.colonne < n.colonne))
            });
            CX + (avant as f32 - (total - 1) as f32 / 2.0) * PAS_COLONNE
        } else {
            let v = self.voie(n.branche)?;
            let i = v.sous[..v.n_sous].iter().position(|&c| c == n.colonne)?;
            let decalage = i as f32 - (v.n_sous - 1) as f32 / 2.0;
            self.axe(v, n.ligne) + decalage * PAS_COLONNE
        };
        Some(Vec2::new(x, self.y(n.ligne)))
    }
}

/// Trait de `a` à `b` (un nœud tourné autour de son milieu).
fn trait_(commandes: &mut Commands, zone: Entity, (a, b): (Vec2, Vec2), couleur: Color) {
    let d = b - a;
    let m = (a + b) / 2.0;
    let l = d.length();
    commandes.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(m.x - l / 2.0),
            top: Val::Px(m.y - 1.5),
            width: Val::Px(l),
            height: Val::Px(3.0),
            ..default()
        },
        UiTransform::from_rotation(Rot2::radians(d.y.atan2(d.x))),
        BackgroundColor(couleur),
        ChildOf(zone),
    ));
}

pub fn construire(commandes: &mut Commands, c: &Contexte, ic: &mut Icones, parent: Entity) {
    let def = c.arbre();
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let plan = Plan::nouveau(def);
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
    // Traits : chaque talent vers le plus proche de la ligne du dessous, dans sa
    // branche (ou le tronc) ; le bas d'une branche part du haut du tronc.
    let y_tronc = plan.y(plan.debut - 1);
    let haut_tronc = Vec2::new(CX, y_tronc - case::CASE / 2.0 - 18.0);
    let pied = Vec2::new(CX, y_tronc);
    trait_(commandes, zone, (pied, haut_tronc), IVOIRE.with_alpha(0.5));
    for n in &def.noeuds {
        let Some(ici) = plan.place(def, n) else {
            continue;
        };
        if n.ligne == 0 {
            continue;
        }
        let meme = if n.branche == Branche::Tronc || n.ligne == plan.debut {
            Branche::Tronc
        } else {
            n.branche
        };
        let dessous = def
            .noeuds
            .iter()
            .filter(|m| m.branche == meme && m.ligne < n.ligne)
            .filter_map(|m| Some((m, plan.place(def, m)?)))
            .max_by(|(m1, p1), (m2, p2)| {
                (m1.ligne, -(p1.x - ici.x).abs())
                    .partial_cmp(&(m2.ligne, -(p2.x - ici.x).abs()))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        let (bas, rang_bas) = match dessous {
            Some((m, _)) if meme == Branche::Tronc && n.branche != Branche::Tronc => {
                (haut_tronc, prog.rang(p, a, m))
            }
            Some((m, la)) => (la, prog.rang(p, a, m)),
            None => (haut_tronc, 1),
        };
        let vif = prog.rang(p, a, n) > 0 && rang_bas > 0;
        let couleur = teinte(n.branche).with_alpha(if vif { 0.85 } else { 0.28 });
        trait_(commandes, zone, (bas, ici), couleur);
    }
    for v in plan.voies.iter().flatten() {
        bout(commandes, c, ic, zone, &plan, v);
    }
    let lignes = def.noeuds.iter().map(|n| n.ligne).max().unwrap_or(0);
    for l in 0..=lignes {
        seuil(commandes, c, ic, zone, l, plan.y(l));
    }
    for (k, n) in def.noeuds.iter().enumerate() {
        if let Some(centre) = plan.place(def, n) {
            case::construire(commandes, c, ic, zone, (k, n), centre);
        }
    }
}

/// Nom de la branche et points qui y sont, au-dessus de son talent le plus haut.
fn bout(
    commandes: &mut Commands,
    c: &Contexte,
    ic: &mut Icones,
    zone: Entity,
    plan: &Plan,
    v: &Voie,
) {
    let def = c.arbre();
    let prog = &c.sauvegarde.progression;
    let (p, a) = (c.id(), c.etat.arbre);
    let noeuds = def.noeuds.iter().filter(|n| n.branche == v.branche);
    let (haut, points) = noeuds.fold((0, 0), |(h, s), n| (h.max(n.ligne), s + prog.rang(p, a, n)));
    let x = plan.axe(v, haut);
    let y = plan.y(haut) - case::CASE / 2.0 - 52.0;
    let boite = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(x - 100.0),
                top: Val::Px(y),
                width: Val::Px(200.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            ChildOf(zone),
        ))
        .id();
    let nom = def.branches.get(&v.branche).map_or("", String::as_str);
    let t = teinte(v.branche);
    let e = ecrire(commandes, ic, boite, nom, (true, 19.0, t));
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
        (false, charte::T_AIDE - 1.0, couleur),
    );
}

/// Seuil de points de la ligne `l`, écrit à gauche (cadenas s'il manque).
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
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                top: Val::Px(y - 11.0),
                height: Val::Px(22.0),
                align_items: AlignItems::Center,
                column_gap: Val::Px(5.0),
                ..default()
            },
            ChildOf(zone),
        ))
        .id();
    let couleur = if ouverte {
        AIDE.with_alpha(0.6)
    } else {
        OR_VIF
    };
    if !ouverte {
        charte::cadenas(commandes, marque, 9.0, couleur);
    }
    let t = format!("{} pts", palier.rangs);
    ecrire(commandes, ic, marque, &t, (true, 13.0, couleur));
}
