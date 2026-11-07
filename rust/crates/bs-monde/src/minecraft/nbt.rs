//! Lecture du format NBT de Minecraft (src/minecraft/nbt.ts) : gros-boutiste,
//! compressé en gzip le plus souvent (zlib parfois). Les tableaux d'octets, d'entiers
//! et de longs gardent leur forme typée. Bedrock (.mcstructure) : petit-boutiste, sans
//! compression.

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
    lire_racine(Lecteur {
        octets: if decompresse { &brut } else { octets },
        position: 0,
        petit: false,
    })
}

/// NBT de Bedrock : petit-boutiste, non compressé.
pub fn lire_nbt_bedrock(octets: &[u8]) -> Result<(String, Compound), String> {
    lire_racine(Lecteur {
        octets,
        position: 0,
        petit: true,
    })
}

fn lire_racine(mut l: Lecteur) -> Result<(String, Compound), String> {
    if l.octet()? != 10 {
        return Err("Fichier NBT invalide : la racine doit être un compound.".into());
    }
    let nom = l.chaine()?;
    Ok((nom, l.compound()?))
}

struct Lecteur<'a> {
    octets: &'a [u8],
    position: usize,
    /// Petit-boutiste (Bedrock).
    petit: bool,
}

/// Nombre de N octets dans l'ordre du fichier.
macro_rules! nombre {
    ($l:expr, $t:ty) => {{
        let o = $l.prendre()?;
        if $l.petit {
            <$t>::from_le_bytes(o)
        } else {
            <$t>::from_be_bytes(o)
        }
    }};
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
        Ok(nombre!(self, i8))
    }

    fn entier(&mut self) -> Result<i32, String> {
        Ok(nombre!(self, i32))
    }

    fn longueur(&mut self) -> Result<usize, String> {
        usize::try_from(self.entier()?)
            .map_err(|_| "Fichier NBT invalide : longueur négative.".into())
    }

    fn chaine(&mut self) -> Result<String, String> {
        let n = nombre!(self, u16) as usize;
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
            2 => Nbt::Court(nombre!(self, i16)),
            3 => Nbt::Entier(self.entier()?),
            4 => Nbt::Long(nombre!(self, i64)),
            5 => Nbt::Flottant(nombre!(self, f32)),
            6 => Nbt::Double(nombre!(self, f64)),
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
                    v.push(nombre!(self, i64));
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

/// Écrit `racine` en NBT de Java (gros-boutiste, non compressé).
pub fn ecrire_nbt(nom: &str, racine: &Compound) -> Vec<u8> {
    let mut o = vec![10];
    ecrire_chaine(&mut o, nom);
    ecrire_compound(&mut o, racine);
    o
}

fn type_de(v: &Nbt) -> u8 {
    match v {
        Nbt::Octet(_) => 1,
        Nbt::Court(_) => 2,
        Nbt::Entier(_) => 3,
        Nbt::Long(_) => 4,
        Nbt::Flottant(_) => 5,
        Nbt::Double(_) => 6,
        Nbt::Octets(_) => 7,
        Nbt::Chaine(_) => 8,
        Nbt::Liste(_) => 9,
        Nbt::Compound(_) => 10,
        Nbt::Entiers(_) => 11,
        Nbt::Longs(_) => 12,
    }
}

fn ecrire_chaine(o: &mut Vec<u8>, s: &str) {
    o.extend((s.len().min(u16::MAX as usize) as u16).to_be_bytes());
    o.extend(&s.as_bytes()[..s.len().min(u16::MAX as usize)]);
}

fn ecrire_compound(o: &mut Vec<u8>, c: &Compound) {
    for (nom, v) in &c.0 {
        o.push(type_de(v));
        ecrire_chaine(o, nom);
        ecrire_valeur(o, v);
    }
    o.push(0);
}

fn ecrire_valeur(o: &mut Vec<u8>, v: &Nbt) {
    match v {
        Nbt::Octet(x) => o.push(*x as u8),
        Nbt::Court(x) => o.extend(x.to_be_bytes()),
        Nbt::Entier(x) => o.extend(x.to_be_bytes()),
        Nbt::Long(x) => o.extend(x.to_be_bytes()),
        Nbt::Flottant(x) => o.extend(x.to_be_bytes()),
        Nbt::Double(x) => o.extend(x.to_be_bytes()),
        Nbt::Octets(t) => {
            o.extend((t.len() as i32).to_be_bytes());
            o.extend(t.iter().map(|&x| x as u8));
        }
        Nbt::Chaine(s) => ecrire_chaine(o, s),
        Nbt::Liste(l) => {
            o.push(l.first().map_or(0, type_de));
            o.extend((l.len() as i32).to_be_bytes());
            l.iter().for_each(|x| ecrire_valeur(o, x));
        }
        Nbt::Compound(c) => ecrire_compound(o, c),
        Nbt::Entiers(t) => {
            o.extend((t.len() as i32).to_be_bytes());
            t.iter().for_each(|x| o.extend(x.to_be_bytes()));
        }
        Nbt::Longs(t) => {
            o.extend((t.len() as i32).to_be_bytes());
            t.iter().for_each(|x| o.extend(x.to_be_bytes()));
        }
    }
}

#[cfg(test)]
mod essais {
    use super::*;

    #[test]
    fn ecrire_puis_relire() {
        let c = Compound(vec![
            ("a".into(), Nbt::Court(-3)),
            ("b".into(), Nbt::Liste(vec![Nbt::Chaine("x".into())])),
            ("c".into(), Nbt::Longs(vec![1, -1])),
            ("d".into(), Nbt::Octets(vec![-1, 2])),
        ]);
        let (nom, relu) = lire_nbt(&ecrire_nbt("racine", &c)).unwrap();
        assert_eq!(nom, "racine");
        assert_eq!(relu, c);
    }
}
