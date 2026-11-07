//! Réglages du relais (délais et limites, injectables dans les tests) et limites par
//! adresse IP : salons ouverts par adresse, tentatives de `Rejoindre`/`Reprendre` sur une
//! fenêtre glissante (contre l'essai de tous les codes).

use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::ops::RangeInclusive;
use std::time::{Duration, Instant};

use bs_protocole::VERSION_PROTOCOLE;

/// Délai pendant lequel la place d'un joueur coupé (hôte ou invité) lui reste réservée.
pub const DELAI_RECONNEXION: Duration = Duration::from_secs(60);
/// Intervalle des Ping WebSocket envoyés par le relais (Cloudflare coupe vers 100 s).
pub const BATTEMENT: Duration = Duration::from_secs(30);
/// Une connexion silencieuse (pas même un Pong) plus longtemps est fermée.
pub const SILENCE_MAX: Duration = Duration::from_secs(90);
/// Un salon sans aucun message aussi longtemps est fermé.
pub const INACTIVITE_SALON: Duration = Duration::from_secs(10 * 60);
/// Salons ouverts en même temps par une même adresse.
pub const SALONS_PAR_ADRESSE: u32 = 3;
/// Tentatives de `Rejoindre`/`Reprendre` par adresse sur `FENETRE_TENTATIVES`.
pub const TENTATIVES: usize = 20;
pub const FENETRE_TENTATIVES: Duration = Duration::from_secs(60);
/// Taille maximale d'un message WebSocket reçu.
pub const MESSAGE_MAX: usize = 256 * 1024;
/// Intervalle de l'entretien (places réservées expirées, salons inactifs).
pub const ENTRETIEN: Duration = Duration::from_secs(1);

/// Délais et limites du relais ; `Default` donne les vraies valeurs, les tests les
/// raccourcissent.
#[derive(Debug, Clone)]
pub struct Reglages {
    pub delai_reconnexion: Duration,
    pub battement: Duration,
    pub silence_max: Duration,
    pub inactivite_salon: Duration,
    pub salons_par_adresse: u32,
    pub tentatives: usize,
    pub fenetre_tentatives: Duration,
    pub message_max: usize,
    pub entretien: Duration,
    /// Versions du protocole acceptées ; les salons sont rangés par version.
    pub versions: RangeInclusive<u16>,
    /// Prendre l'adresse dans l'en-tête `CF-Connecting-IP` quand il est présent (relais
    /// derrière Cloudflare). À désactiver si le relais est exposé directement : n'importe
    /// qui pourrait alors forger l'en-tête.
    pub cloudflare: bool,
}

impl Default for Reglages {
    fn default() -> Self {
        Self {
            delai_reconnexion: DELAI_RECONNEXION,
            battement: BATTEMENT,
            silence_max: SILENCE_MAX,
            inactivite_salon: INACTIVITE_SALON,
            salons_par_adresse: SALONS_PAR_ADRESSE,
            tentatives: TENTATIVES,
            fenetre_tentatives: FENETRE_TENTATIVES,
            message_max: MESSAGE_MAX,
            entretien: ENTRETIEN,
            versions: VERSION_PROTOCOLE..=VERSION_PROTOCOLE,
            cloudflare: true,
        }
    }
}

/// Compteurs par adresse IP.
#[derive(Debug, Default)]
pub struct Limites {
    /// Instants des dernières tentatives (au plus `tentatives`), du plus ancien au plus récent.
    tentatives: HashMap<IpAddr, VecDeque<Instant>>,
    salons: HashMap<IpAddr, u32>,
}

impl Limites {
    /// Compte une tentative de `Rejoindre`/`Reprendre` ; `false` si l'adresse a déjà
    /// atteint la limite sur la fenêtre (la tentative refusée n'est pas comptée).
    pub fn tenter(&mut self, ip: IpAddr, maintenant: Instant, r: &Reglages) -> bool {
        let essais = self.tentatives.entry(ip).or_default();
        while essais
            .front()
            .is_some_and(|&t| maintenant.saturating_duration_since(t) >= r.fenetre_tentatives)
        {
            essais.pop_front();
        }
        if essais.len() >= r.tentatives {
            return false;
        }
        essais.push_back(maintenant);
        true
    }

    /// `true` si l'adresse peut encore ouvrir un salon.
    pub fn peut_ouvrir(&self, ip: IpAddr, r: &Reglages) -> bool {
        self.salons.get(&ip).copied().unwrap_or(0) < r.salons_par_adresse
    }

    pub fn salon_ouvert(&mut self, ip: IpAddr) {
        *self.salons.entry(ip).or_default() += 1;
    }

    pub fn salon_ferme(&mut self, ip: IpAddr) {
        if let Some(n) = self.salons.get_mut(&ip) {
            *n -= 1;
            if *n == 0 {
                self.salons.remove(&ip);
            }
        }
    }

    /// Oublie les adresses dont toutes les tentatives sont sorties de la fenêtre.
    pub fn purger(&mut self, maintenant: Instant, r: &Reglages) {
        self.tentatives.retain(|_, essais| {
            essais
                .back()
                .is_some_and(|&t| maintenant.saturating_duration_since(t) < r.fenetre_tentatives)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tentatives_sur_fenetre_glissante() {
        let r = Reglages {
            tentatives: 3,
            ..Reglages::default()
        };
        let (mut l, t0) = (Limites::default(), Instant::now());
        let ip: IpAddr = [10, 0, 0, 1].into();
        let autre: IpAddr = [10, 0, 0, 2].into();
        for k in 0..3 {
            assert!(l.tenter(ip, t0 + Duration::from_secs(k * 10), &r));
        }
        assert!(!l.tenter(ip, t0 + Duration::from_secs(30), &r));
        assert!(l.tenter(autre, t0 + Duration::from_secs(30), &r));
        // La première tentative sort de la fenêtre à t0 + 60 s
        assert!(l.tenter(ip, t0 + Duration::from_secs(60), &r));
        assert!(!l.tenter(ip, t0 + Duration::from_secs(61), &r));
        l.purger(t0 + Duration::from_secs(200), &r);
        assert!(l.tentatives.is_empty());
    }

    #[test]
    fn salons_par_adresse() {
        let r = Reglages {
            salons_par_adresse: 2,
            ..Reglages::default()
        };
        let mut l = Limites::default();
        let ip: IpAddr = [10, 0, 0, 1].into();
        l.salon_ouvert(ip);
        assert!(l.peut_ouvrir(ip, &r));
        l.salon_ouvert(ip);
        assert!(!l.peut_ouvrir(ip, &r));
        l.salon_ferme(ip);
        assert!(l.peut_ouvrir(ip, &r));
        l.salon_ferme(ip);
        assert!(l.salons.is_empty());
    }
}
