//! Recherche des mises à jour en arrière-plan, pour le jeu (plan, section 5) : un fil lit
//! le manifeste signé au lancement puis toutes les 30 min, télécharge la nouvelle version
//! sur sa voie (stable ou bêta) et la garde prête. Le jeu lit l'`Etat` à chaque image
//! sans attendre ; au menu, « Redémarrer » lance l'installeur de Velopack, qui attend la
//! fermeture du jeu, applique la version et relance. Sans redémarrage, Velopack
//! l'applique au lancement suivant. Hors installation Velopack (développement, archive)
//! ou sans clé publique compilée, rien n'est cherché.

use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use velopack::locator::{LocationContext, auto_locate_app_manifest};
use velopack::{UpdateCheck, UpdateManager, UpdateOptions, VelopackAsset};

use crate::source::{DepotHttps, SourceSignee};

/// Intervalle entre deux recherches.
const INTERVALLE: Duration = Duration::from_secs(30 * 60);
/// Dépôt public des versions : une release par canal (« rust-win-stable »…).
pub const DEPOT_PAR_DEFAUT: &str = "https://github.com/TakeosOfficiel/Block-Survivors-version/releases/download/rust-{canal}/{nom}";

#[derive(Clone, Debug, PartialEq)]
pub enum Etat {
    /// Pas de mises à jour ici (pourquoi).
    Inactive(String),
    Recherche,
    AJour,
    /// Pourcentage téléchargé.
    Telechargement(u8),
    /// Version téléchargée, appliquée au redémarrage.
    Prete(String),
    Erreur(String),
}

pub struct Reglages {
    /// Fichier du dernier numéro de série vu.
    pub fichier_serie: PathBuf,
    /// Voie bêta plutôt que stable.
    pub beta: bool,
    /// Version qui plantait au démarrage (`garde`) : jamais reproposée.
    pub refusee: Option<String>,
}

struct Partage {
    etat: Mutex<Etat>,
    /// Voie voulue, et « chercher maintenant » (réveille le fil).
    demande: Mutex<(bool, bool)>,
    reveil: Condvar,
    prete: Mutex<Option<(UpdateManager, VelopackAsset)>>,
}

/// Le service, tenu par le jeu ; son fil vit tant que le jeu tourne.
#[derive(Clone)]
pub struct MiseAJour {
    partage: Arc<Partage>,
}

impl MiseAJour {
    /// Démarre la recherche (ou un service inactif, sans fil).
    pub fn demarrer(r: Reglages) -> Self {
        let cles = crate::cles::cles_du_jeu();
        let installe = auto_locate_app_manifest(LocationContext::FromCurrentExe)
            .map(|l| l.get_manifest_channel());
        let etat = match (&installe, cles.is_empty()) {
            (Err(_), _) => Some("jeu non installé par Velopack"),
            (_, true) => Some("aucune clé de mise à jour dans ce build"),
            _ => None,
        };
        let partage = Arc::new(Partage {
            etat: Mutex::new(etat.map_or(Etat::Recherche, |e| Etat::Inactive(e.into()))),
            demande: Mutex::new((r.beta, false)),
            reveil: Condvar::new(),
            prete: Mutex::new(None),
        });
        if let (Ok(canal), None) = (installe, etat) {
            let p = partage.clone();
            let modele = std::env::var("BS_DEPOT_MAJ").unwrap_or_else(|_| DEPOT_PAR_DEFAUT.into());
            let lancement = std::thread::Builder::new()
                .name("mises à jour".into())
                .spawn(move || boucle(&p, &modele, cles, &canal, r));
            if let Err(e) = lancement {
                *partage.etat.lock().unwrap_or_else(|e| e.into_inner()) =
                    Etat::Erreur(e.to_string());
            }
        }
        Self { partage }
    }

    pub fn etat(&self) -> Etat {
        self.partage
            .etat
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Change de voie (option « bêta ») et cherche aussitôt.
    pub fn changer_voie(&self, beta: bool) {
        let mut d = self
            .partage
            .demande
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if d.0 != beta {
            *d = (beta, true);
            self.partage.reveil.notify_all();
        }
    }

    /// Lance l'installeur, qui attend la fin du jeu pour appliquer la version et le
    /// relancer. Le jeu doit ensuite se fermer (sauvegardes écrites).
    pub fn redemarrer(&self) -> Result<(), String> {
        let prete = self.partage.prete.lock().unwrap_or_else(|e| e.into_inner());
        let (um, paquet) = prete.as_ref().ok_or("aucune version prête")?;
        um.wait_exit_then_apply_updates(paquet, false, true, Vec::<String>::new())
            .map_err(|e| e.to_string())
    }
}

/// Canal Velopack voulu : système du canal installé (« win », « linux ») et voie.
pub fn canal_voulu(installe: &str, beta: bool) -> String {
    let systeme = installe.split('-').next().unwrap_or(installe);
    format!("{systeme}-{}", if beta { "beta" } else { "stable" })
}

fn boucle(
    p: &Partage,
    modele: &str,
    cles: Vec<ed25519_dalek::VerifyingKey>,
    installe: &str,
    r: Reglages,
) {
    let poser = |e: Etat| *p.etat.lock().unwrap_or_else(|e| e.into_inner()) = e;
    loop {
        let beta = p.demande.lock().unwrap_or_else(|e| e.into_inner()).0;
        let canal = canal_voulu(installe, beta);
        let options = UpdateOptions {
            // Revenir de la bêta à la stable descend de version : seulement en changeant de voie
            AllowVersionDowngrade: canal != installe,
            ExplicitChannel: Some(canal),
            // Paquets complets seulement (deltas sans gain sur l'exécutable)
            MaximumDeltasBeforeFallback: -1,
        };
        let source = SourceSignee::new(
            DepotHttps {
                modele: modele.into(),
            },
            cles.clone(),
            r.fichier_serie.clone(),
        );
        poser(Etat::Recherche);
        match chercher(source, options, r.refusee.as_deref(), &poser) {
            Ok(Some(prete)) => {
                poser(Etat::Prete(prete.1.Version.clone()));
                *p.prete.lock().unwrap_or_else(|e| e.into_inner()) = Some(prete);
            }
            Ok(None) => poser(Etat::AJour),
            Err(e) => poser(Etat::Erreur(e)),
        }
        // Attente : 30 min, ou un changement de voie
        let d = p.demande.lock().unwrap_or_else(|e| e.into_inner());
        let (mut d, _) = p
            .reveil
            .wait_timeout_while(d, INTERVALLE, |d| !d.1)
            .unwrap_or_else(|e| e.into_inner());
        d.1 = false;
    }
}

/// Une recherche : `Some` si une version a été téléchargée.
fn chercher(
    source: SourceSignee<DepotHttps>,
    options: UpdateOptions,
    refusee: Option<&str>,
    poser: &(dyn Fn(Etat) + Sync),
) -> Result<Option<(UpdateManager, VelopackAsset)>, String> {
    let um = UpdateManager::new(source, Some(options), None).map_err(|e| e.to_string())?;
    let info = match um.check_for_updates().map_err(|e| e.to_string())? {
        UpdateCheck::UpdateAvailable(info) => info,
        _ => return Ok(None),
    };
    if refusee == Some(info.TargetFullRelease.Version.as_str()) {
        return Ok(None);
    }
    let (envoi, reception) = mpsc::channel::<i16>();
    std::thread::scope(|s| {
        s.spawn(|| {
            for pct in reception {
                poser(Etat::Telechargement(pct.clamp(0, 100) as u8));
            }
        });
        um.download_updates(&info, Some(envoi))
    })
    .map_err(|e| e.to_string())?;
    Ok(Some((um, info.TargetFullRelease)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canaux() {
        assert_eq!(canal_voulu("win-stable", true), "win-beta");
        assert_eq!(canal_voulu("linux-beta", false), "linux-stable");
        // Canal par défaut de Velopack (sans voie)
        assert_eq!(canal_voulu("win", false), "win-stable");
    }

    #[test]
    fn hors_installation_inactif() {
        let m = MiseAJour::demarrer(Reglages {
            fichier_serie: std::env::temp_dir().join("bs-maj-essai-serie"),
            refusee: None,
            beta: false,
        });
        assert!(matches!(m.etat(), Etat::Inactive(_)));
        assert!(m.redemarrer().is_err());
    }
}
