//! Pièces solides des armes, dessinées avec le matériau des entités (et non en
//! lueurs) : les blocs des météores, un lot par texture. Une texture animée (bande
//! d'images, comme le magma) ne montre que sa première image.

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use bs_donnees::Donnees;
use bs_donnees::armes::Mecanique;

use crate::effets::Lot;
use crate::materiau::ParametresBloc;
use crate::materiau_entite::{MateriauEntite, tag_surcouche};
use crate::modeles::rendu::Composants;
use crate::textures::Textures;

/// Côté d'un bloc de météore (part de la taille du météore).
const COTE_METEORE: f32 = 0.9;
/// Tournoiement des météores (rad/s).
const TOURNOIEMENT: f32 = 7.0;

pub struct Solides {
    /// Blocs des météores, par texture.
    meteores: Vec<(&'static str, Lot)>,
}

/// Textures des météores.
pub fn textures_meteores(d: &'static Donnees) -> impl Iterator<Item = &'static str> {
    d.armes.values().filter_map(|a| match &a.mecanique {
        Mecanique::Meteore { texture, .. } => Some(texture.as_str()),
        _ => None,
    })
}

/// Cube unité dont chaque face montre la première image carrée de la texture.
fn cube(proportion: f32) -> Mesh {
    let mut m = Mesh::from(Cuboid::from_length(1.0));
    let n = m.count_vertices();
    if let Some(VertexAttributeValues::Float32x2(uv)) = m.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
        for c in uv.iter_mut() {
            c[1] *= proportion;
        }
    }
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0f32; 4]; n]);
    m
}

impl Solides {
    pub fn nouveaux(
        d: &'static Donnees,
        ambiance: ParametresBloc,
        serveur: &AssetServer,
        textures: &Textures,
        images: &Assets<Image>,
        maillages: &mut Assets<Mesh>,
        materiaux: &mut Assets<MateriauEntite>,
    ) -> Self {
        // Bloc incandescent : non éclairé, découpé
        let mut parametres = ambiance;
        parametres.brouillard.z = 0.5;
        parametres.brouillard.w = 1.0;
        let mut meteores: Vec<(&'static str, Lot)> = Vec::new();
        for chemin in textures_meteores(d) {
            if meteores.iter().any(|(c, _)| *c == chemin) {
                continue;
            }
            let texture = textures.obtenir(serveur, chemin);
            let proportion = texture
                .as_ref()
                .and_then(|t| images.get(t))
                .map_or(1.0, |i| i.width() as f32 / i.height().max(1) as f32);
            let lot = Lot::nouveau(
                "bloc-meteore",
                maillages.add(cube(proportion)),
                materiaux.add(MateriauEntite {
                    parametres,
                    texture,
                }),
            );
            meteores.push((chemin, lot));
        }
        Self { meteores }
    }

    /// Commence l'image : aucun bloc posé.
    pub fn commencer(&mut self) {
        for (_, l) in &mut self.meteores {
            l.commencer();
        }
    }

    /// Pose un bloc de météore de texture `chemin`.
    pub fn meteore(
        &mut self,
        commandes: &mut Commands,
        q: &mut Composants,
        chemin: &str,
        position: Vec3,
        taille: f32,
        temps: f32,
    ) {
        let Some((_, lot)) = self.meteores.iter_mut().find(|(c, _)| *c == chemin) else {
            return;
        };
        let rotation =
            Quat::from_axis_angle(Vec3::new(1.0, 0.3, 1.0).normalize(), temps * TOURNOIEMENT);
        let tf = Transform {
            translation: position,
            rotation,
            scale: Vec3::splat(COTE_METEORE * taille),
        };
        lot.ajouter(commandes, q, tf, &tag_surcouche([0.0; 4]));
    }

    pub fn terminer(&mut self, q: &mut Composants) {
        for (_, l) in &mut self.meteores {
            l.terminer(q);
        }
    }
}
