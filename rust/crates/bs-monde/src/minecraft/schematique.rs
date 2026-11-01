//! Schématiques Minecraft (src/minecraft/schematique.ts) : .schem de WorldEdit / FAWE
//! (formats Sponge 1, 2 et 3) et .nbt des blocs de structure. Le résultat est un
//! volume de blocs indexés dans une palette d'états (« oak_stairs[facing=east] »),
//! avec le texte des panneaux (repères posés par le constructeur).

use std::collections::HashMap;

use super::nbt::{Compound, Nbt, lire_nbt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Panneau {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub texte: String,
}

#[derive(Debug, Clone)]
pub struct Schematique {
    /// Blocs d'ouest en est (x), de bas en haut (y), du nord au sud (z).
    pub largeur: usize,
    pub hauteur: usize,
    pub profondeur: usize,
    /// États sans espace de noms ; l'index 0 est toujours « air ».
    pub palette: Vec<String>,
    /// Index dans la palette de la case (x, y, z) : (y × profondeur + z) × largeur + x.
    pub blocs: Vec<u16>,
    pub panneaux: Vec<Panneau>,
    /// Version des données Minecraft du fichier (3465 = 1.20.1).
    pub version_donnees: i64,
}

impl Schematique {
    pub fn index(&self, x: usize, y: usize, z: usize) -> usize {
        (y * self.profondeur + z) * self.largeur + x
    }

    /// Index de palette de la case, `None` hors du volume.
    pub fn bloc(&self, x: i32, y: i32, z: i32) -> Option<u16> {
        let (l, h, p) = (
            self.largeur as i32,
            self.hauteur as i32,
            self.profondeur as i32,
        );
        if x < 0 || y < 0 || z < 0 || x >= l || y >= h || z >= p {
            return None;
        }
        Some(self.blocs[self.index(x as usize, y as usize, z as usize)])
    }
}

/// Blocs renommés depuis les anciennes versions → nom actuel.
const RENOMMES: [(&str, &str); 2] = [("grass", "short_grass"), ("chain", "iron_chain")];
const AIRS: [&str; 3] = ["air", "cave_air", "void_air"];

pub fn lire_schematique(octets: &[u8]) -> Result<Schematique, String> {
    let (_, racine) = lire_nbt(octets)?;
    if let Some(s) = racine.compound("Schematic") {
        return lire_sponge(s, 3);
    }
    if racine.get("BlockData").is_some() {
        return lire_sponge(&racine, 2);
    }
    if racine.get("blocks").is_some() && racine.get("size").is_some() {
        return lire_structure(&racine);
    }
    if racine.get("Regions").is_some() {
        return Err("Fichier Litematica (.litematic) pas encore pris en charge : exporte la zone en .schem avec WorldEdit.".into());
    }
    Err("Format de schématique inconnu (attendu : .schem de WorldEdit ou .nbt de bloc de structure).".into())
}

/// Normalise un état : sans « minecraft: », avec les noms actuels, air unique.
fn normaliser(etat: &str) -> String {
    let sans = etat.strip_prefix("minecraft:").unwrap_or(etat);
    let (nom, reste) = sans.find('[').map_or((sans, ""), |i| sans.split_at(i));
    if AIRS.contains(&nom) {
        return "air".into();
    }
    let actuel = RENOMMES
        .iter()
        .find(|(a, _)| *a == nom)
        .map_or(nom, |(_, n)| n);
    format!("{actuel}{reste}")
}

/// Palette normalisée (air en 0).
struct Palette {
    etats: Vec<String>,
    index: HashMap<String, u16>,
}

impl Palette {
    fn nouvelle() -> Self {
        Self {
            etats: vec!["air".into()],
            index: HashMap::from([("air".into(), 0)]),
        }
    }

    fn ajouter(&mut self, etat: &str) -> u16 {
        let n = normaliser(etat);
        if let Some(&i) = self.index.get(&n) {
            return i;
        }
        let i = self.etats.len() as u16;
        self.etats.push(n.clone());
        self.index.insert(n, i);
        i
    }
}

fn dimension(s: &Compound, nom: &str) -> Result<usize, String> {
    s.get(nom)
        .and_then(Nbt::entier)
        .and_then(|v| usize::try_from(v & 0xffff).ok())
        .ok_or_else(|| format!("Schématique invalide : {nom} manquant."))
}

fn lire_sponge(s: &Compound, version: u32) -> Result<Schematique, String> {
    let (largeur, hauteur, profondeur) = (
        dimension(s, "Width")?,
        dimension(s, "Height")?,
        dimension(s, "Length")?,
    );
    let blocs_nbt = if version >= 3 {
        s.compound("Blocks")
            .ok_or("Schématique invalide : Blocks manquant.")?
    } else {
        s
    };
    let palette_nbt = blocs_nbt
        .compound("Palette")
        .ok_or("Schématique invalide : Palette manquante.")?;
    let donnees = match blocs_nbt.get(if version >= 3 { "Data" } else { "BlockData" }) {
        Some(Nbt::Octets(d)) => d,
        _ => return Err("Schématique invalide : données des blocs manquantes.".into()),
    };
    let mut palette = Palette::nouvelle();
    let mut passage = vec![0u16; palette_nbt.0.len() + 1];
    for (etat, index) in &palette_nbt.0 {
        let i = index.entier().unwrap_or(0) as usize;
        if i >= passage.len() {
            passage.resize(i + 1, 0);
        }
        passage[i] = palette.ajouter(etat);
    }
    // Indices en varint, dans l'ordre (y × profondeur + z) × largeur + x
    let mut blocs = vec![0u16; largeur * hauteur * profondeur];
    let mut lu = 0;
    for b in blocs.iter_mut() {
        if lu >= donnees.len() {
            break;
        }
        let (mut valeur, mut decalage) = (0usize, 0);
        loop {
            let o = donnees.get(lu).map_or(0, |&o| o as u8);
            lu += 1;
            valeur |= ((o & 0x7f) as usize) << decalage;
            if o & 0x80 == 0 || decalage > 28 {
                break;
            }
            decalage += 7;
        }
        *b = passage.get(valeur).copied().unwrap_or(0);
    }
    let entites = ["BlockEntities", "TileEntities"]
        .iter()
        .find_map(|n| blocs_nbt.get(n).or_else(|| s.get(n)))
        .and_then(Nbt::liste)
        .unwrap_or(&[]);
    let mut panneaux = Vec::new();
    for e in entites {
        let Nbt::Compound(e) = e else { continue };
        let id = e
            .get("Id")
            .or_else(|| e.get("id"))
            .and_then(Nbt::chaine)
            .unwrap_or("");
        if !id.contains("sign") {
            continue;
        }
        let Some(pos) = e.get("Pos").and_then(Nbt::entiers).filter(|p| p.len() >= 3) else {
            continue;
        };
        let texte = texte_de(e.get("Data").unwrap_or(&Nbt::Compound(e.clone())));
        panneaux.push(Panneau {
            x: pos[0] as i32,
            y: pos[1] as i32,
            z: pos[2] as i32,
            texte,
        });
    }
    Ok(Schematique {
        largeur,
        hauteur,
        profondeur,
        palette: palette.etats,
        blocs,
        panneaux,
        version_donnees: s.get("DataVersion").and_then(Nbt::entier).unwrap_or(0),
    })
}

fn lire_structure(s: &Compound) -> Result<Schematique, String> {
    let taille = s
        .get("size")
        .and_then(Nbt::entiers)
        .filter(|t| t.len() >= 3);
    let taille = taille.ok_or("Structure invalide : size manquant.")?;
    let [largeur, hauteur, profondeur] = [0, 1, 2].map(|k| taille[k].max(0) as usize);
    let palette_nbt = s
        .get("palette")
        .or_else(|| {
            s.get("palettes")
                .and_then(Nbt::liste)
                .and_then(|l| l.first())
        })
        .and_then(Nbt::liste)
        .ok_or("Structure invalide : palette manquante.")?;
    let mut palette = Palette::nouvelle();
    let passage: Vec<u16> = palette_nbt
        .iter()
        .map(|p| {
            let Nbt::Compound(p) = p else { return 0 };
            let nom = p.get("Name").and_then(Nbt::chaine).unwrap_or("air");
            let suffixe = match p.compound("Properties") {
                Some(pr) if !pr.0.is_empty() => {
                    let paires: Vec<String> =
                        pr.0.iter()
                            .map(|(k, v)| format!("{k}={}", v.chaine().unwrap_or("")))
                            .collect();
                    format!("[{}]", paires.join(","))
                }
                _ => String::new(),
            };
            palette.ajouter(&format!("{nom}{suffixe}"))
        })
        .collect();
    let mut blocs = vec![0u16; largeur * hauteur * profondeur];
    let mut panneaux = Vec::new();
    for b in s.get("blocks").and_then(Nbt::liste).unwrap_or(&[]) {
        let Nbt::Compound(b) = b else { continue };
        let Some(pos) = b.get("pos").and_then(Nbt::entiers).filter(|p| p.len() >= 3) else {
            continue;
        };
        let [x, y, z] = [0, 1, 2].map(|k| pos[k] as usize);
        if x >= largeur || y >= hauteur || z >= profondeur {
            continue;
        }
        let etat = b.get("state").and_then(Nbt::entier).unwrap_or(0) as usize;
        blocs[(y * profondeur + z) * largeur + x] = passage.get(etat).copied().unwrap_or(0);
        if let Some(nbt) = b.compound("nbt")
            && nbt
                .get("id")
                .and_then(Nbt::chaine)
                .is_some_and(|i| i.contains("sign"))
        {
            panneaux.push(Panneau {
                x: x as i32,
                y: y as i32,
                z: z as i32,
                texte: texte_de(&Nbt::Compound(nbt.clone())),
            });
        }
    }
    Ok(Schematique {
        largeur,
        hauteur,
        profondeur,
        palette: palette.etats,
        blocs,
        panneaux,
        version_donnees: s.get("DataVersion").and_then(Nbt::entier).unwrap_or(0),
    })
}

/// Tout le texte d'un panneau (lignes JSON ou texte brut, faces avant et arrière).
fn texte_de(v: &Nbt) -> String {
    let mut morceaux = Vec::new();
    parcourir(v, &mut morceaux);
    morceaux.join(" ").trim().to_owned()
}

fn parcourir(v: &Nbt, morceaux: &mut Vec<String>) {
    match v {
        Nbt::Chaine(s) => {
            // Ligne au format JSON ({"text":"depart"}) ou texte simple
            let textes = textes_json(s);
            if textes.is_empty() {
                morceaux.push(s.clone());
            } else {
                morceaux.extend(textes);
            }
        }
        Nbt::Liste(l) => l.iter().for_each(|x| parcourir(x, morceaux)),
        Nbt::Compound(c) => c.0.iter().for_each(|(_, x)| parcourir(x, morceaux)),
        _ => {}
    }
}

/// Valeurs des clés `"text": "…"` d'une ligne JSON.
fn textes_json(s: &str) -> Vec<String> {
    let mut r = Vec::new();
    let mut reste = s;
    while let Some(i) = reste.find("\"text\"") {
        reste = &reste[i + 6..];
        let apres = reste.trim_start();
        let Some(apres) = apres.strip_prefix(':') else {
            continue;
        };
        let apres = apres.trim_start();
        let Some(apres) = apres.strip_prefix('"') else {
            continue;
        };
        let Some(fin) = apres.find('"') else { break };
        r.push(apres[..fin].to_owned());
        reste = &apres[fin..];
    }
    r
}
