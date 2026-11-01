//! Matériau des entités (shader `shaders/entites.wgsl`) : l'ambiance de l'arène comme
//! les blocs, découpe, deux faces, et la surcouche de couleur de chaque instance
//! rangée dans son `MeshTag` (les instances d'un même maillage restent groupées en un
//! seul appel de dessin). Sert aussi aux effets (projectiles, orbes, ombres) : voir
//! les options du shader.

use bevy::asset::embedded_asset;
use bevy::mesh::MeshTag;
use bevy::mesh::{MeshVertexBufferLayoutRef, VertexAttributeDescriptor};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

use crate::materiau::ParametresBloc;

const SHADER: &str = "embedded://bs_rendu/shaders/entites.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct MateriauEntite {
    #[uniform(0)]
    pub parametres: ParametresBloc,
    /// Absente : blanc (la couleur des sommets donne la couleur de secours).
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
}

impl Material for MateriauEntite {
    /// Translucide si `animation.w` vaut 1 (ombres) ; sinon opaque découpé.
    fn alpha_mode(&self) -> AlphaMode {
        if self.parametres.animation.w > 0.5 {
            AlphaMode::Blend
        } else {
            AlphaMode::Opaque
        }
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
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(5),
        ];
        descriptor.vertex.buffers = vec![layout.0.get_layout(&attributs)?];
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

/// Surcouche (r, g, b, force de 0 à 1) rangée dans un `MeshTag`.
pub fn tag_surcouche(s: [f32; 4]) -> MeshTag {
    let octet = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u32;
    MeshTag(octet(s[0]) | octet(s[1]) << 8 | octet(s[2]) << 16 | octet(s[3]) << 24)
}

pub struct PluginMateriauEntite;

impl Plugin for PluginMateriauEntite {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/entites.wgsl");
        // Shader des lueurs des armes (`armes::lueur`), rangé avec les autres
        embedded_asset!(app, "shaders/lueur.wgsl");
        app.add_plugins(MaterialPlugin::<MateriauEntite>::default());
    }
}
