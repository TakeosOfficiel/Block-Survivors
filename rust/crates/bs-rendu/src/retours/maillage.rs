//! Maillage réécrit à chaque image (particules, chiffres, barres de vie) : des carrés
//! orientés par l'appelant (face à la caméra), rassemblés en un seul appel de dessin.
//! Les tampons gardent leur capacité d'une image à l'autre : aucune allocation une
//! fois le record de carrés atteint.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;

use crate::materiau_entite::MateriauEntite;

pub struct MaillageDynamique {
    poignee: Handle<Mesh>,
    entite: Entity,
    positions: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    couleurs: Vec<[f32; 4]>,
    indices: Vec<u32>,
    /// Vide à l'image précédente (rien à renvoyer à la carte graphique).
    etait_vide: bool,
}

impl MaillageDynamique {
    pub fn nouveau(
        commandes: &mut Commands,
        maillages: &mut Assets<Mesh>,
        materiau: Handle<MateriauEntite>,
        nom: &str,
    ) -> Self {
        let m = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<[f32; 4]>::new())
        .with_inserted_indices(Indices::U32(Vec::new()));
        let poignee = maillages.add(m);
        let entite = commandes
            .spawn((
                Name::new(nom.to_owned()),
                Mesh3d(poignee.clone()),
                MeshMaterial3d(materiau),
                Transform::default(),
                Visibility::Hidden,
                NoFrustumCulling,
            ))
            .id();
        Self {
            poignee,
            entite,
            positions: Vec::new(),
            uvs: Vec::new(),
            couleurs: Vec::new(),
            indices: Vec::new(),
            etait_vide: true,
        }
    }

    pub fn commencer(&mut self) {
        self.positions.clear();
        self.uvs.clear();
        self.couleurs.clear();
        self.indices.clear();
    }

    /// Carré de centre `centre`, demi-côtés `droite` et `haut` ; `uv` = (u0, v0, u1, v1),
    /// v vers le bas.
    pub fn carre(
        &mut self,
        centre: Vec3,
        droite: Vec3,
        haut: Vec3,
        [u0, v0, u1, v1]: [f32; 4],
        couleur: [f32; 4],
    ) {
        let b = self.positions.len() as u32;
        self.positions.extend([
            (centre - droite + haut).to_array(),
            (centre + droite + haut).to_array(),
            (centre + droite - haut).to_array(),
            (centre - droite - haut).to_array(),
        ]);
        self.uvs.extend([[u0, v0], [u1, v0], [u1, v1], [u0, v1]]);
        self.couleurs.extend([couleur; 4]);
        self.indices.extend([b, b + 2, b + 1, b, b + 3, b + 2]);
    }

    /// Envoie les carrés de l'image (et cache le maillage s'il n'y en a aucun).
    pub fn terminer(
        &mut self,
        maillages: &mut Assets<Mesh>,
        visibilites: &mut Query<&mut Visibility>,
    ) {
        let vide = self.positions.is_empty();
        if let Ok(mut v) = visibilites.get_mut(self.entite) {
            v.set_if_neq(if vide {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            });
        }
        if vide && self.etait_vide {
            return;
        }
        self.etait_vide = vide;
        let Some(m) = maillages.get_mut(&self.poignee) else {
            return;
        };
        let n = self.positions.len();
        if let Some(VertexAttributeValues::Float32x3(v)) = m.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            v.clear();
            v.extend_from_slice(&self.positions);
        }
        // Normales sans usage (matériaux non éclairés), mais exigées par le shader
        if let Some(VertexAttributeValues::Float32x3(v)) = m.attribute_mut(Mesh::ATTRIBUTE_NORMAL) {
            v.resize(n, [0.0, 0.0, 1.0]);
        }
        if let Some(VertexAttributeValues::Float32x2(v)) = m.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
            v.clear();
            v.extend_from_slice(&self.uvs);
        }
        if let Some(VertexAttributeValues::Float32x4(v)) = m.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
            v.clear();
            v.extend_from_slice(&self.couleurs);
        }
        if let Some(Indices::U32(v)) = m.indices_mut() {
            v.clear();
            v.extend_from_slice(&self.indices);
        }
    }
}
