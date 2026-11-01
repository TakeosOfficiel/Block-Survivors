//! Lecture du format NBT de Minecraft (src/minecraft/nbt.ts) : gros-boutiste,
//! compressé en gzip le plus souvent (zlib parfois). Les tableaux d'octets, d'entiers
//! et de longs gardent leur forme typée.

use std::io::Read;

use flate2::read::{GzDecoder, ZlibDecoder};

#[derive(Debug, Clone, PartialEq)]
pub enum Nbt {
    Octet(i8),
    Court(i16),
    Entier(i32),
    Long(i64),
    Flottant(f32),
    Double(f64),
    Octets(Vec<i8>),
    Chaine(String),
    Liste(Vec<Nbt>),
    Compound(Compound),
    Entiers(Vec<i32>),
    Longs(Vec<i64>),
}

/// Étiquettes nommées, dans l'ordre du fichier.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Compound(pub Vec<(String, Nbt)>);

impl Compound {
    pub fn get(&self, nom: &str) -> Option<&Nbt> {
        self.0.iter().find(|(n, _)| n == nom).map(|(_, v)| v)
    }

    pub fn compound(&self, nom: &str) -> Option<&Compound> {
        match self.get(nom)? {
            Nbt::Compound(c) => Some(c),
            _ => None,
        }
    }
}

impl Nbt {
    /// Valeur entière, quel que soit son type numérique.
    pub fn entier(&self) -> Option<i64> {
        match *self {
            Nbt::Octet(v) => Some(v.into()),
            Nbt::Court(v) => Some(v.into()),
            Nbt::Entier(v) => Some(v.into()),
            Nbt::Long(v) => Some(v),
            _ => None,
        }
    }

    pub fn chaine(&self) -> Option<&str> {
        match self {
            Nbt::Chaine(s) => Some(s),
            _ => None,
        }
    }

    pub fn liste(&self) -> Option<&[Nbt]> {
        match self {
            Nbt::Liste(l) => Some(l),
            _ => None,
        }
    }

    /// Tableau d'entiers (`Entiers`, ou liste de nombres).
    pub fn entiers(&self) -> Option<Vec<i64>> {
        match self {
            Nbt::Entiers(v) => Some(v.iter().map(|&x| x.into()).collect()),
            Nbt::Liste(l) => l.iter().map(Nbt::entier).collect(),
            _ => None,
        }
    }
}

/// Décompresse si besoin (gzip ou zlib), puis lit l'étiquette racine (un compound).
pub fn lire_nbt(octets: &[u8]) -> Result<(String, Compound), String> {
    let mut brut = Vec::new();
    let decompresse = match octets {
        [0x1f, 0x8b, ..] => GzDecoder::new(octets).read_to_end(&mut brut).map(|_| true),
        [0x78, ..] => ZlibDecoder::new(octets)
            .read_to_end(&mut brut)
            .map(|_| true),
        _ => Ok(false),
    }
    .map_err(|e| format!("Fichier NBT illisible : {e}"))?;
    let mut l = Lecteur {
        octets: if decompresse { &brut } else { octets },
        position: 0,
    };
    if l.octet()? != 10 {
        return Err("Fichier NBT invalide : la racine doit être un compound.".into());
    }
    let nom = l.chaine()?;
    Ok((nom, l.compound()?))
}

struct Lecteur<'a> {
    octets: &'a [u8],
    position: usize,
}

impl Lecteur<'_> {
    fn prendre<const N: usize>(&mut self) -> Result<[u8; N], String> {
        let fin = self.position + N;
        let tranche = self
            .octets
            .get(self.position..fin)
            .ok_or("Fichier NBT tronqué.")?;
        self.position = fin;
        Ok(tranche.try_into().expect("taille"))
    }

    fn octet(&mut self) -> Result<i8, String> {
        Ok(i8::from_be_bytes(self.prendre()?))
    }

    fn entier(&mut self) -> Result<i32, String> {
        Ok(i32::from_be_bytes(self.prendre()?))
    }

    fn longueur(&mut self) -> Result<usize, String> {
        usize::try_from(self.entier()?)
            .map_err(|_| "Fichier NBT invalide : longueur négative.".into())
    }

    fn chaine(&mut self) -> Result<String, String> {
        let n = u16::from_be_bytes(self.prendre()?) as usize;
        let fin = self.position + n;
        let t = self
            .octets
            .get(self.position..fin)
            .ok_or("Fichier NBT tronqué.")?;
        self.position = fin;
        Ok(String::from_utf8_lossy(t).into_owned())
    }

    fn compound(&mut self) -> Result<Compound, String> {
        let mut c = Compound::default();
        loop {
            let t = self.octet()?;
            if t == 0 {
                return Ok(c);
            }
            let nom = self.chaine()?;
            let v = self.valeur(t)?;
            c.0.push((nom, v));
        }
    }

    fn valeur(&mut self, t: i8) -> Result<Nbt, String> {
        Ok(match t {
            1 => Nbt::Octet(self.octet()?),
            2 => Nbt::Court(i16::from_be_bytes(self.prendre()?)),
            3 => Nbt::Entier(self.entier()?),
            4 => Nbt::Long(i64::from_be_bytes(self.prendre()?)),
            5 => Nbt::Flottant(f32::from_be_bytes(self.prendre()?)),
            6 => Nbt::Double(f64::from_be_bytes(self.prendre()?)),
            7 => {
                let n = self.longueur()?;
                let fin = self.position + n;
                let t = self
                    .octets
                    .get(self.position..fin)
                    .ok_or("Fichier NBT tronqué.")?;
                self.position = fin;
                Nbt::Octets(t.iter().map(|&o| o as i8).collect())
            }
            8 => Nbt::Chaine(self.chaine()?),
            9 => {
                let te = self.octet()?;
                let n = self.longueur()?;
                let mut l = Vec::with_capacity(n.min(1 << 16));
                for _ in 0..n {
                    l.push(self.valeur(te)?);
                }
                Nbt::Liste(l)
            }
            10 => Nbt::Compound(self.compound()?),
            11 => {
                let n = self.longueur()?;
                let mut v = Vec::with_capacity(n.min(1 << 20));
                for _ in 0..n {
                    v.push(self.entier()?);
                }
                Nbt::Entiers(v)
            }
            12 => {
                let n = self.longueur()?;
                let mut v = Vec::with_capacity(n.min(1 << 20));
                for _ in 0..n {
                    v.push(i64::from_be_bytes(self.prendre()?));
                }
                Nbt::Longs(v)
            }
            _ => {
                return Err(format!(
                    "Fichier NBT invalide : type d'étiquette {t} inconnu."
                ));
            }
        })
    }
}
