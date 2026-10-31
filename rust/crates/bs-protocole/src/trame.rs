//! Trames échangées avec le relais (un message WebSocket binaire chacune). Le premier
//! octet donne le type ; le relais ne lit jamais la charge utile.

/// Une trame, empruntant ses octets au message reçu.
#[derive(Debug, PartialEq, Eq)]
pub enum Trame<'a> {
    /// Client → relais : ouvrir un salon (l'expéditeur en devient l'hôte).
    Creer { version: u16 },
    /// Client → relais : rejoindre le salon `code` (6 lettres).
    Rejoindre { version: u16, code: [u8; 6] },
    /// Relais → client : salon ouvert (pour l'hôte) ou rejoint (pour un invité, `place` ≥ 1).
    Salon { code: [u8; 6], place: u8 },
    /// Relais → client : refus (version différente, salon inconnu ou plein).
    Refus { raison: u8 },
    /// Invité → hôte (le relais remplit `place` avec celle de l'expéditeur).
    VersHote { place: u8, charge: &'a [u8] },
    /// Hôte → tous les invités.
    VersInvites { charge: &'a [u8] },
}

pub const REFUS_VERSION: u8 = 1;
pub const REFUS_SALON: u8 = 2;
pub const REFUS_PLEIN: u8 = 3;

impl Trame<'_> {
    /// Écrit la trame dans `sortie` (vidé d'abord).
    pub fn ecrire(&self, sortie: &mut Vec<u8>) {
        sortie.clear();
        match self {
            Trame::Creer { version } => {
                sortie.push(0);
                sortie.extend_from_slice(&version.to_le_bytes());
            }
            Trame::Rejoindre { version, code } => {
                sortie.push(1);
                sortie.extend_from_slice(&version.to_le_bytes());
                sortie.extend_from_slice(code);
            }
            Trame::Salon { code, place } => {
                sortie.push(2);
                sortie.extend_from_slice(code);
                sortie.push(*place);
            }
            Trame::Refus { raison } => sortie.extend_from_slice(&[3, *raison]),
            Trame::VersHote { place, charge } => {
                sortie.extend_from_slice(&[4, *place]);
                sortie.extend_from_slice(charge);
            }
            Trame::VersInvites { charge } => {
                sortie.push(5);
                sortie.extend_from_slice(charge);
            }
        }
    }
}

/// Lit une trame ; `None` si elle est mal formée.
pub fn lire_trame(o: &[u8]) -> Option<Trame<'_>> {
    let version = |o: &[u8]| Some(u16::from_le_bytes(o.get(1..3)?.try_into().ok()?));
    match *o.first()? {
        0 => Some(Trame::Creer {
            version: version(o)?,
        }),
        1 => Some(Trame::Rejoindre {
            version: version(o)?,
            code: o.get(3..9)?.try_into().ok()?,
        }),
        2 => Some(Trame::Salon {
            code: o.get(1..7)?.try_into().ok()?,
            place: *o.get(7)?,
        }),
        3 => Some(Trame::Refus { raison: *o.get(1)? }),
        4 => Some(Trame::VersHote {
            place: *o.get(1)?,
            charge: &o[2..],
        }),
        5 => Some(Trame::VersInvites { charge: &o[1..] }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aller_retour() {
        let mut o = Vec::new();
        for t in [
            Trame::Creer { version: 7 },
            Trame::Rejoindre {
                version: 7,
                code: *b"ABCDEF",
            },
            Trame::Salon {
                code: *b"ABCDEF",
                place: 2,
            },
            Trame::Refus {
                raison: REFUS_PLEIN,
            },
            Trame::VersHote {
                place: 1,
                charge: &[9, 8],
            },
            Trame::VersInvites { charge: &[1, 2, 3] },
        ] {
            t.ecrire(&mut o);
            assert_eq!(lire_trame(&o), Some(t));
        }
        assert_eq!(lire_trame(&[1, 0]), None);
    }
}
