//! Matériau « lueur » (shader `shaders/lueur.wgsl`) et son maillage réécrit à chaque
//! image : des quadrilatères dont la forme lumineuse (disque, faisceau, anneau,
//! flamme, traînée, étincelle) est calculée par le shader, ajoutés à la scène. Toutes
//! les lueurs des armes tiennent dans un seul appel de dessin.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::mesh::{
    Indices, MeshVertexBufferLayoutRef, PrimitiveTopology, VertexAttributeDescriptor,
    VertexAttributeValues,
};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

/// Embarqué par `materiau_entite` (les chemins d'`embedded_asset!` partent du fichier).
const SHADER: &str = "embedded://bs_rendu/shaders/lueur.wgsl";

/// Formes du shader.
#[derive(Clone, Copy)]
pub enum Forme {
    Disque = 0,
    Faisceau = 1,
    Anneau = 2,
    Flamme = 3,
    Trainee = 4,
    Etincelle = 5,
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct ParametresLueur {
    /// Début, fin du brouillard, intensité générale.
    pub brouillard: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct MateriauLueur {
    #[uniform(0)]
    pub parametres: ParametresLueur,
}

impl Material for MateriauLueur {
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }

    /// De la lumière : aucune ombre.
    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn vertex_shader() -> ShaderRef {
        SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let attributs: [VertexAttributeDescriptor; 4] = [
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            Mesh::ATTRIBUTE_UV_1.at_shader_location(3),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(5),
        ];
        descriptor.vertex.buffers = vec![layout.0.get_layout(&attributs)?];
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

pub struct PluginLueur;

impl Plugin for PluginLueur {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<MateriauLueur>::default());
    }
}

/// Maillage des lueurs de l'image (tampons gardés d'une image à l'autre).
pub struct MaillageLueur {
    poignee: Handle<Mesh>,
    entite: Entity,
    positions: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    formes: Vec<[f32; 2]>,
    couleurs: Vec<[f32; 4]>,
    indices: Vec<u32>,
    etait_vide: bool,
}

impl MaillageLueur {
    pub fn nouveau(
        commandes: &mut Commands,
        maillages: &mut Assets<Mesh>,
        materiau: Handle<MateriauLueur>,
    ) -> Self {
        let m = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, Vec::<[f32; 2]>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<[f32; 4]>::new())
        .with_inserted_indices(Indices::U32(Vec::new()));
        let poignee = maillages.add(m);
        let entite = commandes
            .spawn((
                Name::new("lueurs-armes"),
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
            formes: Vec::new(),
            couleurs: Vec::new(),
            indices: Vec::new(),
            etait_vide: true,
        }
    }

    pub fn commencer(&mut self) {
        self.positions.clear();
        self.uvs.clear();
        self.formes.clear();
        self.couleurs.clear();
        self.indices.clear();
    }

    /// Quadrilatère quelconque : coins dans l'ordre du tour, coordonnées de forme et
    /// couleurs par coin.
    pub fn quad(
        &mut self,
        coins: [Vec3; 4],
        uvs: [[f32; 2]; 4],
        couleurs: [[f32; 4]; 4],
        forme: Forme,
        reglage: f32,
    ) {
        let b = self.positions.len() as u32;
        self.positions.extend(coins.map(|c| c.to_array()));
        self.uvs.extend(uvs);
        self.formes.extend([[forme as u32 as f32, reglage]; 4]);
        self.couleurs.extend(couleurs);
        self.indices.extend([b, b + 2, b + 1, b, b + 3, b + 2]);
    }

    /// Carré de centre `centre` et de demi-côtés `droite`, `haut`.
    pub fn carre(
        &mut self,
        centre: Vec3,
        droite: Vec3,
        haut: Vec3,
        couleur: [f32; 4],
        forme: Forme,
        reglage: f32,
    ) {
        self.quad(
            [
                centre - droite + haut,
                centre + droite + haut,
                centre + droite - haut,
                centre - droite - haut,
            ],
            [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
            [couleur; 4],
            forme,
            reglage,
        );
    }

    /// Faisceau de `a` à `b`, tourné vers l'œil, de largeur `la` puis `lb`.
    #[allow(clippy::too_many_arguments)]
    pub fn faisceau(
        &mut self,
        oeil: Vec3,
        a: Vec3,
        b: Vec3,
        la: f32,
        lb: f32,
        ca: [f32; 4],
        cb: [f32; 4],
    ) {
        let axe = b - a;
        let vue = oeil - (a + b) * 0.5;
        let cote = axe.cross(vue).normalize_or_zero();
        self.quad(
            [a - cote * la, b - cote * lb, b + cote * lb, a + cote * la],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            [ca, cb, cb, ca],
            Forme::Faisceau,
            0.0,
        );
    }

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
        if let Some(VertexAttributeValues::Float32x3(v)) = m.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            v.clear();
            v.extend_from_slice(&self.positions);
        }
        if let Some(VertexAttributeValues::Float32x2(v)) = m.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
            v.clear();
            v.extend_from_slice(&self.uvs);
        }
        if let Some(VertexAttributeValues::Float32x2(v)) = m.attribute_mut(Mesh::ATTRIBUTE_UV_1) {
            v.clear();
            v.extend_from_slice(&self.formes);
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
