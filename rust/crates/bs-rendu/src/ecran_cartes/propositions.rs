//! Présentations à l'essai des choix (variable `BS_CARTES_STYLE`) : « inventaire »
//! (case d'inventaire et infobulle Minecraft), « parchemin » (feuillets de parchemin),
//! « bandes » (l'écran partagé en grandes bandes verticales) et « verre » (verre dépoli).
//! La présentation par défaut (médaillons) est dans `vue`.

use bevy::prelude::*;
use bs_regles::textes::{Carte, Icone};

use super::vue::{BoutonCarte, Contexte, Halo, couleur_famille};
use crate::ecrans::charte::{IVOIRE, VERT_GAIN};
use crate::ecrans::nuit::{self, Style};
use crate::hud::police::TextePixel;

const DESCRIPTION: Color = Color::srgb_u8(0xc8, 0xcc, 0xd6);

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

fn chiffre(k: usize) -> String {
    ((b'1' + k as u8) as char).to_string()
}

fn bouton(commandes: &mut Commands, liste: Entity, k: usize, n: Node) -> Entity {
    commandes
        .spawn((Button, BoutonCarte(k), n, ChildOf(liste)))
        .id()
}

fn centre(commandes: &mut Commands, e: Entity) {
    commandes
        .entity(e)
        .insert(TextLayout::new_with_justify(Justify::Center));
}

/// Case d'inventaire (grise, en creux) avec l'objet, infobulle Minecraft dessous : fond
/// violet nuit, liseré violet, nom de la couleur de la rareté, effets en bleu.
pub fn inventaire(commandes: &mut Commands, c: &Contexte, liste: Entity, k: usize, carte: &Carte) {
    let (famille, _) = couleur_famille(carte.famille);
    let b = bouton(
        commandes,
        liste,
        k,
        Node {
            width: Val::Px(360.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(14.0),
            ..default()
        },
    );
    let case = commandes
        .spawn((
            Node {
                width: Val::Px(128.0),
                height: Val::Px(128.0),
                border: UiRect::all(Val::Px(4.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb_u8(0x8b, 0x8b, 0x8b)),
            BorderColor {
                top: Color::srgb_u8(0x37, 0x37, 0x37),
                left: Color::srgb_u8(0x37, 0x37, 0x37),
                bottom: Color::WHITE,
                right: Color::WHITE,
            },
            Outline::new(Val::Px(3.0), Val::Px(3.0), Color::NONE),
            ChildOf(b),
        ))
        .id();
    commandes.entity(b).insert(Halo(case, Color::NONE));
    icone(commandes, c, case, carte, 96.0);
    let bulle = commandes
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(0x10, 0x00, 0x10, 0xf0)),
            BorderColor {
                top: Color::srgb_u8(0x50, 0x00, 0xff),
                left: Color::srgb_u8(0x3c, 0x00, 0xbf),
                right: Color::srgb_u8(0x3c, 0x00, 0xbf),
                bottom: Color::srgb_u8(0x28, 0x00, 0x7f),
            },
            ChildOf(b),
        ))
        .id();
    let l = 360.0 - 30.0;
    let nom = format!("{}  [{}]", carte.nom, chiffre(k));
    commandes.spawn((
        TextePixel::nouveau(&nom, famille, 2.5).bloc(l),
        ChildOf(bulle),
    ));
    let gris = Color::srgb_u8(0xa8, 0xa8, 0xa8);
    let s = surtitre(carte);
    commandes.spawn((TextePixel::nouveau(&s, gris, 2.0), ChildOf(bulle)));
    let bleu = Color::srgb_u8(0x55, 0x55, 0xff);
    for ligne in &carte.lignes {
        commandes.spawn((
            TextePixel::nouveau(ligne, bleu, 2.0).bloc(l),
            ChildOf(bulle),
        ));
    }
    let violet = Color::srgb_u8(0xaa, 0x55, 0xcc);
    let d = &carte.description;
    commandes.spawn((TextePixel::nouveau(d, violet, 2.0).bloc(l), ChildOf(bulle)));
}

/// Feuillet de parchemin un peu de travers, encre brune, nom à la cire rouge.
pub fn parchemin(
    commandes: &mut Commands,
    c: &Contexte,
    liste: Entity,
    (k, n): (usize, usize),
    carte: &Carte,
) {
    let (famille, _) = couleur_famille(carte.famille);
    let encre = Color::srgb_u8(0x3b, 0x2a, 0x14);
    let angle = if k % 2 == 0 { -1.5 } else { 1.5 } * (n > 1) as u8 as f32;
    let b = bouton(
        commandes,
        liste,
        k,
        Node {
            width: Val::Px(270.0),
            min_height: Val::Px(380.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(10.0),
            padding: UiRect::axes(Val::Px(22.0), Val::Px(22.0)),
            border: UiRect::all(Val::Px(3.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
    );
    commandes.entity(b).insert((
        UiTransform {
            rotation: Rot2::degrees(angle),
            ..default()
        },
        BorderColor::all(Color::srgb_u8(0x8a, 0x64, 0x2c)),
        Outline::new(Val::Px(3.0), Val::Px(3.0), Color::NONE),
        BackgroundGradient::from(RadialGradient::new(
            UiPosition::CENTER,
            RadialGradientShape::FarthestCorner,
            vec![
                ColorStop::new(Color::srgb_u8(0xf3, 0xe4, 0xbc), Val::Percent(0.0)),
                ColorStop::new(Color::srgb_u8(0xd9, 0xbf, 0x86), Val::Percent(100.0)),
            ],
        )),
        Halo(b, Color::NONE),
    ));
    let haut = nuit::rangee(commandes, b, 8.0);
    commandes.entity(haut).entry::<Node>().and_modify(|mut n| {
        n.align_self = AlignSelf::Stretch;
        n.justify_content = JustifyContent::SpaceBetween;
    });
    nuit::point(commandes, haut, 12.0, famille);
    c.ecrit(
        commandes,
        haut,
        &surtitre(carte),
        (Style::Grasse, 12.0, encre),
    );
    c.ecrit(commandes, haut, &chiffre(k), (Style::Logo, 22.0, encre));
    icone(commandes, c, b, carte, 88.0);
    let cire = Color::srgb_u8(0x8e, 0x1f, 0x14);
    let nom = c.ecrit(commandes, b, &carte.nom, (Style::Logo, 27.0, cire));
    commandes.entity(nom).remove::<TextShadow>();
    centre(commandes, nom);
    let vert = Color::srgb_u8(0x2f, 0x6b, 0x1f);
    if let Some(cle) = carte.lignes.first() {
        let e = c.ecrit(commandes, b, cle, (Style::Grasse, 15.0, vert));
        centre(commandes, e);
    }
    let d = c.ecrit(
        commandes,
        b,
        &carte.description,
        (Style::Normale, 15.0, encre),
    );
    centre(commandes, d);
}

/// Grande bande verticale du haut en bas de l'écran, lueur de la famille montant du bas,
/// grand numéro en filigrane, trait de la famille dessous (or au survol).
pub fn bande(commandes: &mut Commands, c: &Contexte, liste: Entity, k: usize, carte: &Carte) {
    let (famille, lueur) = couleur_famille(carte.famille);
    let b = bouton(
        commandes,
        liste,
        k,
        Node {
            flex_grow: 1.0,
            flex_basis: Val::ZERO,
            max_width: Val::Px(420.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            row_gap: Val::Px(10.0),
            padding: UiRect::new(Val::Px(24.0), Val::Px(24.0), Val::Px(20.0), Val::Px(30.0)),
            ..default()
        },
    );
    commandes
        .entity(b)
        .insert(BackgroundGradient::from(LinearGradient::to_top(vec![
            ColorStop::new(famille.with_alpha(lueur + 0.25), Val::Percent(0.0)),
            ColorStop::new(famille.with_alpha(lueur * 0.4), Val::Percent(55.0)),
            ColorStop::new(Color::NONE, Val::Percent(100.0)),
        ])));
    let filigrane = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                ..default()
            },
            ChildOf(b),
        ))
        .id();
    c.ecrit(
        commandes,
        filigrane,
        &chiffre(k),
        (Style::Logo, 150.0, IVOIRE.with_alpha(0.12)),
    );
    icone(commandes, c, b, carte, 120.0);
    c.ecrit(
        commandes,
        b,
        &surtitre(carte),
        (Style::Grasse, 14.0, famille),
    );
    let nom = c.ecrit(commandes, b, &carte.nom, (Style::Logo, 38.0, IVOIRE));
    centre(commandes, nom);
    if let Some(cle) = carte.lignes.first() {
        let e = c.ecrit(commandes, b, cle, (Style::Grasse, 17.0, VERT_GAIN));
        centre(commandes, e);
    }
    let d = c.ecrit(
        commandes,
        b,
        &carte.description,
        (Style::Normale, 16.0, DESCRIPTION),
    );
    centre(commandes, d);
    let trait_ = commandes
        .spawn((
            Node {
                width: Val::Percent(60.0),
                height: Val::Px(5.0),
                margin: UiRect::top(Val::Px(10.0)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(famille),
            Outline::new(Val::Px(3.0), Val::ZERO, famille),
            ChildOf(b),
        ))
        .id();
    commandes.entity(b).insert(Halo(trait_, famille));
}

/// Plaque de verre dépoli : blanc très léger, bord fin, texte aligné à gauche.
pub fn verre(commandes: &mut Commands, c: &Contexte, liste: Entity, k: usize, carte: &Carte) {
    let (famille, _) = couleur_famille(carte.famille);
    let b = bouton(
        commandes,
        liste,
        k,
        Node {
            width: Val::Px(280.0),
            min_height: Val::Px(360.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            padding: UiRect::all(Val::Px(24.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(28.0)),
            ..default()
        },
    );
    commandes.entity(b).insert((
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.22)),
        Outline::new(Val::Px(2.0), Val::Px(3.0), Color::NONE),
        Halo(b, Color::NONE),
    ));
    let haut = nuit::rangee(commandes, b, 12.0);
    commandes.entity(haut).entry::<Node>().and_modify(|mut n| {
        n.justify_content = JustifyContent::SpaceBetween;
    });
    let fond = commandes
        .spawn((
            Node {
                width: Val::Px(84.0),
                height: Val::Px(84.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(22.0)),
                ..default()
            },
            BackgroundColor(famille.with_alpha(0.28)),
            ChildOf(haut),
        ))
        .id();
    icone(commandes, c, fond, carte, 60.0);
    c.ecrit(
        commandes,
        haut,
        &chiffre(k),
        (Style::Logo, 30.0, IVOIRE.with_alpha(0.4)),
    );
    c.ecrit(
        commandes,
        b,
        &surtitre(carte),
        (Style::Grasse, 12.0, famille),
    );
    c.ecrit(commandes, b, &carte.nom, (Style::Logo, 28.0, IVOIRE));
    if let Some(cle) = carte.lignes.first() {
        c.ecrit(commandes, b, cle, (Style::Grasse, 15.0, VERT_GAIN));
    }
    c.ecrit(
        commandes,
        b,
        &carte.description,
        (Style::Normale, 15.0, DESCRIPTION),
    );
}
