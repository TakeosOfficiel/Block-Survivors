//! Vue de la page Multijoueur : hors salon (créer, code, salons publics) ou dans le
//! salon (code, joueurs, lancement), et le message de la dernière coop arrêtée ; la
//! panneau des amis à droite (`vue_amis`) ; hors salon, les succès de coop (repliés).
//! Couleurs et boutons de la charte (docs/CHARTE-UI.md).

use bevy::prelude::*;
use bs_reseau::PhaseCoop;

use bs_sauvegarde::amis::Ami;
use bs_sauvegarde::progression::Progression;

use super::{ActionMulti, Annuaire};
use crate::amis::ReseauAmis;
use crate::coop::Coop;
use crate::ecrans::charte::{self, AIDE, BORD, IVOIRE, OR_VIF, Relief};
use crate::hud::police::TextePixel;
use crate::lancement::{DonneesJeu, SourceMenu};
use crate::menu::{ActionMenu, Menu};

const LARGEUR: f32 = 560.0;
const ROUGE: Color = charte::ROUGE;

/// Ce que la page montre de la coop.
pub struct VueMulti<'a> {
    pub coop: Option<&'a Coop>,
    pub message: Option<&'a str>,
    pub annuaire: &'a Annuaire,
    pub amis: Option<&'a ReseauAmis>,
    pub liste: &'a [Ami],
    pub progression: &'a Progression,
}

pub(super) fn texte(
    commandes: &mut Commands,
    parent: Entity,
    t: &str,
    couleur: Color,
    taille: f32,
) {
    commandes.spawn((TextePixel::nouveau(t, couleur, taille), ChildOf(parent)));
}

/// Bouton en relief (`relief`), éteint sans action.
pub(super) fn bouton_teinte(
    commandes: &mut Commands,
    parent: Entity,
    t: &str,
    relief: Relief,
    action: Option<ActionMulti>,
) -> Entity {
    let b = charte::bouton_relief(commandes, parent, relief, 40.0, ());
    commandes.entity(b).entry::<Node>().and_modify(|mut n| {
        n.padding = UiRect::horizontal(Val::Px(12.0));
        n.flex_shrink = 0.0;
    });
    let couleur = if action.is_some() { IVOIRE } else { AIDE };
    texte(commandes, b, t, couleur, 1.75);
    match action {
        Some(a) => {
            commandes.entity(b).insert(ActionMenu::Multi(a));
        }
        None => {
            commandes
                .entity(b)
                .remove::<(Button, Relief)>()
                .insert(BackgroundColor(charte::CREUX));
        }
    }
    b
}

pub(super) fn bouton(
    commandes: &mut Commands,
    parent: Entity,
    t: &str,
    action: Option<ActionMulti>,
) {
    bouton_teinte(commandes, parent, t, Relief::SECONDAIRE, action);
}

/// Champ de saisie en creux, cliquable (`action`).
pub(super) fn champ(commandes: &mut Commands, parent: Entity, t: &str, action: ActionMulti) {
    let c = commandes
        .spawn((
            Button,
            ActionMenu::Multi(action),
            Node {
                flex_grow: 1.0,
                height: Val::Px(40.0),
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(0x0b, 0x0e, 0x10)),
            BorderColor {
                top: Color::BLACK,
                left: Color::BLACK,
                right: BORD,
                bottom: BORD,
            },
            ChildOf(parent),
        ))
        .id();
    texte(commandes, c, t, IVOIRE, 1.75);
}

/// Ligne posée dans un panneau (surface, bord de 2).
pub(super) fn ligne(commandes: &mut Commands, parent: Entity, _largeur: f32) -> Entity {
    commandes
        .spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                flex_shrink: 0.0,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(charte::SURFACE),
            BorderColor::all(BORD),
            ChildOf(parent),
        ))
        .id()
}

/// Rangée sans fond (boutons côte à côte).
fn rangee(commandes: &mut Commands, parent: Entity) -> Entity {
    commandes
        .spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(parent),
        ))
        .id()
}

/// Titre de section d'un panneau, filet dessous.
pub(super) fn section(commandes: &mut Commands, parent: Entity, t: &str) {
    let s = commandes
        .spawn((
            Node {
                margin: UiRect::top(Val::Px(4.0)),
                padding: UiRect::bottom(Val::Px(6.0)),
                border: UiRect::bottom(Val::Px(2.0)),
                flex_shrink: 0.0,
                ..default()
            },
            BorderColor::all(BORD),
            ChildOf(parent),
        ))
        .id();
    texte(commandes, s, t, IVOIRE, 2.0);
}

/// Hors salon : créer (visibilité au choix), rejoindre par code, salons publics.
fn accueil(commandes: &mut Commands, racine: Entity, menu: &Menu, vue: &VueMulti, saisie: bool) {
    section(commandes, racine, "Créer un salon");
    let public = vue.annuaire.public;
    let l = rangee(commandes, racine);
    for (t, p) in [("Privé", false), ("Public", true)] {
        let (relief, t) = if p == public {
            (Relief::CHOISI, format!("✓ {t}"))
        } else {
            (Relief::SECONDAIRE, t.to_owned())
        };
        let b = bouton_teinte(commandes, l, &t, relief, Some(ActionMulti::Visibilite(p)));
        commandes
            .entity(b)
            .entry::<Node>()
            .and_modify(|mut n| n.width = Val::Px(120.0));
    }
    let b = bouton_teinte(
        commandes,
        l,
        "Créer le salon",
        Relief::PRINCIPAL,
        Some(ActionMulti::Creer),
    );
    commandes
        .entity(b)
        .entry::<Node>()
        .and_modify(|mut n| n.margin = UiRect::left(Val::Auto));
    let aide = if public {
        "Public : visible de tous dans la liste des salons."
    } else {
        "Privé : on n'entre qu'avec le code du salon."
    };
    texte(commandes, racine, aide, AIDE, 1.5);

    section(commandes, racine, "Rejoindre par code");
    let l = rangee(commandes, racine);
    let valeur = match &menu.saisie {
        Some(s) if saisie => format!("{s}|"),
        _ => "Taper le code".into(),
    };
    champ(commandes, l, &valeur, ActionMulti::Code);
    bouton(commandes, l, "Rejoindre", Some(ActionMulti::Rejoindre));
    if menu.saisie.is_some() && saisie {
        let t = "Entrée : rejoindre · Échap : annuler";
        texte(commandes, racine, t, AIDE, 1.5);
    }

    let l = rangee(commandes, racine);
    commandes.entity(l).insert(Node {
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        flex_shrink: 0.0,
        ..default()
    });
    section(commandes, l, "Salons publics");
    let recherche = vue.annuaire.session.is_some();
    let t = if recherche {
        "Recherche…"
    } else {
        "Actualiser"
    };
    bouton(
        commandes,
        l,
        t,
        (!recherche).then_some(ActionMulti::Actualiser),
    );
    if vue.annuaire.salons.is_empty() && !recherche {
        texte(commandes, racine, "Aucun salon public ouvert.", AIDE, 1.5);
    }
    for (k, s) in vue.annuaire.salons.iter().enumerate() {
        let l = ligne(commandes, racine, LARGEUR);
        let hote = String::from_utf8_lossy(&s.annonce);
        let t = format!("{hote} · {}/{} joueurs", s.joueurs, s.places);
        let nom = commandes
            .spawn((
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
                ChildOf(l),
            ))
            .id();
        texte(commandes, nom, &t, IVOIRE, 1.75);
        let plein = s.joueurs >= s.places;
        bouton(
            commandes,
            l,
            "Rejoindre",
            (!plein).then_some(ActionMulti::Public(k)),
        );
    }
}

/// Succès de coop : replié en « Succès coop · 2/5 », déplié au clic (gagnés en or, les
/// autres avec leur condition).
fn succes(commandes: &mut Commands, racine: Entity, dj: &DonneesJeu, vue: &VueMulti) {
    let (liste, p) = (&dj.d.succes, vue.progression);
    if liste.is_empty() {
        return;
    }
    let gagnes = liste.iter().filter(|s| p.succes(&s.id)).count();
    let ouverts = vue.annuaire.succes_ouverts;
    let fleche = if ouverts { "⌄" } else { "›" };
    let t = format!("Succès coop · {gagnes}/{} {fleche}", liste.len());
    let b = bouton_teinte(
        commandes,
        racine,
        &t,
        Relief::SECONDAIRE,
        Some(ActionMulti::Succes),
    );
    commandes
        .entity(b)
        .entry::<Node>()
        .and_modify(|mut n| n.margin = UiRect::top(Val::Px(8.0)));
    if !ouverts {
        return;
    }
    for s in liste {
        let (t, couleur) = if p.succes(&s.id) {
            (format!("✓ {} · {}", s.nom, s.texte), OR_VIF)
        } else {
            (format!("{} · {}", s.nom, s.texte), AIDE)
        };
        commandes.spawn((
            TextePixel::nouveau(&t, couleur, 1.5).bloc(LARGEUR - 40.0),
            ChildOf(racine),
        ));
    }
}

/// Dans le salon : code, joueurs, lancement.
fn salon(commandes: &mut Commands, racine: Entity, dj: &DonneesJeu, menu: &Menu, coop: &Coop) {
    if *coop.phase() == PhaseCoop::Connexion {
        texte(commandes, racine, "Connexion au relais…", IVOIRE, 2.0);
        bouton(commandes, racine, "Annuler", Some(ActionMulti::Quitter));
        return;
    }
    let code = coop.code().unwrap_or_default();
    texte(
        commandes,
        racine,
        &format!("Code du salon : {code}"),
        OR_VIF,
        2.5,
    );
    texte(
        commandes,
        racine,
        "Donnez ce code à vos amis pour qu'ils vous rejoignent.",
        AIDE,
        1.5,
    );
    for j in coop.joueurs() {
        let l = ligne(commandes, racine, LARGEUR);
        let p = &j.presentation;
        let classe =
            dj.d.personnages
                .personnages
                .get(&p.personnage)
                .map_or("?", |c| c.nom.as_str());
        let moi = if j.index == coop.place() {
            " (vous)"
        } else {
            ""
        };
        let hote = if j.index == 0 { "Hôte · " } else { "" };
        texte(
            commandes,
            l,
            &format!("{hote}{}{moi}", p.pseudo),
            IVOIRE,
            2.0,
        );
        texte(commandes, l, classe, AIDE, 1.5);
    }
    let l = ligne(commandes, racine, LARGEUR);
    if coop.est_hote() {
        let arene = &dj.arenes[menu.choix.arene];
        let generee = matches!(arene.source, SourceMenu::Generee(_));
        texte(
            commandes,
            l,
            &format!("Arène : {}", arene.def.nom),
            IVOIRE,
            2.0,
        );
        bouton_teinte(
            commandes,
            l,
            "Lancer",
            Relief::PRINCIPAL,
            generee.then_some(ActionMulti::Lancer),
        );
        if !generee {
            texte(commandes, l, "(arène générée seulement)", AIDE, 1.5);
        }
    } else {
        texte(
            commandes,
            l,
            "En attente du lancement par l'hôte…",
            AIDE,
            2.0,
        );
    }
    bouton(
        commandes,
        racine,
        "Quitter le salon",
        Some(ActionMulti::Quitter),
    );
}

/// Construit la page ; renvoie sa racine.
pub fn construire(
    commandes: &mut Commands,
    dj: &DonneesJeu,
    menu: &Menu,
    vue: &VueMulti,
) -> Entity {
    let racine = commandes
        .spawn((
            Name::new("multijoueur"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(charte::MARGE)),
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(charte::FOND),
        ))
        .id();
    commandes.spawn((
        TextePixel::nouveau("Multijoueur", IVOIRE, 4.0),
        ChildOf(racine),
    ));
    let corps = commandes
        .spawn((
            Node {
                column_gap: Val::Px(24.0),
                align_items: AlignItems::FlexStart,
                min_height: Val::ZERO,
                flex_shrink: 1.0,
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    let colonne = |largeur: f32| Node {
        width: Val::Px(largeur),
        row_gap: Val::Px(8.0),
        max_height: Val::Percent(100.0),
        overflow: Overflow::clip_y(),
        ..default()
    };
    let gauche = charte::panneau(commandes, corps, colonne(LARGEUR));
    let droite = charte::panneau(commandes, corps, colonne(super::vue_amis::LARGEUR));
    let saisie_salon = !vue.annuaire.champ_ami;
    match vue.coop {
        Some(coop) => salon(commandes, gauche, dj, menu, coop),
        None => {
            accueil(commandes, gauche, menu, vue, saisie_salon);
            succes(commandes, gauche, dj, vue);
        }
    }
    super::vue_amis::construire(commandes, droite, menu, vue);
    if let Some(m) = vue.message {
        commandes.spawn((
            TextePixel::nouveau(m, ROUGE, 1.5).bloc(LARGEUR),
            ChildOf(racine),
        ));
    }
    let retour = bouton_teinte(
        commandes,
        racine,
        "Retour",
        Relief::SECONDAIRE,
        Some(ActionMulti::Quitter),
    );
    // Retour : ferme la page (et quitte le salon, comme Échap)
    commandes.entity(retour).insert(ActionMenu::Fermer);
    commandes
        .entity(retour)
        .entry::<Node>()
        .and_modify(|mut n| {
            n.width = Val::Px(200.0);
            n.margin = UiRect::top(Val::Auto);
        });
    racine
}
