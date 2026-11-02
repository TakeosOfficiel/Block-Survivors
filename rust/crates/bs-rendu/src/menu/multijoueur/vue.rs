//! Vue de la page Multijoueur : hors salon (créer, code, salons publics) ou dans le
//! salon (code, joueurs, lancement), et le message de la dernière coop arrêtée ; la
//! colonne des amis à droite (`vue_amis`) ; hors salon, les succès de coop.

use bevy::prelude::*;
use bs_reseau::PhaseCoop;

use bs_sauvegarde::amis::Ami;
use bs_sauvegarde::progression::Progression;

use super::{ActionMulti, Annuaire};
use crate::amis::ReseauAmis;
use crate::coop::Coop;
use crate::ecrans::style::{self, GRIS, OR, OR_CLAIR, TEXTE};
use crate::hud::police::TextePixel;
use crate::lancement::{DonneesJeu, SourceMenu};
use crate::menu::{ActionMenu, Menu};

const LARGEUR: f32 = 560.0;
const ROUGE: Color = Color::srgb_u8(0xff, 0x8a, 0x8a);

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

pub(super) fn bouton(
    commandes: &mut Commands,
    parent: Entity,
    t: &str,
    action: Option<ActionMulti>,
) {
    let b = style::bouton(commandes, parent, t, 0.0, false, ());
    let mut e = commandes.entity(b);
    e.entry::<Node>().and_modify(|mut n| {
        n.width = Val::Auto;
        n.height = Val::Px(36.0);
        n.padding = UiRect::horizontal(Val::Px(12.0));
    });
    match action {
        Some(a) => {
            e.insert(ActionMenu::Multi(a));
        }
        None => {
            e.remove::<(Button, style::BoutonPage)>();
            e.insert(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)));
        }
    }
}

pub(super) fn ligne(commandes: &mut Commands, parent: Entity, largeur: f32) -> Entity {
    commandes
        .spawn((
            Node {
                width: Val::Px(largeur),
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                flex_shrink: 0.0,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(crate::menu::vue::FOND),
            BorderColor::all(Color::srgb_u8(0x2a, 0x2a, 0x2a)),
            ChildOf(parent),
        ))
        .id()
}

/// Hors salon : créer, rejoindre par code, salons publics.
fn accueil(commandes: &mut Commands, racine: Entity, menu: &Menu, vue: &VueMulti, saisie: bool) {
    let l = ligne(commandes, racine, LARGEUR);
    texte(commandes, l, "Créer un salon :", OR, 2.0);
    bouton(commandes, l, "Public", Some(ActionMulti::Creer(true)));
    bouton(commandes, l, "Privé", Some(ActionMulti::Creer(false)));

    let l = ligne(commandes, racine, LARGEUR);
    texte(commandes, l, "Rejoindre :", OR, 2.0);
    let champ = match &menu.saisie {
        Some(s) if saisie => format!("{s}_"),
        _ => "Taper le code".into(),
    };
    bouton(commandes, l, &champ, Some(ActionMulti::Code));
    bouton(commandes, l, "Rejoindre", Some(ActionMulti::Rejoindre));
    if menu.saisie.is_some() && saisie {
        texte(
            commandes,
            racine,
            "Entrée : rejoindre · Échap : annuler",
            GRIS,
            1.5,
        );
    }

    let l = ligne(commandes, racine, LARGEUR);
    texte(commandes, l, "Salons publics", OR, 2.0);
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
        texte(commandes, racine, "Aucun salon public ouvert.", GRIS, 1.5);
    }
    for (k, s) in vue.annuaire.salons.iter().enumerate() {
        let l = ligne(commandes, racine, LARGEUR);
        let hote = String::from_utf8_lossy(&s.annonce);
        let t = format!("{hote} · {}/{} joueurs", s.joueurs, s.places);
        texte(commandes, l, &t, TEXTE, 2.0);
        let plein = s.joueurs >= s.places;
        bouton(
            commandes,
            l,
            "Rejoindre",
            (!plein).then_some(ActionMulti::Public(k)),
        );
    }
}

/// Succès de coop : gagnés en or, les autres en gris avec leur condition.
fn succes(commandes: &mut Commands, racine: Entity, dj: &DonneesJeu, p: &Progression) {
    let liste = &dj.d.talents.succes;
    if liste.is_empty() {
        return;
    }
    let gagnes = liste.iter().filter(|s| p.succes(&s.id)).count();
    let l = ligne(commandes, racine, LARGEUR);
    let t = format!("Succès de coop · {gagnes}/{}", liste.len());
    texte(commandes, l, &t, OR, 2.0);
    for s in liste {
        let (t, couleur) = if p.succes(&s.id) {
            (format!("{} · {} · gagné", s.nom, s.texte), OR_CLAIR)
        } else {
            (format!("{} · {}", s.nom, s.texte), GRIS)
        };
        texte(commandes, racine, &t, couleur, 1.5);
    }
}

/// Dans le salon : code, joueurs, lancement.
fn salon(commandes: &mut Commands, racine: Entity, dj: &DonneesJeu, menu: &Menu, coop: &Coop) {
    if *coop.phase() == PhaseCoop::Connexion {
        texte(commandes, racine, "Connexion au relais…", TEXTE, 2.0);
        bouton(commandes, racine, "Annuler", Some(ActionMulti::Quitter));
        return;
    }
    let code = coop.code().unwrap_or_default();
    texte(
        commandes,
        racine,
        &format!("Code du salon : {code}"),
        OR_CLAIR,
        2.5,
    );
    texte(
        commandes,
        racine,
        "Donnez ce code à vos amis pour qu'ils vous rejoignent.",
        GRIS,
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
            TEXTE,
            2.0,
        );
        texte(commandes, l, classe, GRIS, 1.5);
    }
    let l = ligne(commandes, racine, LARGEUR);
    if coop.est_hote() {
        let arene = &dj.arenes[menu.choix.arene];
        let generee = matches!(arene.source, SourceMenu::Generee(_));
        texte(
            commandes,
            l,
            &format!("Arène : {}", arene.def.nom),
            TEXTE,
            2.0,
        );
        bouton(
            commandes,
            l,
            "Lancer",
            generee.then_some(ActionMulti::Lancer),
        );
        if !generee {
            texte(commandes, l, "(arène générée seulement)", GRIS, 1.5);
        }
    } else {
        texte(
            commandes,
            l,
            "En attente du lancement par l'hôte…",
            GRIS,
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
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(crate::menu::vue::FOND),
        ))
        .id();
    texte(commandes, racine, "Multijoueur", OR_CLAIR, 3.0);
    let colonne = |largeur: f32| Node {
        width: Val::Px(largeur),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: Val::Px(8.0),
        ..default()
    };
    let corps = commandes
        .spawn((
            Node {
                column_gap: Val::Px(24.0),
                align_items: AlignItems::FlexStart,
                ..default()
            },
            ChildOf(racine),
        ))
        .id();
    let gauche = commandes.spawn((colonne(LARGEUR), ChildOf(corps))).id();
    let droite = commandes
        .spawn((colonne(super::vue_amis::LARGEUR), ChildOf(corps)))
        .id();
    let saisie_salon = !vue.annuaire.champ_ami;
    match vue.coop {
        Some(coop) => salon(commandes, gauche, dj, menu, coop),
        None => {
            accueil(commandes, gauche, menu, vue, saisie_salon);
            succes(commandes, gauche, dj, vue.progression);
        }
    }
    super::vue_amis::construire(commandes, droite, menu, vue);
    if let Some(m) = vue.message {
        commandes.spawn((
            TextePixel::nouveau(m, ROUGE, 1.5).bloc(LARGEUR),
            ChildOf(racine),
        ));
    }
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
        .entry::<Node>()
        .and_modify(|mut n| n.margin = UiRect::top(Val::Auto));
    racine
}
