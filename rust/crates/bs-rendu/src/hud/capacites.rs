//! Armes, bonus passifs et équipement portés, en bas à gauche, au style des menus :
//! tuiles de nuit arrondies avec l'icône et le rang en pastille (« MAX » en or) ; arme
//! éveillée cerclée de violet, objet cerclé de la couleur de sa rareté, emplacement vide
//! en simple contour. Au survol, une infobulle dit ce que c'est : nom, rang, effet. Une
//! ligne n'est reconstruite que quand son contenu change.

use bevy::prelude::*;
use bs_donnees::Donnees;
use bs_donnees::equipement::Rarete;
use bs_regles::equipement::niveau_max_objet;
use bs_regles::partie::Partie;

use super::police::TextePixel;
use super::{BLANC, Hud, absolu};
use crate::dessins::Dessins;
use crate::ecrans::charte::{AIDE, IVOIRE, OR_VIF};
use crate::ecrans::nuit::{Style, ecrit, nuit};
use crate::polices::PolicesMenu;
use crate::textures::Textures;

/// Texte de l'infobulle d'une tuile : nom, rang, effet.
#[derive(Component)]
pub struct Bulle {
    nom: String,
    rang: String,
    effet: String,
}

/// L'infobulle commune et ses trois textes.
pub struct Infobulle {
    racine: Entity,
    nom: Entity,
    rang: Entity,
    effet: Entity,
}

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
    polices: PolicesMenu,
    bulle: Infobulle,
}

pub fn construire(commandes: &mut Commands, p: &PolicesMenu, racine: Entity) -> Capacites {
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
    let (equipement, passifs, armes) = (ligne(), ligne(), ligne());
    // Infobulle au-dessus des tuiles, cachée hors survol
    let bulle = commandes
        .spawn((
            Node {
                bottom: Val::Percent(100.0),
                left: Val::ZERO,
                width: Val::Px(300.0),
                margin: UiRect::bottom(Val::Px(10.0)),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                border_radius: BorderRadius::all(Val::Px(12.0)),
                ..absolu()
            },
            BackgroundColor(nuit(0.92)),
            Visibility::Hidden,
            ChildOf(colonne),
        ))
        .id();
    let bulle = Infobulle {
        racine: bulle,
        nom: ecrit(commandes, p, bulle, "", (Style::Grasse, 17.0, IVOIRE)),
        rang: ecrit(commandes, p, bulle, "", (Style::Grasse, 12.0, OR_VIF)),
        effet: ecrit(commandes, p, bulle, "", (Style::Normale, 14.0, AIDE)),
    };
    Capacites {
        equipement,
        passifs,
        armes,
        polices: p.clone(),
        bulle,
    }
}

/// Montre l'infobulle de la tuile survolée, la cache sinon.
pub fn survoler(
    hud: Option<Res<Hud>>,
    tuiles: Query<(&Interaction, &Bulle), Changed<Interaction>>,
    mut textes: Query<&mut Text>,
    mut visibles: Query<&mut Visibility>,
) {
    let Some(hud) = hud else { return };
    let b = &hud.capacites.bulle;
    for (i, bulle) in &tuiles {
        let montre = *i != Interaction::None;
        if montre {
            for (e, t) in [
                (b.nom, &bulle.nom),
                (b.rang, &bulle.rang),
                (b.effet, &bulle.effet),
            ] {
                if let Ok(mut texte) = textes.get_mut(e) {
                    texte.0.clone_from(t);
                }
            }
        }
        if let Ok(mut v) = visibles.get_mut(b.racine) {
            v.set_if_neq(if montre {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
        }
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

/// Tuile 42 × 42 : fond de nuit arrondi, contour coloré (rareté, éveil) ou discret.
fn case(commandes: &mut Commands, parent: Entity, contour: Option<Color>, vide: bool) -> Entity {
    let fond = if vide { Color::NONE } else { nuit(0.72) };
    let trait_ = contour.unwrap_or(IVOIRE.with_alpha(if vide { 0.18 } else { 0.1 }));
    commandes
        .spawn((
            Node {
                width: Val::Px(42.0),
                height: Val::Px(42.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(fond),
            Outline::new(Val::Px(2.0), Val::ZERO, trait_),
            ChildOf(parent),
        ))
        .id()
}

/// Icône (ou initiale du nom), rang en pastille et infobulle.
fn remplir_case(
    commandes: &mut Commands,
    p: &PolicesMenu,
    c: Entity,
    icone: Option<Handle<Image>>,
    (nom, effet): (&str, &str),
    (rang, max): (u32, u32),
) {
    match icone {
        Some(h) => {
            commandes.spawn((
                ImageNode::new(h),
                Node {
                    width: Val::Px(30.0),
                    height: Val::Px(30.0),
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
        ("MAX".to_owned(), OR_VIF)
    } else {
        (rang.to_string(), IVOIRE)
    };
    let pastille = commandes
        .spawn((
            Node {
                right: Val::Px(-5.0),
                bottom: Val::Px(-5.0),
                min_width: Val::Px(17.0),
                height: Val::Px(17.0),
                padding: UiRect::horizontal(Val::Px(4.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::MAX,
                ..absolu()
            },
            BackgroundColor(nuit(0.95)),
            ChildOf(c),
        ))
        .id();
    ecrit(
        commandes,
        p,
        pastille,
        &texte,
        (Style::Grasse, 11.0, couleur),
    );
    let rang = if rang >= max {
        format!("Rang {rang} · maximum")
    } else {
        format!("Rang {rang} / {max}")
    };
    commandes.entity(c).insert((
        Interaction::None,
        Bulle {
            nom: nom.to_owned(),
            rang,
            effet: effet.to_owned(),
        },
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
            let i = (niveau as usize).saturating_sub(1);
            let effet = def.description.get(i).or(def.description.first());
            let effet = effet.map_or("", String::as_str);
            let pol = &cap.polices;
            remplir_case(commandes, pol, c, icone, (&def.nom, effet), (niveau, max));
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
            let icone = textures.obtenir(serveur, &def.icone);
            let textes = (def.nom.as_str(), def.description.as_str());
            remplir_case(commandes, &cap.polices, c, icone, textes, (rang, max));
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
            let icone = textures.obtenir(serveur, &a.def.icone);
            let textes = (a.def.nom.as_str(), a.def.description.as_str());
            remplir_case(commandes, &cap.polices, c, icone, textes, (rang, max));
        }
    }
}
