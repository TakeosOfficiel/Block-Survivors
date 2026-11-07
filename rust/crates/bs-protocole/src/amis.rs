//! Trames des amis (docs/PLAN-RUST.md, section 4, « Ce qu'on garde de notre version »),
//! sur une connexion au relais sans salon. Le relais ne voit jamais un code ami en clair :
//! la présence d'un joueur est rangée sous une **empreinte** de son code (SHA-256), sa
//! boîte aux lettres sous une autre. Seul qui connaît le code peut suivre l'une ou écrire
//! dans l'autre.
//!
//! Chaque profil a sa clé ed25519 : présences et lettres sont **signées** (avec l'adresse
//! visée et un instant croissant, contre le rejeu). Le relais lie une empreinte de présence
//! à la première clé qui y publie ; le jeu, lui, retient la clé de chaque ami à sa première
//! présence : personne ne peut se faire passer pour un autre.
//!
//! | Type | Trame | Octets après le type |
//! |---|---|---|
//! | 17 | `Ecouter` | boîte (32), présences suivies (32 chacune, `SUIVIS_MAX` au plus) |
//! | 18 | `Publier` | empreinte (32), signé |
//! | 19 | `Presence` | empreinte (32), signé (la trame `Publier` refaite suivre) |
//! | 20 | `Poster` | boîte (32), signé |
//! | 21 | `Courrier` | boîte (32), signé (la trame `Poster` refaite suivre) |
//! | 22 | `Absent` | empreinte (32) : son auteur s'est déconnecté (dit par le relais) |
//!
//! « Signé » : clé publique (32), signature (64), instant u64 (ms), contenu (le reste,
//! `CONTENU_MAX` au plus, décrit par le jeu).

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

/// Présences suivies au plus par une connexion (comme les amis d'un profil).
pub const SUIVIS_MAX: usize = 50;
/// Taille maximale du contenu d'une présence ou d'une lettre.
pub const CONTENU_MAX: usize = 512;

const ECOUTER: u8 = 17;
const PUBLIER: u8 = 18;
const PRESENCE: u8 = 19;
const POSTER: u8 = 20;
const COURRIER: u8 = 21;
const ABSENT: u8 = 22;

/// Une adresse sur le relais : empreinte de présence ou de boîte.
pub type Adresse = [u8; 32];

/// Contenu signé par son auteur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signe<'a> {
    pub cle: [u8; 32],
    pub signature: [u8; 64],
    /// Instant de l'auteur (ms) : chaque envoi doit dépasser le précédent.
    pub instant: u64,
    pub contenu: &'a [u8],
}

/// Une trame des amis, empruntant ses octets au message reçu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrameAmi<'a> {
    /// Client → relais : recevoir les lettres de `boite` et les présences `suivis`
    /// (remplace l'abonnement précédent de la connexion).
    Ecouter { boite: Adresse, suivis: &'a [u8] },
    /// Client → relais : sa présence.
    Publier {
        empreinte: Adresse,
        signe: Signe<'a>,
    },
    /// Relais → client : présence d'un joueur suivi.
    Presence {
        empreinte: Adresse,
        signe: Signe<'a>,
    },
    /// Client → relais : une lettre pour `boite`.
    Poster { boite: Adresse, signe: Signe<'a> },
    /// Relais → client : une lettre reçue.
    Courrier { boite: Adresse, signe: Signe<'a> },
    /// Relais → client : le joueur suivi s'est déconnecté.
    Absent { empreinte: Adresse },
}

/// Genre de ce qui est signé (la signature ne vaut que pour lui).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Genre {
    Presence,
    Courrier,
}

impl Genre {
    fn domaine(self) -> &'static [u8] {
        match self {
            Self::Presence => b"bs-ami/presence\0",
            Self::Courrier => b"bs-ami/courrier\0",
        }
    }
}

fn empreinte(domaine: &str, code: &str) -> Adresse {
    let mut h = Sha256::new();
    h.update(domaine.as_bytes());
    h.update(code.as_bytes());
    h.finalize().into()
}

/// Adresse de la présence d'un code ami.
pub fn empreinte_presence(code: &str) -> Adresse {
    empreinte("bs-ami/presence/", code)
}

/// Adresse de la boîte aux lettres d'un code ami.
pub fn empreinte_boite(code: &str) -> Adresse {
    empreinte("bs-ami/boite/", code)
}

fn message_signe(genre: Genre, adresse: &Adresse, instant: u64, contenu: &[u8]) -> Vec<u8> {
    let mut m = Vec::with_capacity(16 + 32 + 8 + contenu.len());
    m.extend_from_slice(genre.domaine());
    m.extend_from_slice(adresse);
    m.extend_from_slice(&instant.to_le_bytes());
    m.extend_from_slice(contenu);
    m
}

/// Signe `contenu` pour `adresse`.
pub fn signer<'a>(
    cle: &SigningKey,
    genre: Genre,
    adresse: &Adresse,
    instant: u64,
    contenu: &'a [u8],
) -> Signe<'a> {
    let signature = cle.sign(&message_signe(genre, adresse, instant, contenu));
    Signe {
        cle: cle.verifying_key().to_bytes(),
        signature: signature.to_bytes(),
        instant,
        contenu,
    }
}

impl Signe<'_> {
    /// Vrai si la signature est de `cle` pour ce genre et cette adresse.
    pub fn verifier(&self, genre: Genre, adresse: &Adresse) -> bool {
        let Ok(cle) = VerifyingKey::from_bytes(&self.cle) else {
            return false;
        };
        let signature = Signature::from_bytes(&self.signature);
        let m = message_signe(genre, adresse, self.instant, self.contenu);
        cle.verify(&m, &signature).is_ok()
    }

    fn ecrire(&self, sortie: &mut Vec<u8>) {
        sortie.extend_from_slice(&self.cle);
        sortie.extend_from_slice(&self.signature);
        sortie.extend_from_slice(&self.instant.to_le_bytes());
        sortie.extend_from_slice(self.contenu);
    }
}

/// Présences suivies d'un `Ecouter`, une à une.
pub fn suivis(octets: &[u8]) -> impl Iterator<Item = Adresse> + '_ {
    octets
        .chunks_exact(32)
        .map(|c| c.try_into().expect("32 octets"))
}

impl TrameAmi<'_> {
    /// Écrit la trame dans `sortie` (vidé d'abord).
    pub fn ecrire(&self, sortie: &mut Vec<u8>) {
        sortie.clear();
        let (genre, adresse, signe) = match self {
            Self::Ecouter { boite, suivis } => {
                sortie.push(ECOUTER);
                sortie.extend_from_slice(boite);
                sortie.extend_from_slice(suivis);
                return;
            }
            Self::Absent { empreinte } => {
                sortie.push(ABSENT);
                sortie.extend_from_slice(empreinte);
                return;
            }
            Self::Publier { empreinte, signe } => (PUBLIER, empreinte, signe),
            Self::Presence { empreinte, signe } => (PRESENCE, empreinte, signe),
            Self::Poster { boite, signe } => (POSTER, boite, signe),
            Self::Courrier { boite, signe } => (COURRIER, boite, signe),
        };
        sortie.push(genre);
        sortie.extend_from_slice(adresse);
        signe.ecrire(sortie);
    }
}

/// La même trame, refaite suivre par le relais : `Publier` devient `Presence`, `Poster`
/// devient `Courrier` (seul le premier octet change). `None` pour les autres.
pub fn faire_suivre(trame: &[u8]) -> Option<Vec<u8>> {
    let genre = match *trame.first()? {
        PUBLIER => PRESENCE,
        POSTER => COURRIER,
        _ => return None,
    };
    let mut o = trame.to_vec();
    o[0] = genre;
    Some(o)
}

fn adresse(o: &[u8]) -> Option<(Adresse, &[u8])> {
    let (a, reste) = o.split_first_chunk::<32>()?;
    Some((*a, reste))
}

fn signe(o: &[u8]) -> Option<Signe<'_>> {
    let (cle, o) = o.split_first_chunk::<32>()?;
    let (signature, o) = o.split_first_chunk::<64>()?;
    let (instant, contenu) = o.split_first_chunk::<8>()?;
    (contenu.len() <= CONTENU_MAX).then_some(Signe {
        cle: *cle,
        signature: *signature,
        instant: u64::from_le_bytes(*instant),
        contenu,
    })
}

/// Lit une trame des amis ; `None` si elle est mal formée ou d'un autre genre.
pub fn lire_trame_ami(o: &[u8]) -> Option<TrameAmi<'_>> {
    let (&genre, corps) = o.split_first()?;
    let (a, reste) = adresse(corps)?;
    match genre {
        ECOUTER => {
            (reste.len() % 32 == 0 && reste.len() / 32 <= SUIVIS_MAX).then_some(TrameAmi::Ecouter {
                boite: a,
                suivis: reste,
            })
        }
        PUBLIER => Some(TrameAmi::Publier {
            empreinte: a,
            signe: signe(reste)?,
        }),
        PRESENCE => Some(TrameAmi::Presence {
            empreinte: a,
            signe: signe(reste)?,
        }),
        POSTER => Some(TrameAmi::Poster {
            boite: a,
            signe: signe(reste)?,
        }),
        COURRIER => Some(TrameAmi::Courrier {
            boite: a,
            signe: signe(reste)?,
        }),
        ABSENT => reste
            .is_empty()
            .then_some(TrameAmi::Absent { empreinte: a }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cle(n: u8) -> SigningKey {
        SigningKey::from_bytes(&[n; 32])
    }

    #[test]
    fn aller_retour_et_signature() {
        let k = cle(7);
        let p = empreinte_presence("ABC-DEF-GHJ");
        assert_ne!(p, empreinte_boite("ABC-DEF-GHJ"));
        assert_ne!(p, empreinte_presence("ABC-DEF-GHK"));
        let s = signer(&k, Genre::Presence, &p, 42, b"{\"t\":\"presence\"}");
        let mut o = Vec::new();
        TrameAmi::Publier {
            empreinte: p,
            signe: s,
        }
        .ecrire(&mut o);
        let Some(TrameAmi::Publier { empreinte, signe }) = lire_trame_ami(&o) else {
            panic!("trame illisible");
        };
        assert_eq!(empreinte, p);
        assert!(signe.verifier(Genre::Presence, &p));
        // Pas pour une autre adresse, un autre genre, ni modifiée
        assert!(!signe.verifier(Genre::Presence, &empreinte_presence("X")));
        assert!(!signe.verifier(Genre::Courrier, &p));
        let mut faux = signe;
        faux.instant += 1;
        assert!(!faux.verifier(Genre::Presence, &p));
        // Refaite suivre : même contenu, autre genre
        let suivie = faire_suivre(&o).unwrap();
        assert!(matches!(
            lire_trame_ami(&suivie),
            Some(TrameAmi::Presence { signe: s2, .. }) if s2 == signe
        ));
    }

    #[test]
    fn trames_mal_formees() {
        let b = empreinte_boite("A");
        let mut o = Vec::new();
        let liste = [7u8; 64];
        TrameAmi::Ecouter {
            boite: b,
            suivis: &liste,
        }
        .ecrire(&mut o);
        let Some(TrameAmi::Ecouter { suivis: l, .. }) = lire_trame_ami(&o) else {
            panic!("Ecouter illisible");
        };
        assert_eq!(suivis(l).count(), 2);
        o.push(0);
        assert_eq!(lire_trame_ami(&o), None);
        let trop = vec![0u8; 32 * (SUIVIS_MAX + 1)];
        TrameAmi::Ecouter {
            boite: b,
            suivis: &trop,
        }
        .ecrire(&mut o);
        assert_eq!(lire_trame_ami(&o), None);
        TrameAmi::Absent { empreinte: b }.ecrire(&mut o);
        assert_eq!(lire_trame_ami(&o), Some(TrameAmi::Absent { empreinte: b }));
        let contenu = vec![0u8; CONTENU_MAX + 1];
        let s = signer(&cle(1), Genre::Courrier, &b, 1, &contenu);
        TrameAmi::Poster { boite: b, signe: s }.ecrire(&mut o);
        assert_eq!(lire_trame_ami(&o), None);
        assert_eq!(lire_trame_ami(&[PUBLIER; 40]), None);
        assert_eq!(lire_trame_ami(&[0; 40]), None);
        assert_eq!(faire_suivre(&[ECOUTER; 40]), None);
    }
}
