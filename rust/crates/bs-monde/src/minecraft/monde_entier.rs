//! Monde Minecraft entier (dossier de sauvegarde de Java : `level.dat` et
//! `region/*.mca`) : on découpe la zone autour du panneau « depart » (sinon autour du
//! point d'apparition) et on l'enregistre en schématique Sponge (.schem), lue ensuite
//! comme toutes les maps. Formats des tronçons : 1.18 et plus (`sections`,
//! `block_states`), et 1.16-1.17 (`Level`, `Sections`, `Palette`, `BlockStates`).

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use flate2::Compression;
use flate2::write::GzEncoder;

use super::nbt::{Compound, Nbt, ecrire_nbt, lire_nbt};
use super::schematique::{Palette, etat_de, texte_de};

/// Demi-côté de la zone découpée (blocs), autour du départ.
const DEMI_COTE: i32 = 48;
/// Blocs gardés sous le départ, et au-dessus.
const DESSOUS: i32 = 16;
const DESSUS: i32 = 48;

/// Le dossier est-il une sauvegarde de monde de Java ?
pub fn est_un_monde(dossier: &Path) -> bool {
    dossier.join("level.dat").is_file() && dossier.join("region").is_dir()
}

/// Tronçon n° `i` (0 à 1023) d'un fichier de région, décompressé.
fn troncon(region: &[u8], i: usize) -> Option<Compound> {
    let e = region.get(4 * i..4 * i + 4)?;
    let debut = (u32::from_be_bytes([0, e[0], e[1], e[2]]) as usize) * 4096;
    if debut == 0 {
        return None;
    }
    let l = region.get(debut..debut + 4)?;
    let longueur = u32::from_be_bytes([l[0], l[1], l[2], l[3]]) as usize;
    // Octet de compression (1 gzip, 2 zlib, 3 aucune) : lire_nbt reconnaît les trois
    let donnees = region.get(debut + 5..debut + 4 + longueur)?;
    Some(lire_nbt(donnees).ok()?.1)
}

/// Contenu d'un tronçon (sous `Level` avant la 1.18).
fn contenu(c: &Compound) -> &Compound {
    c.compound("Level").unwrap_or(c)
}

fn liste<'a>(c: &'a Compound, noms: &[&str]) -> &'a [Nbt] {
    noms.iter()
        .find_map(|n| c.get(n).and_then(Nbt::liste))
        .unwrap_or(&[])
}

/// Entités de blocs d'un tronçon.
fn entites(c: &Compound) -> &[Nbt] {
    liste(contenu(c), &["block_entities", "TileEntities"])
}

fn position(e: &Compound) -> Option<[i32; 3]> {
    let [x, y, z] = ["x", "y", "z"].map(|k| e.get(k).and_then(Nbt::entier));
    Some([x? as i32, y? as i32, z? as i32])
}

fn est_le_depart(e: &Compound) -> bool {
    let id = e.get("id").and_then(Nbt::chaine).unwrap_or("");
    let t = texte_de(&Nbt::Compound(e.clone())).to_lowercase();
    id.contains("sign") && (t.contains("depart") || t.contains("départ"))
}

/// Fichiers de région : (rx, rz) → chemin.
fn regions(dossier: &Path) -> Vec<((i32, i32), std::path::PathBuf)> {
    let mut r: Vec<_> = std::fs::read_dir(dossier.join("region"))
        .map(|d| {
            d.filter_map(Result::ok)
                .map(|e| e.path())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
        .into_iter()
        .filter_map(|p| {
            let nom = p.file_name()?.to_str()?.to_owned();
            let mut morceaux = nom.strip_suffix(".mca")?.strip_prefix("r.")?.split('.');
            let rx = morceaux.next()?.parse().ok()?;
            let rz = morceaux.next()?.parse().ok()?;
            Some(((rx, rz), p))
        })
        .collect();
    r.sort();
    r
}

/// Point d'apparition du monde (level.dat).
fn apparition(dossier: &Path) -> [i32; 3] {
    let lu = std::fs::read(dossier.join("level.dat"))
        .ok()
        .and_then(|o| lire_nbt(&o).ok());
    let Some((_, racine)) = lu else {
        return [0, 64, 0];
    };
    let d = racine.compound("Data").unwrap_or(&racine);
    if let Some(p) = d
        .compound("spawn")
        .and_then(|s| s.get("pos"))
        .and_then(Nbt::entiers)
        && p.len() >= 3
    {
        return [p[0] as i32, p[1] as i32, p[2] as i32];
    }
    let v = ["SpawnX", "SpawnY", "SpawnZ"].map(|k| d.get(k).and_then(Nbt::entier));
    [v[0].unwrap_or(0), v[1].unwrap_or(64), v[2].unwrap_or(0)].map(|x| x as i32)
}

/// Valeur n° `i` de `bits` bits, sans chevauchement entre deux longs (depuis la 1.16).
fn valeur(longs: &[i64], i: usize, bits: usize) -> usize {
    let par_long = 64 / bits;
    let l = longs.get(i / par_long).copied().unwrap_or(0) as u64;
    ((l >> ((i % par_long) * bits)) & ((1u64 << bits) - 1)) as usize
}

fn bits_pour(n: usize, minimum: usize) -> usize {
    ((usize::BITS - n.saturating_sub(1).leading_zeros()) as usize).max(minimum)
}

/// Zone découpée : blocs, palette, panneaux et biomes comptés.
struct Decoupe {
    min: [i32; 3],
    taille: [usize; 3],
    palette: Palette,
    blocs: Vec<u16>,
    entites: Vec<Nbt>,
    biomes: HashMap<String, u32>,
    version: i64,
}

impl Decoupe {
    fn ajouter_troncon(&mut self, c: &Compound, tx: i32, tz: i32) {
        self.version = self
            .version
            .max(c.get("DataVersion").and_then(Nbt::entier).unwrap_or(0));
        let [l, h, p] = self.taille.map(|t| t as i32);
        for s in liste(contenu(c), &["sections", "Sections"]) {
            let Nbt::Compound(s) = s else { continue };
            let Some(sy) = s.get("Y").and_then(Nbt::entier) else {
                continue;
            };
            let y0 = sy as i32 * 16;
            if y0 + 16 <= self.min[1] || y0 >= self.min[1] + h {
                continue;
            }
            let etats = s.compound("block_states");
            let palette = etats.map_or_else(|| liste(s, &["Palette"]), |e| liste(e, &["palette"]));
            let passage: Vec<u16> = palette
                .iter()
                .map(|e| match e {
                    Nbt::Compound(e) => self.palette.ajouter(&etat_de(e)),
                    _ => 0,
                })
                .collect();
            let longs = match etats.map_or_else(|| s.get("BlockStates"), |e| e.get("data")) {
                Some(Nbt::Longs(l)) => l.as_slice(),
                _ => &[],
            };
            let bits = bits_pour(passage.len(), 4);
            for i in 0..4096 {
                let (x, y, z) = (i as i32 & 15, i as i32 >> 8, (i as i32 >> 4) & 15);
                let [bx, by, bz] = [
                    tx * 16 + x - self.min[0],
                    y0 + y - self.min[1],
                    tz * 16 + z - self.min[2],
                ];
                if bx < 0 || by < 0 || bz < 0 || bx >= l || by >= h || bz >= p {
                    continue;
                }
                let k = if longs.is_empty() {
                    0
                } else {
                    valeur(longs, i, bits)
                };
                self.blocs[((by * p + bz) * l + bx) as usize] =
                    passage.get(k).copied().unwrap_or(0);
            }
            self.compter_biomes(s);
        }
        for e in entites(c) {
            let Nbt::Compound(e) = e else { continue };
            let Some(pos) = position(e) else { continue };
            let r = [0, 1, 2].map(|k| pos[k] - self.min[k]);
            if (0..3).any(|k| r[k] < 0 || r[k] >= self.taille[k] as i32)
                || !e
                    .get("id")
                    .and_then(Nbt::chaine)
                    .unwrap_or("")
                    .contains("sign")
            {
                continue;
            }
            self.entites.push(Nbt::Compound(Compound(vec![
                ("Pos".into(), Nbt::Entiers(r.to_vec())),
                (
                    "Id".into(),
                    e.get("id").cloned().unwrap_or(Nbt::Chaine(String::new())),
                ),
                ("Data".into(), Nbt::Compound(e.clone())),
            ])));
        }
    }

    /// Biomes de la section (1.18 et plus : 64 cases de 4 × 4 × 4).
    fn compter_biomes(&mut self, s: &Compound) {
        let Some(b) = s.compound("biomes") else {
            return;
        };
        let noms: Vec<&str> = liste(b, &["palette"])
            .iter()
            .filter_map(Nbt::chaine)
            .collect();
        let longs = match b.get("data") {
            Some(Nbt::Longs(l)) => l.as_slice(),
            _ => &[],
        };
        let bits = bits_pour(noms.len(), 1);
        for i in 0..64 {
            let k = if longs.is_empty() {
                0
            } else {
                valeur(longs, i, bits)
            };
            if let Some(n) = noms.get(k) {
                *self.biomes.entry((*n).to_owned()).or_default() += 1;
            }
        }
    }

    /// Schématique Sponge (version 3), compressée.
    fn en_schem(self) -> Vec<u8> {
        let mut donnees = Vec::with_capacity(self.blocs.len());
        for &b in &self.blocs {
            let mut v = b as u32;
            loop {
                let o = (v & 0x7f) as u8;
                v >>= 7;
                donnees.push((if v == 0 { o } else { o | 0x80 }) as i8);
                if v == 0 {
                    break;
                }
            }
        }
        let palette = Compound(
            (self.palette.etats.iter().enumerate())
                .map(|(i, e)| (format!("minecraft:{e}"), Nbt::Entier(i as i32)))
                .collect(),
        );
        let biome = self
            .biomes
            .iter()
            .max_by_key(|(n, c)| (**c, std::cmp::Reverse((*n).clone())))
            .map_or("minecraft:plains".to_owned(), |(n, _)| n.clone());
        let [l, h, p] = self.taille.map(|t| Nbt::Court(t as i16));
        let schematique = Compound(vec![
            ("Version".into(), Nbt::Entier(3)),
            ("DataVersion".into(), Nbt::Entier(self.version as i32)),
            ("Width".into(), l),
            ("Height".into(), h),
            ("Length".into(), p),
            (
                "Blocks".into(),
                Nbt::Compound(Compound(vec![
                    ("Palette".into(), Nbt::Compound(palette)),
                    ("Data".into(), Nbt::Octets(donnees)),
                    ("BlockEntities".into(), Nbt::Liste(self.entites)),
                ])),
            ),
            (
                "Biomes".into(),
                Nbt::Compound(Compound(vec![
                    (
                        "Palette".into(),
                        Nbt::Compound(Compound(vec![(biome, Nbt::Entier(0))])),
                    ),
                    ("Data".into(), Nbt::Octets(vec![0])),
                ])),
            ),
        ]);
        let racine = Compound(vec![("Schematic".into(), Nbt::Compound(schematique))]);
        let mut gz = GzEncoder::new(Vec::new(), Compression::default());
        gz.write_all(&ecrire_nbt("", &racine))
            .expect("écriture en mémoire");
        gz.finish().expect("écriture en mémoire")
    }
}

/// Découpe la zone de jeu d'un monde et la renvoie en .schem (gzip).
pub fn extraire_monde(dossier: &Path) -> Result<Vec<u8>, String> {
    let fichiers = regions(dossier);
    if fichiers.is_empty() {
        return Err("Monde sans région (dossier region vide).".into());
    }
    let spawn = apparition(dossier);
    // Les régions les plus proches du point d'apparition d'abord
    let mut ordre: Vec<usize> = (0..fichiers.len()).collect();
    let pres = |&(rx, rz): &(i32, i32)| {
        (rx * 512 + 256 - spawn[0]).abs() + (rz * 512 + 256 - spawn[2]).abs()
    };
    ordre.sort_by_key(|&i| pres(&fichiers[i].0));
    let mut lus: HashMap<(i32, i32), Vec<u8>> = HashMap::new();
    let mut depart = None;
    'recherche: for &i in &ordre {
        let (cle, chemin) = &fichiers[i];
        let octets = std::fs::read(chemin).unwrap_or_default();
        for t in 0..1024 {
            let Some(c) = troncon(&octets, t) else {
                continue;
            };
            for e in entites(&c) {
                if let Nbt::Compound(e) = e
                    && est_le_depart(e)
                {
                    depart = position(e);
                    lus.insert(*cle, octets);
                    break 'recherche;
                }
            }
        }
        lus.insert(*cle, octets);
    }
    let centre = depart.unwrap_or(spawn);
    let min = [
        centre[0] - DEMI_COTE,
        centre[1] - DESSOUS,
        centre[2] - DEMI_COTE,
    ];
    let taille = [2 * DEMI_COTE, DESSOUS + DESSUS, 2 * DEMI_COTE].map(|t| t as usize);
    let mut d = Decoupe {
        min,
        taille,
        palette: Palette::nouvelle(),
        blocs: vec![0; taille.iter().product()],
        entites: Vec::new(),
        biomes: HashMap::new(),
        version: 0,
    };
    let chemins: HashMap<_, _> = fichiers.into_iter().collect();
    for tx in min[0].div_euclid(16)..=(min[0] + taille[0] as i32 - 1).div_euclid(16) {
        for tz in min[2].div_euclid(16)..=(min[2] + taille[2] as i32 - 1).div_euclid(16) {
            let cle = (tx.div_euclid(32), tz.div_euclid(32));
            let octets = lus.entry(cle).or_insert_with(|| {
                (chemins.get(&cle).and_then(|c| std::fs::read(c).ok())).unwrap_or_default()
            });
            let i = (tz.rem_euclid(32) * 32 + tx.rem_euclid(32)) as usize;
            if let Some(c) = troncon(octets, i) {
                d.ajouter_troncon(&c, tx, tz);
            }
        }
    }
    if d.palette.etats.len() <= 1 {
        return Err("Rien autour du départ : le monde n'a pas été généré à cet endroit.".into());
    }
    Ok(d.en_schem())
}

#[cfg(test)]
#[path = "monde_entier_essais.rs"]
mod essais;
