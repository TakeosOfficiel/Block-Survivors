//! Scènes de droite de la page Multijoueur : créer un salon (médaillons Privé / Public,
//! les cinq places, gros bouton vert), rejoindre par code (champ en pilule), salons
//! publics (avatars ronds et liens), et, dans un salon, son code, ses joueurs et le
//! lancement. Sans cadre ; textes et pilules posés sur la nuit.

use bevy::prelude::*;
use bs_reseau::PhaseCoop;

use super::ActionMulti;
use super::vue::{
    Police, TERNE, VueMulti, accroche, aide, appel, avatar, ecrit, initiale, m, rond, surtitre,
};
use crate::coop::Coop;
use crate::ecrans::charte::{self, AIDE, IVOIRE, OR_VIF};
use crate::lancement::{DonneesJeu, SourceMenu};
use crate::menu::air;
use crate::menu::vue::Lien;
use crate::menu::{ActionMenu, Icones, Menu};

const VERT: Color = Color::srgb_u8(0x3e, 0x8e, 0x4e);

/// Rangée espacée du haut.
fn bloc(c: &mut Commands, parent: Entity, ecart: f32, haut: f32) -> Entity {
    let r = air::rangee(c, parent, ecart);
    c.entity(r)
        .entry::<Node>()
        .and_modify(move |mut n| n.margin = UiRect::top(Val::Px(haut)));
    r
}

/// Médaillon de visibilité : rond doré si choisi, icône, titre et description.
fn medaillon(c: &mut Commands, ic: &Icones, parent: Entity, public: bool, choisi: bool) {
    let b = air::rangee(c, parent, 20.0);
    if !choisi {
        c.entity(b).insert((
            Button,
            Lien(TERNE),
            ActionMenu::Multi(ActionMulti::Visibilite(public)),
        ));
    }
    let o = air::point(
        c,
        b,
        96.0,
        if choisi {
            OR_VIF.with_alpha(0.08)
        } else {
            Color::NONE
        },
    );
    let bord = if choisi {
        OR_VIF
    } else {
        IVOIRE.with_alpha(0.2)
    };
    c.entity(o)
        .insert(BorderColor::all(bord))
        .entry::<Node>()
        .and_modify(|mut n| {
            n.border = UiRect::all(Val::Px(3.0));
            n.justify_content = JustifyContent::Center;
            n.align_items = AlignItems::Center;
        });
    let trait_ = if choisi {
        OR_VIF
    } else {
        IVOIRE.with_alpha(0.55)
    };
    if public {
        globe(c, o, trait_);
    } else {
        charte::cadenas(c, o, 36.0, trait_);
    }
    let t = air::colonne(c, b, 6.0);
    let (titre, desc) = if public {
        ("PUBLIC", "Visible dans les salons publics")
    } else {
        ("PRIVÉ", "Sur invitation ou avec le code")
    };
    ecrit(
        c,
        ic,
        t,
        titre,
        (Police::Logo, 30.0, if choisi { OR_VIF } else { TERNE }),
    );
    let d = ecrit(c, ic, t, desc, (Police::Normale, 14.0, AIDE));
    c.entity(d).insert(Node {
        max_width: Val::Px(190.0),
        ..default()
    });
}

/// Globe au trait : cercle, méridien en ovale, équateur.
fn globe(c: &mut Commands, parent: Entity, couleur: Color) {
    let g = air::point(c, parent, 38.0, Color::NONE);
    c.entity(g)
        .insert(BorderColor::all(couleur))
        .entry::<Node>()
        .and_modify(|mut n| {
            n.border = UiRect::all(Val::Px(2.5));
            n.justify_content = JustifyContent::Center;
            n.align_items = AlignItems::Center;
        });
    c.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(14.0),
            height: Val::Percent(100.0),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BorderColor::all(couleur),
        ChildOf(g),
    ));
    c.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Px(2.0),
            ..default()
        },
        BackgroundColor(couleur),
        ChildOf(g),
    ));
}

/// Les cinq places : pseudos présents en ronds pleins, places libres en « + ».
fn places(c: &mut Commands, ic: &Icones, parent: Entity, joueurs: &[(&str, bool)]) {
    let r = bloc(c, parent, 14.0, 22.0);
    for k in 0..5 {
        match joueurs.get(k) {
            Some((p, moi)) => {
                let fond = if *moi {
                    Color::srgb_u8(0x56, 0xb0, 0x67)
                } else {
                    IVOIRE.with_alpha(0.85)
                };
                rond(c, ic, r, (initiale(p), 46.0), fond);
            }
            None => {
                let v = air::point(c, r, 46.0, Color::NONE);
                c.entity(v)
                    .insert(BorderColor::all(IVOIRE.with_alpha(0.22)))
                    .entry::<Node>()
                    .and_modify(|mut n| {
                        n.border = UiRect::all(Val::Px(2.0));
                        n.justify_content = JustifyContent::Center;
                        n.align_items = AlignItems::Center;
                    });
                ecrit(
                    c,
                    ic,
                    v,
                    "+",
                    (Police::Grasse, 16.0, IVOIRE.with_alpha(0.3)),
                );
            }
        }
    }
    let t = format!("{} / 5 survivants", joueurs.len());
    let e = aide(c, ic, r, &t);
    c.entity(e)
        .entry::<Node>()
        .and_modify(|mut n| n.margin = UiRect::left(Val::Px(10.0)));
}

/// Créer un salon.
pub fn creer(c: &mut Commands, ic: &Icones, d: Entity, vue: &VueMulti) {
    surtitre(c, ic, d, "NOUVEAU SALON");
    accroche(c, ic, d, "Qui peut entrer dans ta nuit ?");
    aide(
        c,
        ic,
        d,
        "Tu seras l'hôte : tes amis te rejoignent avec le code du salon.",
    );
    let r = bloc(c, d, 64.0, 30.0);
    let public = vue.annuaire.public;
    medaillon(c, ic, r, false, !public);
    medaillon(c, ic, r, true, public);
    let moi = if vue.pseudo.is_empty() {
        "?"
    } else {
        vue.pseudo
    };
    places(c, ic, d, &[(moi, true)]);
    appel(c, ic, d, "CRÉER", m(ActionMulti::Creer));
}

/// Champ de saisie en grande pilule et son bouton vert accolé.
fn saisie(
    c: &mut Commands,
    ic: &Icones,
    d: Entity,
    (t, actif): (String, bool),
    champ: ActionMulti,
    ok: (&str, ActionMulti),
) {
    let r = bloc(c, d, 14.0, 26.0);
    let f = air::champ(c, ic, r, &t, actif, ActionMenu::Multi(champ));
    c.entity(f).entry::<Node>().and_modify(|mut n| {
        n.width = Val::Px(300.0);
        n.height = Val::Px(56.0);
        n.padding = UiRect::horizontal(Val::Px(26.0));
    });
    let b = air::bouton_vert(c, ic, r, ok.0, m(ok.1));
    c.entity(b).entry::<Node>().and_modify(|mut n| {
        n.height = Val::Px(52.0);
        n.padding = UiRect::horizontal(Val::Px(26.0));
    });
}

/// Rejoindre par code.
pub fn rejoindre(c: &mut Commands, ic: &Icones, d: Entity, menu: &Menu, vue: &VueMulti) {
    surtitre(c, ic, d, "REJOINDRE PAR CODE");
    accroche(c, ic, d, "Entre le code que ton ami t'a donné.");
    aide(c, ic, d, "Six caractères, par exemple K7Q-2MX.");
    let valeur = match &menu.saisie {
        Some(s) if !vue.annuaire.champ_ami => (format!("{s}|"), true),
        _ => ("XXX-XXX".to_owned(), false),
    };
    let actif = valeur.1;
    saisie(
        c,
        ic,
        d,
        valeur,
        ActionMulti::Code,
        ("VALIDER", ActionMulti::Rejoindre),
    );
    if actif {
        aide(c, ic, d, "Entrée : rejoindre · Échap : annuler");
    }
}

/// Ligne d'une liste : avatar, nom qui prend la place, puis le reste.
pub(super) fn ligne(c: &mut Commands, ic: &Icones, d: Entity, nom: &str) -> Entity {
    let l = air::rangee(c, d, 18.0);
    c.entity(l)
        .entry::<Node>()
        .and_modify(|mut n| n.width = Val::Percent(100.0));
    avatar(c, ic, l, nom, 44.0);
    let e = ecrit(c, ic, l, nom, (Police::Grasse, 18.0, IVOIRE));
    c.entity(e).insert(Node {
        flex_grow: 1.0,
        ..default()
    });
    l
}

/// Lien texte d'une ligne (ivoire, or au survol).
pub(super) fn lien(c: &mut Commands, ic: &Icones, l: Entity, t: &str, a: Option<ActionMenu>) {
    let couleur = if a.is_some() { IVOIRE } else { AIDE };
    air::lien(c, ic, l, t, (15.0, couleur), a);
}

/// Salons publics.
pub fn publics(c: &mut Commands, ic: &Icones, d: Entity, vue: &VueMulti) {
    surtitre(c, ic, d, "SALONS PUBLICS");
    accroche(c, ic, d, "Des survivants cherchent du renfort.");
    let liste = air::colonne(c, d, 18.0);
    c.entity(liste).entry::<Node>().and_modify(|mut n| {
        n.margin = UiRect::top(Val::Px(24.0));
        n.width = Val::Percent(100.0);
    });
    let recherche = vue.annuaire.session.is_some();
    if vue.annuaire.salons.is_empty() {
        let t = if recherche {
            "Recherche des salons…"
        } else {
            "Aucun salon public ouvert pour l'instant."
        };
        aide(c, ic, liste, t);
    }
    for (k, s) in vue.annuaire.salons.iter().enumerate() {
        let hote = String::from_utf8_lossy(&s.annonce);
        let l = ligne(c, ic, liste, &hote);
        let t = format!("{}/{} joueurs", s.joueurs, s.places);
        let e = ecrit(c, ic, l, &t, (Police::Normale, 14.0, AIDE));
        c.entity(e).insert(Node {
            min_width: Val::Px(140.0),
            ..default()
        });
        let plein = s.joueurs >= s.places;
        let (t, a) = if plein {
            ("Complet", None)
        } else {
            ("Rejoindre", m(ActionMulti::Public(k)))
        };
        lien(c, ic, l, t, a);
    }
    let a = (!recherche).then_some(ActionMenu::Multi(ActionMulti::Actualiser));
    let r = bloc(c, d, 0.0, 22.0);
    air::lien(c, ic, r, "↻ Actualiser", (15.0, AIDE), a);
}

/// Dans le salon : code, places, joueurs, lancement.
pub fn salon(
    c: &mut Commands,
    ic: &Icones,
    d: Entity,
    (dj, menu): (&DonneesJeu, &Menu),
    coop: &Coop,
) {
    let quitter = m(ActionMulti::Quitter);
    if *coop.phase() == PhaseCoop::Connexion {
        surtitre(c, ic, d, "CONNEXION");
        accroche(c, ic, d, "Connexion au relais…");
        let r = bloc(c, d, 0.0, 20.0);
        air::lien(c, ic, r, "Annuler", (15.0, AIDE), quitter);
        return;
    }
    surtitre(c, ic, d, "CODE DU SALON");
    let code = coop.code().unwrap_or_default();
    ecrit(c, ic, d, &code, (Police::Logo, 60.0, OR_VIF));
    aide(
        c,
        ic,
        d,
        "Donne ce code à tes amis pour qu'ils te rejoignent.",
    );
    let joueurs = coop.joueurs();
    let mut noms = [("", false); 5];
    for (n, j) in noms.iter_mut().zip(joueurs) {
        *n = (j.presentation.pseudo.as_str(), j.index == coop.place());
    }
    places(c, ic, d, &noms[..joueurs.len().min(5)]);
    let liste = air::colonne(c, d, 12.0);
    c.entity(liste).entry::<Node>().and_modify(|mut n| {
        n.margin = UiRect::top(Val::Px(14.0));
        n.width = Val::Percent(100.0);
    });
    for j in joueurs {
        let p = &j.presentation;
        let l = ligne(c, ic, liste, &p.pseudo);
        if j.index == coop.place() {
            ecrit(
                c,
                ic,
                l,
                "toi",
                (Police::Normale, 14.0, Color::srgb_u8(0x56, 0xb0, 0x67)),
            );
        }
        if j.index == 0 {
            ecrit(c, ic, l, "★ Hôte", (Police::Grasse, 14.0, OR_VIF));
        }
        let classe = dj.d.personnages.personnages.get(&p.personnage);
        let classe = classe.map_or("?", |x| x.nom.as_str());
        ecrit(c, ic, l, classe, (Police::Normale, 14.0, AIDE));
    }
    if coop.est_hote() {
        let arene = &dj.arenes[menu.choix.arene];
        let generee = matches!(arene.source, SourceMenu::Generee(_));
        let r = bloc(c, d, 8.0, 18.0);
        ecrit(c, ic, r, "Arène", (Police::Normale, 15.0, AIDE));
        ecrit(c, ic, r, &arene.def.nom, (Police::Grasse, 15.0, IVOIRE));
        if !generee {
            aide(c, ic, d, "La coop ne se joue que sur une arène générée.");
        }
        appel(
            c,
            ic,
            d,
            "LANCER",
            generee.then_some(ActionMenu::Multi(ActionMulti::Lancer)),
        );
    } else {
        let r = bloc(c, d, 10.0, 18.0);
        air::point(c, r, 10.0, VERT);
        ecrit(
            c,
            ic,
            r,
            "En attente du lancement par l'hôte…",
            (Police::Grasse, 16.0, IVOIRE),
        );
    }
    let r = bloc(c, d, 0.0, 18.0);
    air::lien(c, ic, r, "Quitter le salon", (15.0, charte::ROUGE), quitter);
}
