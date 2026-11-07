//! Trames échangées avec le relais (un message WebSocket binaire chacune). Le premier
//! octet donne le type, les nombres sont petit-boutistes. La lecture n'alloue rien : une
//! `Trame` emprunte ses octets au message reçu. Le relais ne lit jamais la charge utile
//! (`VersHote`, `VersInvites`) ni l'annonce d'un salon (octets décrits par le jeu).
//!
//! | Type | Trame | Octets après le type |
//! |---|---|---|
//! | 0 | `Creer` | version u16, public (0 ou 1), annonce (le reste, ≤ 256) |
//! | 1 | `Rejoindre` | version u16, code (6) |
//! | 2 | `Salon` | code (6), place, jeton u64 |
//! | 3 | `Refus` | raison |
//! | 4 | `VersHote` | place, charge (le reste) |
//! | 5 | `VersInvites` | charge (le reste) |
//! | 6 | `Reprendre` | version u16, code (6), jeton u64 |
//! | 7 | `Annoncer` | annonce (le reste, ≤ 256) |
//! | 8 | `Lister` | version u16 |
//! | 9 | `Liste` | entrées jusqu'à la fin : code (6), joueurs, places, taille u16, annonce |
//! | 10 / 11 | `Arrivee` / `Depart` | place |
//! | 12 / 13 | `HotePerdu` / `HoteRevenu` | rien |
//! | 14 | `Exclure` | place |
//! | 15 / 16 | `Exclu` / `Ferme` | rien |
//!
//! Les trames de taille fixe doivent avoir exactement leur taille.

/// Joueurs par salon, hôte compris (place 0 = hôte, 1 à 4 = invités).
pub const PLACES_SALON: u8 = 5;
/// Taille maximale de l'annonce d'un salon (arène, pseudo de l'hôte… décrits par le jeu).
pub const ANNONCE_MAX: usize = 256;
/// Nombre maximal de salons dans une `Liste`.
pub const LISTE_MAX: usize = 64;

/// Raisons de refus (`Trame::Refus`).
pub const REFUS_VERSION: u8 = 1;
pub const REFUS_SALON: u8 = 2;
pub const REFUS_PLEIN: u8 = 3;
/// `Reprendre` avec un jeton qui ne correspond à aucune place du salon.
pub const REFUS_JETON: u8 = 4;
/// Trop de `Rejoindre`/`Reprendre` depuis cette adresse (essai de tous les codes).
pub const REFUS_TENTATIVES: u8 = 5;
/// Trop de salons ouverts depuis cette adresse.
pub const REFUS_SALONS: u8 = 6;

const CREER: u8 = 0;
const REJOINDRE: u8 = 1;
const SALON: u8 = 2;
const REFUS: u8 = 3;
const VERS_HOTE: u8 = 4;
const VERS_INVITES: u8 = 5;
const REPRENDRE: u8 = 6;
const ANNONCER: u8 = 7;
const LISTER: u8 = 8;
const LISTE: u8 = 9;
const ARRIVEE: u8 = 10;
const DEPART: u8 = 11;
const HOTE_PERDU: u8 = 12;
const HOTE_REVENU: u8 = 13;
const EXCLURE: u8 = 14;
const EXCLU: u8 = 15;
const FERME: u8 = 16;

/// Une trame, empruntant ses octets au message reçu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trame<'a> {
    /// Client → relais : ouvrir un salon (l'expéditeur en devient l'hôte). Un salon public
    /// apparaît dans la `Liste` avec son annonce.
    Creer {
        version: u16,
        public: bool,
        annonce: &'a [u8],
    },
    /// Client → relais : rejoindre le salon `code` (6 lettres).
    Rejoindre { version: u16, code: [u8; 6] },
    /// Relais → client : salon ouvert (hôte, place 0), rejoint ou repris. Le `jeton` permet
    /// de revenir à sa place après une coupure (`Reprendre`).
    Salon {
        code: [u8; 6],
        place: u8,
        jeton: u64,
    },
    /// Relais → client : refus (`REFUS_*`).
    Refus { raison: u8 },
    /// Invité → hôte (le relais remplit `place` avec celle de l'expéditeur).
    VersHote { place: u8, charge: &'a [u8] },
    /// Hôte → tous les invités.
    VersInvites { charge: &'a [u8] },
    /// Client → relais : revenir à sa place (hôte ou invité) après une coupure.
    Reprendre {
        version: u16,
        code: [u8; 6],
        jeton: u64,
    },
    /// Hôte → relais : nouvelle annonce du salon (places, arène changées…).
    Annoncer { annonce: &'a [u8] },
    /// Client sans salon → relais : demander les salons publics de cette version.
    Lister { version: u16 },
    /// Relais → client : salons publics (lecture par `ListeSalons::iter`).
    Liste { salons: ListeSalons<'a> },
    /// Relais → hôte : un invité arrive à `place` (ou revient : renvoyer un instantané complet).
    Arrivee { place: u8 },
    /// Relais → hôte : l'invité de `place` est parti (coupure : sa place lui reste réservée).
    Depart { place: u8 },
    /// Relais → invités : l'hôte a perdu la connexion (mettre en pause, attendre).
    HotePerdu,
    /// Relais → invités : l'hôte est revenu.
    HoteRevenu,
    /// Hôte → relais : retirer l'invité de `place`.
    Exclure { place: u8 },
    /// Relais → invité : l'hôte vous a retiré du salon (la connexion est ensuite fermée).
    Exclu,
    /// Relais → invités : le salon est fermé.
    Ferme,
}

/// Salons d'une `Liste`, encore encodés (validés à la lecture : l'itération ne peut pas
/// échouer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ListeSalons<'a>(&'a [u8]);

/// Un salon public, tel que la `Liste` le décrit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntreeListe<'a> {
    pub code: [u8; 6],
    /// Places occupées (hôte compris, places réservées comprises).
    pub joueurs: u8,
    pub places: u8,
    pub annonce: &'a [u8],
}

impl<'a> ListeSalons<'a> {
    /// Parcourt les salons, sans allocation.
    pub fn iter(&self) -> IterListe<'a> {
        IterListe(self.0)
    }

    pub fn est_vide(&self) -> bool {
        self.0.is_empty()
    }

    /// Valide les entrées (toutes complètes, annonces ≤ `ANNONCE_MAX`).
    fn lire(o: &'a [u8]) -> Option<Self> {
        let mut reste = o;
        while !reste.is_empty() {
            reste = lire_entree(reste)?.1;
        }
        Some(Self(o))
    }
}

impl<'a> IntoIterator for ListeSalons<'a> {
    type Item = EntreeListe<'a>;
    type IntoIter = IterListe<'a>;
    fn into_iter(self) -> IterListe<'a> {
        self.iter()
    }
}

/// Itérateur sur les salons d'une `Liste`.
#[derive(Debug, Clone)]
pub struct IterListe<'a>(&'a [u8]);

impl<'a> Iterator for IterListe<'a> {
    type Item = EntreeListe<'a>;
    fn next(&mut self) -> Option<EntreeListe<'a>> {
        let (e, reste) = lire_entree(self.0)?;
        self.0 = reste;
        Some(e)
    }
}

impl EntreeListe<'_> {
    /// Ajoute l'entrée à une `Liste` en cours d'écriture : écrire d'abord
    /// `Trame::Liste { salons: ListeSalons::default() }`, puis chaque entrée.
    pub fn ajouter(&self, sortie: &mut Vec<u8>) {
        debug_assert!(self.annonce.len() <= ANNONCE_MAX);
        sortie.extend_from_slice(&self.code);
        sortie.extend_from_slice(&[self.joueurs, self.places]);
        sortie.extend_from_slice(&(self.annonce.len() as u16).to_le_bytes());
        sortie.extend_from_slice(self.annonce);
    }
}

fn lire_entree(o: &[u8]) -> Option<(EntreeListe<'_>, &[u8])> {
    let mut l = Lecteur(o);
    let code = l.octets()?;
    let joueurs = l.u8()?;
    let places = l.u8()?;
    let taille = usize::from(l.u16()?);
    if taille > ANNONCE_MAX {
        return None;
    }
    let (annonce, reste) = l.0.split_at_checked(taille)?;
    let e = EntreeListe {
        code,
        joueurs,
        places,
        annonce,
    };
    Some((e, reste))
}

impl Trame<'_> {
    /// Écrit la trame dans `sortie` (vidé d'abord).
    pub fn ecrire(&self, sortie: &mut Vec<u8>) {
        sortie.clear();
        match *self {
            Trame::Creer {
                version,
                public,
                annonce,
            } => {
                sortie.push(CREER);
                sortie.extend_from_slice(&version.to_le_bytes());
                sortie.push(u8::from(public));
                sortie.extend_from_slice(annonce);
            }
            Trame::Rejoindre { version, code } => {
                sortie.push(REJOINDRE);
                sortie.extend_from_slice(&version.to_le_bytes());
                sortie.extend_from_slice(&code);
            }
            Trame::Salon { code, place, jeton } => {
                sortie.push(SALON);
                sortie.extend_from_slice(&code);
                sortie.push(place);
                sortie.extend_from_slice(&jeton.to_le_bytes());
            }
            Trame::Refus { raison } => sortie.extend_from_slice(&[REFUS, raison]),
            Trame::VersHote { place, charge } => {
                sortie.extend_from_slice(&[VERS_HOTE, place]);
                sortie.extend_from_slice(charge);
            }
            Trame::VersInvites { charge } => {
                sortie.push(VERS_INVITES);
                sortie.extend_from_slice(charge);
            }
            Trame::Reprendre {
                version,
                code,
                jeton,
            } => {
                sortie.push(REPRENDRE);
                sortie.extend_from_slice(&version.to_le_bytes());
                sortie.extend_from_slice(&code);
                sortie.extend_from_slice(&jeton.to_le_bytes());
            }
            Trame::Annoncer { annonce } => {
                sortie.push(ANNONCER);
                sortie.extend_from_slice(annonce);
            }
            Trame::Lister { version } => {
                sortie.push(LISTER);
                sortie.extend_from_slice(&version.to_le_bytes());
            }
            Trame::Liste { salons } => {
                sortie.push(LISTE);
                sortie.extend_from_slice(salons.0);
            }
            Trame::Arrivee { place } => sortie.extend_from_slice(&[ARRIVEE, place]),
            Trame::Depart { place } => sortie.extend_from_slice(&[DEPART, place]),
            Trame::HotePerdu => sortie.push(HOTE_PERDU),
            Trame::HoteRevenu => sortie.push(HOTE_REVENU),
            Trame::Exclure { place } => sortie.extend_from_slice(&[EXCLURE, place]),
            Trame::Exclu => sortie.push(EXCLU),
            Trame::Ferme => sortie.push(FERME),
        }
    }
}

/// Lecture pas à pas d'une trame (sans allocation).
struct Lecteur<'a>(&'a [u8]);

impl<'a> Lecteur<'a> {
    fn octets<const N: usize>(&mut self) -> Option<[u8; N]> {
        let (t, reste) = self.0.split_first_chunk::<N>()?;
        self.0 = reste;
        Some(*t)
    }
    fn u8(&mut self) -> Option<u8> {
        self.octets::<1>().map(|[o]| o)
    }
    fn u16(&mut self) -> Option<u16> {
        self.octets().map(u16::from_le_bytes)
    }
    fn u64(&mut self) -> Option<u64> {
        self.octets().map(u64::from_le_bytes)
    }
    /// Termine une trame de taille fixe : plus aucun octet.
    fn fin<T>(self, t: T) -> Option<T> {
        self.0.is_empty().then_some(t)
    }
    fn annonce(self) -> Option<&'a [u8]> {
        (self.0.len() <= ANNONCE_MAX).then_some(self.0)
    }
}

/// Lit une trame ; `None` si elle est mal formée (type inconnu, taille fausse, annonce
/// trop longue, booléen invalide, liste incomplète).
pub fn lire_trame(o: &[u8]) -> Option<Trame<'_>> {
    let (&genre, corps) = o.split_first()?;
    let mut l = Lecteur(corps);
    match genre {
        CREER => {
            let version = l.u16()?;
            let public = match l.u8()? {
                0 => false,
                1 => true,
                _ => return None,
            };
            Some(Trame::Creer {
                version,
                public,
                annonce: l.annonce()?,
            })
        }
        REJOINDRE => {
            let t = Trame::Rejoindre {
                version: l.u16()?,
                code: l.octets()?,
            };
            l.fin(t)
        }
        SALON => {
            let t = Trame::Salon {
                code: l.octets()?,
                place: l.u8()?,
                jeton: l.u64()?,
            };
            l.fin(t)
        }
        REFUS => {
            let t = Trame::Refus { raison: l.u8()? };
            l.fin(t)
        }
        VERS_HOTE => Some(Trame::VersHote {
            place: l.u8()?,
            charge: l.0,
        }),
        VERS_INVITES => Some(Trame::VersInvites { charge: corps }),
        REPRENDRE => {
            let t = Trame::Reprendre {
                version: l.u16()?,
                code: l.octets()?,
                jeton: l.u64()?,
            };
            l.fin(t)
        }
        ANNONCER => Some(Trame::Annoncer {
            annonce: l.annonce()?,
        }),
        LISTER => {
            let t = Trame::Lister { version: l.u16()? };
            l.fin(t)
        }
        LISTE => Some(Trame::Liste {
            salons: ListeSalons::lire(corps)?,
        }),
        ARRIVEE | DEPART | EXCLURE => {
            let place = l.u8()?;
            l.fin(match genre {
                ARRIVEE => Trame::Arrivee { place },
                DEPART => Trame::Depart { place },
                _ => Trame::Exclure { place },
            })
        }
        HOTE_PERDU => l.fin(Trame::HotePerdu),
        HOTE_REVENU => l.fin(Trame::HoteRevenu),
        EXCLU => l.fin(Trame::Exclu),
        FERME => l.fin(Trame::Ferme),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
