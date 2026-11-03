//! Construction de la page Profil : skins à gauche, pseudo et classe au centre (le
//! héros 3D est dessous), pièces d'armure portées à droite.

use bevy::prelude::*;
use bs_donnees::arenes::Monde;
use bs_sauvegarde::collection::{MONDES, PIECES};
use bs_sauvegarde::profil_joueur::{
    CHEMIN_SKIN_IMPORTE, ChoixSkin, NOMS_SKINS, chemin_skin_vanilla,
};

use super::{ActionProfil, Skin};
use crate::dessins::Dessins;
use crate::ecrans::style::{self, GRIS, OR, OR_CLAIR, TEXTE};
use crate::hud::police::TextePixel;
use crate::icones_armures;
use crate::lancement::DonneesJeu;
use crate::menu::{ActionMenu, Icones};
use crate::sauvegarde::Sauvegarde;

const LARGEUR_SKINS: f32 = 300.0;
const LARGEUR_ARMURE: f32 = 380.0;
const JAUNE: Color = Color::srgb_u8(0xff, 0xe0, 0x66);
const ROUGE: Color = Color::srgb_u8(0xff, 0x8a, 0x8a);
const VERT: Color = Color::srgb_u8(0x55, 0xff, 0x55);

fn texte(commandes: &mut Commands, parent: Entity, t: &str, couleur: Color, taille: f32) {
    commandes.spawn((TextePixel::nouveau(t, couleur, taille), ChildOf(parent)));
}

fn panneau(commandes: &mut Commands, parent: Entity, largeur: f32, titre: &str) -> Entity {
    let p = commandes
        .spawn((
            Node {
                width: Val::Px(largeur),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(5.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(crate::menu::vue::FOND),
            BorderColor::all(Color::srgb_u8(0x2a, 0x2a, 0x2a)),
            ChildOf(parent),
        ))
        .id();
    texte(commandes, p, titre, OR, 2.0);
    p
}

/// Bouton compact ; `choisi` le met en avant, sans action il est éteint.
fn bouton(
    commandes: &mut Commands,
    parent: Entity,
    t: &str,
    (largeur, hauteur): (Val, f32),
    choisi: bool,
    action: Option<ActionProfil>,
) -> Entity {
    let b = style::bouton(commandes, parent, t, 0.0, false, ());
    let mut e = commandes.entity(b);
    e.entry::<Node>().and_modify(move |mut n| {
        n.width = largeur;
        n.height = Val::Px(hauteur);
        n.padding = UiRect::horizontal(Val::Px(6.0));
        n.column_gap = Val::Px(8.0);
    });
    match action {
        Some(a) => {
            e.insert(ActionMenu::Profil(a));
        }
        None => {
            e.remove::<(Button, style::BoutonPage)>();
            e.insert(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)));
        }
    }
    if choisi {
        let c = crate::menu::talents::CHOISI;
        e.insert((c, BorderColor::all(c.bord), BackgroundColor(c.fond)));
    }
    b
}

/// Visage de la skin (et son calque de chapeau), en tête de `parent`.
pub(in crate::menu) fn visage(
    commandes: &mut Commands,
    ic: &mut Icones,
    parent: Entity,
    chemin: &str,
    cote: f32,
) {
    let image = ic.icone(chemin);
    let s = ic
        .images
        .get(&image)
        .map_or(1.0, |i| i.width() as f32 / 64.0);
    let case = |x: f32| Rect::new(x * s, 8.0 * s, (x + 8.0) * s, 16.0 * s);
    let tete = commandes
        .spawn((
            ImageNode {
                image: image.clone(),
                rect: Some(case(8.0)),
                ..default()
            },
            Node {
                width: Val::Px(cote),
                height: Val::Px(cote),
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .id();
    commandes.spawn((
        ImageNode {
            image,
            rect: Some(case(40.0)),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(cote),
            height: Val::Px(cote),
            ..default()
        },
        ChildOf(tete),
    ));
    commandes.entity(parent).insert_children(0, &[tete]);
}

/// Construit la page ; renvoie sa racine (centre transparent : l'aperçu 3D dessous).
pub fn construire(
    commandes: &mut Commands,
    (dj, sauvegarde, dessins): (&DonneesJeu, &Sauvegarde, &Dessins),
    ic: &mut Icones,
    menu: (usize, Option<&str>, Option<&str>),
) -> Entity {
    let (personnage, saisie, message) = menu;
    let d = dj.d;
    let profil = &sauvegarde.profil;
    let classe = &d.personnages.personnages[personnage];
    let racine = commandes
        .spawn((
            Name::new("profil"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
        ))
        .id();
    texte(commandes, racine, "Profil", OR_CLAIR, 3.0);
    let corps = commandes
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            ChildOf(racine),
        ))
        .id();

    // Skins
    let skins = panneau(commandes, corps, LARGEUR_SKINS, "Skin");
    let mut choix = vec![(
        Skin::Classe,
        "Classe".to_owned(),
        classe.apparence.skin.clone(),
    )];
    for (k, nom) in NOMS_SKINS.iter().enumerate() {
        let chemin = chemin_skin_vanilla(k, profil.bras_fins);
        choix.push((Skin::Vanilla(k), (*nom).to_owned(), chemin));
    }
    if profil.skin_importe.is_some() {
        choix.push((
            Skin::Importe,
            "Importée".to_owned(),
            CHEMIN_SKIN_IMPORTE.into(),
        ));
    }
    let largeur = Val::Percent(100.0);
    for (s, nom, chemin) in choix {
        let choisi = profil.skin == s.choix();
        let b = bouton(
            commandes,
            skins,
            &nom,
            (largeur, 32.0),
            choisi,
            Some(ActionProfil::Skin(s)),
        );
        commandes.entity(b).entry::<Node>().and_modify(|mut n| {
            n.justify_content = JustifyContent::FlexStart;
        });
        visage(commandes, ic, b, &chemin, 24.0);
    }
    let bras = if profil.bras_fins {
        "Bras : fins"
    } else {
        "Bras : larges"
    };
    let action = (profil.skin != ChoixSkin::Classe).then_some(ActionProfil::Bras);
    bouton(commandes, skins, bras, (largeur, 32.0), false, action);
    commandes.spawn((
        TextePixel::nouveau(
            "Importer une skin : glissez un fichier PNG sur la fenêtre.",
            GRIS,
            1.5,
        )
        .bloc(LARGEUR_SKINS - 20.0),
        ChildOf(skins),
    ));
    if let Some(m) = message {
        let couleur = if m.starts_with("Skin refusée") {
            ROUGE
        } else {
            VERT
        };
        commandes.spawn((
            TextePixel::nouveau(m, couleur, 1.5).bloc(LARGEUR_SKINS - 20.0),
            ChildOf(skins),
        ));
    }

    // Centre : pseudo (le héros est dessiné dessous)
    let centre = commandes
        .spawn((
            Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(6.0),
                ..default()
            },
            ChildOf(corps),
        ))
        .id();
    let (t, couleur) = match saisie {
        Some(s) => (format!("{s}_"), JAUNE),
        None if profil.pseudo.is_empty() => ("Sans pseudo".to_owned(), GRIS),
        None => (profil.pseudo.clone(), TEXTE),
    };
    let b = bouton(
        commandes,
        centre,
        "",
        (Val::Auto, 44.0),
        saisie.is_some(),
        Some(ActionProfil::Pseudo),
    );
    commandes.entity(b).despawn_related::<Children>();
    texte(commandes, b, &t, couleur, 2.5);
    let aide = if saisie.is_some() {
        "Entrée : valider · Échap : annuler"
    } else {
        "Cliquez sur le pseudo pour le changer"
    };
    texte(commandes, centre, aide, GRIS, 1.5);
    let nom_skin = match profil.skin {
        ChoixSkin::Classe => "skin de la classe".to_owned(),
        ChoixSkin::Importe => "skin importée".to_owned(),
        ChoixSkin::Vanilla(k) => format!("skin {}", NOMS_SKINS[k]),
    };
    texte(
        commandes,
        centre,
        &format!("{} · {nom_skin}", classe.nom),
        OR_CLAIR,
        2.0,
    );

    // Armure portée
    armure(commandes, (dj, sauvegarde, dessins), ic, corps);

    let retour = style::bouton(
        commandes,
        racine,
        "Retour",
        200.0,
        false,
        ActionMenu::Fermer,
    );
    commandes
        .entity(retour)
        .insert(BackgroundColor(crate::menu::vue::FOND));
    racine
}

/// Une rangée par emplacement : rien, ou la pièce d'un monde (éteinte tant qu'elle n'est
/// pas gagnée) ; puis l'ensemble complet porté.
fn armure(
    commandes: &mut Commands,
    (dj, sauvegarde, dessins): (&DonneesJeu, &Sauvegarde, &Dessins),
    ic: &mut Icones,
    parent: Entity,
) {
    let d = dj.d;
    let col = &sauvegarde.collection;
    let p = panneau(commandes, parent, LARGEUR_ARMURE, "Armure portée");
    for (i, &piece) in PIECES.iter().enumerate() {
        let rangee = commandes
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.0),
                    ..default()
                },
                ChildOf(p),
            ))
            .id();
        let nom = d.armures.noms_pieces.get(&piece).map_or("", |n| n.as_str());
        let etiquette = commandes
            .spawn((
                Node {
                    width: Val::Px(110.0),
                    ..default()
                },
                ChildOf(rangee),
            ))
            .id();
        texte(commandes, etiquette, nom, TEXTE, 1.5);
        let portee = col.portee(piece);
        let rien = Some(ActionProfil::Porter(i, None));
        bouton(
            commandes,
            rangee,
            "—",
            (Val::Px(40.0), 40.0),
            portee.is_none(),
            rien,
        );
        for (k, &monde) in MONDES.iter().enumerate() {
            let possedee = col.possede(monde, piece);
            let action = possedee.then_some(ActionProfil::Porter(i, Some(k)));
            let b = bouton(
                commandes,
                rangee,
                "",
                (Val::Px(40.0), 40.0),
                portee == Some(monde),
                action,
            );
            commandes.entity(b).despawn_related::<Children>();
            icone_piece(commandes, (d, dessins), ic, b, (monde, piece), possedee);
        }
    }
    let t = match col.ensemble_complet() {
        Some(m) => {
            let a = &d.armures.armures[&m];
            format!("{} : {}", a.nom, a.ensemble.description)
        }
        None => "Portez les 7 pièces d'un même monde pour son bonus d'ensemble.".to_owned(),
    };
    let couleur = if col.ensemble_complet().is_some() {
        VERT
    } else {
        GRIS
    };
    commandes.spawn((
        TextePixel::nouveau(&t, couleur, 1.5).bloc(LARGEUR_ARMURE - 20.0),
        ChildOf(p),
    ));
}

fn icone_piece(
    commandes: &mut Commands,
    (d, dessins): (&bs_donnees::Donnees, &Dessins),
    ic: &mut Icones,
    parent: Entity,
    (monde, piece): (Monde, bs_donnees::equipement::Piece),
    possedee: bool,
) {
    let icone = icones_armures::icone(d, dessins, monde, piece, |c| ic.icone(c));
    let eteint = if possedee { 1.0 } else { 0.25 };
    let t = icone.teinte.to_srgba();
    let teinte = Color::srgba(t.red * eteint, t.green * eteint, t.blue * eteint, 1.0);
    let image = commandes
        .spawn((
            ImageNode::new(icone.image).with_color(teinte),
            Node {
                width: Val::Px(28.0),
                height: Val::Px(28.0),
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    if let Some(calque) = icone.calque {
        commandes.spawn((
            ImageNode::new(calque).with_color(Color::srgb(eteint, eteint, eteint)),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(28.0),
                height: Val::Px(28.0),
                ..default()
            },
            ChildOf(image),
        ));
    }
}
