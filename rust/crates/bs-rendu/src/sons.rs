//! Sons de la partie (src/audio/sons.ts) : chaque son de la table des données est
//! téléchargé depuis le serveur de ressources de Mojang (source « mcs », cache disque),
//! décodé une fois par un chargeur d'assets, puis confié au mixeur de `bs_audio`. Les
//! sons demandés par la simulation (`Evenement::Son`) sont joués après chaque pas ;
//! un son pas encore téléchargé est simplement muet. `BS_SON=0` coupe tout.

use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::prelude::*;
use bs_audio::{Mixeur, StaticSoundData, decoder};
use bs_donnees::Donnees;
use bs_regles::evenements::Evenement;

use crate::PartieEnCours;

/// Échantillons décodés d'un son.
#[derive(Asset, TypePath)]
pub struct Echantillons(pub StaticSoundData);

#[derive(Default, TypePath)]
struct LecteurOgg;

impl AssetLoader for LecteurOgg {
    type Asset = Echantillons;
    type Settings = ();
    type Error = BevyError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _reglages: &(),
        _contexte: &mut LoadContext<'_>,
    ) -> Result<Echantillons, BevyError> {
        let mut octets = Vec::new();
        reader.read_to_end(&mut octets).await?;
        Ok(Echantillons(decoder(octets)?))
    }

    fn extensions(&self) -> &[&str] {
        &["ogg"]
    }
}

/// Mixeur et sons en cours de téléchargement (ressource du fil principal : la sortie
/// son n'est pas partageable entre fils).
pub struct Sons {
    d: &'static Donnees,
    mixeur: Mixeur,
    poignees: Vec<Handle<Echantillons>>,
    /// Sons déjà passés au mixeur.
    recus: usize,
}

impl Sons {
    pub fn nouveaux(d: &'static Donnees) -> Self {
        let muet = std::env::var("BS_SON").is_ok_and(|v| v == "0");
        let mixeur = Mixeur::nouveau(d.sons.len(), muet);
        if !muet && !mixeur.actif() {
            warn!("pas de sortie son : partie muette");
        }
        Self {
            d,
            mixeur,
            poignees: Vec::new(),
            recus: 0,
        }
    }

    /// Joue le son `id` (identifiant des données) ; `hauteur` multiplie la hauteur tirée.
    pub fn jouer(&mut self, id: &str, hauteur: f32, maintenant: f64) {
        if let Some((k, _, def)) = self.d.sons.get_full(id) {
            self.mixeur.jouer(k, def, hauteur, maintenant);
        }
    }
}

pub struct PluginSons;

impl Plugin for PluginSons {
    fn build(&self, app: &mut App) {
        app.init_asset::<Echantillons>()
            .init_asset_loader::<LecteurOgg>()
            .add_systems(Startup, demander)
            .add_systems(Update, recevoir);
    }
}

/// Lance le téléchargement de tous les sons (rien sans sortie son).
fn demander(mut sons: NonSendMut<Sons>, serveur: Res<AssetServer>) {
    if !sons.mixeur.actif() {
        return;
    }
    let poignees = sons
        .d
        .sons
        .values()
        .map(|s| {
            let h = &s.hachage;
            serveur.load(format!("mcs://{}/{h}.ogg", &h[..2.min(h.len())]))
        })
        .collect();
    sons.poignees = poignees;
}

/// Passe au mixeur les sons arrivés.
fn recevoir(mut sons: NonSendMut<Sons>, echantillons: Res<Assets<Echantillons>>) {
    if sons.recus == sons.poignees.len() {
        return;
    }
    let Sons {
        d,
        mixeur,
        poignees,
        recus,
    } = &mut *sons;
    for (k, p) in poignees.iter().enumerate() {
        if mixeur.pret(k) {
            continue;
        }
        if let Some(e) = echantillons.get(p) {
            let (_, def) = d.sons.get_index(k).expect("son des données");
            mixeur.charger(k, def, e.0.clone());
            *recus += 1;
        }
    }
}

/// Après chaque pas : sons demandés par la simulation.
pub fn jouer(jeu: Res<PartieEnCours>, mut sons: NonSendMut<Sons>, temps: Res<Time<Real>>) {
    let maintenant = temps.elapsed_secs_f64();
    for ev in &jeu.partie.ev.liste {
        if let Evenement::Son { son, hauteur } = *ev {
            sons.jouer(son, hauteur.unwrap_or(1.0), maintenant);
        }
    }
}
