//! Vue de la page Multijoueur, à l'air libre comme les Talents (`menu::air`) : fond nuit
//! en dégradé, aucun panneau. Hors salon, trois choix côte à côte au milieu de l'écran :
//! créer (Privé / Public en pilules, grand bouton vert), rejoindre par code (champ en
//! pilule), salons publics (lignes de texte, liens « Rejoindre ») ; dans le salon : son code en grand,
//! les joueurs, le lancement. Les amis dans un volet repliable à droite (`vue_amis`).
//! En bas : « ‹ Retour » et les succès de coop.

use bevy::prelude::*;
use bs_reseau::PhaseCoop;

use bs_sauvegarde::amis::Ami;
use bs_sauvegarde::progression::Progression;

use super::{ActionMulti, Annuaire};
use crate::amis::ReseauAmis;
use crate::coop::Coop;
use crate::ecrans::charte::{self, AIDE, IVOIRE, OR_VIF, T_AIDE, T_CORPS, T_SECTION};
use crate::hud::police::TextePixel;
use crate::lancement::{DonneesJeu, SourceMenu};
use crate::menu::air::{self, ecrire};
use crate::menu::{ActionMenu, Icones, Menu};

pub(super) const LARGEUR: f32 = 520.0;
/// Largeur d'un des trois choix hors salon, et leur écart.
const PARTIE: f32 = 340.0;
const ESPACE: f32 = 110.0;

/// Ce que la page montre de la coop.
pub struct VueMulti<'a> {
    pub coop: Option<&'a Coop>,
    pub message: Option<&'a str>,
    pub annuaire: &'a Annuaire,
    pub amis: Option<&'a ReseauAmis>,
    pub liste: &'a [Ami],
    pub progression: &'a Progression,
}

/// Action de la page, pour les liens.
pub(super) fn m(a: ActionMulti) -> Option<ActionMenu> {
    Some(ActionMenu::Multi(a))
}

/// Titre de partie (police du logo).
pub(super) fn titre(commandes: &mut Commands, ic: &Icones, parent: Entity, t: &str) {
    ecrire(commandes, ic, parent, t, (true, T_SECTION, IVOIRE));
}

/// Petit texte d'aide, replié sur la largeur de la colonne.
pub(super) fn aide(commandes: &mut Commands, ic: &Icones, parent: Entity, t: &str) {
    let e = ecrire(commandes, ic, parent, t, (false, T_AIDE, AIDE));
    commandes.entity(e).insert(Node {
        max_width: Val::Px(LARGEUR),
        ..default()
    });
}

/// Rangée étirée sur la colonne.
pub(super) fn ligne(commandes: &mut Commands, parent: Entity) -> Entity {
    let l = air::rangee(commandes, parent, 14.0);
    commandes.entity(l).entry::<Node>().and_modify(|mut n| {
        n.width = Val::Percent(100.0);
        n.min_height = Val::Px(40.0);
    });
    l
}

/// Bloc qui prend la place libre d'une ligne.
pub(super) fn etire(commandes: &mut Commands, parent: Entity, ecart: f32) -> Entity {
    let c = air::colonne(commandes, parent, ecart);
    commandes.entity(c).entry::<Node>().and_modify(|mut n| {
        n.flex_grow = 1.0;
        n.flex_shrink = 1.0;
    });
    c
}

/// Texte d'un champ en cours de saisie (curseur) ou son invite.
pub(super) fn valeur(menu: &Menu, saisie: bool, invite: &str) -> (String, bool) {
    match &menu.saisie {
        Some(s) if saisie => (format!("{s}|"), true),
        _ => (invite.into(), false),
    }
}

/// Grand bouton vert, centré.
fn bouton_vert(c: &mut Commands, ic: &Icones, col: Entity, t: &str, a: Option<ActionMenu>) {
    let b = air::bouton_vert(c, ic, col, t, a);
    c.entity(b)
        .entry::<Node>()
        .and_modify(|mut n| n.margin = UiRect::vertical(Val::Px(6.0)));
}

/// Hors salon : créer (visibilité au choix), rejoindre par code, salons publics.
fn accueil(commandes: &mut Commands, ic: &Icones, page: Entity, menu: &Menu, vue: &VueMulti) {
    // Trois choix côte à côte, bien espacés, sans séparation
    let r = air::rangee(commandes, page, ESPACE);
    commandes
        .entity(r)
        .entry::<Node>()
        .and_modify(|mut n| n.align_items = AlignItems::FlexStart);
    let partie = |commandes: &mut Commands| {
        let c = air::colonne(commandes, r, 18.0);
        commandes.entity(c).entry::<Node>().and_modify(|mut n| {
            n.width = Val::Px(PARTIE);
            n.align_items = AlignItems::Center;
        });
        c
    };
    let col = partie(commandes);
    titre(commandes, ic, col, "Créer un salon");
    let public = vue.annuaire.public;
    let l = air::rangee(commandes, col, 10.0);
    for (t, p) in [("Privé", false), ("Public", true)] {
        let action = (p != public).then_some(ActionMenu::Multi(ActionMulti::Visibilite(p)));
        air::pilule(commandes, ic, l, t, p == public, action);
    }
    let t = if public {
        "Visible de tous dans la liste des salons publics."
    } else {
        "On n'y entre qu'avec le code du salon."
    };
    aide(commandes, ic, col, t);
    bouton_vert(commandes, ic, col, "CRÉER LE SALON", m(ActionMulti::Creer));

    let col = partie(commandes);

    titre(commandes, ic, col, "Rejoindre par code");
    let saisie = !vue.annuaire.champ_ami;
    let (t, actif) = valeur(menu, saisie, "Code du salon");
    let champ = ActionMenu::Multi(ActionMulti::Code);
    air::champ(commandes, ic, col, &t, actif, champ);
    let rejoindre = m(ActionMulti::Rejoindre);
    air::lien(
        commandes,
        ic,
        col,
        "Rejoindre ›",
        (T_CORPS, OR_VIF),
        rejoindre,
    );
    if actif {
        aide(commandes, ic, col, "Entrée : rejoindre · Échap : annuler");
    }

    let col = partie(commandes);

    let l = air::rangee(commandes, col, 18.0);
    titre(commandes, ic, l, "Salons publics");
    let recherche = vue.annuaire.session.is_some();
    let (t, action) = if recherche {
        ("Recherche…", None)
    } else {
        ("↻ Actualiser", m(ActionMulti::Actualiser))
    };
    air::lien(commandes, ic, l, t, (T_AIDE, AIDE), action);
    if vue.annuaire.salons.is_empty() && !recherche {
        aide(
            commandes,
            ic,
            col,
            "Aucun salon public ouvert pour l'instant.",
        );
    }
    for (k, s) in vue.annuaire.salons.iter().enumerate() {
        let l = ligne(commandes, col);
        let plein = s.joueurs >= s.places;
        let couleur = if plein { AIDE } else { charte::VERT_GAIN };
        air::point(commandes, l, 8.0, couleur);
        let g = etire(commandes, l, 0.0);
        let hote = String::from_utf8_lossy(&s.annonce);
        ecrire(commandes, ic, g, &hote, (false, T_CORPS, IVOIRE));
        let t = format!("{}/{}", s.joueurs, s.places);
        ecrire(commandes, ic, l, &t, (false, T_AIDE, AIDE));
        let (t, action) = if plein {
            ("Complet", None)
        } else {
            ("Rejoindre ›", m(ActionMulti::Public(k)))
        };
        air::lien(commandes, ic, l, t, (T_CORPS, OR_VIF), action);
    }
}

/// Succès de coop : un lien « Succès coop · 2/5 › », dépliés au clic (gagnés en or, les
/// autres avec leur condition).
fn succes(commandes: &mut Commands, ic: &Icones, parent: Entity, dj: &DonneesJeu, vue: &VueMulti) {
    let (liste, p) = (&dj.d.succes, vue.progression);
    if liste.is_empty() {
        return;
    }
    let gagnes = liste.iter().filter(|s| p.succes(&s.id)).count();
    let ouverts = vue.annuaire.succes_ouverts;
    let fleche = if ouverts { "⌄" } else { "›" };
    let t = format!("Succès coop · {gagnes}/{} {fleche}", liste.len());
    let col = air::colonne(commandes, parent, 4.0);
    if ouverts {
        for s in liste {
            let (t, couleur) = if p.succes(&s.id) {
                (format!("★ {} · {}", s.nom, s.texte), OR_VIF)
            } else {
                (format!("{} · {}", s.nom, s.texte), AIDE)
            };
            ecrire(commandes, ic, col, &t, (false, T_AIDE, couleur));
        }
    }
    air::lien(
        commandes,
        ic,
        col,
        &t,
        (T_CORPS, IVOIRE),
        m(ActionMulti::Succes),
    );
}

/// Dans le salon : code, joueurs, lancement.
fn salon(
    c: &mut Commands,
    ic: &Icones,
    col: Entity,
    (dj, menu): (&DonneesJeu, &Menu),
    coop: &Coop,
) {
    let quitter = m(ActionMulti::Quitter);
    if *coop.phase() == PhaseCoop::Connexion {
        titre(c, ic, col, "Connexion au relais…");
        air::lien(c, ic, col, "Annuler", (T_CORPS, IVOIRE), quitter);
        return;
    }
    ecrire(c, ic, col, "Code du salon", (false, T_AIDE, AIDE));
    let code = coop.code().unwrap_or_default();
    ecrire(c, ic, col, &code, (true, 48.0, OR_VIF));
    aide(
        c,
        ic,
        col,
        "Donnez ce code à vos amis pour qu'ils vous rejoignent.",
    );
    air::trait_fin(c, col);

    let joueurs = coop.joueurs();
    titre(c, ic, col, &format!("Joueurs · {}/5", joueurs.len()));
    for j in joueurs {
        let l = ligne(c, col);
        let p = &j.presentation;
        let moi = j.index == coop.place();
        let couleur = if moi { charte::VERT_GAIN } else { IVOIRE };
        air::point(c, l, 10.0, couleur);
        let g = air::rangee(c, l, 10.0);
        c.entity(g)
            .entry::<Node>()
            .and_modify(|mut n| n.flex_grow = 1.0);
        ecrire(c, ic, g, &p.pseudo, (true, T_CORPS, IVOIRE));
        if moi {
            ecrire(c, ic, g, "(vous)", (false, T_AIDE, AIDE));
        }
        if j.index == 0 {
            ecrire(c, ic, g, "★ Hôte", (false, T_AIDE, OR_VIF));
        }
        let classe = dj.d.personnages.personnages.get(&p.personnage);
        let classe = classe.map_or("?", |c| c.nom.as_str());
        ecrire(c, ic, l, classe, (false, T_AIDE, AIDE));
    }
    air::trait_fin(c, col);

    if coop.est_hote() {
        let arene = &dj.arenes[menu.choix.arene];
        let generee = matches!(arene.source, SourceMenu::Generee(_));
        let t = format!("Arène : {}", arene.def.nom);
        ecrire(c, ic, col, &t, (false, T_CORPS, IVOIRE));
        if !generee {
            aide(c, ic, col, "La coop ne se joue que sur une arène générée.");
        }
        let lancer = generee.then_some(ActionMenu::Multi(ActionMulti::Lancer));
        bouton_vert(c, ic, col, "LANCER", lancer);
    } else {
        let t = "En attente du lancement par l'hôte…";
        ecrire(c, ic, col, t, (false, T_CORPS, AIDE));
    }
    air::lien(
        c,
        ic,
        col,
        "Quitter le salon",
        (T_CORPS, charte::ROUGE),
        quitter,
    );
}

/// Construit la page ; renvoie sa racine.
pub fn construire(
    commandes: &mut Commands,
    dj: &DonneesJeu,
    ic: &Icones,
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
                row_gap: Val::Px(28.0),
                ..default()
            },
            BackgroundColor(charte::FOND),
            air::fond(),
        ))
        .id();
    commandes.spawn((
        TextePixel::nouveau("Multijoueur", IVOIRE, 3.0),
        ChildOf(racine),
    ));
    // Contenu au milieu de l'écran ; les amis dans un volet à droite
    let col = air::colonne(commandes, racine, 16.0);
    let dans_salon = vue.coop.is_some();
    commandes
        .entity(col)
        .entry::<Node>()
        .and_modify(move |mut n| {
            if dans_salon {
                n.width = Val::Px(LARGEUR);
            }
            n.align_items = AlignItems::Center;
            n.justify_content = JustifyContent::Center;
            n.flex_grow = 1.0;
            n.padding = UiRect::bottom(Val::Px(60.0));
            n.flex_shrink = 1.0;
            n.min_height = Val::ZERO;
            n.overflow = Overflow::clip_y();
        });
    match vue.coop {
        Some(coop) => salon(commandes, ic, col, (dj, menu), coop),
        None => accueil(commandes, ic, col, menu, vue),
    }
    if let Some(t) = vue.message {
        ecrire(commandes, ic, racine, t, (false, T_CORPS, charte::ROUGE));
    }
    let pied = air::rangee(commandes, racine, 48.0);
    commandes.entity(pied).entry::<Node>().and_modify(|mut n| {
        n.align_items = AlignItems::FlexEnd;
        n.position_type = PositionType::Absolute;
        n.left = Val::Px(charte::MARGE);
        n.bottom = Val::Px(charte::MARGE);
    });
    // Retour : ferme la page (et quitte le salon, comme Échap)
    let retour = Some(ActionMenu::Fermer);
    air::lien(commandes, ic, pied, "‹ Retour", (T_CORPS, IVOIRE), retour);
    if vue.coop.is_none() {
        succes(commandes, ic, pied, dj, vue);
    }
    super::vue_amis::volet(commandes, ic, racine, menu, vue);
    racine
}
