//! Lumière des blocs à l'écran (`bs_monde::lumiere`) : les niveaux calculés au
//! chargement partent dans une texture 3D (un texel par bloc, lissée entre blocs) que
//! lisent les shaders des blocs et des entités (`shaders/lumiere.wgsl`). Chaque
//! matériau la reçoit à sa création, où qu'il soit créé ; sans source de lumière dans
//! l'arène, rien n'est envoyé.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bs_monde::lumiere::{NIVEAU_MAX, VolumeLumiere};

use crate::materiau::{MateriauBloc, ParametresBloc};
use crate::materiau_entite::MateriauEntite;

/// Côté maximal de la texture 3D (limite courante des cartes graphiques).
const COTE_MAX: usize = 2048;

/// Texture de lumière de l'arène et sa place dans le monde.
#[derive(Resource, Clone)]
pub struct VolumeLumineux {
    image: Handle<Image>,
    /// Coin de la boîte dans le monde, w à 1.
    volume: Vec4,
    taille: Vec4,
}

impl VolumeLumineux {
    fn poser(&self, parametres: &mut ParametresBloc, lumiere: &mut Option<Handle<Image>>) {
        parametres.volume = self.volume;
        parametres.taille = self.taille;
        *lumiere = Some(self.image.clone());
    }
}

/// Envoie la lumière `v` (son bloc d'index (0, 0, 0) a son coin en `coin` dans le monde).
pub fn inserer(
    commandes: &mut Commands,
    images: &mut Assets<Image>,
    v: &VolumeLumiere,
    coin: Vec3,
) {
    if v.sombre() {
        return;
    }
    let [l, h, p] = v.taille;
    if l.max(h).max(p) > COTE_MAX {
        warn!("lumière des blocs ignorée : arène trop grande ({l}×{h}×{p})");
        return;
    }
    let echelle = 255 / NIVEAU_MAX;
    let octets = v.niveaux.iter().map(|n| n * echelle).collect();
    let mut image = Image::new(
        Extent3d {
            width: l as u32,
            height: h as u32,
            depth_or_array_layers: p as u32,
        },
        TextureDimension::D3,
        octets,
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        address_mode_w: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });
    let o = Vec3::from_array(v.origine.map(|c| c as f32)) + coin;
    commandes.insert_resource(VolumeLumineux {
        image: images.add(image),
        volume: o.extend(1.0),
        taille: Vec4::new(l as f32, h as f32, p as f32, 0.0),
    });
}

/// Donne la lumière aux matériaux : à tous quand elle arrive, puis à chaque nouveau.
pub fn distribuer(
    volume: Option<Res<VolumeLumineux>>,
    mut nouveaux_blocs: MessageReader<AssetEvent<MateriauBloc>>,
    mut nouvelles_entites: MessageReader<AssetEvent<MateriauEntite>>,
    mut blocs: ResMut<Assets<MateriauBloc>>,
    mut entites: ResMut<Assets<MateriauEntite>>,
) {
    let Some(v) = volume else {
        nouveaux_blocs.clear();
        nouvelles_entites.clear();
        return;
    };
    if v.is_added() {
        nouveaux_blocs.clear();
        nouvelles_entites.clear();
        for (_, m) in blocs.iter_mut() {
            v.poser(&mut m.parametres, &mut m.lumiere);
        }
        for (_, m) in entites.iter_mut() {
            v.poser(&mut m.parametres, &mut m.lumiere);
        }
        return;
    }
    for e in nouveaux_blocs.read() {
        if let AssetEvent::Added { id } = e
            && let Some(m) = blocs.get_mut(*id)
        {
            v.poser(&mut m.parametres, &mut m.lumiere);
        }
    }
    for e in nouvelles_entites.read() {
        if let AssetEvent::Added { id } = e
            && let Some(m) = entites.get_mut(*id)
        {
            v.poser(&mut m.parametres, &mut m.lumiere);
        }
    }
}
