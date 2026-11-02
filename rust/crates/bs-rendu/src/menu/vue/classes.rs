//! Colonne des classes de l'accueil (src/ui/ecranMenu.ts `afficherClasse`) : une ligne
//! par classe (icône de son arme, nom, rôle ou verrou), puis la description de la
//! choisie, ses pastilles (cœurs, vitesse, arme), ses bonus, ses cartes exclusives et
//! les talents achetés de son arbre actif.

use bevy::prelude::*;
use bs_donnees::progression::TypeDeblocage;
use bs_regles::textes::libelles::{lignes_bonus, nombre};

use super::{BLANC, GRIS_CLAIR, GRIS_MOYEN, ORE_TRANSPARENT, Pinceau, VERRE};
use crate::ecrans::style::{BoutonPage, OR, OR_CLAIR, Teinte};
use crate::lancement::{ChoixPartie, DonneesJeu};
use crate::menu::ActionMenu;
use crate::sauvegarde::Sauvegarde;

const COEUR: &str = "gui/sprites/hud/heart/full";
const VERT_BONUS: Color = Color::srgb_u8(0x6d, 0xff, 0x6d);
const ROLE: Color = Color::srgb_u8(0xb8, 0xb8, 0xb8);
const VERROU: Color = Color::srgb_u8(0xff, 0xb0, 0xa0);
const CHOISIE: Teinte = Teinte {
    bord: OR,
    fond: Color::srgba(1.0, 0.843, 0.369, 0.12),
};
const LIBRE: Teinte = Teinte {
    bord: ORE_TRANSPARENT,
    fond: ORE_TRANSPARENT,
};

/// Rangée qui passe à la ligne.
fn rangee(p: &mut Pinceau, parent: Entity, ecart: f32, haut: f32) -> Entity {
    p.noeud(
        parent,
        Node {
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            column_gap: Val::Px(ecart),
            row_gap: Val::Px(ecart),
            margin: UiRect::top(Val::Px(haut)),
            ..default()
        },
    )
}

pub(super) fn construire(
    p: &mut Pinceau,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    choix: ChoixPartie,
    parent: Entity,
) {
    let d = dj.d;
    p.section(parent, "Classe");
    for (k, (id, perso)) in d.personnages.personnages.iter().enumerate() {
        let verrou = sauvegarde.progression.verrou(TypeDeblocage::Personnage, id);
        let choisie = k == choix.personnage;
        let teinte = if choisie { CHOISIE } else { LIBRE };
        let mut ligne = p.c.spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            },
            teinte,
            BorderColor::all(teinte.bord),
            BackgroundColor(teinte.fond),
            ChildOf(parent),
        ));
        if verrou.is_none() {
            ligne.insert((Button, BoutonPage, ActionMenu::Classe(k)));
        }
        let ligne = ligne.id();
        let icone = perso.apparence.objet_en_main.as_deref().or_else(|| {
            perso
                .armes_depart
                .first()
                .and_then(|a| d.armes.get(a))
                .map(|a| a.icone.as_str())
        });
        if let Some(i) = icone {
            p.case(ligne, i, 44.0, 32.0);
        }
        let textes = p.noeud(
            ligne,
            Node {
                flex_direction: FlexDirection::Column,
                flex_shrink: 1.0,
                row_gap: Val::Px(2.0),
                ..default()
            },
        );
        let couleur = if choisie { OR_CLAIR } else { BLANC };
        p.texte(textes, &perso.nom, true, 16.0, couleur);
        match verrou {
            Some(v) => p.texte(
                textes,
                &format!("Verrouillée · {}", v.texte),
                false,
                13.0,
                VERROU,
            ),
            None => p.texte(textes, &perso.role, false, 13.0, ROLE),
        };
    }
    details(p, dj, sauvegarde, choix, parent);
}

/// Description, pastilles, bonus et cartes exclusives de la classe choisie.
fn details(
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
    let desc = p.texte(parent, &perso.description, false, 15.0, GRIS_CLAIR);
    p.c.entity(desc).insert(Node {
        margin: UiRect::top(Val::Px(14.0)),
        ..default()
    });
    let puces = rangee(p, parent, 6.0, 10.0);
    p.puce(puces, &nombre(perso.pv / 2.0), Some(COEUR), VERRE, BLANC);
    p.puce(
        puces,
        &format!("Vitesse {}", nombre(perso.vitesse)),
        None,
        VERRE,
        BLANC,
    );
    if let Some(a) = perso.armes_depart.first().and_then(|a| d.armes.get(a)) {
        p.puce(puces, &a.nom, Some(&a.icone), VERRE, BLANC);
    }
    for l in lignes_bonus(&perso.bonus) {
        let e = p.texte(parent, &l, false, 14.0, VERT_BONUS);
        p.c.entity(e).insert(Node {
            margin: UiRect::top(Val::Px(4.0)),
            ..default()
        });
    }
    if !perso.cartes_exclusives.is_empty() {
        p.intitule(parent, "Bonus exclusifs");
        let r = rangee(p, parent, 6.0, 6.0);
        for c in &perso.cartes_exclusives {
            if let Some(passif) = d.passifs.get(c) {
                p.case(r, &passif.icone, 36.0, 28.0);
            }
        }
    }
    // Talents achetés de l'arbre actif
    let prog = &sauvegarde.progression;
    let index = prog.arbre_actif(id);
    let Some(arbre) = d.talents.arbres.get(id).and_then(|a| a.get(index)) else {
        return;
    };
    let rangs = prog.points(id, index, None);
    let s = if rangs > 1 { "s" } else { "" };
    p.intitule(
        parent,
        &format!("Talents : {} ({rangs} rang{s})", arbre.nom),
    );
    let r = rangee(p, parent, 6.0, 6.0);
    let mut aucun = true;
    for n in arbre.noeuds.iter().filter(|n| prog.actif(id, index, n)) {
        aucun = false;
        let icone = n
            .options
            .as_ref()
            .map_or(&n.icone, |o| &o[prog.option(id, index, n)].icone);
        let case = p.case(r, icone, 36.0, 28.0);
        if n.rang_max > 1 {
            let rang = prog.rang(id, index, n).to_string();
            let badge = p.noeud(
                case,
                super::place(None, Some(Val::Px(2.0)), None, Some(Val::Px(0.0))),
            );
            p.texte(badge, &rang, true, 12.0, BLANC);
        }
    }
    if aucun {
        p.texte(
            r,
            "Aucun talent acheté : jouez cette classe pour monter sa maîtrise.",
            false,
            13.0,
            GRIS_MOYEN,
        );
    }
}
