//! Matériau des blocs (shader `shaders/blocs.wgsl`) : éclairage, brouillard et ciel de
//! l'arène (comme `appliquerAmbiance` de src/moteur/rendu.ts), textures animées en bande
//! (eau, lave : on avance l'image affichée), découpe nette et faces doubles, feuillage
//! et plantes qui ondulent au vent (attribut `ATTRIBUTE_VENT`, src/monde/vegetation.ts).

use bevy::asset::embedded_asset;
use bevy::mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexAttributeDescriptor};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError, VertexFormat,
};
use bevy::shader::{ShaderDefVal, ShaderRef, load_shader_library};
use bs_donnees::arenes::Arene;

use crate::arene::couleur_lineaire;

/// Lumière de l'ombrage façon Minecraft (src/config.ts, `rendu.directionSoleil`).
pub const DIRECTION_SOLEIL: Vec3 = Vec3::new(0.3, 1.0, 0.5);

/// Vent de chaque sommet : centre x, pied y, centre z du bloc, mode (0 : immobile,
/// 1 : feuillage, 2 : plante).
pub const ATTRIBUTE_VENT: MeshVertexAttribute =
    MeshVertexAttribute::new("Vent", 0x6273_7665, VertexFormat::Float32x4);

const SHADER: &str = "embedded://bs_rendu/shaders/blocs.wgsl";
/// Passe d'ombre des blocs et des entités (`shaders/ombres.wgsl`).
pub const SHADER_OMBRES: &str = "embedded://bs_rendu/shaders/ombres.wgsl";

/// Passe d'ombre (et non rendu) : définie par Bevy pour les shaders de prépasse.
pub fn passe_d_ombre(descriptor: &RenderPipelineDescriptor) -> bool {
    descriptor
        .vertex
        .shader_defs
        .iter()
        .any(|d| matches!(d, ShaderDefVal::Bool(n, true) if n == "PREPASS_PIPELINE"))
}

/// Option `nom` des shaders de sommets et de fragments.
pub fn definir(descriptor: &mut RenderPipelineDescriptor, nom: &str) {
    descriptor.vertex.shader_defs.push(nom.into());
    if let Some(f) = descriptor.fragment.as_mut() {
        f.shader_defs.push(nom.into());
    }
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct ParametresBloc {
    pub ambiante: Vec4,
    pub soleil: Vec4,
    pub direction: Vec4,
    pub ciel: Vec4,
    /// Début, fin du brouillard, seuil de découpe (0 : aucune), lumineux (1).
    pub brouillard: Vec4,
    /// Images de la bande, image affichée, texture répétée (1 : terrain extérieur),
    /// alpha transmis (1 : matériau translucide).
    pub animation: Vec4,
    /// Lumière des blocs (`lumiere`) : coin de la boîte dans le monde, active (1).
    pub volume: Vec4,
    /// Taille de la boîte (blocs).
    pub taille: Vec4,
    /// Vue à travers (`transparence`) : pieds du joueur, rayon dégagé (0 : aucun).
    pub coupe: Vec4,
}

impl ParametresBloc {
    /// Ambiance de l'arène (lumières, ciel, brouillard).
    pub fn de_l_arene(a: &Arene) -> Self {
        let l = &a.lumiere;
        let rgb = |hex: u32, k: f32| {
            let c = couleur_lineaire(hex);
            Vec4::new(c[0] * k, c[1] * k, c[2] * k, 1.0)
        };
        Self {
            ambiante: rgb(l.couleur_ambiante, l.ambiante),
            soleil: rgb(l.couleur_soleil, l.soleil),
            direction: DIRECTION_SOLEIL.normalize().extend(0.0),
            ciel: rgb(a.ciel, 1.0),
            brouillard: Vec4::new(a.brouillard.debut, a.brouillard.fin, 0.0, 0.0),
            animation: Vec4::new(1.0, 0.0, 0.0, 0.0),
            volume: Vec4::ZERO,
            taille: Vec4::ONE,
            coupe: Vec4::ZERO,
        }
    }

    /// Ambiance de l'arène pour `MateriauEntite`, dont le shader lit `animation.x`
    /// autrement (1 : dessiné devant tout, sans profondeur) : remis à 0.
    pub fn pour_entites(a: &Arene) -> Self {
        let mut p = Self::de_l_arene(a);
        p.animation.x = 0.0;
        p
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[bind_group_data(CleMateriauBloc)]
pub struct MateriauBloc {
    #[uniform(0)]
    pub parametres: ParametresBloc,
    /// Absente : blanc (la couleur des sommets donne la couleur de secours).
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
    /// Niveaux de lumière des blocs (texture 3D, voir `lumiere`) ; posée après coup.
    #[texture(3, dimension = "3d")]
    #[sampler(4)]
    pub lumiere: Option<Handle<Image>>,
    /// Faces visibles des deux côtés (croix, feuilles, toiles).
    pub double_face: bool,
    /// Mélange alpha (verre teinté, eau des maps importées) ; l'alpha de la texture est
    /// alors transmis par le shader (`animation.w` à 1).
    pub translucide: bool,
}

#[repr(C)]
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct CleMateriauBloc {
    double_face: bool,
}

impl From<&MateriauBloc> for CleMateriauBloc {
    fn from(m: &MateriauBloc) -> Self {
        Self {
            double_face: m.double_face,
        }
    }
}

impl Material for MateriauBloc {
    /// Découpe : `Mask`, pour que la passe d'ombre découpe aussi (feuilles, fleurs).
    fn alpha_mode(&self) -> AlphaMode {
        if self.translucide {
            AlphaMode::Blend
        } else if self.parametres.brouillard.z > 0.0 {
            AlphaMode::Mask(self.parametres.brouillard.z)
        } else {
            AlphaMode::Opaque
        }
    }

    fn vertex_shader() -> ShaderRef {
        SHADER.into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        SHADER_OMBRES.into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        SHADER_OMBRES.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if key.bind_group_data.double_face {
            descriptor.primitive.cull_mode = None;
        }
        if passe_d_ombre(descriptor) {
            let attributs = [
                Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
                Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
                ATTRIBUTE_VENT.at_shader_location(6),
            ];
            descriptor.vertex.buffers = vec![layout.0.get_layout(&attributs)?];
            definir(descriptor, "BLOC");
            return Ok(());
        }
        let attributs: [VertexAttributeDescriptor; 5] = [
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(5),
            ATTRIBUTE_VENT.at_shader_location(6),
        ];
        descriptor.vertex.buffers = vec![layout.0.get_layout(&attributs)?];
        Ok(())
    }
}

/// Texture animée : son matériau, sa vitesse et son nombre d'images (connu au chargement).
#[derive(Component)]
pub struct AnimationTexture {
    pub materiau: Handle<MateriauBloc>,
    pub images_par_seconde: f32,
    pub aller_retour: bool,
}

pub struct PluginMateriaux;

impl Plugin for PluginMateriaux {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/blocs.wgsl");
        embedded_asset!(app, "shaders/ombres.wgsl");
        load_shader_library!(app, "shaders/vent.wgsl");
        load_shader_library!(app, "shaders/soleil.wgsl");
        load_shader_library!(app, "shaders/lumiere.wgsl");
        app.add_plugins(MaterialPlugin::<MateriauBloc>::default())
            .add_systems(Update, (animer_textures, couper_le_vent));
    }
}

/// Le vent suit l'option des ombres (« Ombres et vent » de la version TS : sans
/// shaders, sa force est nulle). `direction.w` à 1 : vent coupé. Appliqué aux
/// matériaux créés ensuite (nouvelle arène) comme à ceux déjà là.
fn couper_le_vent(
    ombres: Res<crate::soleil::OmbresSoleil>,
    mut evenements: MessageReader<AssetEvent<MateriauBloc>>,
    mut materiaux: ResMut<Assets<MateriauBloc>>,
) {
    let nouveaux = evenements
        .read()
        .any(|e| matches!(e, AssetEvent::Added { .. }));
    if !nouveaux && !ombres.is_changed() {
        return;
    }
    let coupe = if ombres.0 { 0.0 } else { 1.0 };
    let ids: Vec<_> = materiaux
        .iter()
        .filter(|(_, m)| m.parametres.direction.w != coupe)
        .map(|(id, _)| id)
        .collect();
    for id in ids {
        if let Some(m) = materiaux.get_mut(id) {
            m.parametres.direction.w = coupe;
        }
    }
}

/// Image affichée de chaque bande : comme `BibliothequeTextures.animer` en TS.
fn animer_textures(
    temps: Res<Time>,
    animations: Query<&AnimationTexture>,
    mut materiaux: ResMut<Assets<MateriauBloc>>,
    images: Res<Assets<Image>>,
) {
    for a in &animations {
        let Some(m) = materiaux.get(&a.materiau) else {
            continue;
        };
        let Some(image) = m.texture.as_ref().and_then(|t| images.get(t)) else {
            continue;
        };
        let taille = image.size();
        let n = (taille.y / taille.x.max(1)).max(1);
        let pas = (temps.elapsed_secs() * a.images_par_seconde) as u32;
        let image_affichee = if a.aller_retour && n > 1 {
            let cycle = 2 * (n - 1);
            let k = pas % cycle;
            if k < n { k } else { cycle - k }
        } else {
            pas % n
        };
        let anim = Vec4::new(
            n as f32,
            image_affichee as f32,
            m.parametres.animation.z,
            m.parametres.animation.w,
        );
        if m.parametres.animation != anim
            && let Some(m) = materiaux.get_mut(&a.materiau)
        {
            m.parametres.animation = anim;
        }
    }
}
