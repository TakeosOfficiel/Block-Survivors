//! Textures vanilla de Minecraft, jamais dans le dépôt : téléchargées au lancement depuis
//! les mêmes sources que la version TypeScript (GitHub, puis mcasset.cloud en secours),
//! gardées en cache sur le disque du PC.
//!
//! Une source d'assets Bevy « mc:// » essaie les adresses dans l'ordre : on charge
//! `mc://block/dirt.png` comme n'importe quel fichier. Une texture introuvable partout
//! reste absente : chaque système prend alors sa couleur de secours (comme en TS).
//! Même principe pour les fichiers JSON des maps importées (source « mcj », `fichiers`)
//! et pour les sons (source « mcs », serveur de ressources de Mojang, `sons`).

use std::path::{Path, PathBuf};

use bevy::asset::io::web::WebAssetReader;
use bevy::asset::io::{
    AssetReader, AssetReaderError, AssetSourceBuilder, PathStream, Reader, VecReader,
};
use bevy::asset::{LoadState, RenderAssetUsages};
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

/// Version des textures (celle de `VERSION_MINECRAFT` dans src/config.ts).
pub const VERSION_MINECRAFT: &str = "26.3";

/// Serveur de ressources de Mojang (src/config.ts `SOURCES_SONS`) : `<2 premiers
/// caractères de l'empreinte>/<empreinte>`.
const SOURCE_SONS: &str = "resources.download.minecraft.net";

/// Préfixes des sources, essayés dans l'ordre (sans « https:// »), pour un dossier de
/// `assets/minecraft` (« textures », ou vide pour les fichiers JSON).
fn sources(dossier: &str) -> [String; 2] {
    let suite = if dossier.is_empty() {
        String::new()
    } else {
        format!("/{dossier}")
    };
    [
        format!(
            "raw.githubusercontent.com/InventivetalentDev/minecraft-assets/{VERSION_MINECRAFT}/assets/minecraft{suite}"
        ),
        format!("assets.mcasset.cloud/{VERSION_MINECRAFT}/assets/minecraft{suite}"),
    ]
}

/// Lecteur d'une source vanilla (« mc », « mcj ») : cache disque, puis chaque adresse
/// distante.
struct LecteurMinecraft {
    sources: Vec<String>,
    cache: Option<PathBuf>,
    /// Adresses distantes sans l'extension du chemin (sons : rangés par empreinte).
    sans_extension: bool,
}

impl AssetReader for LecteurMinecraft {
    async fn read<'a>(&'a self, chemin: &'a Path) -> Result<Box<dyn Reader>, AssetReaderError> {
        if let Some(octets) = self.cache.as_ref().and_then(|c| lire_cache(c, chemin)) {
            return Ok(Box::new(VecReader::new(octets)));
        }
        let mut erreur = AssetReaderError::NotFound(chemin.to_path_buf());
        for source in &self.sources {
            let adresse = if self.sans_extension {
                Path::new(source).join(chemin.with_extension(""))
            } else {
                Path::new(source).join(chemin)
            };
            match WebAssetReader::Https.read(&adresse).await {
                Ok(mut lecteur) => {
                    let mut octets = Vec::new();
                    lecteur.read_to_end(&mut octets).await?;
                    if let Some(c) = &self.cache {
                        ecrire_cache(c, chemin, &octets);
                    }
                    return Ok(Box::new(VecReader::new(octets)));
                }
                Err(e) => erreur = e,
            }
        }
        Err(erreur)
    }

    async fn read_meta<'a>(
        &'a self,
        chemin: &'a Path,
    ) -> Result<Box<dyn Reader>, AssetReaderError> {
        // Pas de fichiers .meta : réglages par défaut, sans requête inutile
        Err(AssetReaderError::NotFound(chemin.to_path_buf()))
    }

    async fn read_directory<'a>(
        &'a self,
        chemin: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        Err(AssetReaderError::NotFound(chemin.to_path_buf()))
    }

    async fn is_directory<'a>(&'a self, _chemin: &'a Path) -> Result<bool, AssetReaderError> {
        Ok(false)
    }
}

/// Dossier du cache : `sorte` = « textures », « fichiers » ou « sons ».
fn dossier_cache(sorte: &str) -> Option<PathBuf> {
    let var = |nom: &str| std::env::var_os(nom).map(PathBuf::from);
    let base = if cfg!(windows) {
        var("LOCALAPPDATA")?.join("BlockSurvivors")
    } else if cfg!(target_os = "macos") {
        var("HOME")?.join("Library/Caches/BlockSurvivors")
    } else {
        var("XDG_CACHE_HOME")
            .or_else(|| var("HOME").map(|h| h.join(".cache")))?
            .join("block-survivors")
    };
    Some(base.join(sorte).join(VERSION_MINECRAFT))
}

fn lire_cache(dossier: &Path, chemin: &Path) -> Option<Vec<u8>> {
    std::fs::read(dossier.join(chemin)).ok()
}

/// Écriture au mieux : un cache impossible à écrire ne gêne pas le jeu.
fn ecrire_cache(dossier: &Path, chemin: &Path, octets: &[u8]) {
    let fichier = dossier.join(chemin);
    if let Some(parent) = fichier.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(fichier, octets);
}

/// Enregistre les sources « mc » (textures), « mcj » (états et modèles de blocs, JSON) et
/// « mcs » (sons).
/// À ajouter **avant** `DefaultPlugins` (avant `AssetPlugin`).
pub struct PluginSourceTextures;

impl Plugin for PluginSourceTextures {
    fn build(&self, app: &mut App) {
        app.register_asset_source(
            "mc",
            AssetSourceBuilder::new(|| {
                Box::new(LecteurMinecraft {
                    sources: sources("textures").into(),
                    cache: dossier_cache("textures"),
                    sans_extension: false,
                })
            }),
        )
        .register_asset_source(
            "mcj",
            AssetSourceBuilder::new(|| {
                Box::new(LecteurMinecraft {
                    sources: sources("").into(),
                    cache: dossier_cache("fichiers"),
                    sans_extension: false,
                })
            }),
        )
        .register_asset_source(
            "mcs",
            AssetSourceBuilder::new(|| {
                Box::new(LecteurMinecraft {
                    sources: vec![SOURCE_SONS.to_owned()],
                    cache: dossier_cache("sons"),
                    sans_extension: true,
                })
            }),
        );
    }
}

/// Textures demandées, par chemin vanilla (« block/dirt »).
#[derive(Resource, Default)]
pub struct Textures {
    poignees: HashMap<String, Handle<Image>>,
}

/// Où en est le chargement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Avancement {
    pub faites: usize,
    pub total: usize,
    pub manquantes: usize,
}

impl Textures {
    /// Lance le chargement de `chemin` (une seule fois par chemin). Pixels nets.
    pub fn demander(&mut self, serveur: &AssetServer, chemin: &str) {
        if self.poignees.contains_key(chemin) {
            return;
        }
        let poignee = serveur.load_with_settings(
            format!("mc://{chemin}.png"),
            |r: &mut ImageLoaderSettings| {
                r.sampler = ImageSampler::nearest();
                r.asset_usage = RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD;
            },
        );
        self.poignees.insert(chemin.to_owned(), poignee);
    }

    pub fn avancement(&self, serveur: &AssetServer) -> Avancement {
        let mut a = Avancement {
            faites: 0,
            total: self.poignees.len(),
            manquantes: 0,
        };
        for p in self.poignees.values() {
            match serveur.load_state(p) {
                LoadState::Loaded => a.faites += 1,
                LoadState::Failed(_) => {
                    a.faites += 1;
                    a.manquantes += 1;
                }
                _ => {}
            }
        }
        a
    }

    /// Chargement fini, réussi ou non (texture inconnue : faux).
    pub fn termine(&self, serveur: &AssetServer, chemin: &str) -> bool {
        self.poignees.get(chemin).is_some_and(|p| {
            matches!(
                serveur.load_state(p),
                LoadState::Loaded | LoadState::Failed(_)
            )
        })
    }

    /// Texture chargée, ou `None` si elle manque (couleur de secours).
    pub fn obtenir(&self, serveur: &AssetServer, chemin: &str) -> Option<Handle<Image>> {
        let p = self.poignees.get(chemin)?;
        matches!(serveur.load_state(p), LoadState::Loaded).then(|| p.clone())
    }
}
