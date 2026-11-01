//! Armes, bonus passifs et équipement portés, en bas à gauche (src/ui/barreCapacites.ts) :
//! cases d'inventaire avec icône et rang (« MAX » en jaune) ; arme éveillée cerclée de
//! violet, objet cerclé de la couleur de sa rareté, emplacement vide estompé. Une ligne
//! n'est reconstruite que quand son contenu change.

use bevy::prelude::*;
use bs_donnees::Donnees;
use bs_donnees::equipement::Rarete;
use bs_regles::equipement::niveau_max_objet;
use bs_regles::partie::Partie;

use super::police::TextePixel;
use super::{BLANC, JAUNE, absolu};
use crate::dessins::Dessins;
use crate::textures::Textures;

pub fn demander_textures(d: &Donnees, demander: &mut impl FnMut(&str)) {
    for a in d.armes.values() {
        demander(&a.icone);
    }
    for p in d.passifs.values() {
        demander(&p.icone);
    }
}

/// Une ligne de cases et ce qu'elle montre : (index, rang, marque).
struct Ligne {
    racine: Entity,
    vue: Vec<(usize, u32, u8)>,
    neuve: Vec<(usize, u32, u8)>,
}

pub struct Capacites {
    equipement: Ligne,
    passifs: Ligne,
    armes: Ligne,
}

pub fn construire(commandes: &mut Commands, racine: Entity) -> Capacites {
    let colonne = commandes
        .spawn((
            Node {
                bottom: Val::Px(12.0),
                left: Val::Px(12.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..absolu()
            },
            ChildOf(racine),
        ))
        .id();
    let mut ligne = || Ligne {
        racine: commandes
            .spawn((
                Node {
                    column_gap: Val::Px(2.0),
                    ..default()
                },
                ChildOf(colonne),
            ))
            .id(),
        vue: Vec::new(),
        neuve: Vec::with_capacity(16),
    };
    Capacites {
        equipement: ligne(),
        passifs: ligne(),
        armes: ligne(),
    }
}

/// Marques d'une case.
const VIDE: u8 = 1;
const EVEILLEE: u8 = 2;

fn couleur_rarete(r: Rarete) -> Color {
    match r {
        Rarete::Commune => Color::srgb_u8(0x9a, 0xa0, 0xaa),
        Rarete::Rare => Color::srgb_u8(0x3f, 0x86, 0xff),
        Rarete::Epique => Color::srgb_u8(0xa3, 0x5c, 0xff),
        Rarete::Legendaire => Color::srgb_u8(0xff, 0x9a, 0x1f),
    }
}

/// Case d'inventaire 40 × 40 (bords sombres en haut à gauche, clairs en bas à droite).
fn case(
    commandes: &mut Commands,
    parent: Entity,
    contour: Option<Color>,
    estompee: bool,
) -> Entity {
    let alpha = if estompee { 0.45 } else { 1.0 };
    let sombre = Color::srgba_u8(0x37, 0x37, 0x37, (alpha * 255.0) as u8);
    let clair = Color::srgba(1.0, 1.0, 1.0, alpha);
    let mut e = commandes.spawn((
        Node {
            width: Val::Px(40.0),
            height: Val::Px(40.0),
            border: UiRect::all(Val::Px(2.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba_u8(0x8b, 0x8b, 0x8b, (alpha * 255.0) as u8)),
        BorderColor {
            top: sombre,
            left: sombre,
            bottom: clair,
            right: clair,
        },
        ChildOf(parent),
    ));
    if let Some(c) = contour {
        e.insert(Outline::new(Val::Px(2.0), Val::ZERO, c));
    }
    e.id()
}

/// Icône (ou initiale du nom) et rang.
fn remplir_case(
    commandes: &mut Commands,
    c: Entity,
    icone: Option<Handle<Image>>,
    nom: &str,
    rang: u32,
    max: u32,
) {
    match icone {
        Some(h) => {
            commandes.spawn((
                ImageNode::new(h),
                Node {
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    ..default()
                },
                ChildOf(c),
            ));
        }
        None => {
            let initiale: String = nom.chars().take(1).collect();
            commandes.spawn((TextePixel::nouveau(&initiale, BLANC, 2.0), ChildOf(c)));
        }
    }
    let (texte, couleur) = if rang >= max {
        ("MAX".to_owned(), JAUNE)
    } else {
        (rang.to_string(), BLANC)
    };
    commandes.spawn((
        TextePixel::nouveau(&texte, couleur, 2.0),
        Node {
            right: Val::ZERO,
            bottom: Val::Px(-2.0),
            ..absolu()
        },
        ChildOf(c),
    ));
}

/// Reconstruit les lignes dont le contenu a changé.
pub fn afficher(
    cap: &mut Capacites,
    commandes: &mut Commands,
    textures: &Textures,
    serveur: &AssetServer,
    dessins: &Dessins,
    p: &Partie,
) {
    let d = p.d;
    // Bagues puis amulette ; emplacement vide estompé
    let l = &mut cap.equipement;
    l.neuve.clear();
    let portes = p.equipement.bagues.iter().chain(&p.equipement.amulettes);
    l.neuve.extend(portes.map(|o| match o {
        Some(o) => (o.objet, o.niveau, 0),
        None => (0, 0, VIDE),
    }));
    if l.neuve != l.vue {
        std::mem::swap(&mut l.neuve, &mut l.vue);
        commandes.entity(l.racine).despawn_children();
        for &(objet, niveau, marque) in &l.vue {
            if marque == VIDE {
                case(commandes, l.racine, None, true);
                continue;
            }
            let def = &d.equipement.objets[objet];
            let c = case(commandes, l.racine, Some(couleur_rarete(def.rarete)), false);
            let max = niveau_max_objet(d, objet);
            let icone = dessins.objets.get(objet).cloned();
            remplir_case(commandes, c, icone, &def.nom, niveau, max);
        }
    }

    // Passifs (sauf consommables : le totem a son compteur près des cœurs)
    let l = &mut cap.passifs;
    l.neuve.clear();
    l.neuve.extend(
        p.passifs
            .possedes
            .iter()
            .filter(|x| d.passifs[x.index].consommable.is_none())
            .map(|x| (x.index, x.rang, 0)),
    );
    if l.neuve != l.vue {
        std::mem::swap(&mut l.neuve, &mut l.vue);
        commandes.entity(l.racine).despawn_children();
        for &(index, rang, _) in &l.vue {
            let def = &d.passifs[index];
            let c = case(commandes, l.racine, None, false);
            let max = def.niveaux.len() as u32;
            remplir_case(
                commandes,
                c,
                textures.obtenir(serveur, &def.icone),
                &def.nom,
                rang,
                max,
            );
        }
    }

    // Armes ; arme éveillée : reflet violet des objets enchantés
    let l = &mut cap.armes;
    l.neuve.clear();
    l.neuve.extend(p.arsenal.armes.iter().map(|a| {
        let marque = if a.eveil.is_some() { EVEILLEE } else { 0 };
        (a.index, a.rang, marque)
    }));
    if l.neuve != l.vue {
        std::mem::swap(&mut l.neuve, &mut l.vue);
        commandes.entity(l.racine).despawn_children();
        for (k, &(_, rang, marque)) in l.vue.iter().enumerate() {
            let a = &p.arsenal.armes[k];
            let violet = (marque == EVEILLEE).then_some(Color::srgb_u8(0x7a, 0x3f, 0xd0));
            let c = case(commandes, l.racine, violet, false);
            let max = a.rang_max();
            remplir_case(
                commandes,
                c,
                textures.obtenir(serveur, &a.def.icone),
                &a.def.nom,
                rang,
                max,
            );
        }
    }
}
