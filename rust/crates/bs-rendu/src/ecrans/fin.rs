//! Écran de fin (docs/CHARTE-UI.md) : titre sur fond doré (victoire) ou rouge (défaite),
//! personnage et arène, tuiles du résumé, bandeau de la maîtrise gagnée, bilan défilant
//! avec son indicateur, « Rejouer » vert avec le focus (Entrée), « Continuer sans fin »
//! après une victoire hors mode sans fin, « Menu principal ».

use bevy::prelude::*;
use bs_regles::etat::Issue;
use bs_regles::partie::Partie;
use bs_sauvegarde::Recompense;

use super::Action;
use super::bilan::{self, Contexte, texte};
use super::charte::{self, AIDE, IVOIRE, OR_SOURD, OR_VIF, Relief, VERT_GAIN};
use super::style;
use super::vues::{bouton, panneau, racine, rangee};

/// Fin de partie : titre, lieu, tuiles, maîtrise, bilan, boutons.
pub fn fin(
    commandes: &mut Commands,
    c: &Contexte,
    p: &Partie,
    recompense: Option<&Recompense>,
    coop: bool,
) -> Entity {
    let e = &p.etat;
    let victoire = e.issue == Some(Issue::Victoire);
    let (haut, bas) = if victoire {
        (
            Color::srgba_u8(90, 70, 0, 107),
            Color::srgba_u8(14, 12, 8, 235),
        )
    } else {
        (
            Color::srgba_u8(70, 0, 0, 115),
            Color::srgba_u8(12, 8, 10, 235),
        )
    };
    let stops = vec![
        ColorStop::new(haut, Val::Percent(0.0)),
        ColorStop::new(bas, Val::Percent(100.0)),
    ];
    let r = racine(
        commandes,
        BackgroundGradient::from(LinearGradient::to_bottom(stops)),
    );
    let (titre, couleur) = if victoire {
        ("Victoire !", OR_VIF)
    } else {
        ("Vous êtes mort !", charte::ROUGE)
    };
    style::titre(commandes, r, titre, couleur, 5.0);
    let lieu = format!("{} · {}", e.personnage.nom, e.arene.nom);
    let record = recompense.is_some_and(|r| r.record_sans_fin);
    let lieu = if record {
        let t = bilan::temps(e.temps_jeu);
        format!("Mode sans fin · {lieu} · nouveau record : {t} !")
    } else if e.sans_fin {
        format!("Mode sans fin · {lieu}")
    } else {
        lieu
    };
    texte(commandes, r, &lieu, if record { OR_VIF } else { AIDE }, 2.0);
    bilan::tuiles(commandes, c, r, p);
    if let Some(rec) = recompense {
        maitrise(commandes, c, r, p, rec);
    }
    panneau(commandes, c, r, p);
    let boutons = rangee(commandes, r);
    if !coop {
        bouton(
            commandes,
            boutons,
            "Rejouer",
            260.0,
            (Relief::PRINCIPAL, true),
            Action::Rejouer,
        );
    }
    if victoire && !e.sans_fin {
        let violet = Relief {
            fond: Color::srgb_u8(0x3a, 0x22, 0x55),
            bord: Color::srgb_u8(0x8a, 0x6c, 0xcf),
            fond_survol: Color::srgb_u8(0x4a, 0x2d, 0x6c),
            bord_survol: Color::srgb_u8(0xb0, 0x94, 0xf0),
            tranche: Color::srgb_u8(0x24, 0x14, 0x36),
        };
        bouton(
            commandes,
            boutons,
            "Continuer sans fin",
            260.0,
            (violet, false),
            Action::Continuer,
        );
    }
    bouton(
        commandes,
        boutons,
        "Menu principal",
        220.0,
        (Relief::SECONDAIRE, false),
        Action::Menu,
    );
    if !coop {
        texte(commandes, r, "Entrée : rejouer", AIDE, 1.5);
    }
    r
}

/// Bandeau de la maîtrise gagnée : fiole, « Classe · Maîtrise N », barre verte du
/// niveau, XP gagnée (et points de talent), déblocages et succès.
fn maitrise(commandes: &mut Commands, c: &Contexte, parent: Entity, p: &Partie, r: &Recompense) {
    let bloc = commandes
        .spawn((
            Node {
                width: Val::Px(800.0),
                max_width: Val::Percent(96.0),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                column_gap: Val::Px(16.0),
                padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(charte::CHOISI),
            BorderColor::all(OR_SOURD),
            ChildOf(parent),
        ))
        .id();
    let icone = c.image(&p.d.talents.maitrise.icone, 36.0);
    commandes.spawn((icone, ChildOf(bloc)));
    let textes = commandes
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                row_gap: Val::Px(6.0),
                ..default()
            },
            ChildOf(bloc),
        ))
        .id();
    let a = r.apres;
    let entete = commandes
        .spawn((
            Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::End,
                ..default()
            },
            ChildOf(textes),
        ))
        .id();
    let nom = format!("{} · Maîtrise {}", p.etat.personnage.nom, a.niveau);
    texte(commandes, entete, &nom, IVOIRE, 2.0);
    let xp = bs_regles::textes::libelles::nombre(r.xp as f32);
    texte(commandes, entete, &format!("+{xp} XP"), VERT_GAIN, 2.0);
    let barre = commandes
        .spawn((
            Node {
                height: Val::Px(10.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(charte::CREUX),
            BorderColor::all(charte::BORD),
            ChildOf(textes),
        ))
        .id();
    let part = (a.xp - a.debut_niveau) as f32 / (a.fin_niveau - a.debut_niveau).max(1) as f32;
    commandes.spawn((
        Node {
            width: Val::Percent(part.clamp(0.0, 1.0) * 100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(VERT_GAIN),
        ChildOf(barre),
    ));
    let niveaux = a.niveau - r.avant.niveau;
    if niveaux > 0 {
        let s = if niveaux > 1 { "s" } else { "" };
        let t = format!(
            "Niveau {} → {} : +{niveaux} point{s} de talent !",
            r.avant.niveau, a.niveau
        );
        texte(commandes, textes, &t, OR_VIF, 1.5);
    }
    for &i in &r.deblocages {
        let t = format!("Débloqué : {} !", p.d.talents.deblocages[i].nom);
        texte(commandes, textes, &t, OR_VIF, 1.5);
    }
    for &i in &r.succes {
        let s = &p.d.succes[i];
        let t = format!("Succès : {} ({})", s.nom, s.texte);
        texte(commandes, textes, &t, OR_VIF, 1.5);
    }
}
