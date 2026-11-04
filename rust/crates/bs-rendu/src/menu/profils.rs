//! Page Profils du menu (src/ui/ecranProfils.ts) : les profils de cette installation
//! (pseudo, maîtrise de chaque classe jouée), pour jouer avec un autre, en créer un (pseudo
//! tapé, ou un pseudo tiré au hasard), en supprimer un (second clic pour confirmer), ou
//! transférer le profil actif : export en `.bssave`, import en glissant un `.bssave` sur
//! la fenêtre. Changer de profil relit toutes ses données et ramène à l'accueil.
//! En menu de console (`console`) : à gauche les profils en grands intitulés (l'actif
//! ouvert, doré) ; à droite le nouveau profil, le transfert et la suppression. Sans cadre.

use bevy::prelude::*;
use bs_sauvegarde::profil_joueur::{nettoyer_pseudo, pseudo_aleatoire};
use bs_sauvegarde::profils::cle;

use super::console::{self, Police, accroche, aide, ecrit, intitule, surtitre};
use super::{ActionMenu, Icones, Menu, Page, air};
use crate::ecrans::charte::{AIDE, IVOIRE, OR_VIF, ROUGE, VERT_GAIN};
use crate::lancement::{self, DonneesJeu};
use crate::sauvegarde::Sauvegarde;

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

fn p(a: ActionProfils) -> Option<ActionMenu> {
    Some(ActionMenu::Profils(a))
}

/// Section de la scène : surtitre, avec un peu d'air au-dessus.
fn section(c: &mut Commands, ic: &Icones, parent: Entity, t: &str) -> Entity {
    let s = air::colonne(c, parent, 12.0);
    c.entity(s)
        .entry::<Node>()
        .and_modify(|mut n| n.margin = UiRect::top(Val::Px(22.0)));
    surtitre(c, ic, s, t);
    s
}

/// Scène : nouveau profil, transfert, suppression.
fn scene(
    c: &mut Commands,
    (dj, sauvegarde): (&DonneesJeu, &Sauvegarde),
    ic: &Icones,
    d: Entity,
    (menu, a_supprimer): (&Menu, Option<usize>),
) {
    surtitre(c, ic, d, "NOUVEAU PROFIL");
    accroche(c, ic, d, "Repars de zéro, sous un autre pseudo.");
    let libre = sauvegarde.profils.peut_creer();
    let (valeur, saisie) = match &menu.saisie {
        Some(s) => (format!("{s}|"), true),
        None => (menu.suggestion.clone(), false),
    };
    if libre {
        let r = air::rangee(c, d, 14.0);
        c.entity(r)
            .entry::<Node>()
            .and_modify(|mut n| n.margin = UiRect::vertical(Val::Px(8.0)));
        let action = ActionMenu::Profils(ActionProfils::Nouveau);
        let f = air::champ(c, ic, r, &valeur, saisie, action);
        c.entity(f).entry::<Node>().and_modify(|mut n| {
            n.width = Val::Px(320.0);
            n.height = Val::Px(52.0);
        });
        let b = air::bouton_vert(c, ic, r, "CRÉER", p(ActionProfils::Creer));
        c.entity(b)
            .entry::<Node>()
            .and_modify(|mut n| n.height = Val::Px(50.0));
    }
    let t = if !libre {
        "Nombre maximal de profils atteint."
    } else if saisie {
        "Entrée : créer · Échap : annuler"
    } else {
        "Clique sur le pseudo proposé pour le changer."
    };
    aide(c, ic, d, t);

    let s = section(c, ic, d, "TRANSFERT");
    let r = air::rangee(c, s, 18.0);
    air::lien(
        c,
        ic,
        r,
        "Exporter ce profil",
        (18.0, OR_VIF),
        p(ActionProfils::Exporter),
    );
    aide(
        c,
        ic,
        r,
        "ou glisse un fichier .bssave sur la fenêtre pour l'importer",
    );
    if let Some(m) = &menu.message {
        let couleur = if m.starts_with("Profil exporté") {
            VERT_GAIN
        } else {
            ROUGE
        };
        ecrit(c, ic, s, m, (Police::Grasse, 14.0, couleur));
    }

    let ids = sauvegarde.profils.ids();
    if ids.len() < 2 {
        return;
    }
    let s = section(c, ic, d, "SUPPRIMER UN PROFIL");
    for (k, id) in ids.iter().enumerate() {
        let (pseudo, _) = resumer(dj, sauvegarde, id);
        let r = air::rangee(c, s, 18.0);
        let e = ecrit(c, ic, r, &pseudo, (Police::Grasse, 17.0, IVOIRE));
        c.entity(e).insert(Node {
            min_width: Val::Px(180.0),
            ..default()
        });
        if a_supprimer == Some(k) {
            ecrit(
                c,
                ic,
                r,
                "Toute sa progression ?",
                (Police::Grasse, 15.0, ROUGE),
            );
            air::lien(
                c,
                ic,
                r,
                "Supprimer",
                (16.0, ROUGE),
                p(ActionProfils::Supprimer(k)),
            );
            air::lien(c, ic, r, "Annuler", (16.0, AIDE), p(ActionProfils::Annuler));
        } else {
            let suppr = p(ActionProfils::Supprimer(k));
            air::lien(c, ic, r, "Supprimer", (16.0, AIDE), suppr);
        }
    }
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
    let console::Page {
        racine,
        gauche,
        scene: d,
    } = console::page(c, ic, ("profils", "PROFILS"), (620.0, 700.0));
    // Les profils : l'actif ouvert, un clic sur un autre pour jouer avec
    for (k, id) in sauvegarde.profils.ids().iter().enumerate() {
        let actif = id == sauvegarde.profils.actif();
        let (pseudo, maitrises) = resumer(dj, sauvegarde, id);
        let detail = match (actif, maitrises.is_empty()) {
            (true, true) => "Actif · aucune partie jouée".to_owned(),
            (true, false) => format!("Actif · {maitrises}"),
            (false, true) => "Aucune partie jouée".to_owned(),
            (false, false) => maitrises,
        };
        let action = (!actif).then_some(ActionMenu::Profils(ActionProfils::Jouer(k)));
        intitule(c, ic, gauche, (&pseudo, &detail), (60.0, action));
    }
    scene(c, (dj, sauvegarde), ic, d, (menu, a_supprimer));
    let p = console::pied(c, ic, racine);
    aide(c, ic, p, "Clique sur un profil pour jouer avec.");
    racine
}
