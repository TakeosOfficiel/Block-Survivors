//! Cœurs du HUD (src/ui/coeurs.ts) : sprites vanilla 9 × 9 agrandis × 3, 10 par ligne,
//! chacun rempli pixel par pixel (7 colonnes intérieures) ; cœurs dorés du bouclier
//! après eux ; tremblement à faible santé ; PV exacts au dixième à droite.

use bevy::prelude::*;
use bs_regles::partie::Partie;

use super::police::{TextePixel, ecrire};
use super::{BLANC, absolu, formater_dixiemes, image_ou};
use crate::textures::Textures;

const COEUR_PLEIN: &str = "gui/sprites/hud/heart/full";
const COEUR_DORE: &str = "gui/sprites/hud/heart/absorbing_full";
const COEUR_VIDE: &str = "gui/sprites/hud/heart/container";
/// Pixels d'écran par pixel de sprite.
const PX: f32 = 3.0;
/// Les cœurs tremblent à 4 PV ou moins.
const SEUIL_TREMBLEMENT: f32 = 4.0;
const PERIODE_TREMBLEMENT: f32 = 0.05;
const AMPLITUDE_TREMBLEMENT: f32 = 3.0;

pub fn demander_textures(demander: &mut impl FnMut(&str)) {
    for t in [COEUR_PLEIN, COEUR_DORE, COEUR_VIDE] {
        demander(t);
    }
}

/// Un cœur : son entité (tremblement) et son remplissage.
struct Coeur {
    racine: Entity,
    plein: Entity,
    colonnes: i32,
}

pub struct Coeurs {
    racine: Entity,
    pleins: Vec<Coeur>,
    dores: Vec<Coeur>,
    pv_max: f32,
    texte: Entity,
    /// PV et bouclier affichés, en dixièmes.
    affiches: (i64, i64),
    minuterie: f32,
    tremble: bool,
    alea: u32,
    plein: Option<Handle<Image>>,
    dore: Option<Handle<Image>>,
    vide: Option<Handle<Image>>,
}

pub fn construire(
    commandes: &mut Commands,
    serveur: &AssetServer,
    textures: &Textures,
    racine: Entity,
) -> Coeurs {
    // Les lignes s'empilent vers le haut
    let conteneur = commandes
        .spawn((
            Node {
                bottom: Val::Px(74.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-150.0)),
                width: Val::Px(300.0),
                flex_wrap: FlexWrap::WrapReverse,
                column_gap: Val::Px(3.0),
                row_gap: Val::Px(3.0),
                ..absolu()
            },
            ChildOf(racine),
        ))
        .id();
    let texte = commandes
        .spawn((
            TextePixel::nouveau("", BLANC, 2.0),
            Node {
                bottom: Val::Px(78.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(160.0)),
                ..absolu()
            },
            ChildOf(racine),
        ))
        .id();
    Coeurs {
        racine: conteneur,
        pleins: Vec::new(),
        dores: Vec::new(),
        pv_max: -1.0,
        texte,
        affiches: (-1, -1),
        minuterie: 0.0,
        tremble: false,
        alea: 0x9e37_79b9,
        plein: textures.obtenir(serveur, COEUR_PLEIN),
        dore: textures.obtenir(serveur, COEUR_DORE),
        vide: textures.obtenir(serveur, COEUR_VIDE),
    }
}

impl Coeurs {
    /// Un emplacement de cœur par tranche de 2 PV maximum, puis les cœurs dorés.
    fn reconstruire(&mut self, commandes: &mut Commands, pv_max: f32, plafond_bouclier: f32) {
        self.pv_max = pv_max;
        self.affiches = (-1, -1);
        commandes.entity(self.racine).despawn_children();
        let mut creer = |sprite: &Option<Handle<Image>>, secours: Color, cache: bool| {
            let racine = commandes
                .spawn((
                    Node {
                        width: Val::Px(9.0 * PX),
                        height: Val::Px(9.0 * PX),
                        display: if cache { Display::None } else { Display::Flex },
                        ..default()
                    },
                    ChildOf(self.racine),
                ))
                .id();
            commandes.spawn((
                image_ou(&self.vide, Color::srgb(0.1, 0.1, 0.1)),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..absolu()
                },
                ChildOf(racine),
            ));
            let plein = commandes
                .spawn((
                    image_ou(sprite, secours),
                    Node {
                        left: Val::ZERO,
                        top: Val::ZERO,
                        width: Val::Px(9.0 * PX),
                        height: Val::Px(9.0 * PX),
                        ..absolu()
                    },
                    ChildOf(racine),
                ))
                .id();
            Coeur {
                racine,
                plein,
                colonnes: -1,
            }
        };
        let n = (pv_max / 2.0).ceil() as usize;
        let rouge = Color::srgb(0.88, 0.1, 0.1);
        self.pleins = (0..n).map(|_| creer(&self.plein, rouge, false)).collect();
        let n = (pv_max * plafond_bouclier / 2.0).ceil() as usize;
        let or = Color::srgb(0.94, 0.78, 0.12);
        self.dores = (0..n).map(|_| creer(&self.dore, or, true)).collect();
    }
}

/// Rogne un cœur à `colonnes` colonnes intérieures (bord gauche compris).
fn rogner(images: &mut Query<&mut ImageNode>, noeuds: &mut Query<&mut Node>, c: &Coeur) {
    let largeur = (1 + c.colonnes.clamp(0, 7)) as f32;
    let plein = c.colonnes >= 7;
    if let Ok(mut i) = images.get_mut(c.plein) {
        if i.image != Handle::default() {
            i.rect = (!plein).then(|| Rect::new(0.0, 0.0, largeur, 9.0));
        }
        i.color.set_alpha(if c.colonnes == 0 { 0.0 } else { 1.0 });
    }
    if let Ok(mut n) = noeuds.get_mut(c.plein) {
        n.width = Val::Px(if plein { 9.0 } else { largeur } * PX);
    }
}

/// Colonnes remplies d'un cœur (0 à 7) pour une part de 0 à 1.
fn colonnes(part: f32) -> i32 {
    let c = (part * 7.0).round() as i32;
    if part > 0.0 && c == 0 { 1 } else { c }
}

pub fn afficher(
    j: &mut Coeurs,
    commandes: &mut Commands,
    p: &Partie,
    dt: f32,
    textes: &mut Query<&mut TextePixel>,
    images: &mut Query<&mut ImageNode>,
    noeuds: &mut Query<&mut Node>,
) {
    let joueur = &p.etat.joueur;
    if joueur.pv_max != j.pv_max {
        j.reconstruire(commandes, joueur.pv_max, p.d.regles.plafonds.bouclier);
        return; // les cœurs existent au prochain passage
    }
    let (pv, bouclier) = (joueur.pv.max(0.0), joueur.bouclier.max(0.0));
    let dixiemes = (
        (pv * 10.0 - 1e-4).ceil() as i64,
        (bouclier * 10.0 - 1e-4).ceil() as i64,
    );
    if dixiemes != j.affiches {
        j.affiches = dixiemes;
        let mut s = format!(
            "{} / {}",
            formater_dixiemes(dixiemes.0),
            formater_dixiemes((joueur.pv_max * 10.0).round() as i64)
        );
        if dixiemes.1 > 0 {
            s.push_str(" + ");
            s.push_str(&formater_dixiemes(dixiemes.1));
        }
        if let Ok(mut t) = textes.get_mut(j.texte) {
            ecrire(&mut t, &s);
        }
        for (i, c) in j.dores.iter_mut().enumerate() {
            let part = ((bouclier - i as f32 * 2.0) / 2.0).clamp(0.0, 1.0);
            if let Ok(mut n) = noeuds.get_mut(c.racine) {
                let d = if part > 0.0 {
                    Display::Flex
                } else {
                    Display::None
                };
                if n.display != d {
                    n.display = d;
                }
            }
            let col = if part > 0.0 { colonnes(part).max(1) } else { 0 };
            if col != c.colonnes {
                c.colonnes = col;
                rogner(images, noeuds, c);
            }
        }
    }
    // Chaque cœur vaut 2 PV
    for (i, c) in j.pleins.iter_mut().enumerate() {
        let col = colonnes(((pv - i as f32 * 2.0) / 2.0).clamp(0.0, 1.0));
        if col != c.colonnes {
            c.colonnes = col;
            rogner(images, noeuds, c);
        }
    }
    // Santé basse : chaque cœur saute d'un pixel au hasard
    let actif = pv > 0.0 && pv <= SEUIL_TREMBLEMENT;
    if actif || j.tremble {
        j.minuterie -= dt;
        if !actif || j.minuterie <= 0.0 {
            j.minuterie = PERIODE_TREMBLEMENT;
            for c in j.pleins.iter().chain(&j.dores) {
                j.alea ^= j.alea << 13;
                j.alea ^= j.alea >> 17;
                j.alea ^= j.alea << 5;
                let haut = if actif && j.alea & 1 == 0 {
                    -AMPLITUDE_TREMBLEMENT
                } else {
                    0.0
                };
                if let Ok(mut n) = noeuds.get_mut(c.racine) {
                    n.top = Val::Px(haut);
                }
            }
        }
        j.tremble = actif;
    }
}
