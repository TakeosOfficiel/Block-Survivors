//! Présentations à l'essai des choix (variable `BS_CARTES_STYLE`) : « liste » (une ligne
//! par choix, comme les pages du menu) et « jeu » (grandes cartes à jouer illustrées,
//! bord doré, en éventail). La présentation par défaut (médaillons) est dans `vue`.

use bevy::prelude::*;
use bs_regles::textes::{Carte, Icone};

use super::vue::{BoutonCarte, Contexte, Halo, couleur_famille};
use crate::ecrans::charte::{AIDE, IVOIRE, OR_VIF, VERT_GAIN};
use crate::ecrans::nuit::{self, Style, nuit as sombre};

const DESCRIPTION: Color = Color::srgb_u8(0xc8, 0xcc, 0xd6);
const OR_SOURD: Color = Color::srgb_u8(0xb8, 0x8a, 0x2c);

fn icone(commandes: &mut Commands, c: &Contexte, parent: Entity, carte: &Carte, cote: f32) {
    let image = match carte.icone {
        Icone::Texture(chemin) => c.textures.obtenir(c.serveur, chemin),
        Icone::Objet(i) => c.dessins.objets.get(i).cloned(),
    };
    if let Some(h) = image {
        commandes.spawn((
            ImageNode::new(h),
            Node {
                width: Val::Px(cote),
                height: Val::Px(cote),
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(parent),
        ));
    }
}

fn surtitre(carte: &Carte) -> String {
    if carte.etiquette != carte.famille.nom() {
        carte.etiquette.to_uppercase()
    } else {
        carte.famille.nom().to_uppercase()
    }
}

/// Une ligne par choix : touche, icône dans sa lueur, nom et famille, effet à droite.
pub fn ligne(commandes: &mut Commands, c: &Contexte, liste: Entity, k: usize, carte: &Carte) {
    let (famille, lueur) = couleur_famille(carte.famille);
    let b = commandes
        .spawn((
            Button,
            BoutonCarte(k),
            Node {
                width: Val::Px(980.0),
                max_width: Val::Percent(94.0),
                align_items: AlignItems::Center,
                column_gap: Val::Px(22.0),
                padding: UiRect::axes(Val::Px(26.0), Val::Px(14.0)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            Outline::new(Val::Px(2.0), Val::ZERO, Color::NONE),
            BackgroundGradient::from(LinearGradient::to_right(vec![
                ColorStop::new(famille.with_alpha(lueur), Val::Percent(0.0)),
                ColorStop::new(Color::NONE, Val::Percent(60.0)),
            ])),
            ChildOf(liste),
        ))
        .id();
    commandes.entity(b).insert(Halo(b, Color::NONE));
    let chiffre = ((b'1' + k as u8) as char).to_string();
    c.ecrit(commandes, b, &chiffre, (Style::Logo, 30.0, nuit::TERNE));
    icone(commandes, c, b, carte, 64.0);
    let gauche = nuit::colonne(commandes, b, 2.0);
    commandes
        .entity(gauche)
        .entry::<Node>()
        .and_modify(|mut n| {
            n.width = Val::Px(340.0);
        });
    c.ecrit(
        commandes,
        gauche,
        &surtitre(carte),
        (Style::Grasse, 13.0, famille),
    );
    c.ecrit(commandes, gauche, &carte.nom, (Style::Logo, 32.0, IVOIRE));
    let droite = nuit::colonne(commandes, b, 4.0);
    commandes
        .entity(droite)
        .entry::<Node>()
        .and_modify(|mut n| {
            n.flex_grow = 1.0;
            n.flex_shrink = 1.0;
        });
    if let Some(cle) = carte.lignes.first() {
        c.ecrit(commandes, droite, cle, (Style::Grasse, 17.0, VERT_GAIN));
    }
    c.ecrit(
        commandes,
        droite,
        &carte.description,
        (Style::Normale, 15.0, DESCRIPTION),
    );
    for l in carte.lignes.iter().skip(1) {
        c.ecrit(commandes, droite, l, (Style::Normale, 13.0, AIDE));
    }
}

/// Grande carte à jouer : bord doré, fond de la couleur de la famille vers la nuit,
/// grosse icône dans sa lueur ; légèrement inclinée selon sa place dans l'éventail.
pub fn carte_jeu(
    commandes: &mut Commands,
    c: &Contexte,
    liste: Entity,
    (k, n): (usize, usize),
    carte: &Carte,
) {
    let (famille, _) = couleur_famille(carte.famille);
    let angle = (k as f32 - (n.saturating_sub(1)) as f32 / 2.0) * 3.5;
    let b = commandes
        .spawn((
            Button,
            BoutonCarte(k),
            Node {
                width: Val::Px(270.0),
                min_height: Val::Px(410.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(10.0),
                padding: UiRect::new(Val::Px(20.0), Val::Px(20.0), Val::Px(16.0), Val::Px(22.0)),
                border: UiRect::all(Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(16.0)),
                ..default()
            },
            UiTransform {
                rotation: Rot2::degrees(angle),
                ..default()
            },
            BorderColor::all(OR_SOURD),
            Outline::new(Val::Px(3.0), Val::Px(3.0), Color::NONE),
            BackgroundColor(sombre(1.0)),
            BackgroundGradient::from(LinearGradient::to_bottom(vec![
                ColorStop::new(famille.with_alpha(0.75), Val::Percent(0.0)),
                ColorStop::new(famille.with_alpha(0.25), Val::Percent(45.0)),
                ColorStop::new(Color::NONE, Val::Percent(80.0)),
            ])),
            ChildOf(liste),
        ))
        .id();
    commandes.entity(b).insert(Halo(b, Color::NONE));
    let haut = nuit::rangee(commandes, b, 8.0);
    commandes.entity(haut).entry::<Node>().and_modify(|mut n| {
        n.align_self = AlignSelf::Stretch;
        n.justify_content = JustifyContent::SpaceBetween;
    });
    let s = surtitre(carte);
    c.ecrit(commandes, haut, &s, (Style::Logo, 15.0, IVOIRE));
    let chiffre = ((b'1' + k as u8) as char).to_string();
    c.ecrit(commandes, haut, &chiffre, (Style::Logo, 22.0, OR_VIF));
    let halo = commandes
        .spawn((
            Node {
                width: Val::Px(150.0),
                height: Val::Px(150.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundGradient::from(RadialGradient::new(
                UiPosition::CENTER,
                RadialGradientShape::ClosestSide,
                vec![
                    ColorStop::new(Color::srgba(1.0, 0.95, 0.8, 0.45), Val::Percent(0.0)),
                    ColorStop::new(Color::NONE, Val::Percent(100.0)),
                ],
            )),
            ChildOf(b),
        ))
        .id();
    icone(commandes, c, halo, carte, 92.0);
    let nom = c.ecrit(commandes, b, &carte.nom, (Style::Logo, 28.0, IVOIRE));
    commandes
        .entity(nom)
        .insert(TextLayout::new_with_justify(Justify::Center));
    if let Some(cle) = carte.lignes.first() {
        let e = c.ecrit(commandes, b, cle, (Style::Grasse, 15.0, VERT_GAIN));
        commandes
            .entity(e)
            .insert(TextLayout::new_with_justify(Justify::Center));
    }
    let d = (Style::Normale, 15.0, DESCRIPTION);
    let d = c.ecrit(commandes, b, &carte.description, d);
    commandes
        .entity(d)
        .insert(TextLayout::new_with_justify(Justify::Center));
}
