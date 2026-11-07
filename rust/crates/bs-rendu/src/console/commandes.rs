//! Commandes de la console : mesures et réglages pour tous, triches en solo seulement
//! (en coop, l'hôte simule pour tout le monde). Un identifiant inconnu fait lister ceux
//! qui existent.

use bevy::prelude::*;
use bs_regles::cartes::Amelioration;

use super::Console;
use crate::PartieEnCours;

/// Distance d'apparition des ennemis demandés (blocs), comme `distanceApparition` du TS.
const DISTANCE: f32 = 7.0;
const VITESSES: (f32, f32) = (0.1, 16.0);

const AIDE: &str = "aide · mesures · vitesse <x> · immortel · soin · niveau [n] · emeraudes <n> \
     · temps <s> · tuer · arme <id> · passif <id> · ennemi <id> [n]";

pub fn executer(
    ligne: &str,
    c: &mut Console,
    jeu: &mut PartieEnCours,
    en_coop: bool,
    temps: &mut Time<Virtual>,
) {
    let mut mots = ligne.split_whitespace();
    let Some(nom) = mots.next() else {
        return;
    };
    let nom = nom.to_lowercase();
    let arg = mots.next();
    let nombre = |defaut: f32| {
        arg.and_then(|a| a.replace(',', ".").parse::<f32>().ok())
            .unwrap_or(defaut)
    };
    match nom.as_str() {
        "aide" | "help" | "?" => c.ecrire(AIDE),
        "mesures" => {
            c.mesures = !c.mesures;
            c.ecrire(if c.mesures {
                "mesures gardées console fermée"
            } else {
                "mesures seulement console ouverte"
            });
        }
        _ if en_coop => c.ecrire("en coop, seules « aide » et « mesures » marchent"),
        "vitesse" => {
            let v = nombre(1.0).clamp(VITESSES.0, VITESSES.1);
            temps.set_relative_speed(v);
            c.ecrire(format!("vitesse ×{v}"));
        }
        "immortel" | "god" => {
            c.immortel = !c.immortel;
            c.ecrire(if c.immortel { "immortel" } else { "mortel" });
        }
        "soin" => {
            let j = &mut jeu.partie.etat.joueur;
            j.pv = j.pv_max;
            c.ecrire("PV au maximum");
        }
        "niveau" => {
            let n = nombre(1.0).clamp(1.0, 50.0) as u32;
            jeu.partie.etat.niveaux_en_attente += n;
            c.ecrire(format!("{n} niveau(x) à choisir"));
        }
        "emeraudes" | "émeraudes" => {
            let n = nombre(100.0).max(0.0) as u32;
            jeu.partie.etat.emeraudes += n;
            c.ecrire(format!("+{n} émeraudes"));
        }
        "temps" => {
            let s = nombre(0.0).max(0.0);
            jeu.partie.etat.temps_jeu = f64::from(s);
            c.ecrire(format!("temps de jeu : {s} s"));
        }
        "tuer" => {
            jeu.partie.terrasser_tous();
            c.ecrire("ennemis terrassés");
        }
        "arme" => {
            let d = jeu.partie.d;
            match arg.and_then(|a| d.armes.get_index_of(a)) {
                Some(i) => {
                    jeu.partie.appliquer_amelioration(Amelioration::Arme(i));
                    c.ecrire(format!(
                        "arme {} : un rang de plus",
                        d.armes.get_index(i).map_or("", |a| a.0)
                    ));
                }
                None => lister(c, "armes", d.armes.keys()),
            }
        }
        "passif" => {
            let d = jeu.partie.d;
            match arg.and_then(|a| d.passifs.get_index_of(a)) {
                Some(i) => {
                    jeu.partie.appliquer_amelioration(Amelioration::Passif(i));
                    c.ecrire("passif : un rang de plus");
                }
                None => lister(c, "passifs", d.passifs.keys()),
            }
        }
        "ennemi" => {
            let d = jeu.partie.d;
            let n = mots
                .next()
                .and_then(|a| a.parse::<u32>().ok())
                .unwrap_or(1)
                .clamp(1, 100);
            match arg.and_then(|a| d.ennemis.get_index_of(a)) {
                Some(i) => {
                    let (x, z) = (jeu.partie.etat.joueur.x, jeu.partie.etat.joueur.z);
                    let mut poses = 0;
                    for k in 0..n {
                        let a = k as f32 * std::f32::consts::TAU / n as f32;
                        if jeu.partie.apparaitre(
                            i,
                            x + a.cos() * DISTANCE,
                            z + a.sin() * DISTANCE,
                            1.0,
                        ) {
                            poses += 1;
                        }
                    }
                    c.ecrire(format!("{poses} ennemi(s) apparu(s)"));
                }
                None => lister(c, "ennemis", d.ennemis.keys()),
            }
        }
        _ => c.ecrire(format!("commande inconnue « {nom} » ; tapez aide")),
    }
}

/// Liste les identifiants possibles, sur une ligne (coupée au-delà de 160 caractères).
fn lister<'a>(c: &mut Console, quoi: &str, ids: impl Iterator<Item = &'a String>) {
    let mut ligne = format!("{quoi} :");
    for id in ids {
        if ligne.len() > 160 {
            ligne.push_str(" …");
            break;
        }
        ligne.push(' ');
        ligne.push_str(id);
    }
    c.ecrire(ligne);
}
