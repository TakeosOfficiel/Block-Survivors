//! Compilation d'un modèle cubique (src/modeles/compilation.ts) : parties indexées,
//! pose de base et membres animés désignés par index, pour animer sans chercher de
//! nom à chaque image.

use bevy::math::Vec3;
use bs_donnees::blocs::{Axe, Modele};

/// Pixels de texture par bloc (taille des modèles Minecraft).
pub const PIXELS_PAR_BLOC: f32 = 16.0;

#[derive(Debug, Clone)]
pub struct PartieCompilee {
    pub nom: &'static str,
    /// Articulation, en blocs.
    pub pivot: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct MembreMarche {
    pub partie: usize,
    /// 0 = x, 1 = y, 2 = z
    pub axe: usize,
    pub amplitude: f32,
    pub dephasage: f32,
    pub frequence: f32,
    pub absolu: bool,
}

#[derive(Debug, Clone)]
pub struct ModeleCompile {
    pub def: &'static Modele,
    pub parties: Vec<PartieCompilee>,
    /// Rotations (x, y, z) de base de chaque partie.
    pub pose_base: Vec<Vec3>,
    pub marche: Vec<MembreMarche>,
    /// (partie, signe)
    pub repos: Vec<(usize, f32)>,
    /// (partie, amplitude)
    pub attaque: Vec<(usize, f32)>,
    /// (partie, vitesse)
    pub tournoiement: Vec<(usize, f32)>,
    /// Index de la partie qui tient l'objet.
    pub main: Option<usize>,
    /// Sommet du modèle en pose de base (blocs).
    pub hauteur: f32,
}

impl ModeleCompile {
    /// Les parties citées existent : vérifié à la validation des données.
    pub fn nouveau(def: &'static Modele) -> Self {
        let parties: Vec<PartieCompilee> = def
            .parties
            .iter()
            .map(|(nom, c)| PartieCompilee {
                nom,
                pivot: Vec3::from_array(c.pivot) / PIXELS_PAR_BLOC,
            })
            .collect();
        let index = |nom: &str| {
            def.parties
                .get_index_of(nom)
                .unwrap_or_else(|| panic!("partie « {nom} » absente du modèle"))
        };
        let axe = |a: Axe| match a {
            Axe::X => 0,
            Axe::Y => 1,
            Axe::Z => 2,
        };
        Self {
            pose_base: def
                .parties
                .values()
                .map(|c| c.rotation.map_or(Vec3::ZERO, Vec3::from_array))
                .collect(),
            marche: def
                .marche
                .iter()
                .flat_map(|m| &m.membres)
                .map(|m| MembreMarche {
                    partie: index(&m.partie),
                    axe: axe(m.axe),
                    amplitude: m.amplitude,
                    dephasage: m.dephasage,
                    frequence: m.frequence.unwrap_or(1.0),
                    absolu: m.absolu,
                })
                .collect(),
            repos: def
                .repos
                .iter()
                .flat_map(|r| &r.membres)
                .map(|m| (index(&m.partie), f32::from(m.signe)))
                .collect(),
            attaque: def
                .attaque
                .iter()
                .flat_map(|a| &a.membres)
                .map(|m| (index(&m.partie), m.amplitude))
                .collect(),
            tournoiement: def
                .tournoiement
                .iter()
                .flatten()
                .map(|t| (index(&t.partie), t.vitesse))
                .collect(),
            main: def.main.as_ref().map(|m| index(&m.partie)),
            hauteur: def
                .parties
                .values()
                .map(|c| c.pivot[1] + c.centre[1] + c.taille[1] / 2.0)
                .fold(f32::MIN, f32::max)
                / PIXELS_PAR_BLOC,
            parties,
            def,
        }
    }
}
