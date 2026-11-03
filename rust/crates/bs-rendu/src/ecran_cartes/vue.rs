//! Mise en page de l'écran de cartes (docs/CHARTE-UI.md) : voile sombre, titre ivoire
//! entre deux traits d'or sourd, cartes debout au bord neutre (seule la carte survolée
//! passe à l'or vif), bandeau et liseré intérieur de la couleur de la famille (rareté pour
//! les objets), fond teinté ; zones fixes : touche, icône, nom en ivoire, stat clé, puis
//! étiquette, description et autres lignes « avant → après » (nouvelle valeur en vert).
//! Sous les cartes : « Laisser les objets » (refus d'un coffre) et « Relancer ».
//! Tout est reconstruit à chaque ouverture ou relance (jamais pendant le jeu).

use bevy::prelude::*;
use bs_donnees::equipement::Rarete;
use bs_regles::textes::{Carte, Famille, Icone};

use crate::dessins::Dessins;
use crate::ecrans::charte::{self, AIDE, BORD, IVOIRE, OR_SOURD, Relief};
use crate::ecrans::style::{SOMBRE, plein};
use crate::hud::police::TextePixel;
use crate::textures::Textures;

const VOILE: Color = Color::srgba_u8(6, 8, 14, 188);
const FOND_CARTE: Color = Color::srgba_u8(10, 12, 18, 240);
const TEXTE_BANDEAU: Color = Color::srgb_u8(0x10, 0x13, 0x1b);
const DESCRIPTION: Color = Color::srgb_u8(0xc8, 0xcc, 0xd6);
const STATS: Color = Color::srgb_u8(0xcf, 0xe9, 0xc8);
const VALEUR: Color = charte::VERT_GAIN;
const LARGEUR_CARTE: f32 = 250.0;
/// Largeur du texte dans la carte (marges de 18 px et liseré).
const LARGEUR_TEXTE: f32 = LARGEUR_CARTE - 2.0 * 18.0 - 6.0;

/// Carte cliquable (index dans la liste).
#[derive(Component)]
pub struct BoutonCarte(pub usize);

/// Bouton « Relancer ».
#[derive(Component)]
pub struct BoutonRelancer;

/// Ressources nécessaires aux icônes.
pub struct Contexte<'a> {
    pub serveur: &'a AssetServer,
    pub textures: &'a Textures,
    pub dessins: &'a Dessins,
}

/// Couleur de la famille et opacité de son voile (styles.css `.famille-*`).
pub(crate) fn couleur_famille(f: Famille) -> (Color, f32) {
    let (rgb, voile) = match f {
        Famille::Arme => (0xd8dde6, 0.16),
        Famille::Passif => (0x55e055, 0.22),
        Famille::NouvelleArme => (0x4fd8ff, 0.26),
        Famille::Exclusif => (0xff6ad5, 0.3),
        Famille::Eveil => (0xb06cff, 0.36),
        Famille::Soin => (0xffc83a, 0.22),
        Famille::Objet(Rarete::Commune) => (0xc8ccd4, 0.16),
        Famille::Objet(Rarete::Rare) => (0x4f8dff, 0.28),
        Famille::Objet(Rarete::Epique) => (0xb06cff, 0.34),
        Famille::Objet(Rarete::Legendaire) => (0xff9a1f, 0.36),
        Famille::Action => (0x8a93a3, 0.14),
    };
    let c = Color::srgb_u8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8);
    (c, voile)
}

/// Construit l'écran ; `relances` : `None` sans bouton (étape de remplacement).
pub fn construire(
    commandes: &mut Commands,
    c: &Contexte,
    titre: &str,
    cartes: &[Carte],
    relances: Option<u32>,
) -> Entity {
    let racine = commandes
        .spawn((
            Name::new("écran de cartes"),
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(26.0),
                ..plein()
            },
            BackgroundColor(VOILE),
            GlobalZIndex(10),
        ))
        .id();

    // Titre entre deux traits d'or
    let ligne = commandes
        .spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(18.0),
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    let trait_or = || {
        (
            Node {
                width: Val::Px(120.0),
                height: Val::Px(3.0),
                ..default()
            },
            BackgroundColor(OR_SOURD),
            ChildOf(ligne),
        )
    };
    commandes.spawn(trait_or());
    commandes.spawn((TextePixel::nouveau(titre, IVOIRE, 4.0), ChildOf(ligne)));
    commandes.spawn(trait_or());

    let liste = commandes
        .spawn((
            Node {
                column_gap: Val::Px(22.0),
                padding: UiRect::top(Val::Px(12.0)),
                align_items: AlignItems::Stretch,
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    let mut refus = None;
    for (k, carte) in cartes.iter().enumerate() {
        // Refus du coffre : action secondaire sous les cartes, pas une carte
        if carte.famille == Famille::Action && carte.etiquette == "Refuser" {
            refus = Some((k, carte));
            continue;
        }
        construire_carte(commandes, c, liste, k, carte);
    }

    let bas = commandes
        .spawn((
            Node {
                column_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    if let Some((k, carte)) = refus {
        let b = charte::bouton_relief(commandes, bas, Relief::SECONDAIRE, 40.0, BoutonCarte(k));
        let t = format!("{} ({})", carte.nom, k + 1);
        commandes.spawn((TextePixel::nouveau(&t, IVOIRE, 2.0), ChildOf(b)));
    }
    if let Some(n) = relances {
        bouton_relancer(commandes, bas, n);
    }
    racine
}

fn construire_carte(
    commandes: &mut Commands,
    c: &Contexte,
    liste: Entity,
    k: usize,
    carte: &Carte,
) {
    let (famille, voile) = couleur_famille(carte.famille);
    let teinte = famille.with_alpha(voile);
    let bouton = commandes
        .spawn((
            Button,
            BoutonCarte(k),
            Node {
                width: Val::Px(LARGEUR_CARTE),
                min_height: Val::Px(350.0),
                padding: UiRect::new(Val::Px(18.0), Val::Px(18.0), Val::Px(50.0), Val::Px(18.0)),
                border: UiRect::all(Val::Px(3.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BorderColor::all(BORD),
            Outline::new(Val::Px(2.0), Val::ZERO, SOMBRE),
            BackgroundColor(FOND_CARTE),
            BackgroundGradient::from(LinearGradient::to_bottom(vec![
                ColorStop::new(teinte, Val::Percent(0.0)),
                ColorStop::new(Color::NONE, Val::Percent(42.0)),
            ])),
            ChildOf(liste),
        ))
        .id();
    // Liserés intérieurs : sombre puis couleur de la famille
    let sombre = commandes
        .spawn((
            Node {
                border: UiRect::all(Val::Px(3.0)),
                ..plein()
            },
            BorderColor::all(SOMBRE),
            ChildOf(bouton),
        ))
        .id();
    commandes.spawn((
        Node {
            border: UiRect::all(Val::Px(2.0)),
            ..plein()
        },
        BorderColor::all(famille),
        ChildOf(sombre),
    ));
    // Bandeau de la famille
    let bandeau = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(5.0),
                right: Val::Px(5.0),
                top: Val::Px(5.0),
                height: Val::Px(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(famille),
            ChildOf(bouton),
        ))
        .id();
    commandes.spawn((
        TextePixel::nouveau(carte.famille.nom(), TEXTE_BANDEAU, 2.0).sans_bord(),
        ChildOf(bandeau),
    ));
    // Touche
    let touche = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(42.0),
                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.3)),
            ChildOf(bouton),
        ))
        .id();
    let chiffre = (b'1' + k as u8) as char;
    commandes.spawn((
        TextePixel::nouveau(
            chiffre.encode_utf8(&mut [0; 4]),
            Color::srgb_u8(0xcf, 0xd3, 0xdc),
            1.5,
        ),
        ChildOf(touche),
    ));
    // Icône
    let cadre = commandes
        .spawn((
            Node {
                width: Val::Px(84.0),
                height: Val::Px(84.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(14.0)),
                ..default()
            },
            BackgroundGradient::from(RadialGradient::new(
                UiPosition::CENTER,
                RadialGradientShape::FarthestCorner,
                vec![
                    ColorStop::new(teinte, Val::Percent(0.0)),
                    ColorStop::new(Color::NONE, Val::Percent(70.0)),
                ],
            )),
            ChildOf(bouton),
        ))
        .id();
    let image = match carte.icone {
        Icone::Texture(chemin) => c.textures.obtenir(c.serveur, chemin),
        Icone::Objet(i) => c.dessins.objets.get(i).cloned(),
    };
    if let Some(h) = image {
        commandes.spawn((
            ImageNode::new(h),
            Node {
                width: Val::Px(56.0),
                height: Val::Px(56.0),
                ..default()
            },
            ChildOf(cadre),
        ));
    }
    // Zone du nom (hauteur fixe : deux lignes), en ivoire ; stat clé dessous
    let zone_nom = commandes
        .spawn((
            Node {
                height: Val::Px(40.0),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                ..default()
            },
            ChildOf(bouton),
        ))
        .id();
    commandes.spawn((
        TextePixel::nouveau(&carte.nom, IVOIRE, 2.0).bloc(LARGEUR_TEXTE),
        ChildOf(zone_nom),
    ));
    let zone_cle = commandes
        .spawn((
            Node {
                height: Val::Px(18.0),
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(bouton),
        ))
        .id();
    if let Some(cle) = carte.lignes.first() {
        commandes.spawn((
            TextePixel::nouveau(cle, VALEUR, 1.5).bloc(LARGEUR_TEXTE),
            ChildOf(zone_cle),
        ));
    }
    if carte.etiquette != carte.famille.nom() {
        let pastille = commandes
            .spawn((
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(teinte),
                ChildOf(bouton),
            ))
            .id();
        commandes.spawn((
            TextePixel::nouveau(&carte.etiquette.to_uppercase(), famille, 1.25),
            ChildOf(pastille),
        ));
    }
    commandes.spawn((
        TextePixel::nouveau(&carte.description, DESCRIPTION, 1.5).bloc(LARGEUR_TEXTE),
        ChildOf(bouton),
    ));
    // Les autres lignes en pied (la première est la stat clé, sous le nom)
    if carte.lignes.len() < 2 {
        return;
    }
    // Lignes « Dégâts : 10 → 15 » : la nouvelle valeur ressort
    let stats = commandes
        .spawn((
            Node {
                margin: UiRect::top(Val::Auto),
                align_self: AlignSelf::Stretch,
                padding: UiRect::top(Val::Px(10.0)),
                border: UiRect::top(Val::Px(1.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.1)),
            ChildOf(bouton),
        ))
        .id();
    for l in carte.lignes.iter().skip(1) {
        // Valeur à la ligne si elle ne tient pas
        let noeud = Node {
            flex_wrap: FlexWrap::Wrap,
            ..default()
        };
        let ligne = commandes.spawn((noeud, ChildOf(stats))).id();
        let (debut, valeur) = match l.rfind("→ ") {
            Some(i) => l.split_at(i + "→ ".len()),
            None => (l.as_str(), ""),
        };
        commandes.spawn((
            TextePixel::nouveau(debut, STATS, 1.5).bloc(LARGEUR_TEXTE),
            ChildOf(ligne),
        ));
        if !valeur.is_empty() {
            commandes.spawn((TextePixel::nouveau(valeur, VALEUR, 1.5), ChildOf(ligne)));
        }
    }
}

/// « Relancer (R) · 2 restantes », ou « Plus de relance » éteint.
fn bouton_relancer(commandes: &mut Commands, parent: Entity, n: u32) {
    let bouton = charte::bouton_relief(commandes, parent, Relief::SECONDAIRE, 40.0, BoutonRelancer);
    if n == 0 {
        commandes
            .entity(bouton)
            .remove::<(Button, Relief)>()
            .insert(BackgroundColor(charte::CREUX));
        commandes.spawn((
            TextePixel::nouveau("Plus de relance", AIDE, 2.0),
            ChildOf(bouton),
        ));
        return;
    }
    let pluriel = if n > 1 { "s" } else { "" };
    commandes.spawn((
        TextePixel::nouveau("Relancer (R)", IVOIRE, 2.0),
        ChildOf(bouton),
    ));
    commandes.spawn((
        TextePixel::nouveau(&format!("· {n} restante{pluriel}"), AIDE, 2.0),
        ChildOf(bouton),
    ));
}
