//! Vie d'un salon côté jeu (docs/PLAN-RUST.md, section 4), par-dessus une `Connexion` :
//! créer un salon (on en devient l'hôte, place 0), lister les salons publics, en rejoindre
//! un par son code, puis échanger des octets avec l'hôte ou les invités. Une coupure n'est
//! pas une fin : la session se reconnecte seule et reprend sa place avec le jeton donné par
//! le relais, pendant `DELAI_REPRISE` ; au-delà, elle est perdue. `maj` se relève à
//! chaque image et ne bloque jamais.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use bs_protocole::{ListeSalons, Trame, VERSION_PROTOCOLE, lire_trame};

use crate::connexion::{Connexion, Recu};

/// Temps pendant lequel on tente de reprendre sa place (le relais la garde 60 s).
pub const DELAI_REPRISE: Duration = Duration::from_secs(55);
/// Écart entre deux tentatives de reconnexion.
const ECART_TENTATIVES: Duration = Duration::from_secs(2);

/// Un salon public de la liste.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SalonPublic {
    pub code: [u8; 6],
    pub joueurs: u8,
    pub places: u8,
    pub annonce: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvenementSession {
    /// Connecté au relais (on peut lister, créer ou rejoindre).
    Connectee,
    /// Salon ouvert ou rejoint : son code et notre place (0 : hôte).
    Salon {
        code: [u8; 6],
        place: u8,
    },
    /// Revenu à sa place après une coupure.
    Reprise,
    /// Demande refusée (`bs_protocole::REFUS_*`) ; la connexion reste ouverte.
    Refus(u8),
    Liste(Vec<SalonPublic>),
    /// Octets reçus : chez l'hôte, d'un invité (`place`) ; chez un invité, de l'hôte (0).
    Message {
        place: u8,
        octets: Vec<u8>,
    },
    /// Chez l'hôte : un invité arrive (ou revient) à `place`.
    Arrivee(u8),
    /// Chez l'hôte : l'invité de `place` est parti (coupé ou exclu).
    Depart(u8),
    HotePerdu,
    HoteRevenu,
    /// Retiré du salon par l'hôte.
    Exclu,
    /// Le salon est fermé.
    Ferme,
    /// Connexion perdue : reconnexion en cours.
    Coupure,
    /// Fin de la session (raison lisible).
    Perdue(String),
}

/// « ABCDEF » → « ABC-DEF ».
pub fn code_lisible(code: &[u8; 6]) -> String {
    let t = String::from_utf8_lossy(code);
    format!("{}-{}", &t[..3.min(t.len())], &t[3.min(t.len())..])
}

/// Code tapé par le joueur (« abc-def », « ABC DEF »…) → 6 lettres.
pub fn code_saisi(texte: &str) -> Option<[u8; 6]> {
    let lettres: Vec<u8> = texte
        .bytes()
        .filter(u8::is_ascii_alphanumeric)
        .map(|b| b.to_ascii_uppercase())
        .collect();
    lettres.try_into().ok()
}

pub struct Session {
    adresse: String,
    connexion: Connexion,
    tampon: Vec<u8>,
    evenements: VecDeque<EvenementSession>,
    /// Salon et place, une fois dedans ; le jeton pour y revenir.
    pub salon: Option<([u8; 6], u8)>,
    jeton: u64,
    /// Coupure en cours : depuis quand, et prochaine tentative.
    coupure: Option<(Instant, Instant)>,
    quitte: bool,
    perdue: bool,
}

impl Session {
    /// Se connecte au relais (`crate::adresse_relais()` en jeu).
    pub fn ouvrir(adresse: &str) -> Self {
        Self {
            adresse: adresse.to_owned(),
            connexion: Connexion::ouvrir(adresse),
            tampon: Vec::with_capacity(1024),
            evenements: VecDeque::new(),
            salon: None,
            jeton: 0,
            coupure: None,
            quitte: false,
            perdue: false,
        }
    }

    fn envoyer(&mut self, t: Trame) {
        t.ecrire(&mut self.tampon);
        self.connexion.envoyer(self.tampon.clone());
    }

    pub fn creer(&mut self, public: bool, annonce: &[u8]) {
        let annonce = &annonce[..annonce.len().min(bs_protocole::ANNONCE_MAX)];
        self.envoyer(Trame::Creer {
            version: VERSION_PROTOCOLE,
            public,
            annonce,
        });
    }

    pub fn lister(&mut self) {
        self.envoyer(Trame::Lister {
            version: VERSION_PROTOCOLE,
        });
    }

    pub fn rejoindre(&mut self, code: [u8; 6]) {
        self.envoyer(Trame::Rejoindre {
            version: VERSION_PROTOCOLE,
            code,
        });
    }

    /// Hôte : nouvelle annonce du salon public.
    pub fn annoncer(&mut self, annonce: &[u8]) {
        let annonce = &annonce[..annonce.len().min(bs_protocole::ANNONCE_MAX)];
        self.envoyer(Trame::Annoncer { annonce });
    }

    /// Invité → hôte.
    pub fn vers_hote(&mut self, charge: &[u8]) {
        self.envoyer(Trame::VersHote { place: 0, charge });
    }

    /// Hôte → tous les invités.
    pub fn vers_invites(&mut self, charge: &[u8]) {
        self.envoyer(Trame::VersInvites { charge });
    }

    /// Hôte : retirer l'invité de `place`.
    pub fn exclure(&mut self, place: u8) {
        self.envoyer(Trame::Exclure { place });
    }

    /// Départ volontaire : la place est libérée (le salon fermé si on est l'hôte).
    pub fn quitter(&mut self) {
        self.quitte = true;
        self.connexion.fermer();
    }

    /// Coupe la connexion comme une perte de réseau (essais de reprise).
    #[doc(hidden)]
    pub fn couper_pour_essai(&mut self) {
        self.connexion.couper();
    }

    pub fn hote(&self) -> bool {
        matches!(self.salon, Some((_, 0)))
    }

    pub fn en_coupure(&self) -> bool {
        self.coupure.is_some()
    }

    /// Relève le prochain évènement (à appeler en boucle à chaque image).
    pub fn maj(&mut self) -> Option<EvenementSession> {
        self.pomper();
        self.evenements.pop_front()
    }

    fn pomper(&mut self) {
        if self.perdue || self.quitte {
            return;
        }
        while let Some(r) = self.connexion.recevoir() {
            match r {
                Recu::Ouverte if self.coupure.is_none() => {
                    self.evenements.push_back(EvenementSession::Connectee);
                }
                Recu::Ouverte => {}
                Recu::Octets(o) => self.lire(&o),
                Recu::Fermee(raison) => {
                    self.coupee(raison);
                    break;
                }
            }
        }
        // Nouvelle tentative de reprise
        if let Some((debut, prochaine)) = self.coupure
            && self.connexion.finie()
        {
            let maintenant = Instant::now();
            if maintenant - debut > DELAI_REPRISE {
                self.perdre("le relais ne répond plus".into());
            } else if maintenant >= prochaine {
                self.connexion = Connexion::ouvrir(&self.adresse);
                if let Some((code, _)) = self.salon {
                    let jeton = self.jeton;
                    self.envoyer(Trame::Reprendre {
                        version: VERSION_PROTOCOLE,
                        code,
                        jeton,
                    });
                }
                self.coupure = Some((debut, maintenant + ECART_TENTATIVES));
            }
        }
    }

    fn coupee(&mut self, raison: Option<String>) {
        if self.salon.is_none() {
            let r = raison.unwrap_or_else(|| "connexion au relais fermée".into());
            self.perdre(r);
        } else if self.coupure.is_none() {
            let maintenant = Instant::now();
            self.coupure = Some((maintenant, maintenant));
            self.evenements.push_back(EvenementSession::Coupure);
        }
    }

    fn perdre(&mut self, raison: String) {
        self.perdue = true;
        self.coupure = None;
        self.evenements.push_back(EvenementSession::Perdue(raison));
    }

    fn lire(&mut self, o: &[u8]) {
        let Some(t) = lire_trame(o) else {
            return;
        };
        use EvenementSession as E;
        let e = match t {
            Trame::Salon { code, place, jeton } => {
                self.jeton = jeton;
                let reprise = self.coupure.take().is_some();
                self.salon = Some((code, place));
                if reprise {
                    E::Reprise
                } else {
                    E::Salon { code, place }
                }
            }
            Trame::Refus { raison } => {
                if self.coupure.is_some() {
                    // Place perdue (jeton refusé, salon fermé) : la reprise échoue
                    self.perdre(format!("reprise refusée ({raison})"));
                    return;
                }
                E::Refus(raison)
            }
            Trame::Liste { salons } => E::Liste(liste(salons)),
            Trame::VersHote { place, charge } => E::Message {
                place,
                octets: charge.to_vec(),
            },
            Trame::VersInvites { charge } => E::Message {
                place: 0,
                octets: charge.to_vec(),
            },
            Trame::Arrivee { place } => E::Arrivee(place),
            Trame::Depart { place } => E::Depart(place),
            Trame::HotePerdu => E::HotePerdu,
            Trame::HoteRevenu => E::HoteRevenu,
            Trame::Exclu => {
                self.perdue = true;
                E::Exclu
            }
            Trame::Ferme => {
                self.perdue = true;
                E::Ferme
            }
            _ => return,
        };
        self.evenements.push_back(e);
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // Fermer la session est un départ volontaire
        self.connexion.fermer();
    }
}

fn liste(salons: ListeSalons) -> Vec<SalonPublic> {
    salons
        .iter()
        .map(|e| SalonPublic {
            code: e.code,
            joueurs: e.joueurs,
            places: e.places,
            annonce: e.annonce.to_vec(),
        })
        .collect()
}
