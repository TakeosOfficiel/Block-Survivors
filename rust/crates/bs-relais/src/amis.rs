//! Amis sur le relais (trames `bs_protocole::amis`), pure et synchrone comme les salons :
//! l'heure est passée en paramètre, chaque connexion est un canal.
//!
//! - **Présences** : une connexion publie la sienne sous l'empreinte de son code ami ;
//!   le relais la garde `PRESENCE_MAX` (republiée toutes les 25 s par le jeu), la fait
//!   suivre à ceux qui la suivent et la leur donne dès qu'ils s'abonnent. À la
//!   déconnexion de son auteur, les suiveurs reçoivent `Absent`.
//! - **Clés** : une empreinte est liée à la première clé qui y publie (gardée
//!   `LIAISON_MAX` sans nouvelles) ; une présence signée d'une autre clé, ou d'un instant
//!   déjà vu, est ignorée.
//! - **Lettres** : faites suivre aux connexions qui écoutent la boîte, jamais gardées
//!   (l'ami hors ligne ne les reçoit pas, comme avec la version TS).
//!
//! Publications et lettres sont limitées par connexion (`ENVOIS_MAX` par minute).

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use axum::body::Bytes;
use bs_protocole::amis::{Adresse, Genre, TrameAmi, faire_suivre, suivis};

use crate::salons::{Envoi, Sortant};

/// Une présence non republiée depuis aussi longtemps est oubliée (hors ligne).
pub const PRESENCE_MAX: Duration = Duration::from_secs(70);
/// Une empreinte sans présence depuis aussi longtemps perd sa clé.
pub const LIAISON_MAX: Duration = Duration::from_secs(30 * 24 * 3600);
/// Empreintes liées au plus (au-delà, les nouvelles sont refusées).
pub const LIAISONS_MAX: usize = 200_000;
/// Publications et lettres par connexion sur une minute.
pub const ENVOIS_MAX: usize = 30;
const FENETRE_ENVOIS: Duration = Duration::from_secs(60);
/// Empreintes publiées au plus par une connexion (un profil à la fois, en pratique).
const PUBLIEES_MAX: usize = 4;

struct Abonne {
    envoi: Envoi,
    boite: Option<Adresse>,
    suivis: Vec<Adresse>,
    publiees: Vec<Adresse>,
    envois: VecDeque<Instant>,
}

struct Publiee {
    /// Trame `Presence` prête à envoyer.
    trame: Bytes,
    auteur: u64,
    recue: Instant,
}

struct Liaison {
    cle: [u8; 32],
    instant: u64,
    vu: Instant,
}

#[derive(Default)]
pub struct Amis {
    abonnes: HashMap<u64, Abonne>,
    suiveurs: HashMap<Adresse, Vec<u64>>,
    boites: HashMap<Adresse, Vec<u64>>,
    presences: HashMap<Adresse, Publiee>,
    liaisons: HashMap<Adresse, Liaison>,
}

fn retirer(index: &mut HashMap<Adresse, Vec<u64>>, a: &Adresse, id: u64) {
    if let Some(l) = index.get_mut(a) {
        l.retain(|&x| x != id);
        if l.is_empty() {
            index.remove(a);
        }
    }
}

impl Amis {
    /// Connexions abonnées et présences gardées (page d'état).
    pub fn etat(&self) -> (usize, usize) {
        (self.abonnes.len(), self.presences.len())
    }

    fn envoyer(&self, ids: Option<&Vec<u64>>, trame: &Bytes) {
        for id in ids.into_iter().flatten() {
            if let Some(a) = self.abonnes.get(id) {
                let _ = a.envoi.send(Sortant::Octets(trame.clone()));
            }
        }
    }

    fn abonne(&mut self, id: u64, envoi: &Envoi) -> &mut Abonne {
        self.abonnes.entry(id).or_insert_with(|| Abonne {
            envoi: envoi.clone(),
            boite: None,
            suivis: Vec::new(),
            publiees: Vec::new(),
            envois: VecDeque::new(),
        })
    }

    /// Compte un envoi de la connexion ; faux au-delà de la limite.
    fn compter(&mut self, id: u64, envoi: &Envoi, maintenant: Instant) -> bool {
        let e = &mut self.abonne(id, envoi).envois;
        while e
            .front()
            .is_some_and(|&t| maintenant.saturating_duration_since(t) >= FENETRE_ENVOIS)
        {
            e.pop_front();
        }
        if e.len() >= ENVOIS_MAX {
            return false;
        }
        e.push_back(maintenant);
        true
    }

    /// Traite une trame des amis (`brut` : ses octets) venue de la connexion `id`.
    pub fn recevoir(
        &mut self,
        t: TrameAmi,
        brut: &[u8],
        id: u64,
        envoi: &Envoi,
        maintenant: Instant,
    ) {
        match t {
            TrameAmi::Ecouter { boite, suivis: l } => {
                self.desabonner(id);
                let a = self.abonne(id, envoi);
                a.boite = Some(boite);
                a.suivis.clear();
                a.suivis.extend(suivis(l));
                let liste = a.suivis.clone();
                self.boites.entry(boite).or_default().push(id);
                for e in liste {
                    let ids = self.suiveurs.entry(e).or_default();
                    if !ids.contains(&id) {
                        ids.push(id);
                    }
                    if let Some(p) = self.presences.get(&e) {
                        let _ = envoi.send(Sortant::Octets(p.trame.clone()));
                    }
                }
            }
            TrameAmi::Publier { empreinte, signe } => {
                if !self.compter(id, envoi, maintenant)
                    || !signe.verifier(Genre::Presence, &empreinte)
                {
                    return;
                }
                let pleines = self.liaisons.len() >= LIAISONS_MAX;
                match self.liaisons.get_mut(&empreinte) {
                    Some(l) if l.cle != signe.cle || signe.instant <= l.instant => return,
                    Some(l) => {
                        l.instant = signe.instant;
                        l.vu = maintenant;
                    }
                    None if pleines => return,
                    None => {
                        let l = Liaison {
                            cle: signe.cle,
                            instant: signe.instant,
                            vu: maintenant,
                        };
                        self.liaisons.insert(empreinte, l);
                    }
                }
                let Some(trame) = faire_suivre(brut) else {
                    return;
                };
                let a = self.abonne(id, envoi);
                if !a.publiees.contains(&empreinte) {
                    if a.publiees.len() >= PUBLIEES_MAX {
                        return;
                    }
                    a.publiees.push(empreinte);
                }
                let trame = Bytes::from(trame);
                self.envoyer(self.suiveurs.get(&empreinte), &trame);
                let p = Publiee {
                    trame,
                    auteur: id,
                    recue: maintenant,
                };
                self.presences.insert(empreinte, p);
            }
            TrameAmi::Poster { boite, signe } => {
                if !self.compter(id, envoi, maintenant) || !signe.verifier(Genre::Courrier, &boite)
                {
                    return;
                }
                if let Some(trame) = faire_suivre(brut) {
                    self.envoyer(self.boites.get(&boite), &Bytes::from(trame));
                }
            }
            // Trames du relais : jamais attendues d'un client
            TrameAmi::Presence { .. } | TrameAmi::Courrier { .. } | TrameAmi::Absent { .. } => {}
        }
    }

    /// Retire les abonnements de la connexion (boîte et présences suivies).
    fn desabonner(&mut self, id: u64) {
        let Some(a) = self.abonnes.get_mut(&id) else {
            return;
        };
        let boite = a.boite.take();
        let suivis = std::mem::take(&mut a.suivis);
        if let Some(b) = boite {
            retirer(&mut self.boites, &b, id);
        }
        for e in &suivis {
            retirer(&mut self.suiveurs, e, id);
        }
    }

    /// La présence de `empreinte` disparaît : ses suiveurs l'apprennent.
    fn absent(&mut self, empreinte: &Adresse) {
        if self.presences.remove(empreinte).is_none() {
            return;
        }
        let mut o = Vec::new();
        TrameAmi::Absent {
            empreinte: *empreinte,
        }
        .ecrire(&mut o);
        self.envoyer(self.suiveurs.get(empreinte), &Bytes::from(o));
    }

    /// Connexion fermée : abonnements retirés, ses présences effacées.
    pub fn deconnecter(&mut self, id: u64) {
        self.desabonner(id);
        let Some(a) = self.abonnes.remove(&id) else {
            return;
        };
        for e in &a.publiees {
            if self.presences.get(e).is_some_and(|p| p.auteur == id) {
                self.absent(e);
            }
        }
    }

    /// Oublie les présences muettes et les clés trop anciennes.
    pub fn entretenir(&mut self, maintenant: Instant) {
        let muettes: Vec<Adresse> = self
            .presences
            .iter()
            .filter(|(_, p)| maintenant.saturating_duration_since(p.recue) >= PRESENCE_MAX)
            .map(|(e, _)| *e)
            .collect();
        for e in &muettes {
            self.absent(e);
        }
        self.liaisons
            .retain(|_, l| maintenant.saturating_duration_since(l.vu) < LIAISON_MAX);
    }
}

#[cfg(test)]
mod tests;
