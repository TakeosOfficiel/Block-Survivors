//! Console cachée (équivalent du panneau debug de src/ui/debug.ts) : ² ou F3 l'ouvre ou la
//! ferme en partie. En haut à gauche, sans cadre, en texte de console : les mesures
//! (images/s, durée d'une image, simulation, ennemis ; en coop, le réseau), les dernières
//! réponses et la ligne de saisie. Console ouverte, le clavier ne va plus au jeu ; Entrée
//! exécute (`commandes`), Échap ferme. La commande `mesures` garde les mesures affichées
//! console fermée ; `BS_MESURES` les affiche d'emblée. Rafraîchies quatre fois par seconde.

mod commandes;

use std::collections::VecDeque;
use std::fmt::Write;

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crate::coop::{Coop, Role};
use crate::ecrans::charte::{AIDE, IVOIRE, OR_VIF};
use crate::ecrans::nuit::{Style, ecrit};
use crate::polices::PolicesMenu;
use crate::{PartieEnCours, Phase};

const PERIODE: f32 = 0.25;
const TAILLE: f32 = 15.0;
/// Réponses gardées à l'écran.
const JOURNAL: usize = 10;
const SAISIE_MAX: usize = 60;

#[derive(Resource, Default)]
pub struct Console {
    ouverte: bool,
    /// Mesures affichées console fermée.
    mesures: bool,
    saisie: String,
    journal: VecDeque<String>,
    /// Triches en cours (solo).
    pub immortel: bool,
    textes: Option<[Entity; 3]>,
    racine: Option<Entity>,
    a_redessiner: bool,
    // Relevés des mesures
    minuterie: f32,
    images: u32,
    pire: f32,
    instantanes: u32,
    age: f32,
    ligne: String,
}

impl Console {
    fn ecrire(&mut self, texte: impl Into<String>) {
        if self.journal.len() == JOURNAL {
            self.journal.pop_front();
        }
        self.journal.push_back(texte.into());
        self.a_redessiner = true;
    }
}

/// À l'entrée en partie : les trois textes (mesures, réponses, saisie).
pub fn creer(mut commandes: Commands, polices: Res<PolicesMenu>, mut c: ResMut<Console>) {
    if c.racine.is_none() && std::env::var_os("BS_MESURES").is_some() {
        c.mesures = true;
    }
    let racine = commandes
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(12.0),
                top: Val::Px(12.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
            GlobalZIndex(50),
            DespawnOnExit(Phase::Partie),
        ))
        .id();
    let mut texte = |couleur| {
        let e = ecrit(
            &mut commandes,
            &polices,
            racine,
            "",
            (Style::Normale, TAILLE, couleur),
        );
        commandes.entity(e).insert(TextShadow {
            offset: Vec2::splat(1.0),
            color: Color::srgba(0.0, 0.0, 0.0, 0.85),
        });
        e
    };
    c.textes = Some([texte(IVOIRE), texte(AIDE), texte(OR_VIF)]);
    c.racine = Some(racine);
    c.minuterie = PERIODE;
    c.a_redessiner = true;
}

/// En sortie de partie : vitesse normale, triches oubliées.
pub fn quitter(mut c: ResMut<Console>, mut temps: ResMut<Time<Virtual>>) {
    c.ouverte = false;
    c.immortel = false;
    c.racine = None;
    c.textes = None;
    temps.set_relative_speed(1.0);
}

/// Avant le jeu : ouvre, ferme, saisit ; console ouverte, le clavier est vidé pour le jeu.
#[allow(clippy::too_many_arguments)]
pub fn clavier(
    mut lus: MessageReader<KeyboardInput>,
    mut touches: ResMut<ButtonInput<KeyCode>>,
    mut c: ResMut<Console>,
    jeu: Option<ResMut<PartieEnCours>>,
    coop: Option<Res<Coop>>,
    mut temps: ResMut<Time<Virtual>>,
) {
    let Some(mut jeu) = jeu.filter(|_| c.racine.is_some()) else {
        lus.clear();
        return;
    };
    let ouverte_avant = c.ouverte;
    for k in lus.read() {
        if k.state != ButtonState::Pressed {
            continue;
        }
        if matches!(k.key_code, KeyCode::F3 | KeyCode::Backquote) {
            if !k.repeat {
                c.ouverte = !c.ouverte;
                c.a_redessiner = true;
            }
            continue;
        }
        if !c.ouverte {
            continue;
        }
        c.a_redessiner = true;
        match &k.logical_key {
            Key::Escape => c.ouverte = false,
            Key::Backspace => {
                c.saisie.pop();
            }
            Key::Enter => {
                let ligne = std::mem::take(&mut c.saisie);
                if !ligne.trim().is_empty() {
                    c.ecrire(format!("> {ligne}"));
                    let en_coop = coop.is_some();
                    commandes::executer(&ligne, &mut c, &mut jeu, en_coop, &mut temps);
                }
            }
            _ => {
                if let Some(t) = &k.text {
                    for ch in t.chars().filter(|ch| !ch.is_control()) {
                        if c.saisie.chars().count() < SAISIE_MAX {
                            c.saisie.push(ch);
                        }
                    }
                }
            }
        }
    }
    if c.ouverte || ouverte_avant {
        touches.reset_all();
    }
}

/// Mesures, réponses et saisie à l'écran ; triche « immortel ».
pub fn afficher(
    temps: Res<Time<Real>>,
    mut c: ResMut<Console>,
    mut jeu: ResMut<PartieEnCours>,
    mut coop: Option<ResMut<Coop>>,
    mesure: Res<crate::mesure::Mesure>,
    mut noeuds: Query<&mut Node>,
    mut textes: Query<&mut Text>,
) {
    let (Some(racine), Some([t_mesures, t_journal, t_saisie])) = (c.racine, c.textes) else {
        return;
    };
    if c.immortel && coop.is_none() {
        let j = &mut jeu.partie.etat.joueur;
        j.pv = j.pv_max;
    }
    let dt = temps.delta_secs();
    c.images += 1;
    c.pire = c.pire.max(dt);
    c.age += dt;
    let hote = coop
        .as_deref()
        .filter(|k| k.est_hote())
        .map(|k| k.joueurs().len());
    let invite = coop.as_deref_mut().and_then(|k| match &mut k.role {
        Role::Invite(i) => i.recepteur.as_mut(),
        Role::Hote(_) => None,
    });
    if let Some(r) = &invite
        && r.instantanes != c.instantanes
    {
        c.age = 0.0;
    }
    let visible = c.ouverte || c.mesures;
    if std::mem::take(&mut c.a_redessiner) {
        if let Ok(mut n) = noeuds.get_mut(racine) {
            n.display = if visible {
                Display::Flex
            } else {
                Display::None
            };
        }
        let c = &mut *c;
        if let Ok(mut t) = textes.get_mut(t_journal) {
            t.0.clear();
            if c.ouverte {
                for (i, l) in c.journal.iter().enumerate() {
                    if i > 0 {
                        t.0.push('\n');
                    }
                    t.0.push_str(l);
                }
            }
        }
        if let Ok(mut t) = textes.get_mut(t_saisie) {
            t.0.clear();
            if c.ouverte {
                let _ = write!(t.0, "> {}_", c.saisie);
            }
        }
        c.minuterie = PERIODE;
    }
    c.minuterie += dt;
    if !visible || c.minuterie < PERIODE {
        return;
    }
    let duree = std::mem::take(&mut c.minuterie);
    let images = std::mem::take(&mut c.images).max(1);
    let pire = std::mem::take(&mut c.pire);
    let p = &jeu.partie;
    let c = &mut *c;
    c.ligne.clear();
    let _ = writeln!(
        c.ligne,
        "{:.0} images/s   image {:.1} ms (pire {:.1})   simulation {:.0} µs",
        images as f32 / duree,
        duree * 1000.0 / images as f32,
        pire * 1000.0,
        mesure.pas_moyen_us(),
    );
    let _ = write!(
        c.ligne,
        "ennemis {}   projectiles ennemis {}   temps {:.0} s",
        p.ennemis.nombre,
        p.projectiles_ennemis.pool.len(),
        p.etat.temps_jeu,
    );
    match (hote, invite) {
        (_, Some(r)) => {
            let recus = r.instantanes.wrapping_sub(c.instantanes);
            c.instantanes = r.instantanes;
            let _ = write!(
                c.ligne,
                "\ncoop invité   instantanés {:.0}/s   dernier il y a {:.0} ms   écart corrigé max {:.2} bloc",
                recus as f32 / duree,
                c.age * 1000.0,
                std::mem::take(&mut r.ecart_max),
            );
        }
        (Some(n), None) => {
            let _ = write!(c.ligne, "\ncoop hôte   joueurs {n}");
        }
        _ => {}
    }
    if let Ok(mut t) = textes.get_mut(t_mesures) {
        t.0.clone_from(&c.ligne);
    }
}
