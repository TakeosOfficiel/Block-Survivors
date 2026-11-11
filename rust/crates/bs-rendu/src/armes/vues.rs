//! Vues de la simulation pour l'affichage : chaque objet d'arme (du joueur, des
//! coéquipiers si l'option est active, puis de l'étalage de la vitrine) accompagné de ce
//! qu'il faut pour le dessiner, lu dans la définition de son arme, dans l'arsenal de son
//! lanceur. Hors d'un arsenal (vitrine), la première arme du genre.

use bs_donnees::StatsArme;
use bs_donnees::armes::{Arme, Mecanique};
use bs_regles::combat::pose::{FlecheTombante, Piege, ZonePluie};
use bs_regles::combat::sol::{Chute, Nova};
use bs_regles::combat::traque::Orbe;
use bs_regles::combat::{Arsenal, SystemesArmes};

use super::posees::{Fleche, Ronces};
use super::sol::{Givre, Lame};
use crate::PartieEnCours;
use crate::arene::couleur_lineaire;
use crate::sources::sources;

/// Zone de l'arme `arme` de `arsenal` (1 si elle n'y est pas : vitrine).
pub fn zone(arsenal: Option<&Arsenal>, arme: usize) -> f32 {
    arsenal
        .and_then(|a| a.armes.get(arme))
        .map_or(1.0, |a| a.stats.zone)
}

/// L'arme `arme` de `arsenal` et ses stats ; hors d'un arsenal (vitrine), la première
/// arme du genre `genre` dans les données, avec ses stats de base.
pub fn definition(
    jeu: &PartieEnCours,
    arsenal: Option<&Arsenal>,
    arme: usize,
    genre: &str,
) -> Option<(&'static Arme, StatsArme)> {
    match arsenal.and_then(|a| a.armes.get(arme)) {
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

/// Les objets d'un genre (`champ`) de chaque source, avec l'arsenal de leur lanceur, puis
/// ceux de l'étalage (sans arsenal).
pub fn objets<'a, T>(
    jeu: &'a PartieEnCours,
    allies: bool,
    etalage: &'a [T],
    champ: fn(&SystemesArmes) -> &[T],
) -> impl Iterator<Item = (Option<&'a Arsenal>, &'a T)> + Clone {
    sources(&jeu.partie, allies)
        .flat_map(move |s| champ(s.armes).iter().map(move |o| (Some(s.arsenal), o)))
        .chain(etalage.iter().map(|o| (None, o)))
}

/// Texture du météore de l'arme `arme`.
pub fn texture_meteore(
    jeu: &PartieEnCours,
    arsenal: Option<&Arsenal>,
    arme: usize,
) -> Option<&'static str> {
    match definition(jeu, arsenal, arme, "meteore").map(|(a, _)| &a.mecanique) {
        Some(Mecanique::Meteore { texture, .. }) => Some(texture.as_str()),
        _ => None,
    }
}

/// Teinte linéaire d'une arme, ou `defaut` si elle n'en a pas.
fn teinte(a: &Arme, defaut: [f32; 3]) -> [f32; 3] {
    a.teinte.map_or(defaut, |t| {
        let c = couleur_lineaire(t);
        [c[0], c[1], c[2]]
    })
}

/// Lames géantes de la simulation et de l'étalage.
pub fn lames<'a>(
    jeu: &'a PartieEnCours,
    allies: bool,
    etalage: &'a [Chute],
) -> impl Iterator<Item = Lame<'a>> {
    objets(jeu, allies, etalage, |a| &a.sol.chutes).filter_map(move |(arsenal, c)| {
        let (a, s) = definition(jeu, arsenal, c.arme, "chute")?;
        let Mecanique::Chute { image, taille } = &a.mecanique else {
            return None;
        };
        Some(Lame {
            chute: c,
            image,
            taille: *taille,
            zone: s.zone,
            teinte: teinte(a, [0.75, 0.85, 1.0]),
        })
    })
}

/// Novas de la simulation et de l'étalage.
pub fn givres<'a>(
    jeu: &'a PartieEnCours,
    allies: bool,
    etalage: &'a [Nova],
) -> impl Iterator<Item = Givre<'a>> {
    objets(jeu, allies, etalage, |a| &a.sol.novas).filter_map(move |(arsenal, n)| {
        let (a, s) = definition(jeu, arsenal, n.arme, "nova")?;
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
            teinte: teinte(a, [0.35, 0.75, 1.0]),
        })
    })
}

/// Zones de pluie et teinte de leur arme.
pub fn pluies<'a>(
    jeu: &'a PartieEnCours,
    allies: bool,
    etalage: &'a [ZonePluie],
) -> impl Iterator<Item = (&'a ZonePluie, [f32; 3])> {
    objets(jeu, allies, etalage, |a| &a.posees.zones).map(move |(arsenal, z)| {
        let c = definition(jeu, arsenal, z.arme, "pluie").map(|(a, _)| teinte(a, VISEE));
        (z, c.unwrap_or(VISEE))
    })
}

/// Teinte d'une pluie sans arme connue.
const VISEE: [f32; 3] = [1.0, 0.85, 0.55];

/// Flèches de pluie (image : l'icône de l'arme).
pub fn fleches<'a>(
    jeu: &'a PartieEnCours,
    allies: bool,
    etalage: &'a [FlecheTombante],
) -> impl Iterator<Item = Fleche<'a>> {
    objets(jeu, allies, etalage, |a| &a.posees.fleches).filter_map(move |(arsenal, f)| {
        let (a, _) = definition(jeu, arsenal, f.arme, "pluie")?;
        Some(Fleche {
            fleche: f,
            image: &a.icone,
            teinte: teinte(a, [0.85, 0.9, 1.0]),
        })
    })
}

/// Pièges de ronces.
pub fn ronces<'a>(
    jeu: &'a PartieEnCours,
    allies: bool,
    etalage: &'a [Piege],
) -> impl Iterator<Item = Ronces<'a>> {
    objets(jeu, allies, etalage, |a| &a.posees.pieges).filter_map(move |(arsenal, pi)| {
        let (a, s) = definition(jeu, arsenal, pi.arme, "piege")?;
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
            teinte: teinte(a, [0.35, 1.0, 0.3]),
        })
    })
}

/// Orbes arcaniques et texture de leur bloc.
pub fn orbes<'a>(
    jeu: &'a PartieEnCours,
    allies: bool,
    etalage: &'a [Orbe],
) -> impl Iterator<Item = (&'a Orbe, &'a str)> {
    objets(jeu, allies, etalage, |a| &a.traque.orbes).filter_map(move |(arsenal, o)| {
        let (a, _) = definition(jeu, arsenal, o.arme, "orbes")?;
        match &a.mecanique {
            Mecanique::Orbes { texture, .. } => Some((o, texture.as_str())),
            _ => None,
        }
    })
}
