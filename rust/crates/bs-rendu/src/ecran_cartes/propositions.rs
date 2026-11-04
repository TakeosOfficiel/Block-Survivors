//! Présentations à l'essai des choix (variable `BS_CARTES_STYLE`) : « projecteur » (les
//! objets seuls, le texte du choix pointé dessous), « faisceaux » (objets en lévitation
//! dans un rayon de lumière), « bandeaux » (rubans pleine largeur fondus aux bords) et
//! « colonnes » (mise en page de magazine, grands chiffres, filets). La présentation par
//! défaut (médaillons) est dans `vue`.

use bevy::prelude::*;
use bs_regles::textes::{Carte, Icone};

use super::vue::{BoutonCarte, Contexte, Halo, couleur_famille};
use crate::ecrans::charte::{IVOIRE, VERT_GAIN};
use crate::ecrans::nuit::{self, Style};

const DESCRIPTION: Color = Color::srgb_u8(0xc8, 0xcc, 0xd6);

/// Partie d'un choix « projecteur » : texte (montré au survol) ou icône (grossie).
#[derive(Component)]
pub struct Projecteur(pub usize, pub bool);

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

fn aligne(commandes: &mut Commands, e: Entity, j: Justify) {
    commandes.entity(e).insert(TextLayout::new_with_justify(j));
}

/// Trait fin (liseré d'or au survol) : porte le `Halo` du bouton.
fn filet(commandes: &mut Commands, b: Entity, parent: Entity, largeur: Val, couleur: Color) {
    let t = commandes
        .spawn((
            Node {
                width: largeur,
                height: Val::Px(3.0),
                border_radius: BorderRadius::MAX,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(couleur),
            Outline::new(Val::Px(2.0), Val::ZERO, couleur),
            ChildOf(parent),
        ))
        .id();
    commandes.entity(b).insert(Halo(t, couleur));
}

/// Description, stat clé et autres lignes, alignées selon `j`.
fn textes(commandes: &mut Commands, c: &Contexte, p: Entity, carte: &Carte, j: Justify) {
    if let Some(cle) = carte.lignes.first() {
        let e = c.ecrit(commandes, p, cle, (Style::Grasse, 17.0, VERT_GAIN));
        aligne(commandes, e, j);
    }
    let d = (Style::Normale, 16.0, DESCRIPTION);
    let d = c.ecrit(commandes, p, &carte.description, d);
    aligne(commandes, d, j);
}

/// Projecteur : l'objet seul, grand, son nom ; le reste n'apparaît qu'au survol (le
/// premier est montré d'abord), l'objet pointé grossit.
pub fn projecteur(commandes: &mut Commands, c: &Contexte, liste: Entity, k: usize, carte: &Carte) {
    let (famille, lueur) = couleur_famille(carte.famille);
    let b = bouton(
        commandes,
        liste,
        k,
        Node {
            width: Val::Px(320.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(10.0),
            ..default()
        },
    );
    let lumiere = commandes
        .spawn((
            Projecteur(k, false),
            Node {
                width: Val::Px(220.0),
                height: Val::Px(220.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            UiTransform::from_scale(Vec2::splat(if k == 0 { 1.15 } else { 0.9 })),
            BackgroundGradient::from(RadialGradient::new(
                UiPosition::CENTER,
                RadialGradientShape::ClosestSide,
                vec![
                    ColorStop::new(famille.with_alpha(lueur * 1.6), Val::Percent(0.0)),
                    ColorStop::new(Color::NONE, Val::Percent(100.0)),
                ],
            )),
            ChildOf(b),
        ))
        .id();
    icone(commandes, c, lumiere, carte, 140.0);
    let s = c.ecrit(
        commandes,
        b,
        &surtitre(carte),
        (Style::Grasse, 14.0, famille),
    );
    aligne(commandes, s, Justify::Center);
    let nom = c.ecrit(commandes, b, &carte.nom, (Style::Logo, 34.0, IVOIRE));
    aligne(commandes, nom, Justify::Center);
    filet(commandes, b, b, Val::Px(40.0), famille);
    let detail = commandes
        .spawn((
            Projecteur(k, true),
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                min_height: Val::Px(110.0),
                ..default()
            },
            if k == 0 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            },
            ChildOf(b),
        ))
        .id();
    textes(commandes, c, detail, carte, Justify::Center);
    let t = format!("Touche {}", chiffre(k));
    c.ecrit(commandes, detail, &t, (Style::Grasse, 13.0, nuit::TERNE));
}

/// Au survol d'un choix « projecteur » : son texte paraît, son objet grossit.
pub fn projeter(
    survols: Query<(&Interaction, &BoutonCarte), Changed<Interaction>>,
    mut parties: Query<(&Projecteur, &mut Visibility, &mut UiTransform)>,
) {
    for (i, b) in &survols {
        if *i == Interaction::None {
            continue;
        }
        for (p, mut v, mut t) in &mut parties {
            let choisi = p.0 == b.0;
            if p.1 {
                v.set_if_neq(if choisi {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                });
            } else {
                t.scale = Vec2::splat(if choisi { 1.15 } else { 0.9 });
            }
        }
    }
}

/// Faisceau : l'objet flotte dans un rayon de lumière de sa couleur, fondu vers le
/// haut, le texte dessous.
pub fn faisceau(commandes: &mut Commands, c: &Contexte, liste: Entity, k: usize, carte: &Carte) {
    let (famille, _) = couleur_famille(carte.famille);
    let b = bouton(
        commandes,
        liste,
        k,
        Node {
            width: Val::Px(300.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(10.0),
            ..default()
        },
    );
    // Rayon : bande étroite, de la couleur de la famille en bas, fondue en haut
    let colonne = commandes
        .spawn((
            Node {
                width: Val::Px(130.0),
                height: Val::Px(250.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                padding: UiRect::bottom(Val::Px(30.0)),
                border_radius: BorderRadius::top(Val::Px(65.0)),
                ..default()
            },
            BackgroundGradient::from(LinearGradient::to_top(vec![
                ColorStop::new(famille.with_alpha(0.6), Val::Percent(0.0)),
                ColorStop::new(famille.with_alpha(0.18), Val::Percent(45.0)),
                ColorStop::new(Color::NONE, Val::Percent(100.0)),
            ])),
            ChildOf(b),
        ))
        .id();
    icone(commandes, c, colonne, carte, 120.0);
    let s = format!("{}   ·   {}", chiffre(k), surtitre(carte));
    c.ecrit(commandes, b, &s, (Style::Grasse, 14.0, famille));
    let nom = c.ecrit(commandes, b, &carte.nom, (Style::Logo, 32.0, IVOIRE));
    aligne(commandes, nom, Justify::Center);
    textes(commandes, c, b, carte, Justify::Center);
    filet(commandes, b, b, Val::Px(70.0), famille);
}

/// Bandeau : ruban horizontal sur toute la largeur, couleur de la famille fondue aux
/// deux bords ; chiffre, objet, nom à gauche, effets à droite.
pub fn bandeau(commandes: &mut Commands, c: &Contexte, liste: Entity, k: usize, carte: &Carte) {
    let (famille, lueur) = couleur_famille(carte.famille);
    let b = bouton(
        commandes,
        liste,
        k,
        Node {
            width: Val::Percent(100.0),
            min_height: Val::Px(120.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
    );
    let a = (lueur * 1.5).min(0.5);
    commandes
        .entity(b)
        .insert(BackgroundGradient::from(LinearGradient::to_right(vec![
            ColorStop::new(Color::NONE, Val::Percent(0.0)),
            ColorStop::new(famille.with_alpha(a), Val::Percent(30.0)),
            ColorStop::new(famille.with_alpha(a * 0.6), Val::Percent(70.0)),
            ColorStop::new(Color::NONE, Val::Percent(100.0)),
        ])));
    let ligne = nuit::rangee(commandes, b, 28.0);
    commandes
        .entity(ligne)
        .entry::<Node>()
        .and_modify(|mut n| n.width = Val::Px(1000.0));
    c.ecrit(
        commandes,
        ligne,
        &chiffre(k),
        (Style::Logo, 64.0, IVOIRE.with_alpha(0.35)),
    );
    icone(commandes, c, ligne, carte, 88.0);
    let gauche = nuit::colonne(commandes, ligne, 4.0);
    commandes
        .entity(gauche)
        .entry::<Node>()
        .and_modify(|mut n| {
            n.width = Val::Px(380.0);
            n.align_items = AlignItems::FlexStart;
        });
    c.ecrit(
        commandes,
        gauche,
        &surtitre(carte),
        (Style::Grasse, 14.0, famille),
    );
    c.ecrit(commandes, gauche, &carte.nom, (Style::Logo, 34.0, IVOIRE));
    filet(commandes, b, gauche, Val::Px(60.0), famille);
    let droite = nuit::colonne(commandes, ligne, 6.0);
    commandes
        .entity(droite)
        .entry::<Node>()
        .and_modify(|mut n| {
            n.flex_grow = 1.0;
            n.flex_shrink = 1.0;
            n.align_items = AlignItems::FlexStart;
        });
    textes(commandes, c, droite, carte, Justify::Left);
}

/// Colonne de magazine : grand chiffre de la couleur de la famille, objet, nom, filet,
/// texte aligné à gauche ; filets verticaux entre les colonnes.
pub fn colonne(commandes: &mut Commands, c: &Contexte, liste: Entity, k: usize, carte: &Carte) {
    let (famille, _) = couleur_famille(carte.famille);
    let b = bouton(
        commandes,
        liste,
        k,
        Node {
            width: Val::Px(330.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: Val::Px(10.0),
            padding: UiRect::horizontal(Val::Px(34.0)),
            border: UiRect::left(Val::Px(if k > 0 { 1.0 } else { 0.0 })),
            ..default()
        },
    );
    commandes
        .entity(b)
        .insert(BorderColor::all(IVOIRE.with_alpha(0.18)));
    let haut = nuit::rangee(commandes, b, 18.0);
    commandes
        .entity(haut)
        .entry::<Node>()
        .and_modify(|mut n| n.align_items = AlignItems::FlexEnd);
    c.ecrit(commandes, haut, &chiffre(k), (Style::Logo, 110.0, famille));
    icone(commandes, c, haut, carte, 72.0);
    c.ecrit(
        commandes,
        b,
        &surtitre(carte),
        (Style::Grasse, 14.0, famille),
    );
    let nom = c.ecrit(commandes, b, &carte.nom, (Style::Logo, 36.0, IVOIRE));
    aligne(commandes, nom, Justify::Left);
    filet(commandes, b, b, Val::Px(56.0), famille);
    textes(commandes, c, b, carte, Justify::Left);
}
