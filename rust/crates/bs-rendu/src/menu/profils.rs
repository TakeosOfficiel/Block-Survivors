//! Page Profils du menu (src/ui/ecranProfils.ts) : les profils de cette installation
//! (pseudo, maîtrise de chaque classe jouée), pour jouer avec un autre, en créer un (pseudo
//! tapé, ou un pseudo tiré au hasard), en supprimer un (second clic pour confirmer), ou
//! transférer le profil actif : export en `.bssave`, import en glissant un `.bssave` sur
//! la fenêtre. Changer de profil relit toutes ses données et ramène à l'accueil.

use bevy::prelude::*;
use bs_sauvegarde::profil_joueur::{nettoyer_pseudo, pseudo_aleatoire};
use bs_sauvegarde::profils::cle;

use super::{ActionMenu, Menu, Page};
use crate::ecrans::style::{self, GRIS, OR, OR_CLAIR, TEXTE};
use crate::hud::police::TextePixel;
use crate::lancement::{self, DonneesJeu};
use crate::sauvegarde::Sauvegarde;

const LARGEUR: f32 = 760.0;
const ROUGE: Color = Color::srgb_u8(0xff, 0x8a, 0x8a);
const VERT: Color = Color::srgb_u8(0x55, 0xff, 0x55);

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

fn texte(commandes: &mut Commands, parent: Entity, t: &str, couleur: Color, taille: f32) {
    commandes.spawn((TextePixel::nouveau(t, couleur, taille), ChildOf(parent)));
}

fn bouton(commandes: &mut Commands, parent: Entity, t: &str, action: Option<ActionProfils>) {
    let b = style::bouton(commandes, parent, t, 0.0, false, ());
    let mut e = commandes.entity(b);
    e.entry::<Node>().and_modify(|mut n| {
        n.width = Val::Auto;
        n.height = Val::Px(36.0);
        n.padding = UiRect::horizontal(Val::Px(12.0));
    });
    match action {
        Some(a) => {
            e.insert(ActionMenu::Profils(a));
        }
        None => {
            e.remove::<(Button, style::BoutonPage)>();
            e.insert(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)));
        }
    }
}

fn ligne(commandes: &mut Commands, parent: Entity, actif: bool) -> Entity {
    let bord = if actif {
        OR
    } else {
        Color::srgb_u8(0x2a, 0x2a, 0x2a)
    };
    commandes
        .spawn((
            Node {
                width: Val::Px(LARGEUR),
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(super::vue::FOND),
            BorderColor::all(bord),
            ChildOf(parent),
        ))
        .id()
}

/// Construit la page ; renvoie sa racine.
pub fn construire(
    commandes: &mut Commands,
    dj: &DonneesJeu,
    sauvegarde: &Sauvegarde,
    menu: &Menu,
) -> Entity {
    let Page::Profils(a_supprimer) = menu.page else {
        return commandes.spawn(Node::default()).id();
    };
    let racine = commandes
        .spawn((
            Name::new("profils"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(super::vue::FOND),
        ))
        .id();
    texte(commandes, racine, "Profils", OR_CLAIR, 3.0);
    let ids = sauvegarde.profils.ids();
    for (k, id) in ids.iter().enumerate() {
        let actif = id == sauvegarde.profils.actif();
        let (pseudo, maitrises) = resumer(dj, sauvegarde, id);
        let l = ligne(commandes, racine, actif);
        let infos = commandes
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    flex_grow: 1.0,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                ChildOf(l),
            ))
            .id();
        let nom = if actif {
            format!("{pseudo} (profil actuel)")
        } else {
            pseudo
        };
        texte(commandes, infos, &nom, TEXTE, 2.0);
        let detail = if maitrises.is_empty() {
            "Aucune partie jouée"
        } else {
            &maitrises
        };
        texte(commandes, infos, detail, GRIS, 1.5);
        if a_supprimer == Some(k) {
            texte(commandes, l, "Toute sa progression ?", ROUGE, 1.5);
            bouton(commandes, l, "Supprimer", Some(ActionProfils::Supprimer(k)));
            bouton(commandes, l, "Annuler", Some(ActionProfils::Annuler));
            continue;
        }
        if !actif {
            bouton(commandes, l, "Jouer avec", Some(ActionProfils::Jouer(k)));
        }
        if ids.len() > 1 {
            bouton(commandes, l, "Supprimer", Some(ActionProfils::Supprimer(k)));
        }
    }

    // Nouveau profil
    let l = ligne(commandes, racine, false);
    let libre = sauvegarde.profils.peut_creer();
    texte(commandes, l, "Nouveau profil :", OR, 2.0);
    let champ = match &menu.saisie {
        Some(s) => format!("{s}_"),
        None => menu.suggestion.clone(),
    };
    let saisie = libre.then_some(ActionProfils::Nouveau);
    bouton(commandes, l, &champ, saisie);
    bouton(commandes, l, "Créer", libre.then_some(ActionProfils::Creer));
    let aide = if !libre {
        "Nombre maximal de profils atteint."
    } else if menu.saisie.is_some() {
        "Entrée : créer · Échap : annuler"
    } else {
        "Cliquez sur le pseudo pour le changer"
    };
    texte(commandes, racine, aide, GRIS, 1.5);

    // Transfert
    let l = ligne(commandes, racine, false);
    bouton(
        commandes,
        l,
        "Exporter ce profil",
        Some(ActionProfils::Exporter),
    );
    let t = "Importer : glissez un fichier .bssave sur la fenêtre.";
    texte(commandes, l, t, GRIS, 1.5);
    if let Some(m) = &menu.message {
        let couleur = if m.starts_with("Profil exporté") {
            VERT
        } else {
            ROUGE
        };
        commandes.spawn((
            TextePixel::nouveau(m, couleur, 1.5).bloc(LARGEUR),
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
        .and_modify(|mut n| {
            n.margin = UiRect::top(Val::Auto);
        });
    racine
}
