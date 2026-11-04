//! Colonne gauche de l'accueil, sous le logo : le grand « JOUER » vert (l'arme de la
//! classe à côté), le résumé de la partie dessous (classe · arène, chacun ouvre son
//! volet), puis les liens sans cadre, une icône et un libellé : Multijoueur (deux
//! visages), Talents (livre enchanté, points libres en pastille d'or), Collection
//! (coffre). Un lien survolé passe à l'or (`survoler_liens`).

use bevy::prelude::*;

use super::Pinceau;
use crate::ecrans::charte::{self, AIDE, IVOIRE, OR_VIF, Relief, T_AIDE, T_SECTION};
use crate::lancement::{ChoixPartie, DonneesJeu};
use crate::menu::{ActionMenu, Volet};
use crate::sauvegarde::Sauvegarde;

/// Largeur de la colonne (celle du logo).
pub(super) const LARGEUR: f32 = 400.0;
const H_JOUER: f32 = 84.0;
const LIVRE: &str = "item/enchanted_book";
const COFFRE: &str = "entity/chest/normal";
/// Lien sans cadre : ses libellés à la couleur de repos passent à l'or au survol.
#[derive(Component, Clone, Copy)]
pub struct Lien(pub Color);

/// Bouton sans cadre ; renvoie (lien, place de l'icône).
pub(super) fn lien(
    p: &mut Pinceau,
    parent: Entity,
    t: &str,
    taille: f32,
    action: ActionMenu,
) -> (Entity, Entity) {
    let b =
        p.c.spawn((
            Button,
            Lien(IVOIRE),
            action,
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(14.0),
                padding: UiRect::vertical(Val::Px(6.0)),
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    let icone = p.noeud(
        b,
        Node {
            width: Val::Px(44.0),
            height: Val::Px(44.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(2.0),
            flex_shrink: 0.0,
            ..default()
        },
    );
    p.texte(b, t, true, taille, IVOIRE);
    (b, icone)
}

/// Coffre vu de face, découpé dans sa texture (couvercle, serrure, caisse).
fn coffre(p: &mut Pinceau, parent: Entity) {
    let image = p.ic.icone(COFFRE);
    let c = p.noeud(
        parent,
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
    );
    for (rect, h) in [
        (Rect::new(14.0, 14.0, 28.0, 19.0), 10.0),
        (Rect::new(14.0, 33.0, 28.0, 43.0), 20.0),
    ] {
        p.c.spawn((
            ImageNode {
                image: image.clone(),
                rect: Some(rect),
                ..default()
            },
            Node {
                width: Val::Px(36.0),
                height: Val::Px(h * 1.2),
                ..default()
            },
            ChildOf(c),
        ));
    }
    p.c.spawn((
        ImageNode {
            image,
            rect: Some(Rect::new(1.0, 1.0, 3.0, 5.0)),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(15.6),
            top: Val::Px(7.2),
            width: Val::Px(4.8),
            height: Val::Px(9.6),
            ..default()
        },
        ChildOf(c),
    ));
}

/// Pastille d'or (points de talent libres).
fn pastille(p: &mut Pinceau, parent: Entity, n: u32) {
    let b = p.noeud(
        parent,
        Node {
            min_width: Val::Px(24.0),
            height: Val::Px(24.0),
            padding: UiRect::horizontal(Val::Px(6.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(12.0)),
            ..default()
        },
    );
    p.c.entity(b).insert(BackgroundColor(OR_VIF));
    let t =
        p.ic.polices
            .texte(n.to_string(), true, 14.0, Color::srgb_u8(0x2a, 0x1d, 0x02));
    p.c.spawn((t, ChildOf(b)));
}

pub(super) fn construire(
    p: &mut Pinceau,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    choix: ChoixPartie,
    parent: Entity,
) {
    let d = dj.d;
    let (id, perso) = d
        .personnages
        .personnages
        .get_index(choix.personnage)
        .expect("classe choisie");

    // JOUER, l'arme de la classe en tête
    let jouer = charte::bouton_relief(p.c, parent, Relief::PRINCIPAL, H_JOUER, ActionMenu::Jouer);
    p.c.entity(jouer).insert(BoxShadow::new(
        Color::srgba(0.0, 0.0, 0.0, 0.6),
        Val::ZERO,
        Val::Px(8.0),
        Val::ZERO,
        Val::Px(18.0),
    ));
    p.c.entity(jouer).entry::<Node>().and_modify(|mut n| {
        n.width = Val::Px(LARGEUR);
        n.column_gap = Val::Px(18.0);
        n.border = UiRect::new(Val::Px(3.0), Val::Px(3.0), Val::Px(3.0), Val::Px(7.0));
    });
    let arme = perso.apparence.objet_en_main.as_deref().or_else(|| {
        perso
            .armes_depart
            .first()
            .and_then(|a| d.armes.get(a))
            .map(|a| a.icone.as_str())
    });
    if let Some(i) = arme {
        p.icone(jouer, i, 44.0);
    }
    // Même police que le logo, ombrée
    let police = p.ic.polices.logo.clone();
    p.c.spawn((
        Text::new("JOUER"),
        TextFont {
            font: police,
            font_size: 46.0,
            ..default()
        },
        TextColor(IVOIRE),
        TextShadow {
            offset: Vec2::new(0.0, 4.0),
            color: Color::srgba(0.0, 0.12, 0.02, 0.7),
        },
        ChildOf(jouer),
    ));

    // Résumé : classe · arène, chacun ouvre son volet
    let resume = p.noeud(
        parent,
        Node {
            width: Val::Px(LARGEUR),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
            margin: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(-6.0), Val::Px(18.0)),
            ..default()
        },
    );
    let arene = dj.arenes[choix.arene].def;
    for (k, (t, volet)) in [(&perso.nom, Volet::Classes), (&arene.nom, Volet::Arenes)]
        .into_iter()
        .enumerate()
    {
        if k > 0 {
            p.texte(resume, "·", true, T_SECTION - 2.0, AIDE);
        }
        let (b, icone) = lien(p, resume, t, T_AIDE + 1.0, ActionMenu::Volet(volet));
        p.c.entity(icone).despawn();
        p.c.entity(b).entry::<Node>().and_modify(|mut n| {
            n.column_gap = Val::Px(4.0);
            n.padding = UiRect::axes(Val::Px(4.0), Val::Px(4.0));
        });
        p.texte(b, "⌄", true, T_AIDE + 2.0, AIDE);
    }

    // Liens
    let (_, icone) = lien(p, parent, "Multijoueur", T_SECTION, ActionMenu::OuvrirMulti);
    let skin = &sauvegarde
        .depart(d, choix.personnage)
        .apparence
        .unwrap_or(&perso.apparence)
        .skin;
    let autre = d
        .personnages
        .personnages
        .values()
        .map(|p| &p.apparence.skin)
        .find(|s| *s != skin)
        .unwrap_or(skin);
    crate::menu::profil::visage(p.c, p.ic, icone, skin, 21.0);
    crate::menu::profil::visage(p.c, p.ic, icone, autre, 21.0);

    let (b, icone) = lien(p, parent, "Talents", T_SECTION, ActionMenu::Talents);
    p.icone(icone, LIVRE, 40.0);
    let prog = &sauvegarde.progression;
    let libres = prog.points_libres(id);
    if libres > 0 {
        pastille(p, b, libres);
    }
    let (_, icone) = lien(p, parent, "Collection", T_SECTION, ActionMenu::Collection);
    coffre(p, icone);
}

/// Survol d'un lien : ses textes (et ceux d'une colonne enfant) passent à l'or ;
/// ceux d'une autre couleur (séparateurs, flèches, détails) ne bougent pas.
pub fn survoler_liens(
    liens: Query<(&Interaction, &Lien, &Children), Changed<Interaction>>,
    enfants: Query<&Children>,
    mut textes: Query<&mut TextColor>,
) {
    for (i, lien, fils) in &liens {
        let couleur = if *i == Interaction::None {
            lien.0
        } else {
            OR_VIF
        };
        let petits = fils
            .iter()
            .flat_map(|e| enfants.get(e).into_iter().flatten().copied());
        for e in fils.iter().chain(petits) {
            if let Ok(mut t) = textes.get_mut(e)
                && (t.0 == lien.0 || t.0 == OR_VIF)
            {
                t.0 = couleur;
            }
        }
    }
}
