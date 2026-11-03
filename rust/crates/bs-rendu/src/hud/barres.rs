//! Barres 182 × 5 du HUD, agrandies × 2 (src/ui/barreXp.ts, barreBoss.ts) : expérience
//! avec le niveau au-dessus (vert, contour noir), et vie du boss sous le chrono, aux
//! couleurs de sa définition. Le remplissage est rogné au pixel du sprite.

use bevy::prelude::*;
use bs_donnees::Donnees;
use bs_donnees::entites::{CouleurBarre, Ennemi as DefinitionEnnemi};
use bs_regles::partie::Partie;

use super::police::{TextePixel, ecrire};
use super::{BLANC, absolu, image_ou};
use crate::textures::Textures;

const XP_FOND: &str = "gui/sprites/hud/experience_bar_background";
const XP_PROGRESSION: &str = "gui/sprites/hud/experience_bar_progress";
const VERT_XP: Color = Color::srgb(0.502, 1.0, 0.125);

fn sprites_boss(c: CouleurBarre) -> [String; 2] {
    let n = match c {
        CouleurBarre::Pink => "pink",
        CouleurBarre::Blue => "blue",
        CouleurBarre::Red => "red",
        CouleurBarre::Green => "green",
        CouleurBarre::Yellow => "yellow",
        CouleurBarre::Purple => "purple",
        CouleurBarre::White => "white",
    };
    [
        format!("gui/sprites/boss_bar/{n}_background"),
        format!("gui/sprites/boss_bar/{n}_progress"),
    ]
}

pub fn demander_textures(d: &Donnees, demander: &mut impl FnMut(&str)) {
    demander(XP_FOND);
    demander(XP_PROGRESSION);
    for e in d.ennemis.values() {
        if let Some(b) = &e.boss {
            for t in sprites_boss(b.couleur_barre) {
                demander(&t);
            }
        }
    }
}

/// Une barre : fond et remplissage.
struct Barre {
    fond: Entity,
    remplissage: Entity,
    pixels: i32,
}

pub struct Barres {
    xp: Barre,
    niveau: Entity,
    niveau_affiche: u32,
    boss: Entity,
    boss_nom: Entity,
    boss_barre: Barre,
    /// Définition du boss affiché (adresse : identité seulement).
    boss_affiche: Option<usize>,
    sprites_boss: Vec<(CouleurBarre, [Option<Handle<Image>>; 2])>,
}

/// Barre de 364 × 10 px : fond et remplissage.
fn barre(
    commandes: &mut Commands,
    parent: Entity,
    sprites: [&Option<Handle<Image>>; 2],
    couleurs: [Color; 2],
    noeud: Node,
) -> Barre {
    let fond = commandes
        .spawn((
            Node {
                width: Val::Px(364.0),
                height: Val::Px(10.0),
                ..noeud
            },
            image_ou(sprites[0], couleurs[0]),
            ChildOf(parent),
        ))
        .id();
    let remplissage = commandes
        .spawn((
            Node {
                left: Val::ZERO,
                top: Val::ZERO,
                width: Val::Px(0.0),
                height: Val::Px(10.0),
                ..absolu()
            },
            image_ou(sprites[1], couleurs[1]),
            ChildOf(fond),
        ))
        .id();
    Barre {
        fond,
        remplissage,
        pixels: -1,
    }
}

pub fn construire(
    commandes: &mut Commands,
    serveur: &AssetServer,
    textures: &Textures,
    d: &Donnees,
    racine: Entity,
    haut: Entity,
) -> Barres {
    let obtenir = |t: &str| textures.obtenir(serveur, t);
    let sprites_boss = d
        .ennemis
        .values()
        .filter_map(|e| e.boss.as_ref())
        .map(|b| {
            let [f, p] = sprites_boss(b.couleur_barre);
            (b.couleur_barre, [obtenir(&f), obtenir(&p)])
        })
        .collect();

    // Boss, sous le chrono : cartouche sombre, nom puis barre
    let boss = commandes
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                padding: UiRect::new(Val::Px(14.0), Val::Px(14.0), Val::Px(6.0), Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.063, 0.078, 0.094, 0.82)),
            BorderColor {
                top: crate::ecrans::charte::OR_SOURD,
                ..BorderColor::all(crate::ecrans::charte::BORD)
            },
            Visibility::Hidden,
            ChildOf(haut),
        ))
        .id();
    let boss_nom = commandes
        .spawn((TextePixel::nouveau("", BLANC, 2.0), ChildOf(boss)))
        .id();
    let boss_barre = barre(
        commandes,
        boss,
        [&None, &None],
        [Color::srgb(0.29, 0.1, 0.29), Color::srgb(0.88, 0.25, 0.88)],
        Node::default(),
    );

    // Expérience, niveau au-dessus au centre
    let xp = barre(
        commandes,
        racine,
        [&obtenir(XP_FOND), &obtenir(XP_PROGRESSION)],
        [Color::srgb(0.165, 0.165, 0.165), VERT_XP],
        Node {
            bottom: Val::Px(40.0),
            left: Val::Percent(50.0),
            margin: UiRect::left(Val::Px(-182.0)),
            ..absolu()
        },
    );
    let ligne = commandes
        .spawn((
            Node {
                top: Val::Px(-22.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..absolu()
            },
            ChildOf(xp.fond),
        ))
        .id();
    let niveau = commandes
        .spawn((
            TextePixel::nouveau("1", VERT_XP, 2.0).contour(),
            ChildOf(ligne),
        ))
        .id();
    Barres {
        xp,
        niveau,
        niveau_affiche: 0,
        boss,
        boss_nom,
        boss_barre,
        boss_affiche: None,
        sprites_boss,
    }
}

pub fn afficher(
    b: &mut Barres,
    p: &Partie,
    textes: &mut Query<&mut TextePixel>,
    images: &mut Query<&mut ImageNode>,
    noeuds: &mut Query<&mut Node>,
    visibles: &mut Query<&mut Visibility>,
) {
    let joueur = &p.etat.joueur;
    let ratio = (joueur.xp / joueur.xp_niveau.max(1e-6)).min(1.0);
    remplir(&mut b.xp, (ratio * 182.0).floor() as i32, images, noeuds);
    if joueur.niveau != b.niveau_affiche {
        b.niveau_affiche = joueur.niveau;
        if let Ok(mut t) = textes.get_mut(b.niveau) {
            ecrire(&mut t, &joueur.niveau.to_string());
        }
    }

    let boss = p.ennemis.boss_actif().map(|i| &p.ennemis.actifs()[i]);
    let def = boss.map(|e| std::ptr::from_ref::<DefinitionEnnemi>(e.def) as usize);
    if def != b.boss_affiche {
        b.boss_affiche = def;
        b.boss_barre.pixels = -1;
        if let Ok(mut v) = visibles.get_mut(b.boss) {
            *v = if boss.is_some() {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
        if let Some(e) = boss {
            if let Ok(mut t) = textes.get_mut(b.boss_nom) {
                ecrire(&mut t, &e.def.nom);
            }
            let sprites = e
                .def
                .boss
                .as_ref()
                .and_then(|bb| b.sprites_boss.iter().find(|(c, _)| *c == bb.couleur_barre));
            if let Some((_, [f, pr])) = sprites {
                for (entite, h) in [(b.boss_barre.fond, f), (b.boss_barre.remplissage, pr)] {
                    if let (Ok(mut i), Some(h)) = (images.get_mut(entite), h) {
                        *i = ImageNode::new(h.clone());
                    }
                }
            }
        }
    }
    if let Some(e) = boss {
        let pixels = ((e.pv / e.pv_max).max(0.0) * 182.0).ceil() as i32;
        remplir(&mut b.boss_barre, pixels, images, noeuds);
    }
}

/// Remplissage au pixel du sprite (largeur à l'écran : × 2).
fn remplir(
    b: &mut Barre,
    pixels: i32,
    images: &mut Query<&mut ImageNode>,
    noeuds: &mut Query<&mut Node>,
) {
    if pixels == b.pixels {
        return;
    }
    b.pixels = pixels;
    if let Ok(mut n) = noeuds.get_mut(b.remplissage) {
        n.width = Val::Px(pixels as f32 * 2.0);
    }
    if let Ok(mut i) = images.get_mut(b.remplissage)
        && i.image != Handle::default()
    {
        i.rect = Some(Rect::new(0.0, 0.0, pixels.max(0) as f32, 5.0));
    }
}
