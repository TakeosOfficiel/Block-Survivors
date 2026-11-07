//! Construction de la page Profil, à la manière des pages « console » (`console`), sans
//! cadre ni case : à gauche le pseudo en grand (clic pour le changer), la classe, les
//! skins en visages dans des anneaux et le gabarit des bras en pilules ; au centre, le
//! héros 3D tourne sous un halo (fond transparent : la caméra de l'aperçu est dessous) ;
//! à droite l'armure portée, une rangée de médaillons ronds par emplacement.

use bevy::prelude::*;
use bs_donnees::arenes::Monde;
use bs_sauvegarde::collection::{MONDES, PIECES};
use bs_sauvegarde::profil_joueur::{
    CHEMIN_SKIN_IMPORTE, ChoixSkin, NOMS_SKINS, chemin_skin_vanilla,
};

use super::{ActionProfil, Skin};
use crate::dessins::Dessins;
use crate::ecrans::charte::{AIDE, IVOIRE, OR_VIF, VERT_GAIN};
use crate::icones_armures;
use crate::lancement::DonneesJeu;
use crate::menu::console::{self, Police, TERNE, aide, ecrit, pilule, surtitre};
use crate::menu::vue::Lien;
use crate::menu::{ActionMenu, Icones, air};
use crate::sauvegarde::Sauvegarde;

const ROUGE: Color = Color::srgb_u8(0xff, 0x8a, 0x8a);
/// Largeur des colonnes de part et d'autre du héros.
const COTE: f32 = 470.0;

fn a(action: ActionProfil) -> Option<ActionMenu> {
    Some(ActionMenu::Profil(action))
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

/// Anneau rond (doré si choisi) prêt à recevoir une icône ; cliquable si `action`.
fn anneau(
    c: &mut Commands,
    parent: Entity,
    (cote, choisi): (f32, bool),
    action: Option<ActionMenu>,
) -> Entity {
    let (bord, fond) = match (choisi, action.is_some()) {
        (true, _) => (OR_VIF, OR_VIF.with_alpha(0.12)),
        (false, true) => (
            Color::srgba(1.0, 1.0, 1.0, 0.2),
            Color::srgba(1.0, 1.0, 1.0, 0.04),
        ),
        (false, false) => (Color::srgba(1.0, 1.0, 1.0, 0.06), Color::NONE),
    };
    let r = air::point(c, parent, cote, fond);
    c.entity(r)
        .insert(BorderColor::all(bord))
        .entry::<Node>()
        .and_modify(|mut n| {
            n.border = UiRect::all(Val::Px(2.0));
            n.justify_content = JustifyContent::Center;
            n.align_items = AlignItems::Center;
        });
    if let Some(a) = action {
        c.entity(r).insert((Button, a));
    }
    r
}

/// Colonne de gauche : pseudo, classe, skins, bras.
fn gauche(
    c: &mut Commands,
    (dj, sauvegarde): (&DonneesJeu, &Sauvegarde),
    ic: &mut Icones,
    parent: Entity,
    (personnage, saisie, message): (usize, Option<&str>, Option<&str>),
) {
    let profil = &sauvegarde.profil;
    let classe = &dj.d.personnages.personnages[personnage];
    surtitre(c, ic, parent, "PSEUDO");
    let b = air::rangee(c, parent, 14.0);
    c.entity(b).insert((
        Button,
        Lien(OR_VIF),
        ActionMenu::Profil(ActionProfil::Pseudo),
    ));
    let (t, couleur) = match saisie {
        Some(s) => (format!("{s}|"), IVOIRE),
        None if profil.pseudo.is_empty() => ("Sans pseudo".to_owned(), TERNE),
        None => (profil.pseudo.clone(), OR_VIF),
    };
    ecrit(c, ic, b, &t, (Police::Logo, 56.0, couleur));
    let t = if saisie.is_some() {
        "Entrée : valider · Échap : annuler"
    } else {
        "Clique sur ton pseudo pour le changer"
    };
    aide(c, ic, parent, t);
    let nom_skin = match profil.skin {
        ChoixSkin::Classe => "skin de la classe".to_owned(),
        ChoixSkin::Importe => "skin importée".to_owned(),
        ChoixSkin::Vanilla(k) => format!("skin {}", NOMS_SKINS[k]),
    };
    let t = format!("{} · {nom_skin}", classe.nom);
    ecrit(
        c,
        ic,
        parent,
        &t,
        (Police::Grasse, 16.0, IVOIRE.with_alpha(0.8)),
    );

    let s = air::colonne(c, parent, 12.0);
    c.entity(s)
        .entry::<Node>()
        .and_modify(|mut n| n.margin = UiRect::top(Val::Px(26.0)));
    surtitre(c, ic, s, "SKIN");
    let mut choix = vec![(Skin::Classe, "Classe", classe.apparence.skin.clone())];
    for (k, nom) in NOMS_SKINS.iter().enumerate() {
        choix.push((
            Skin::Vanilla(k),
            *nom,
            chemin_skin_vanilla(k, profil.bras_fins),
        ));
    }
    if profil.skin_importe.is_some() {
        choix.push((Skin::Importe, "Importée", CHEMIN_SKIN_IMPORTE.into()));
    }
    let g = c
        .spawn((
            Node {
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::px(6, 66.0),
                row_gap: Val::Px(12.0),
                column_gap: Val::Px(8.0),
                ..default()
            },
            ChildOf(s),
        ))
        .id();
    for (skin, nom, chemin) in choix {
        let choisi = profil.skin == skin.choix();
        let e = air::colonne(c, g, 5.0);
        c.entity(e)
            .entry::<Node>()
            .and_modify(|mut n| n.align_items = AlignItems::Center);
        let couleur = if choisi { OR_VIF } else { TERNE };
        if !choisi {
            let action = ActionMenu::Profil(ActionProfil::Skin(skin));
            c.entity(e).insert((Button, Lien(couleur), action));
        }
        let r = anneau(c, e, (52.0, choisi), None);
        visage(c, ic, r, &chemin, 28.0);
        ecrit(c, ic, e, nom, (Police::Grasse, 12.0, couleur));
    }
    let r = air::rangee(c, s, 10.0);
    c.entity(r)
        .entry::<Node>()
        .and_modify(|mut n| n.margin = UiRect::top(Val::Px(6.0)));
    let libre = profil.skin != ChoixSkin::Classe;
    for (fins, t) in [(false, "Bras larges"), (true, "Bras fins")] {
        let choisie = libre && profil.bras_fins == fins;
        let action = (libre && !choisie).then_some(ActionMenu::Profil(ActionProfil::Bras));
        pilule(c, ic, r, t, (choisie, action));
    }
    aide(
        c,
        ic,
        s,
        "Importer une skin : glisse un fichier PNG sur la fenêtre.",
    );
    if let Some(m) = message {
        let couleur = if m.starts_with("Skin refusée") {
            ROUGE
        } else {
            VERT_GAIN
        };
        ecrit(c, ic, s, m, (Police::Grasse, 14.0, couleur));
    }
}

/// Colonne de droite : une rangée par emplacement (rien, ou la pièce d'un monde, éteinte
/// tant qu'elle n'est pas gagnée), puis l'ensemble complet porté.
fn armure(
    c: &mut Commands,
    (dj, sauvegarde, dessins): (&DonneesJeu, &Sauvegarde, &Dessins),
    ic: &mut Icones,
    parent: Entity,
) {
    let d = dj.d;
    let col = &sauvegarde.collection;
    surtitre(c, ic, parent, "ARMURE PORTÉE");
    for (i, &piece) in PIECES.iter().enumerate() {
        let rangee = air::rangee(c, parent, 10.0);
        let nom = d.armures.noms_pieces.get(&piece).map_or("", |n| n.as_str());
        let e = ecrit(
            c,
            ic,
            rangee,
            nom,
            (Police::Grasse, 15.0, IVOIRE.with_alpha(0.85)),
        );
        c.entity(e).insert(Node {
            width: Val::Px(110.0),
            ..default()
        });
        let portee = col.portee(piece);
        let rien = anneau(
            c,
            rangee,
            (42.0, portee.is_none()),
            a(ActionProfil::Porter(i, None)),
        );
        let couleur = if portee.is_none() { OR_VIF } else { AIDE };
        ecrit(c, ic, rien, "—", (Police::Grasse, 16.0, couleur));
        for (k, &monde) in MONDES.iter().enumerate() {
            let possedee = col.possede(monde, piece);
            let action = possedee.then_some(ActionMenu::Profil(ActionProfil::Porter(i, Some(k))));
            let r = anneau(c, rangee, (42.0, portee == Some(monde)), action);
            icone_piece(c, (d, dessins), ic, r, (monde, piece), possedee);
        }
    }
    let (t, couleur) = match col.ensemble_complet() {
        Some(m) => {
            let a = &d.armures.armures[&m];
            (
                format!("★ {} : {}", a.nom, a.ensemble.description),
                VERT_GAIN,
            )
        }
        None => (
            "Porte les 7 pièces d'un même monde pour son bonus d'ensemble.".to_owned(),
            AIDE,
        ),
    };
    let e = ecrit(c, ic, parent, &t, (Police::Normale, 14.0, couleur));
    c.entity(e).insert(Node {
        margin: UiRect::top(Val::Px(6.0)),
        ..default()
    });
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
    let (eteint, alpha) = if possedee { (1.0, 1.0) } else { (0.35, 0.6) };
    let t = icone.teinte.to_srgba();
    let teinte = Color::srgba(t.red * eteint, t.green * eteint, t.blue * eteint, alpha);
    let taille = Node {
        width: Val::Px(26.0),
        height: Val::Px(26.0),
        ..default()
    };
    let image = commandes
        .spawn((
            ImageNode::new(icone.image).with_color(teinte),
            taille.clone(),
            ChildOf(parent),
        ))
        .id();
    if let Some(calque) = icone.calque {
        commandes.spawn((
            ImageNode::new(calque).with_color(Color::srgba(eteint, eteint, eteint, alpha)),
            Node {
                position_type: PositionType::Absolute,
                ..taille
            },
            ChildOf(image),
        ));
    }
}

/// Construit la page ; renvoie sa racine (centre transparent : l'aperçu 3D dessous).
pub fn construire(
    commandes: &mut Commands,
    (dj, sauvegarde, dessins): (&DonneesJeu, &Sauvegarde, &Dessins),
    ic: &mut Icones,
    menu: (usize, Option<&str>, Option<&str>),
) -> Entity {
    let c = commandes;
    let racine = console::racine(c, ic, ("profil", "PROFIL"));
    // Halo : le centre laisse voir le héros, la nuit s'assombrit vers les bords
    let nuit =
        |a: f32, p: f32| ColorStop::new(Color::srgba(0.008, 0.012, 0.03, a), Val::Percent(p));
    c.entity(racine)
        .remove::<BackgroundColor>()
        .insert(BackgroundGradient::from(RadialGradient::new(
            UiPosition::CENTER,
            RadialGradientShape::FarthestCorner,
            vec![
                nuit(0.0, 0.0),
                nuit(0.35, 30.0),
                nuit(0.95, 75.0),
                nuit(1.0, 100.0),
            ],
        )));
    let milieu = air::rangee(c, racine, 0.0);
    c.entity(milieu).entry::<Node>().and_modify(|mut n| {
        n.flex_grow = 1.0;
        n.min_height = Val::ZERO;
        n.justify_content = JustifyContent::SpaceBetween;
        n.align_items = AlignItems::Center;
    });
    let g = air::colonne(c, milieu, 10.0);
    c.entity(g)
        .entry::<Node>()
        .and_modify(|mut n| n.width = Val::Px(COTE));
    gauche(c, (dj, sauvegarde), ic, g, menu);
    let d = air::colonne(c, milieu, 10.0);
    c.entity(d).entry::<Node>().and_modify(|mut n| {
        n.width = Val::Px(COTE);
        n.align_items = AlignItems::FlexEnd;
    });
    armure(c, (dj, sauvegarde, dessins), ic, d);
    let p = console::pied(c, ic, racine);
    aide(c, ic, p, "Chaque changement est enregistré aussitôt.");
    racine
}
