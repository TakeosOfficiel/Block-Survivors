//! Amis par le relais (trames `bs_protocole::amis`), sans Bevy : une connexion à part,
//! sans salon, tenue tant que le jeu tourne.
//!
//! - **Présence** : publiée à l'ouverture, à chaque changement et toutes les
//!   `BATTEMENT` ; celle d'un ami est acceptée si elle est signée de sa clé (retenue à sa
//!   première présence, évènement `Cle`), puis oubliée après `SILENCE` sans nouvelles ou à
//!   son `Absent`.
//! - **Lettres** (ajout, invitation, demande) : signées ; celles d'un ami doivent l'être
//!   de sa clé connue. Un ajout est accepté d'un inconnu (le joueur décide).
//! - **Coupure** : nouvel essai après 2 s, puis de plus en plus espacés (60 s au plus).
//!
//! Le service ne touche pas à la sauvegarde : il dit au jeu ce qu'il faut y retenir
//! (clés, pseudos) par ses évènements.

mod contenu;

use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bs_protocole::amis::{
    Adresse, Genre, SUIVIS_MAX, TrameAmi, empreinte_boite, empreinte_presence, lire_trame_ami,
    signer,
};
use ed25519_dalek::SigningKey;

pub use contenu::{Etat, EtatPresence, Lettre};

use crate::{Connexion, Recu};

/// Republication de sa présence.
pub const BATTEMENT: Duration = Duration::from_secs(25);
/// Un ami silencieux aussi longtemps est hors ligne.
pub const SILENCE: Duration = Duration::from_secs(70);
const ESSAI_MIN: Duration = Duration::from_secs(2);
const ESSAI_MAX: Duration = Duration::from_secs(60);

/// Ce que le service dit au jeu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvenementAmis {
    /// Clé d'un ami vue pour la première fois : à retenir dans la sauvegarde.
    Cle { code: String, cle: [u8; 32] },
    /// Pseudo publié par un ami.
    Pseudo { code: String, pseudo: String },
    /// Quelqu'un nous a ajouté.
    Ajout { code: String, pseudo: String },
    /// Un ami nous invite dans son salon.
    Invitation { code: String, salon: [u8; 6] },
    /// Un ami voudrait jouer avec nous.
    Demande { code: String },
    /// Présences changées (qui est en ligne, où).
    Change,
}

/// Un ami suivi.
#[derive(Debug, Clone)]
pub struct AmiSuivi {
    pub code: String,
    pub cle: Option<[u8; 32]>,
    empreinte: Adresse,
    /// Sa présence, s'il est en ligne.
    pub presence: Option<Etat>,
    vu: Instant,
    /// Dernier instant signé vu de lui (présences et lettres), contre le rejeu.
    instant: u64,
}

pub struct ServiceAmis {
    adresse: String,
    connexion: Option<Connexion>,
    ouverte: bool,
    prochain_essai: Instant,
    attente: Duration,
    code: String,
    cle: SigningKey,
    boite: Adresse,
    empreinte: Adresse,
    amis: Vec<AmiSuivi>,
    etat: Etat,
    prochaine_publication: Option<Instant>,
    dernier_instant: u64,
    evenements: VecDeque<EvenementAmis>,
    tampon: Vec<u8>,
}

fn maintenant_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

impl ServiceAmis {
    /// Service du profil `code` (sa graine de clé `secret`) ; se connecte à `adresse`.
    pub fn nouveau(adresse: &str, code: &str, secret: &[u8; 32], etat: Etat) -> Self {
        Self {
            adresse: adresse.to_owned(),
            connexion: None,
            ouverte: false,
            prochain_essai: Instant::now(),
            attente: ESSAI_MIN,
            code: code.to_owned(),
            cle: SigningKey::from_bytes(secret),
            boite: empreinte_boite(code),
            empreinte: empreinte_presence(code),
            amis: Vec::new(),
            etat,
            prochaine_publication: None,
            dernier_instant: 0,
            evenements: VecDeque::new(),
            tampon: Vec::new(),
        }
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn connecte(&self) -> bool {
        self.ouverte
    }

    pub fn amis(&self) -> &[AmiSuivi] {
        &self.amis
    }

    pub fn ami(&self, code: &str) -> Option<&AmiSuivi> {
        self.amis.iter().find(|a| a.code == code)
    }

    /// Suit un ami (sa clé si elle est connue) ; réabonne la connexion.
    pub fn suivre(&mut self, code: &str, cle: Option<[u8; 32]>) {
        if self.amis.iter().any(|a| a.code == code) || self.amis.len() >= SUIVIS_MAX {
            return;
        }
        self.amis.push(AmiSuivi {
            code: code.to_owned(),
            cle,
            empreinte: empreinte_presence(code),
            presence: None,
            vu: Instant::now(),
            instant: 0,
        });
        self.ecouter();
    }

    /// Ne suit plus un ami.
    pub fn oublier(&mut self, code: &str) {
        let avant = self.amis.len();
        self.amis.retain(|a| a.code != code);
        if self.amis.len() != avant {
            self.ecouter();
            self.evenements.push_back(EvenementAmis::Change);
        }
    }

    /// Change ce qu'on publie de soi (publié tout de suite s'il a changé).
    pub fn publier(&mut self, etat: Etat) {
        if etat != self.etat {
            self.etat = etat;
            self.prochaine_publication = Some(Instant::now());
        }
    }

    /// Poste une lettre à `code` (perdue s'il n'est pas en ligne).
    pub fn poster(&mut self, code: &str, lettre: &Lettre) {
        let contenu = lettre.ecrire(&self.code, &self.etat.pseudo);
        let boite = empreinte_boite(code);
        let instant = self.instant();
        let signe = signer(&self.cle, Genre::Courrier, &boite, instant, &contenu);
        TrameAmi::Poster { boite, signe }.ecrire(&mut self.tampon);
        self.envoyer();
    }

    /// Prochain évènement.
    pub fn evenement(&mut self) -> Option<EvenementAmis> {
        self.evenements.pop_front()
    }

    /// À chaque image : connexion, réception, présences.
    pub fn maj(&mut self) {
        let maintenant = Instant::now();
        if self.connexion.is_none() && maintenant >= self.prochain_essai {
            self.connexion = Some(Connexion::ouvrir(&self.adresse));
        }
        while let Some(r) = self.connexion.as_mut().and_then(Connexion::recevoir) {
            match r {
                Recu::Ouverte => {
                    self.ouverte = true;
                    self.attente = ESSAI_MIN;
                    self.ecouter();
                    self.prochaine_publication = Some(maintenant);
                }
                Recu::Octets(o) => self.lire(&o, maintenant),
                Recu::Fermee(_) => {
                    self.coupure(maintenant);
                    break;
                }
            }
        }
        if self.ouverte && self.prochaine_publication.is_some_and(|t| maintenant >= t) {
            self.publier_presence();
            self.prochaine_publication = Some(maintenant + BATTEMENT);
        }
        let mut change = false;
        for a in &mut self.amis {
            if a.presence.is_some() && maintenant.duration_since(a.vu) >= SILENCE {
                a.presence = None;
                change = true;
            }
        }
        if change {
            self.evenements.push_back(EvenementAmis::Change);
        }
    }

    /// Ferme la connexion (départ volontaire : le relais dit `Absent` aux amis).
    pub fn fermer(&mut self) {
        if let Some(c) = self.connexion.take() {
            c.fermer();
        }
        self.ouverte = false;
    }

    fn coupure(&mut self, maintenant: Instant) {
        self.connexion = None;
        self.ouverte = false;
        self.prochain_essai = maintenant + self.attente;
        self.attente = (self.attente * 2).min(ESSAI_MAX);
        let mut change = false;
        for a in &mut self.amis {
            change |= a.presence.take().is_some();
        }
        if change {
            self.evenements.push_back(EvenementAmis::Change);
        }
    }

    /// Instant signé : l'heure en ms, toujours croissant.
    fn instant(&mut self) -> u64 {
        self.dernier_instant = maintenant_ms().max(self.dernier_instant + 1);
        self.dernier_instant
    }

    fn envoyer(&mut self) {
        if let Some(c) = &self.connexion
            && self.ouverte
        {
            c.envoyer(self.tampon.clone());
        }
    }

    fn ecouter(&mut self) {
        let suivis: Vec<u8> = self.amis.iter().flat_map(|a| a.empreinte).collect();
        TrameAmi::Ecouter {
            boite: self.boite,
            suivis: &suivis,
        }
        .ecrire(&mut self.tampon);
        self.envoyer();
    }

    fn publier_presence(&mut self) {
        let contenu = self.etat.ecrire();
        let instant = self.instant();
        let signe = signer(
            &self.cle,
            Genre::Presence,
            &self.empreinte,
            instant,
            &contenu,
        );
        TrameAmi::Publier {
            empreinte: self.empreinte,
            signe,
        }
        .ecrire(&mut self.tampon);
        self.envoyer();
    }

    fn lire(&mut self, o: &[u8], maintenant: Instant) {
        match lire_trame_ami(o) {
            Some(TrameAmi::Presence { empreinte, signe }) => {
                let Some(a) = self.amis.iter_mut().find(|a| a.empreinte == empreinte) else {
                    return;
                };
                if !signe.verifier(Genre::Presence, &empreinte)
                    || a.cle.is_some_and(|c| c != signe.cle)
                    // Égal : la même présence redonnée par le relais (réabonnement)
                    || signe.instant < a.instant
                {
                    return;
                }
                let Some(etat) = Etat::lire(signe.contenu) else {
                    return;
                };
                a.instant = signe.instant;
                a.vu = maintenant;
                if a.cle.is_none() {
                    a.cle = Some(signe.cle);
                    self.evenements.push_back(EvenementAmis::Cle {
                        code: a.code.clone(),
                        cle: signe.cle,
                    });
                }
                if a.presence.as_ref().is_none_or(|p| p.pseudo != etat.pseudo) {
                    self.evenements.push_back(EvenementAmis::Pseudo {
                        code: a.code.clone(),
                        pseudo: etat.pseudo.clone(),
                    });
                }
                if a.presence.as_ref() != Some(&etat) {
                    a.presence = Some(etat);
                    self.evenements.push_back(EvenementAmis::Change);
                }
            }
            Some(TrameAmi::Absent { empreinte }) => {
                if let Some(a) = self.amis.iter_mut().find(|a| a.empreinte == empreinte)
                    && a.presence.take().is_some()
                {
                    self.evenements.push_back(EvenementAmis::Change);
                }
            }
            Some(TrameAmi::Courrier { boite, signe }) => {
                if boite != self.boite || !signe.verifier(Genre::Courrier, &boite) {
                    return;
                }
                let Some((lettre, de, pseudo)) = Lettre::lire(signe.contenu) else {
                    return;
                };
                if de == self.code {
                    return;
                }
                let ami = self.amis.iter_mut().find(|a| a.code == de);
                let e = match (lettre, ami) {
                    // Ajout : d'un inconnu seulement (un ami l'est déjà)
                    (Lettre::Ajout, None) => EvenementAmis::Ajout { code: de, pseudo },
                    (lettre, Some(a)) if a.cle == Some(signe.cle) && signe.instant > a.instant => {
                        a.instant = signe.instant;
                        match lettre {
                            Lettre::Invitation(salon) => {
                                EvenementAmis::Invitation { code: de, salon }
                            }
                            Lettre::Demande => EvenementAmis::Demande { code: de },
                            Lettre::Ajout => return,
                        }
                    }
                    _ => return,
                };
                self.evenements.push_back(e);
            }
            _ => {}
        }
    }
}

impl Drop for ServiceAmis {
    fn drop(&mut self) {
        self.fermer();
    }
}
