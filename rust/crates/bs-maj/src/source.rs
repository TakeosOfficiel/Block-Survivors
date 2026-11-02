//! `SourceSignee` : source de mises à jour Velopack qui lit d'abord le manifeste signé
//! (`version.json`) sur un dépôt, le vérifie, puis ne propose **que** ses paquets ; chaque
//! fichier téléchargé est revérifié (SHA-256 du manifeste signé) avant d'être rendu à
//! Velopack, qui vérifie à son tour SHA-1 et SHA-256. Le dernier numéro de série vu est
//! gardé dans un fichier (refus des retours en arrière d'une session à l'autre).
//! Dépôts : un dossier local (essais) ou HTTPS (les releases du dépôt public de versions).

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::mpsc::Sender;

use ed25519_dalek::VerifyingKey;
use sha2::{Digest, Sha256};
use velopack::bundle::Manifest;
use velopack::sources::UpdateSource;
use velopack::{Error, VelopackAsset, VelopackAssetFeed};

use crate::manifeste::{Manifeste, verifier};

/// Nom du manifeste signé sur le dépôt.
pub const NOM_MANIFESTE: &str = "version.json";

/// Là où sont publiés manifeste et paquets, par canal Velopack (« win-stable »…).
pub trait Depot: Send + Sync {
    fn lire(&self, canal: &str, nom: &str) -> Result<Vec<u8>, Error>;
    /// Copie `nom` vers `vers` en donnant la progression (0 à 100).
    fn copier(
        &self,
        canal: &str,
        nom: &str,
        vers: &Path,
        progression: &mut dyn FnMut(i16),
    ) -> Result<(), Error>;
}

/// Dossier local (essais) : le même pour tous les canaux.
pub struct Dossier(pub PathBuf);

impl Depot for Dossier {
    fn lire(&self, _canal: &str, nom: &str) -> Result<Vec<u8>, Error> {
        Ok(std::fs::read(self.0.join(nom))?)
    }
    fn copier(
        &self,
        _canal: &str,
        nom: &str,
        vers: &Path,
        progression: &mut dyn FnMut(i16),
    ) -> Result<(), Error> {
        std::fs::copy(self.0.join(nom), vers)?;
        progression(100);
        Ok(())
    }
}

/// Dépôt HTTPS : `modele` est l'adresse d'un fichier où `{canal}` et `{nom}` sont
/// remplacés (ex. `https://github.com/…/releases/download/rust-{canal}/{nom}`).
pub struct DepotHttps {
    pub modele: String,
}

/// Délai maximal d'une requête (manifeste ou paquet).
const DELAI_MS: u64 = 10 * 60 * 1000;

impl DepotHttps {
    fn adresse(&self, canal: &str, nom: &str) -> String {
        self.modele.replace("{canal}", canal).replace("{nom}", nom)
    }
    fn options() -> velopack::HttpOptions {
        velopack::HttpOptions {
            TimeoutMilliseconds: DELAI_MS,
            ..Default::default()
        }
    }
}

impl Depot for DepotHttps {
    fn lire(&self, canal: &str, nom: &str) -> Result<Vec<u8>, Error> {
        let texte = velopack::download::download_url_as_string(
            &self.adresse(canal, nom),
            Some(&Self::options()),
        )?;
        Ok(texte.into_bytes())
    }
    fn copier(
        &self,
        canal: &str,
        nom: &str,
        vers: &Path,
        progression: &mut dyn FnMut(i16),
    ) -> Result<(), Error> {
        velopack::download::download_url_to_file(
            &self.adresse(canal, nom),
            vers,
            Some(&Self::options()),
            progression,
        )
    }
}

pub struct SourceSignee<D: Depot> {
    depot: D,
    cles: Vec<VerifyingKey>,
    /// Fichier du dernier numéro de série vu.
    fichier_serie: PathBuf,
    verifie: Mutex<Option<Manifeste>>,
}

impl<D: Depot> SourceSignee<D> {
    pub fn new(depot: D, cles: Vec<VerifyingKey>, fichier_serie: PathBuf) -> Self {
        Self {
            depot,
            cles,
            fichier_serie,
            verifie: Mutex::new(None),
        }
    }

    fn derniere_serie(&self) -> u64 {
        std::fs::read_to_string(&self.fichier_serie)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
    }
}

fn maintenant() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

impl<D: Depot> UpdateSource for SourceSignee<D> {
    fn get_release_feed(
        &self,
        voie: &str,
        _app: &Manifest,
        _id: &str,
    ) -> Result<VelopackAssetFeed, Error> {
        let octets = self.depot.lire(voie, NOM_MANIFESTE)?;
        let m = verifier(&octets, &self.cles, self.derniere_serie(), maintenant())
            .map_err(|r| Error::Other(r.to_string()))?;
        if m.voie != voie {
            return Err(Error::Other(format!(
                "manifeste de la voie « {} », attendu « {voie} »",
                m.voie
            )));
        }
        if let Some(dossier) = self.fichier_serie.parent() {
            let _ = std::fs::create_dir_all(dossier);
        }
        std::fs::write(&self.fichier_serie, m.serie.to_string())?;
        let paquets = m
            .paquets
            .iter()
            .map(|p| VelopackAsset {
                PackageId: p.id.clone(),
                Version: p.version.clone(),
                Type: p.genre.clone(),
                FileName: p.fichier.clone(),
                SHA1: p.sha1.clone(),
                SHA256: p.sha256.clone(),
                Size: p.taille,
                NotesMarkdown: p.notes.clone(),
                NotesHtml: String::new(),
            })
            .collect();
        *self.verifie.lock().unwrap_or_else(|e| e.into_inner()) = Some(m);
        Ok(VelopackAssetFeed { Assets: paquets })
    }

    fn download_release_entry(
        &self,
        asset: &VelopackAsset,
        local: &Path,
        progression: Option<Sender<i16>>,
    ) -> Result<(), Error> {
        // Seul un fichier du manifeste vérifié peut être téléchargé
        let (canal, attendu) = {
            let m = self.verifie.lock().unwrap_or_else(|e| e.into_inner());
            let m = m
                .as_ref()
                .ok_or_else(|| Error::Other("aucun manifeste vérifié".into()))?;
            let attendu = m
                .paquets
                .iter()
                .find(|p| p.fichier == asset.FileName)
                .map(|p| p.sha256.to_lowercase());
            (m.voie.clone(), attendu)
        };
        let attendu = attendu
            .ok_or_else(|| Error::Other(format!("{} absent du manifeste signé", asset.FileName)))?;
        self.depot
            .copier(&canal, &asset.FileName, local, &mut |pct| {
                if let Some(p) = &progression {
                    let _ = p.send(pct);
                }
            })?;
        let obtenu = sha256_fichier(local)?;
        if obtenu != attendu {
            let _ = std::fs::remove_file(local);
            return Err(Error::ChecksumInvalid(local.to_path_buf(), attendu, obtenu));
        }
        Ok(())
    }
}

pub fn sha256_fichier(chemin: &Path) -> std::io::Result<String> {
    let mut h = Sha256::new();
    let mut f = std::fs::File::open(chemin)?;
    std::io::copy(&mut f, &mut h)?;
    Ok(h.finalize().iter().map(|o| format!("{o:02x}")).collect())
}
