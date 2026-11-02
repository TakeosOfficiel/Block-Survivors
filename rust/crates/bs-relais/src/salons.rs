//! Logique des salons, pure et synchrone : aucun réseau, l'heure est passée en paramètre.
//! Chaque connexion est représentée par un canal (`Envoi`) où la logique dépose les
//! messages à lui envoyer, dans l'ordre ; la colle axum (`lib.rs`) les écrit sur le
//! WebSocket. On la teste donc sans réseau, avec des délais aussi courts qu'on veut.
//!
//! Une place (0 = hôte, 1 à 4 = invités) a un jeton de session tiré au hasard. Coupée,
//! elle reste réservée `delai_reconnexion` : son joueur la reprend avec `Reprendre` et son
//! jeton. Un départ volontaire (fermeture WebSocket normale) la libère tout de suite ; celui
//! de l'hôte ferme le salon.

use std::collections::HashMap;
use std::hash::{BuildHasher, RandomState};
use std::net::IpAddr;
use std::time::Instant;

use axum::body::Bytes;
use bs_protocole::{PLACES_SALON, Trame, lire_trame};
use tokio::sync::mpsc;

use crate::limites::{Limites, Reglages};

const PLACES: usize = PLACES_SALON as usize;

/// Ce que la logique demande d'envoyer sur une connexion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sortant {
    /// Un message binaire.
    Octets(Bytes),
    /// Fermer la connexion (après les messages déjà déposés).
    Fermer,
}

/// Canal vers une connexion.
pub type Envoi = mpsc::UnboundedSender<Sortant>;

/// Place qu'occupe une connexion admise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Siege {
    pub code: [u8; 6],
    pub place: u8,
}

/// Résumé pour la page d'état.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Etat {
    pub salons: usize,
    pub publics: usize,
    /// Joueurs connectés dans un salon.
    pub joueurs: usize,
    /// Places réservées à un joueur coupé.
    pub reservees: usize,
}

struct Lien {
    /// Identifiant de la connexion : une ancienne connexion (remplacée par `Reprendre`)
    /// n'agit plus sur la place.
    id: u64,
    envoi: Envoi,
}

struct Occupant {
    jeton: u64,
    /// `None` : joueur coupé, place réservée depuis `coupe_depuis`.
    lien: Option<Lien>,
    coupe_depuis: Instant,
}

struct Salon {
    version: u16,
    public: bool,
    annonce: Vec<u8>,
    /// Adresse de l'hôte à l'ouverture (limite de salons par adresse).
    adresse: IpAddr,
    places: [Option<Occupant>; PLACES],
    dernier_message: Instant,
}

impl Salon {
    fn lien(&self, place: usize) -> Option<&Lien> {
        self.places.get(place)?.as_ref()?.lien.as_ref()
    }

    fn est_lie(&self, place: u8, id: u64) -> bool {
        self.lien(usize::from(place)).is_some_and(|l| l.id == id)
    }

    /// Liens des invités connectés, avec leur place.
    fn invites(&self) -> impl Iterator<Item = (u8, &Lien)> {
        (1..PLACES).filter_map(|p| Some((p as u8, self.lien(p)?)))
    }

    /// Envoie la trame à tous les invités connectés (un seul message partagé).
    fn diffuser(&self, t: Trame, tampon: &mut Vec<u8>) {
        t.ecrire(tampon);
        let m = Bytes::copy_from_slice(tampon);
        for (_, l) in self.invites() {
            let _ = l.envoi.send(Sortant::Octets(m.clone()));
        }
    }

    /// Prévient tout le monde (`Ferme`) et ferme les connexions.
    fn fermer(&self, tampon: &mut Vec<u8>) {
        self.diffuser(Trame::Ferme, tampon);
        for (_, l) in self.invites() {
            let _ = l.envoi.send(Sortant::Fermer);
        }
        if let Some(hote) = self.lien(0) {
            envoyer(&hote.envoi, Trame::Ferme, tampon);
            let _ = hote.envoi.send(Sortant::Fermer);
        }
    }
}

fn envoyer(envoi: &Envoi, t: Trame, tampon: &mut Vec<u8>) {
    t.ecrire(tampon);
    let _ = envoi.send(Sortant::Octets(Bytes::copy_from_slice(tampon)));
}

/// Tous les salons du relais.
pub struct Salons {
    salons: HashMap<[u8; 6], Salon>,
    limites: Limites,
    reglages: Reglages,
    /// Clé aléatoire (SipHash) : codes et jetons = hachage d'un compteur, imprévisibles.
    cle: RandomState,
    compteur: u64,
    /// Tampon d'écriture des trames, réutilisé.
    tampon: Vec<u8>,
}

impl Salons {
    pub fn nouveaux(reglages: Reglages) -> Self {
        Self {
            salons: HashMap::new(),
            limites: Limites::default(),
            reglages,
            cle: RandomState::new(),
            compteur: 0,
            tampon: Vec::new(),
        }
    }

    fn tirer(&mut self) -> u64 {
        self.compteur += 1;
        self.cle.hash_one(self.compteur)
    }

    /// Code de salon libre (lettres sans I ni O, pour éviter les confusions).
    fn nouveau_code(&mut self) -> [u8; 6] {
        const LETTRES: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";
        loop {
            let mut x = self.tirer();
            let code = core::array::from_fn(|_| {
                let l = LETTRES[(x % LETTRES.len() as u64) as usize];
                x /= LETTRES.len() as u64;
                l
            });
            if !self.salons.contains_key(&code) {
                return code;
            }
        }
    }

    /// Message reçu d'une connexion admise. Hôte → tous les invités (`VersInvites`),
    /// invité → hôte (`VersHote`, place réécrite), et pour l'hôte `Annoncer` et `Exclure`.
    /// Le reste est ignoré.
    pub fn recevoir(&mut self, siege: Siege, id: u64, octets: Bytes, maintenant: Instant) {
        let Some(salon) = self.salons.get_mut(&siege.code) else {
            return;
        };
        if !salon.est_lie(siege.place, id) {
            return;
        }
        salon.dernier_message = maintenant;
        let (place, hote) = (siege.place, siege.place == 0);
        match lire_trame(&octets) {
            Some(Trame::VersInvites { .. }) if hote => {
                // Même message pour tous : les octets sont partagés, pas copiés
                for (_, l) in salon.invites() {
                    let _ = l.envoi.send(Sortant::Octets(octets.clone()));
                }
            }
            Some(Trame::VersHote { .. }) if !hote => {
                let Some(l) = salon.lien(0) else { return };
                // Réécrit la place sur place si le message n'est pas partagé, sinon copie
                let m = match octets.try_into_mut() {
                    Ok(mut m) => {
                        m[1] = place;
                        m.freeze()
                    }
                    Err(o) => {
                        let mut v = o.to_vec();
                        v[1] = place;
                        v.into()
                    }
                };
                let _ = l.envoi.send(Sortant::Octets(m));
            }
            Some(Trame::Annoncer { annonce }) if hote => {
                salon.annonce.clear();
                salon.annonce.extend_from_slice(annonce);
            }
            Some(Trame::Exclure { place: p }) if hote && p > 0 && usize::from(p) < PLACES => {
                let t = &mut self.tampon;
                if let Some(Some(lien)) = salon.places[usize::from(p)].take().map(|o| o.lien) {
                    envoyer(&lien.envoi, Trame::Exclu, t);
                    let _ = lien.envoi.send(Sortant::Fermer);
                    if let Some(h) = salon.lien(0) {
                        envoyer(&h.envoi, Trame::Depart { place: p }, t);
                    }
                }
            }
            _ => {}
        }
    }

    /// Fin d'une connexion admise. `volontaire` (fermeture WebSocket normale) : la place est
    /// libérée (le salon fermé si c'est l'hôte) ; sinon elle reste réservée.
    pub fn deconnecter(&mut self, siege: Siege, id: u64, volontaire: bool, maintenant: Instant) {
        let Some(salon) = self.salons.get_mut(&siege.code) else {
            return;
        };
        if !salon.est_lie(siege.place, id) {
            return;
        }
        let (p, t) = (usize::from(siege.place), &mut self.tampon);
        if volontaire && p == 0 {
            if let Some(salon) = self.salons.remove(&siege.code) {
                salon.fermer(t);
                self.limites.salon_ferme(salon.adresse);
            }
            return;
        }
        if volontaire {
            salon.places[p] = None;
        } else if let Some(o) = salon.places[p].as_mut() {
            o.lien = None;
            o.coupe_depuis = maintenant;
        }
        if p == 0 {
            salon.diffuser(Trame::HotePerdu, t);
        } else if let Some(h) = salon.lien(0) {
            envoyer(&h.envoi, Trame::Depart { place: siege.place }, t);
        }
    }

    /// À appeler régulièrement : libère les places réservées expirées, ferme les salons
    /// dont l'hôte n'est pas revenu à temps ou sans message depuis `inactivite_salon`.
    pub fn entretenir(&mut self, maintenant: Instant) {
        let (r, t, limites) = (&self.reglages, &mut self.tampon, &mut self.limites);
        let expire = |o: &Occupant| {
            o.lien.is_none()
                && maintenant.saturating_duration_since(o.coupe_depuis) >= r.delai_reconnexion
        };
        self.salons.retain(|_, s| {
            let inactif =
                maintenant.saturating_duration_since(s.dernier_message) >= r.inactivite_salon;
            if inactif || s.places[0].as_ref().is_some_and(expire) {
                s.fermer(t);
                limites.salon_ferme(s.adresse);
                return false;
            }
            for place in &mut s.places[1..] {
                if place.as_ref().is_some_and(expire) {
                    *place = None;
                }
            }
            true
        });
        limites.purger(maintenant, r);
    }

    pub fn etat(&self) -> Etat {
        let mut e = Etat {
            salons: self.salons.len(),
            ..Etat::default()
        };
        for s in self.salons.values() {
            e.publics += usize::from(s.public);
            for o in s.places.iter().flatten() {
                if o.lien.is_some() {
                    e.joueurs += 1;
                } else {
                    e.reservees += 1;
                }
            }
        }
        e
    }
}

mod accueil;
#[cfg(test)]
mod tests;
