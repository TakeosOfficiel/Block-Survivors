//! Fichiers JSON vanilla des maps importées (états et modèles de blocs), comme
//! `chargerJsonVanilla` de src/assets/fichiers.ts : chargés par la source « mcj »
//! (`mcj://blockstates/stone.json`), lus par la cuisson de `bs_monde::minecraft`.
//! Un fichier introuvable partout vaut `Absent` (le bloc prend un modèle de secours).

use bevy::asset::{AssetLoader, LoadContext, LoadState, io::Reader};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bs_monde::minecraft::modeles::{Fichier, SourceFichiers};
use serde_json::Value;

/// Un fichier JSON lu.
#[derive(Asset, TypePath, Debug)]
pub struct Json(pub Value);

#[derive(Default, TypePath)]
struct LecteurJson;

impl AssetLoader for LecteurJson {
    type Asset = Json;
    type Settings = ();
    type Error = BevyError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _reglages: &(),
        _contexte: &mut LoadContext<'_>,
    ) -> Result<Json, BevyError> {
        let mut octets = Vec::new();
        reader.read_to_end(&mut octets).await?;
        Ok(Json(serde_json::from_slice(&octets)?))
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

pub struct PluginFichiers;

impl Plugin for PluginFichiers {
    fn build(&self, app: &mut App) {
        app.init_asset::<Json>()
            .init_asset_loader::<LecteurJson>()
            .init_resource::<Fichiers>();
    }
}

/// Fichiers demandés, par chemin relatif à `assets/minecraft`.
#[derive(Resource, Default)]
pub struct Fichiers {
    poignees: HashMap<String, Handle<Json>>,
}

impl Fichiers {
    /// Lance le chargement de `chemin` (une seule fois par chemin).
    pub fn demander(&mut self, serveur: &AssetServer, chemin: &str) {
        if !self.poignees.contains_key(chemin) {
            let poignee = serveur.load(format!("mcj://{chemin}"));
            self.poignees.insert(chemin.to_owned(), poignee);
        }
    }

    /// Vue des fichiers pour la cuisson.
    pub fn vue<'a>(&'a self, serveur: &'a AssetServer, json: &'a Assets<Json>) -> Vue<'a> {
        Vue {
            fichiers: self,
            serveur,
            json,
        }
    }
}

pub struct Vue<'a> {
    fichiers: &'a Fichiers,
    serveur: &'a AssetServer,
    json: &'a Assets<Json>,
}

impl SourceFichiers for Vue<'_> {
    fn fichier(&self, chemin: &str) -> Fichier<'_> {
        let Some(p) = self.fichiers.poignees.get(chemin) else {
            return Fichier::EnAttente;
        };
        match self.serveur.load_state(p) {
            LoadState::Failed(_) => Fichier::Absent,
            LoadState::Loaded => self
                .json
                .get(p)
                .map_or(Fichier::Absent, |j| Fichier::Present(&j.0)),
            _ => Fichier::EnAttente,
        }
    }
}
