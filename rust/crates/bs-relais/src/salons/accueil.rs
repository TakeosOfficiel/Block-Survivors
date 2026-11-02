//! Accueil d'une connexion sans salon : `Lister`, `Creer`, `Rejoindre`, `Reprendre`
//! (versions, limites par adresse, places libres, jetons), puis les nouvelles à donner
//! après une admission (`Arrivee`, `HotePerdu`, `HoteRevenu`).

use std::net::IpAddr;
use std::time::Instant;

use axum::body::Bytes;
use bs_protocole::{
    EntreeListe, LISTE_MAX, ListeSalons, PLACES_SALON, REFUS_JETON, REFUS_PLEIN, REFUS_SALON,
    REFUS_SALONS, REFUS_TENTATIVES, REFUS_VERSION, Trame,
};

use super::{Envoi, Lien, Occupant, PLACES, Salon, Salons, Siege, Sortant, envoyer};

impl Salons {
    /// Trame d'une connexion sans salon : `Lister`, `Creer`, `Rejoindre` ou `Reprendre`
    /// (les autres sont ignorées). Les réponses (`Salon`, `Refus`, `Liste`) passent par
    /// `envoi` ; renvoie la place obtenue.
    pub fn accueillir(
        &mut self,
        trame: Trame,
        ip: IpAddr,
        id: u64,
        envoi: &Envoi,
        maintenant: Instant,
    ) -> Option<Siege> {
        let resultat = match trame {
            Trame::Lister { version } => {
                self.lister(version, envoi);
                return None;
            }
            Trame::Creer { version, .. }
            | Trame::Rejoindre { version, .. }
            | Trame::Reprendre { version, .. }
                if !self.reglages.versions.contains(&version) =>
            {
                Err(REFUS_VERSION)
            }
            Trame::Creer {
                version,
                public,
                annonce,
            } => self.creer(version, public, annonce, ip, maintenant),
            Trame::Rejoindre { version, code } => self.rejoindre(version, code, ip, maintenant),
            Trame::Reprendre {
                version,
                code,
                jeton,
            } => self.reprendre(version, code, jeton, ip, maintenant),
            _ => return None,
        };
        let lien = Lien {
            id,
            envoi: envoi.clone(),
        };
        let tampon = &mut self.tampon;
        match resultat {
            Err(raison) => {
                envoyer(envoi, Trame::Refus { raison }, tampon);
                None
            }
            Ok((code, place)) => {
                let salon = self.salons.get_mut(&code)?;
                let occupant = salon.places[usize::from(place)].as_mut()?;
                if let Some(ancien) = occupant.lien.replace(lien) {
                    // Reprise sur une connexion qu'on croyait encore vivante
                    let _ = ancien.envoi.send(Sortant::Fermer);
                }
                let jeton = occupant.jeton;
                envoyer(envoi, Trame::Salon { code, place, jeton }, tampon);
                annoncer_arrivee(salon, place, tampon);
                Some(Siege { code, place })
            }
        }
    }

    fn creer(
        &mut self,
        version: u16,
        public: bool,
        annonce: &[u8],
        ip: IpAddr,
        maintenant: Instant,
    ) -> Result<([u8; 6], u8), u8> {
        if !self.limites.peut_ouvrir(ip, &self.reglages) {
            return Err(REFUS_SALONS);
        }
        let code = self.nouveau_code();
        let jeton = self.tirer();
        let mut places: [Option<Occupant>; PLACES] = Default::default();
        places[0] = Some(Occupant {
            jeton,
            lien: None,
            coupe_depuis: maintenant,
        });
        let salon = Salon {
            version,
            public,
            annonce: annonce.to_vec(),
            adresse: ip,
            places,
            dernier_message: maintenant,
        };
        self.salons.insert(code, salon);
        self.limites.salon_ouvert(ip);
        Ok((code, 0))
    }

    fn rejoindre(
        &mut self,
        version: u16,
        code: [u8; 6],
        ip: IpAddr,
        maintenant: Instant,
    ) -> Result<([u8; 6], u8), u8> {
        if !self.limites.tenter(ip, maintenant, &self.reglages) {
            return Err(REFUS_TENTATIVES);
        }
        let jeton = self.tirer();
        // Salon dont l'hôte est coupé : on ne peut pas y entrer (il ne nous verrait pas)
        let salon = self
            .salons
            .get_mut(&code)
            .filter(|s| s.lien(0).is_some())
            .ok_or(REFUS_SALON)?;
        if salon.version != version {
            return Err(REFUS_VERSION);
        }
        let libre = (1..PLACES)
            .find(|&p| salon.places[p].is_none())
            .ok_or(REFUS_PLEIN)?;
        salon.places[libre] = Some(Occupant {
            jeton,
            lien: None,
            coupe_depuis: maintenant,
        });
        salon.dernier_message = maintenant;
        Ok((code, libre as u8))
    }

    fn reprendre(
        &mut self,
        version: u16,
        code: [u8; 6],
        jeton: u64,
        ip: IpAddr,
        maintenant: Instant,
    ) -> Result<([u8; 6], u8), u8> {
        if !self.limites.tenter(ip, maintenant, &self.reglages) {
            return Err(REFUS_TENTATIVES);
        }
        let salon = self.salons.get_mut(&code).ok_or(REFUS_SALON)?;
        if salon.version != version {
            return Err(REFUS_VERSION);
        }
        let place = salon
            .places
            .iter()
            .position(|o| o.as_ref().is_some_and(|o| o.jeton == jeton))
            .ok_or(REFUS_JETON)?;
        salon.dernier_message = maintenant;
        Ok((code, place as u8))
    }

    /// Envoie la liste des salons publics de `version` dont l'hôte est connecté.
    fn lister(&mut self, version: u16, envoi: &Envoi) {
        let t = &mut self.tampon;
        Trame::Liste {
            salons: ListeSalons::default(),
        }
        .ecrire(t);
        let publics = self
            .salons
            .iter()
            .filter(|(_, s)| s.public && s.version == version && s.lien(0).is_some());
        for (code, s) in publics.take(LISTE_MAX) {
            EntreeListe {
                code: *code,
                joueurs: s.places.iter().flatten().count() as u8,
                places: PLACES_SALON,
                annonce: &s.annonce,
            }
            .ajouter(t);
        }
        let _ = envoi.send(Sortant::Octets(Bytes::copy_from_slice(t)));
    }
}

/// Après une admission à `place` : l'hôte apprend l'arrivée d'un invité ; un invité dont
/// l'hôte est coupé l'apprend ; un hôte qui revient prévient les invités et apprend qui est
/// là (pour leur renvoyer un instantané complet).
fn annoncer_arrivee(salon: &Salon, place: u8, tampon: &mut Vec<u8>) {
    let Some(moi) = salon.lien(usize::from(place)) else {
        return;
    };
    if place == 0 {
        salon.diffuser(Trame::HoteRevenu, tampon);
        for (p, _) in salon.invites() {
            envoyer(&moi.envoi, Trame::Arrivee { place: p }, tampon);
        }
    } else if let Some(h) = salon.lien(0) {
        envoyer(&h.envoi, Trame::Arrivee { place }, tampon);
    } else {
        envoyer(&moi.envoi, Trame::HotePerdu, tampon);
    }
}
