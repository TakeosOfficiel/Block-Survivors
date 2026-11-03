//! Volet des classes de l'accueil : à gauche une ligne par classe (icône de son arme,
//! nom, rôle, ou cadenas et condition), la choisie marquée d'un liseré d'or et d'une
//! coche ; à droite sa fiche (description, vie, vitesse, arme, bonus, cartes
//! exclusives), sa maîtrise et le bouton des Talents.

use bevy::prelude::*;
use bs_donnees::progression::TypeDeblocage;
use bs_regles::textes::libelles::{lignes_bonus, nombre};

use super::{Pinceau, place};
use crate::ecrans::charte::{self, AIDE, BORD, IVOIRE, OR_VIF, Relief, T_AIDE, T_CORPS, VERT_GAIN};
use crate::lancement::{ChoixPartie, DonneesJeu};
use crate::menu::ActionMenu;
use crate::sauvegarde::Sauvegarde;

const COEUR: &str = "gui/sprites/hud/heart/full";
const HAUTEUR_LIGNE: f32 = 52.0;

pub(super) fn construire(
    p: &mut Pinceau,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    choix: ChoixPartie,
    (parent, detail): (Entity, Entity),
) {
    let d = dj.d;
    p.section(parent, "Classe");
    let liste = p.noeud(
        parent,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        },
    );
    for (k, (id, perso)) in d.personnages.personnages.iter().enumerate() {
        let verrou = sauvegarde.progression.verrou(TypeDeblocage::Personnage, id);
        let choisie = k == choix.personnage;
        let relief = if choisie {
            Relief::CHOISI
        } else {
            Relief::SECONDAIRE
        };
        let mut ligne = p.c.spawn((
            Node {
                height: Val::Px(HAUTEUR_LIGNE),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                padding: UiRect::horizontal(Val::Px(8.0)),
                border: charte::bord_relief(),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            relief.composants(),
            ChildOf(liste),
        ));
        if verrou.is_none() {
            ligne.insert((Button, ActionMenu::Classe(k)));
        }
        let ligne = ligne.id();
        if choisie {
            lisere(p, ligne);
        }
        let icone = perso.apparence.objet_en_main.as_deref().or_else(|| {
            perso
                .armes_depart
                .first()
                .and_then(|a| d.armes.get(a))
                .map(|a| a.icone.as_str())
        });
        if let Some(i) = icone {
            let e = p.icone(ligne, i, 32.0);
            if verrou.is_some() {
                p.c.entity(e)
                    .entry::<ImageNode>()
                    .and_modify(|mut i| i.color = Color::srgb(0.35, 0.35, 0.35));
            }
        }
        let textes = p.noeud(
            ligne,
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                flex_shrink: 1.0,
                row_gap: Val::Px(1.0),
                ..default()
            },
        );
        p.texte(textes, &perso.nom, true, T_CORPS + 1.0, IVOIRE);
        match verrou {
            Some(v) => {
                p.texte(textes, &v.texte, false, T_AIDE - 1.0, AIDE);
                charte::cadenas(p.c, ligne, 14.0, AIDE);
            }
            None => {
                p.texte(textes, &perso.role, false, T_AIDE, AIDE);
                if choisie {
                    p.texte(ligne, "✓", true, 20.0, OR_VIF);
                }
            }
        }
    }
    fiche(p, dj, choix, detail);
    maitrise(p, dj, sauvegarde, choix, detail);
}

/// Liseré d'or de 4 au bord gauche d'une ligne choisie.
pub(super) fn lisere(p: &mut Pinceau, ligne: Entity) {
    let l = p.noeud(
        ligne,
        Node {
            width: Val::Px(4.0),
            ..place(
                Some(Val::Px(-2.0)),
                None,
                Some(Val::Px(-2.0)),
                Some(Val::Px(-4.0)),
            )
        },
    );
    p.c.entity(l).insert(BackgroundColor(OR_VIF));
}

/// Ligne « libellé … valeur » de la fiche, icône facultative devant la valeur.
fn propriete(p: &mut Pinceau, parent: Entity, libelle: &str, valeur: &str, icone: Option<&str>) {
    let l = p.noeud(
        parent,
        Node {
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            min_height: Val::Px(24.0),
            ..default()
        },
    );
    p.texte(l, libelle, false, T_AIDE, AIDE);
    let v = p.noeud(
        l,
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
            ..default()
        },
    );
    if let Some(i) = icone {
        p.icone(v, i, 18.0);
    }
    p.texte(v, valeur, true, T_CORPS - 1.0, IVOIRE);
}

/// Fiche de la classe choisie : description, vie, vitesse, arme, bonus, exclusifs.
fn fiche(p: &mut Pinceau, dj: &DonneesJeu, choix: ChoixPartie, parent: Entity) {
    let d = dj.d;
    let perso = &d.personnages.personnages[choix.personnage];
    let f = p.noeud(
        parent,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        },
    );
    p.texte(f, &perso.nom, true, charte::T_SECTION, IVOIRE);
    let desc = p.texte(f, &perso.description, false, T_AIDE, AIDE);
    p.c.entity(desc).insert(Node {
        margin: UiRect::bottom(Val::Px(6.0)),
        ..default()
    });
    propriete(p, f, "Vie", &nombre(perso.pv / 2.0), Some(COEUR));
    propriete(p, f, "Vitesse", &nombre(perso.vitesse), None);
    if let Some(a) = perso.armes_depart.first().and_then(|a| d.armes.get(a)) {
        propriete(p, f, "Arme", &a.nom, Some(&a.icone));
    }
    for l in lignes_bonus(&perso.bonus) {
        p.texte(f, &l, true, T_AIDE, VERT_GAIN);
    }
    if !perso.cartes_exclusives.is_empty() {
        let r = p.noeud(
            f,
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },
        );
        p.texte(r, "Bonus exclusifs", false, T_AIDE, AIDE);
        let icones = p.noeud(
            r,
            Node {
                margin: UiRect::left(Val::Auto),
                column_gap: Val::Px(6.0),
                ..default()
            },
        );
        for c in &perso.cartes_exclusives {
            if let Some(passif) = d.passifs.get(c) {
                p.case(icones, &passif.icone, 32.0, 24.0);
            }
        }
    }
}

/// Maîtrise de la classe (niveau, points libres, barre d'XP) et bouton des Talents.
fn maitrise(
    p: &mut Pinceau,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    choix: ChoixPartie,
    parent: Entity,
) {
    let d = dj.d;
    let (id, _) = d
        .personnages
        .personnages
        .get_index(choix.personnage)
        .expect("classe choisie");
    let prog = &sauvegarde.progression;
    let m = prog.maitrise(id);
    let arbre = prog.arbre_actif(id);
    let libres = prog.points_libres(id, arbre);
    let bloc = p.noeud(
        parent,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            margin: UiRect::top(Val::Px(16.0)),
            padding: UiRect::top(Val::Px(12.0)),
            border: UiRect::top(Val::Px(2.0)),
            ..default()
        },
    );
    p.c.entity(bloc).insert(BorderColor::all(BORD));
    let ligne = p.noeud(
        bloc,
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        },
    );
    p.icone(ligne, &d.talents.maitrise.icone, 20.0);
    let niveau = format!("Maîtrise {}", m.niveau);
    p.texte(ligne, &niveau, true, T_CORPS - 1.0, IVOIRE);
    let s = if libres > 1 { "s" } else { "" };
    let couleur = if libres > 0 { OR_VIF } else { AIDE };
    let points = p.texte(ligne, &format!("{libres} point{s}"), true, T_AIDE, couleur);
    p.c.entity(points).insert(Node {
        margin: UiRect::left(Val::Auto),
        ..default()
    });
    barre_xp(p, bloc, &m);
    let nom = d.talents.arbres.get(id).and_then(|a| a.get(arbre));
    let t = nom.map_or("Talents".to_owned(), |a| format!("Talents · {}", a.nom));
    let b = p.bouton(bloc, &t, Relief::SECONDAIRE, ActionMenu::Talents);
    p.c.entity(b).entry::<Node>().and_modify(|mut n| {
        n.margin = UiRect::top(Val::Px(6.0));
    });
}

/// Barre d'XP de maîtrise, verte sur fond creusé.
fn barre_xp(p: &mut Pinceau, parent: Entity, m: &bs_sauvegarde::progression::EtatMaitrise) {
    let barre = p.noeud(
        parent,
        Node {
            height: Val::Px(10.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
    );
    p.c.entity(barre).insert((
        BackgroundColor(charte::FOND),
        BorderColor::all(Color::BLACK),
    ));
    let etendue = m.fin_niveau.saturating_sub(m.debut_niveau).max(1) as f32;
    let part = (m.xp.saturating_sub(m.debut_niveau) as f32 / etendue).clamp(0.0, 1.0);
    let rempli = p.noeud(
        barre,
        Node {
            width: Val::Percent(part * 100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    );
    p.c.entity(rempli).insert(BackgroundColor(VERT_GAIN));
}
