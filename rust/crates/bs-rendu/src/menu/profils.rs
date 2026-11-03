//! Page Profils du menu (src/ui/ecranProfils.ts) : les profils de cette installation
//! (pseudo, maîtrise de chaque classe jouée), pour jouer avec un autre, en créer un (pseudo
//! tapé, ou un pseudo tiré au hasard), en supprimer un (second clic pour confirmer), ou
//! transférer le profil actif : export en `.bssave`, import en glissant un `.bssave` sur
//! la fenêtre. Changer de profil relit toutes ses données et ramène à l'accueil.
//! Un seul panneau central (docs/CHARTE-UI.md) : profils (badge « Actif »), nouveau
//! profil (champ en creux), transfert.

use bevy::prelude::*;
use bs_sauvegarde::profil_joueur::{nettoyer_pseudo, pseudo_aleatoire};
use bs_sauvegarde::profils::cle;

use super::{ActionMenu, Icones, Menu, Page};
use crate::ecrans::charte::{
    self, AIDE, BORD, IVOIRE, OR_SOURD, OR_VIF, Relief, T_AIDE, T_CORPS, T_SECTION,
};
use crate::hud::police::TextePixel;
use crate::lancement::{self, DonneesJeu};
use crate::sauvegarde::Sauvegarde;

const LARGEUR: f32 = 760.0;

/// Bouton de la page (profils par leur rang dans la liste).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionProfils {
    Jouer(usize),
    /// Premier clic : demande confirmation ; second : supprime.
    Supprimer(usize),
    Annuler,
    /// Commence la saisie du pseudo du nouveau profil (ou crée le profil).
    Nouveau,
    Creer,
    Exporter,
}

/// Ce qu'il reste à faire après une action.
#[derive(PartialEq, Eq)]
pub enum Suite {
    Refaire,
    /// Le profil actif a changé : tout est à relire.
    Recharger,
}

/// Pseudo proposé pour un nouveau profil.
pub fn suggestion() -> String {
    let mut x = lancement::graine_au_hasard() | 1;
    pseudo_aleatoire(|| {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        (x >> 8) as f32 / (1u32 << 24) as f32
    })
}

/// Applique une action de la page.
pub fn agir(sauvegarde: &mut Sauvegarde, menu: &mut Menu, action: ActionProfils) -> Suite {
    let Page::Profils(a_supprimer) = &mut menu.page else {
        return Suite::Refaire;
    };
    let id = |k: usize| sauvegarde.profils.ids().get(k).cloned();
    match action {
        ActionProfils::Jouer(k) => {
            if let Some(id) = id(k) {
                sauvegarde.profils.changer(&id);
                return Suite::Recharger;
            }
        }
        ActionProfils::Supprimer(k) if *a_supprimer == Some(k) => {
            *a_supprimer = None;
            if let Some(id) = id(k) {
                let actif = id == sauvegarde.profils.actif();
                sauvegarde.profils.supprimer(&id);
                if actif {
                    return Suite::Recharger;
                }
            }
        }
        ActionProfils::Supprimer(k) => *a_supprimer = Some(k),
        ActionProfils::Annuler => *a_supprimer = None,
        ActionProfils::Nouveau if menu.saisie.is_none() => menu.saisie = Some(String::new()),
        ActionProfils::Nouveau | ActionProfils::Creer => {
            let saisie = menu.saisie.take().unwrap_or_default();
            return creer(sauvegarde, menu, &saisie);
        }
        ActionProfils::Exporter => {
            menu.message = Some(match sauvegarde.exporter() {
                Ok(f) => format!("Profil exporté : {}", f.display()),
                Err(e) => format!("Export impossible : {e}"),
            });
        }
    }
    Suite::Refaire
}

/// Nouveau profil vierge avec ce pseudo (ou le pseudo proposé) ; il devient l'actif.
pub fn creer(sauvegarde: &mut Sauvegarde, menu: &mut Menu, saisie: &str) -> Suite {
    if !sauvegarde.profils.peut_creer() {
        return Suite::Refaire;
    }
    let mut pseudo = nettoyer_pseudo(saisie);
    if pseudo.is_empty() {
        pseudo.clone_from(&menu.suggestion);
    }
    let profil = serde_json::json!({
        "pseudo": pseudo, "skin": "classe", "brasFins": false, "skinImporte": null,
    })
    .to_string();
    sauvegarde
        .profils
        .creer(|s, id| s.ecrire_brut(&cle(id, "profil"), &profil));
    Suite::Recharger
}

/// Importe un `.bssave` glissé sur la fenêtre.
pub fn importer(sauvegarde: &mut Sauvegarde, menu: &mut Menu, chemin: &std::path::Path) -> Suite {
    let lu = std::fs::read_to_string(chemin);
    match lu.map(|c| sauvegarde.profils.importer(&c)) {
        Ok(Ok(_)) => return Suite::Recharger,
        Ok(Err(e)) => menu.message = Some(e.message().to_owned()),
        Err(_) => menu.message = Some("Fichier illisible.".to_owned()),
    }
    Suite::Refaire
}

/// Pseudo et maîtrises (« Guerrier 3 · Mage 1 ») d'un profil, lus sans le charger.
fn resumer(dj: &DonneesJeu, sauvegarde: &Sauvegarde, id: &str) -> (String, String) {
    let d = dj.d;
    let stockage = sauvegarde.profils.stockage();
    let lire = |nom: &str| -> Option<serde_json::Value> {
        serde_json::from_str(&stockage.lire_brut(&cle(id, nom))?).ok()
    };
    let pseudo = lire("profil")
        .and_then(|p| p["pseudo"].as_str().map(nettoyer_pseudo))
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| "Joueur".to_owned());
    let progression = lire("progression");
    let maitrises: Vec<String> = d
        .personnages
        .personnages
        .iter()
        .filter_map(|(p, perso)| {
            let xp = progression.as_ref()?["maitrise"][p.as_str()].as_f64()?;
            (xp > 0.0).then(|| {
                let niveau = d.talents.maitrise.niveau(xp as u64);
                format!("{} {niveau}", perso.nom)
            })
        })
        .collect();
    (pseudo, maitrises.join(" · "))
}

/// Texte Inter.
fn texte(
    c: &mut Commands,
    ic: &mut Icones,
    parent: Entity,
    t: &str,
    (gras, taille, couleur): (bool, f32, Color),
) {
    let t = ic.polices.texte(t, gras, taille, couleur);
    c.spawn((t, ChildOf(parent)));
}

/// Bouton en relief ; sans action, éteint.
fn bouton(
    c: &mut Commands,
    ic: &mut Icones,
    parent: Entity,
    t: &str,
    relief: Relief,
    action: Option<ActionProfils>,
) {
    let b = charte::bouton_relief(c, parent, relief, charte::H_SECONDAIRE, ());
    let couleur = if action.is_some() { IVOIRE } else { AIDE };
    texte(c, ic, b, t, (true, T_CORPS, couleur));
    match action {
        Some(a) => {
            c.entity(b).insert(ActionMenu::Profils(a));
        }
        None => {
            c.entity(b)
                .remove::<(Button, Relief)>()
                .insert(BackgroundColor(charte::CREUX));
        }
    }
}

/// Ligne d'un profil : surface, ou dorée (liseré à gauche) pour le profil actif.
fn ligne(c: &mut Commands, parent: Entity, actif: bool) -> Entity {
    let (fond, bord) = if actif {
        (charte::CHOISI, OR_SOURD)
    } else {
        (charte::SURFACE, BORD)
    };
    let l = c
        .spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                min_height: Val::Px(60.0),
                padding: UiRect::new(Val::Px(16.0), Val::Px(8.0), Val::Px(6.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(fond),
            BorderColor::all(bord),
            ChildOf(parent),
        ))
        .id();
    if actif {
        c.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(-2.0),
                top: Val::Px(-2.0),
                bottom: Val::Px(-2.0),
                width: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(OR_VIF),
            ChildOf(l),
        ));
    }
    l
}

/// Titre de section du panneau, filet dessous.
fn section(c: &mut Commands, ic: &mut Icones, parent: Entity, t: &str) {
    let s = c
        .spawn((
            Node {
                margin: UiRect::top(Val::Px(8.0)),
                padding: UiRect::bottom(Val::Px(6.0)),
                border: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(BORD),
            ChildOf(parent),
        ))
        .id();
    texte(c, ic, s, t, (true, T_SECTION, IVOIRE));
}

/// Champ de saisie en creux (bord sombre en haut, clair en bas), cliquable.
fn champ(
    c: &mut Commands,
    ic: &mut Icones,
    parent: Entity,
    t: &str,
    action: Option<ActionProfils>,
    actif: bool,
) {
    let mut e = c.spawn((
        Node {
            flex_grow: 1.0,
            height: Val::Px(charte::H_SECONDAIRE),
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(12.0)),
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
    ));
    if let Some(a) = action {
        e.insert((Button, ActionMenu::Profils(a)));
    }
    if actif {
        e.insert(Outline::new(Val::Px(2.0), Val::ZERO, OR_VIF));
    }
    let id = e.id();
    texte(c, ic, id, t, (false, T_CORPS, IVOIRE));
}

/// Construit la page ; renvoie sa racine.
pub fn construire(
    commandes: &mut Commands,
    (dj, sauvegarde): (&DonneesJeu, &Sauvegarde),
    ic: &mut Icones,
    menu: &Menu,
) -> Entity {
    let c = commandes;
    let Page::Profils(a_supprimer) = menu.page else {
        return c.spawn(Node::default()).id();
    };
    let racine = c
        .spawn((
            Name::new("profils"),
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
    c.spawn((TextePixel::nouveau("Profils", IVOIRE, 4.0), ChildOf(racine)));
    let panneau = charte::panneau(
        c,
        racine,
        Node {
            width: Val::Px(LARGEUR),
            max_width: Val::Percent(100.0),
            row_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(24.0)),
            ..default()
        },
    );
    section(c, ic, panneau, "Profils de cette installation");
    let ids = sauvegarde.profils.ids();
    for (k, id) in ids.iter().enumerate() {
        let actif = id == sauvegarde.profils.actif();
        let (pseudo, maitrises) = resumer(dj, sauvegarde, id);
        let l = ligne(c, panneau, actif);
        let infos = c
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                ChildOf(l),
            ))
            .id();
        let haut = c
            .spawn((
                Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                ChildOf(infos),
            ))
            .id();
        texte(c, ic, haut, &pseudo, (true, T_CORPS + 1.0, IVOIRE));
        if actif {
            let badge = c
                .spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(OR_VIF),
                    ChildOf(haut),
                ))
                .id();
            let sombre = Color::srgb_u8(0x2a, 0x1d, 0x02);
            texte(c, ic, badge, "Actif", (true, T_AIDE - 2.0, sombre));
        }
        let detail = if maitrises.is_empty() {
            "Aucune partie jouée"
        } else {
            &maitrises
        };
        texte(c, ic, infos, detail, (false, T_AIDE, AIDE));
        if a_supprimer == Some(k) {
            texte(
                c,
                ic,
                l,
                "Supprimer toute sa progression ?",
                (true, T_AIDE, charte::ROUGE),
            );
            bouton(
                c,
                ic,
                l,
                "Annuler",
                Relief::SECONDAIRE,
                Some(ActionProfils::Annuler),
            );
            bouton(
                c,
                ic,
                l,
                "Supprimer",
                rouge(),
                Some(ActionProfils::Supprimer(k)),
            );
            continue;
        }
        if !actif {
            let jouer = Some(ActionProfils::Jouer(k));
            bouton(c, ic, l, "Jouer avec", Relief::PRINCIPAL, jouer);
        }
        if ids.len() > 1 {
            let suppr = Some(ActionProfils::Supprimer(k));
            bouton(c, ic, l, "Supprimer", Relief::SECONDAIRE, suppr);
        }
    }

    // Nouveau profil : champ en creux, « Créer »
    section(c, ic, panneau, "Nouveau profil");
    let libre = sauvegarde.profils.peut_creer();
    let l = c
        .spawn((
            Node {
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            },
            ChildOf(panneau),
        ))
        .id();
    let (valeur, saisie) = match &menu.saisie {
        Some(s) => (format!("{s}|"), true),
        None => (menu.suggestion.clone(), false),
    };
    champ(
        c,
        ic,
        l,
        &valeur,
        libre.then_some(ActionProfils::Nouveau),
        saisie,
    );
    let creer = libre.then_some(ActionProfils::Creer);
    bouton(c, ic, l, "Créer", Relief::PRINCIPAL, creer);
    let aide = if !libre {
        "Nombre maximal de profils atteint."
    } else if saisie {
        "Entrée : créer · Échap : annuler"
    } else {
        "Cliquez dans le champ pour changer le pseudo proposé."
    };
    texte(c, ic, panneau, aide, (false, T_AIDE, AIDE));

    // Transfert
    section(c, ic, panneau, "Transfert");
    let l = c
        .spawn((
            Node {
                column_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                ..default()
            },
            ChildOf(panneau),
        ))
        .id();
    let exporter = Some(ActionProfils::Exporter);
    bouton(c, ic, l, "Exporter ce profil", Relief::SECONDAIRE, exporter);
    let t = "Importer : glissez un fichier .bssave sur la fenêtre.";
    texte(c, ic, l, t, (false, T_AIDE, AIDE));
    if let Some(m) = &menu.message {
        let couleur = if m.starts_with("Profil exporté") {
            charte::VERT_GAIN
        } else {
            charte::ROUGE
        };
        texte(c, ic, panneau, m, (true, T_AIDE, couleur));
    }

    let retour = charte::bouton_relief(
        c,
        racine,
        Relief::SECONDAIRE,
        charte::H_SECONDAIRE,
        ActionMenu::Fermer,
    );
    c.entity(retour).entry::<Node>().and_modify(|mut n| {
        n.width = Val::Px(200.0);
        n.margin = UiRect::top(Val::Auto);
    });
    texte(c, ic, retour, "Retour", (true, T_CORPS, IVOIRE));
    racine
}

/// Relief rouge d'une suppression confirmée.
fn rouge() -> Relief {
    Relief {
        fond: Color::srgb_u8(0x6e, 0x2a, 0x2a),
        bord: charte::ROUGE,
        fond_survol: Color::srgb_u8(0x86, 0x34, 0x34),
        bord_survol: Color::srgb_u8(0xff, 0x9a, 0x9a),
        tranche: Color::srgb_u8(0x45, 0x18, 0x18),
    }
}
