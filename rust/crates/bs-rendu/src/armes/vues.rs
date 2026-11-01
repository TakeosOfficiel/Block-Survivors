//! Vues de la simulation pour l'affichage : chaque objet d'arme (de la partie, puis de
//! l'étalage de la vitrine) accompagné de ce qu'il faut pour le dessiner, lu dans la
//! définition de son arme. Hors de l'arsenal (vitrine), la première arme du genre.

use bs_donnees::StatsArme;
use bs_donnees::armes::{Arme, Mecanique};
use bs_regles::combat::pose::{FlecheTombante, Piege, ZonePluie};
use bs_regles::combat::sol::{Chute, Nova};
use bs_regles::combat::traque::Orbe;

use super::posees::{Fleche, Ronces};
use super::sol::{Givre, Lame};
use crate::PartieEnCours;

/// Zone de l'arme `arme` de l'arsenal (1 si elle n'y est pas : vitrine).
pub fn zone(jeu: &PartieEnCours, arme: usize) -> f32 {
    jeu.partie
        .arsenal
        .armes
        .get(arme)
        .map_or(1.0, |a| a.stats.zone)
}

/// L'arme `arme` de l'arsenal et ses stats ; hors de l'arsenal (vitrine), la première
/// arme du genre `genre` dans les données, avec ses stats de base.
pub fn definition(
    jeu: &PartieEnCours,
    arme: usize,
    genre: &str,
) -> Option<(&'static Arme, StatsArme)> {
    match jeu.partie.arsenal.armes.get(arme) {
        Some(a) if a.def.mecanique.nom() == genre => Some((a.def, a.stats)),
        _ => jeu
            .partie
            .d
            .armes
            .values()
            .find(|a| a.mecanique.nom() == genre)
            .map(|a| (a, a.stats)),
    }
}

/// Texture du météore de l'arme `arme`.
pub fn texture_meteore(jeu: &PartieEnCours, arme: usize) -> Option<&'static str> {
    match definition(jeu, arme, "meteore").map(|(a, _)| &a.mecanique) {
        Some(Mecanique::Meteore { texture, .. }) => Some(texture.as_str()),
        _ => None,
    }
}

/// Lames géantes de la simulation et de l'étalage.
pub fn lames<'a>(jeu: &'a PartieEnCours, etalage: &'a [Chute]) -> impl Iterator<Item = Lame<'a>> {
    let p = &jeu.partie;
    p.armes
        .sol
        .chutes
        .iter()
        .chain(etalage)
        .filter_map(move |c| {
            let (a, s) = definition(jeu, c.arme, "chute")?;
            let Mecanique::Chute { image, taille } = &a.mecanique else {
                return None;
            };
            Some(Lame {
                chute: c,
                image,
                taille: *taille,
                zone: s.zone,
            })
        })
}

/// Novas de la simulation et de l'étalage.
pub fn givres<'a>(jeu: &'a PartieEnCours, etalage: &'a [Nova]) -> impl Iterator<Item = Givre<'a>> {
    let p = &jeu.partie;
    p.armes
        .sol
        .novas
        .iter()
        .chain(etalage)
        .filter_map(move |n| {
            let (a, s) = definition(jeu, n.arme, "nova")?;
            let Mecanique::Nova {
                texture,
                expansion,
                duree_sol,
                ..
            } = &a.mecanique
            else {
                return None;
            };
            Some(Givre {
                nova: n,
                texture,
                zone: s.zone,
                expansion: *expansion,
                duree_sol: *duree_sol,
            })
        })
}

/// Zones de pluie de flèches.
pub fn pluies<'a>(
    jeu: &'a PartieEnCours,
    etalage: &'a [ZonePluie],
) -> impl Iterator<Item = &'a ZonePluie> {
    jeu.partie.armes.posees.zones.iter().chain(etalage)
}

/// Flèches de pluie (image : l'icône de l'arme).
pub fn fleches<'a>(
    jeu: &'a PartieEnCours,
    etalage: &'a [FlecheTombante],
) -> impl Iterator<Item = Fleche<'a>> {
    let p = &jeu.partie;
    p.armes
        .posees
        .fleches
        .iter()
        .chain(etalage)
        .filter_map(move |f| {
            let (a, _) = definition(jeu, f.arme, "pluie")?;
            Some(Fleche {
                fleche: f,
                image: &a.icone,
            })
        })
}

/// Pièges de ronces.
pub fn ronces<'a>(
    jeu: &'a PartieEnCours,
    etalage: &'a [Piege],
) -> impl Iterator<Item = Ronces<'a>> {
    let p = &jeu.partie;
    p.armes
        .posees
        .pieges
        .iter()
        .chain(etalage)
        .filter_map(move |pi| {
            let (a, s) = definition(jeu, pi.arme, "piege")?;
            let Mecanique::Piege {
                image, intervalle, ..
            } = &a.mecanique
            else {
                return None;
            };
            Some(Ronces {
                piege: pi,
                image,
                zone: s.zone,
                duree: s.duree,
                intervalle: *intervalle,
            })
        })
}

/// Orbes arcaniques et texture de leur bloc.
pub fn orbes<'a>(
    jeu: &'a PartieEnCours,
    etalage: &'a [Orbe],
) -> impl Iterator<Item = (&'a Orbe, &'a str)> {
    let p = &jeu.partie;
    p.armes
        .traque
        .orbes
        .iter()
        .chain(etalage)
        .filter_map(move |o| {
            let (a, _) = definition(jeu, o.arme, "orbes")?;
            match &a.mecanique {
                Mecanique::Orbes { texture, .. } => Some((o, texture.as_str())),
                _ => None,
            }
        })
}
