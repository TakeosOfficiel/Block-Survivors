//! Mise à jour du HUD à chaque image (src/ui/hud.ts `majVisuel`) et après chaque pas
//! (annonces de boss) : textes, bandeaux, voile rouge, totem qui jaillit, jauges et
//! cases d'armes. Rien n'est réécrit tant que la valeur affichée ne change pas.

use bevy::prelude::*;
use bs_regles::evenements::Evenement;

use super::police::{TextePixel, colorer, ecrire};
use super::*;
use crate::PartieEnCours;
use crate::dessins::Dessins;
use crate::textures::Textures;

/// s d'affichage du message de butin d'un boss.
const DUREE_BUTIN: f32 = 4.0;

/// Après chaque pas : annonces de boss et prolongation.
pub fn recevoir(
    jeu: Res<PartieEnCours>,
    hud: Option<ResMut<Hud>>,
    mut textes: Query<&mut TextePixel>,
) {
    let Some(mut hud) = hud else {
        return;
    };
    for ev in &jeu.partie.ev.liste {
        match *ev {
            Evenement::AnnonceBoss { nom, duree } => alerter(
                &mut hud,
                &mut textes,
                &format!("Un boss approche : {nom} !"),
                duree,
            ),
            Evenement::ButinBoss { monde, pieces } => {
                let armure = jeu.partie.d.armures.armures.get(&monde);
                let noms: Vec<&str> = pieces
                    .iter()
                    .flatten()
                    .filter_map(|p| armure?.pieces.get(p).map(|a| a.nom.as_str()))
                    .collect();
                let texte = format!("Butin du boss : {} !", noms.join(" et "));
                afficher_message(&mut hud, &mut textes, &texte, DUREE_BUTIN);
            }
            _ => {}
        }
    }
    if jeu.partie.prolongation && !hud.prolongation_vue {
        alerter(
            &mut hud,
            &mut textes,
            "Vainquez le boss pour gagner !",
            DUREE_ANNONCE_PROLONGATION,
        );
    }
    hud.prolongation_vue = jeu.partie.prolongation;
}

/// Message jaune au-dessus du centre pendant `duree` s.
pub fn afficher_message(
    hud: &mut Hud,
    textes: &mut Query<&mut TextePixel>,
    texte: &str,
    duree: f32,
) {
    if let Ok(mut t) = textes.get_mut(hud.message) {
        ecrire(&mut t, texte);
    }
    hud.minuterie_message = duree;
}

/// Bandeau d'alerte rouge clignotant pendant `duree` s.
pub fn alerter(hud: &mut Hud, textes: &mut Query<&mut TextePixel>, texte: &str, duree: f32) {
    if let Ok(mut t) = textes.get_mut(hud.alerte_texte) {
        ecrire(&mut t, texte);
    }
    hud.minuterie_alerte = duree;
}

/// Chrono, prochain boss, compteurs, bandeaux, voile et totems.
#[allow(clippy::too_many_arguments)]
pub fn afficher(
    mut commandes: Commands,
    serveur: Res<AssetServer>,
    textures: Res<Textures>,
    dessins: Res<Dessins>,
    jeu: Res<PartieEnCours>,
    hud: Option<ResMut<Hud>>,
    temps: Res<Time>,
    mut noeuds: Query<&mut Node>,
    mut textes: Query<&mut TextePixel>,
    mut visibles: Query<&mut Visibility>,
    mut images: Query<&mut ImageNode>,
    mut transformations: Query<&mut UiTransform>,
    mut cache: Local<Option<(i64, i64, u32, u32, u32)>>,
) {
    let Some(mut hud) = hud else {
        return;
    };
    let dt = temps.delta_secs();
    hud.temps += dt;
    let p = &jeu.partie;
    let e = &p.etat;
    let seconde = e.temps_jeu.floor() as i64;
    // −2 : temps écoulé, boss encore vivant ; −1 : plus de boss avant la fin
    let dans = p.temps_avant_boss();
    let boss = if p.prolongation {
        -2
    } else {
        dans.map_or(-1, |s| s.ceil() as i64)
    };
    let restants = e.modificateurs.vies.max(0.0) as u32;
    let restants = restants.saturating_sub(e.joueur.vies_utilisees);
    let valeurs = (seconde, boss, e.ennemis_tues, e.emeraudes, restants);
    let avant = cache.replace(valeurs);
    if avant != Some(valeurs) {
        let mut texte = |entite: Entity, s: &str, couleur: Option<Color>| {
            if let Ok(mut t) = textes.get_mut(entite) {
                ecrire(&mut t, s);
                if let Some(c) = couleur {
                    colorer(&mut t, c);
                }
            }
        };
        let reste = i64::from(p.d.regles.progression.duree_partie as u32) - seconde;
        let final_ = (0..=DERNIER_COMPTE).contains(&reste);
        texte(
            hud.chrono,
            &formater_temps(seconde),
            Some(if final_ { OR } else { BLANC }),
        );
        let (s, proche) = match boss {
            -2 => ("Vainquez le boss pour gagner !".to_owned(), true),
            -1 => (String::new(), false),
            b => (format!("Prochain boss dans {}", formater_temps(b)), b <= 10),
        };
        texte(
            hud.prochain_boss,
            &s,
            Some(if proche { ROUGE } else { GRIS }),
        );
        texte(hud.tues, &e.ennemis_tues.to_string(), None);
        texte(hud.emeraudes, &e.emeraudes.to_string(), None);
        texte(hud.totems_nombre, &format!("×{restants}"), None);
        if let Ok(mut v) = visibles.get_mut(hud.totems) {
            v.set_if_neq(if restants > 0 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
        }
    }

    // Bandeaux
    hud.minuterie_message -= dt;
    hud.minuterie_alerte -= dt;
    let message = hud.minuterie_message > 0.0;
    let alerte = hud.minuterie_alerte > 0.0;
    let mut montrer = |entite: Entity, oui: bool| {
        if let Ok(mut v) = visibles.get_mut(entite) {
            v.set_if_neq(if oui {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
        }
    };
    montrer(hud.message, message);
    // Clignote : deux états par demi-seconde
    let clignote = (hud.temps * 4.0) as i64 % 2 == 0;
    montrer(hud.alerte, alerte);
    if let Ok(mut i) = images.get_mut(hud.alerte_texte) {
        let a = if clignote { 1.0 } else { 0.35 };
        if i.color.alpha() != a {
            i.color.set_alpha(a);
        }
    }

    // Voile rouge : fort juste après un coup, pulsant quand la santé est basse
    let j = &e.joueur;
    let blessure = if j.mort {
        0.0
    } else {
        (j.invulnerabilite / p.d.regles.joueur.invulnerabilite).clamp(0.0, 1.0) * VOILE_BLESSURE
    };
    let sante_basse = if !j.mort && j.pv / j.pv_max <= SEUIL_SANTE_BASSE {
        VOILE_SANTE_BASSE * (0.6 + 0.4 * (hud.temps * PULSATION).sin())
    } else {
        0.0
    };
    let opacite = (blessure.max(sante_basse) * 100.0).round() / 100.0;
    if let Ok(mut i) = images.get_mut(hud.voile)
        && i.color.alpha() != opacite
    {
        i.color.set_alpha(opacite);
    }

    // Totem qui jaillit à chaque résurrection
    if j.vies_utilisees != hud.vies_vues {
        hud.vies_vues = j.vies_utilisees;
        hud.animation_totem = 0.0;
    }
    if hud.animation_totem >= 0.0 {
        hud.animation_totem += dt;
        let fini = hud.animation_totem >= DUREE_TOTEM;
        montrer(hud.totem_anim, !fini);
        let (ty, s, r, a) = cles_totem(hud.animation_totem / DUREE_TOTEM);
        if let Ok(mut t) = transformations.get_mut(hud.totem_anim) {
            t.translation = Val2::px(0.0, ty * 192.0);
            t.scale = Vec2::splat(s);
            t.rotation = Rot2::degrees(r);
        }
        if let Ok(mut i) = images.get_mut(hud.totem_anim) {
            i.color.set_alpha(a);
        }
        if fini {
            hud.animation_totem = -1.0;
        }
    }
    let hud = &mut *hud;
    coeurs::afficher(
        &mut hud.coeurs,
        &mut commandes,
        p,
        dt,
        &mut textes,
        &mut images,
        &mut noeuds,
    );
    barres::afficher(
        &mut hud.barres,
        p,
        &mut textes,
        &mut images,
        &mut noeuds,
        &mut visibles,
    );
    capacites::afficher(
        &mut hud.capacites,
        &mut commandes,
        &textures,
        &serveur,
        &dessins,
        p,
    );
}

/// Images clés de l'animation du totem (styles.css `@keyframes totem`) : décalage
/// vertical (part de la taille), échelle, rotation (degrés), opacité.
fn cles_totem(t: f32) -> (f32, f32, f32, f32) {
    const CLES: [(f32, f32, f32, f32, f32); 5] = [
        (0.0, 0.7, 0.2, -30.0, 1.0),
        (0.25, 0.0, 1.15, 8.0, 1.0),
        (0.4, 0.0, 1.0, 0.0, 1.0),
        (0.75, -0.07, 1.03, 0.0, 1.0),
        (1.0, -0.1, 1.05, 0.0, 0.0),
    ];
    let t = t.clamp(0.0, 1.0);
    let k = CLES
        .iter()
        .rposition(|c| c.0 <= t)
        .unwrap_or(0)
        .min(CLES.len() - 2);
    let (a, b) = (CLES[k], CLES[k + 1]);
    let f = ((t - a.0) / (b.0 - a.0)).clamp(0.0, 1.0);
    // Sortie adoucie (ease-out)
    let f = 1.0 - (1.0 - f) * (1.0 - f);
    let m = |x: f32, y: f32| x + (y - x) * f;
    (m(a.1, b.1), m(a.2, b.2), m(a.3, b.3), m(a.4, b.4))
}
