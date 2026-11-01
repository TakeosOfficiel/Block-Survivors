//! Modèles de blocs vanilla (src/minecraft/modeles.ts) : lit les fichiers officiels
//! (blockstates/*.json, models/block/*.json) et les « cuit » en quadrilatères dans le
//! repère du bloc (0 à 1 sur chaque axe). C'est ce qui affiche n'importe quel bloc
//! de Minecraft sans le décrire à la main : escaliers, dalles, barrières, plantes…
//!
//! Les fichiers arrivent par une `SourceFichiers` (téléchargés par le rendu) : tant
//! qu'il en manque, `cuire` rend la liste des fichiers à demander, et l'appelant
//! recommence quand ils sont là (états, puis modèles, puis leurs parents).

use std::collections::{BTreeSet, HashMap};

use serde_json::{Map, Value};

use super::cuisson::{Direction, ModeleCuit, ModeleResolu, cuire_modele};

/// Texture de l'eau et de la lave (dessinées à part, comme des cubes abaissés).
pub const TEXTURE_EAU: &str = "block/water_still";
pub const TEXTURE_LAVE: &str = "block/lava_still";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Liquide {
    Eau,
    Lave,
}

#[derive(Debug, Clone)]
pub struct Variante {
    pub modele: ModeleCuit,
    pub poids: f32,
}

/// Un état de bloc : ses parties (multipart) et, pour chacune, des variantes pondérées.
#[derive(Debug, Clone)]
pub struct BlocCuit {
    pub etat: String,
    pub nom: String,
    pub proprietes: Vec<(String, String)>,
    pub parties: Vec<Vec<Variante>>,
    /// Eau ou lave ; « waterlogged » ajoute de l'eau (voir `propriete`).
    pub liquide: Option<Liquide>,
    /// Aucun modèle trouvé : affiché en damier rose et noir.
    pub inconnu: bool,
}

impl BlocCuit {
    pub fn propriete(&self, cle: &str) -> Option<&str> {
        self.proprietes
            .iter()
            .find(|(k, _)| k == cle)
            .map(|(_, v)| v.as_str())
    }

    /// Eau dedans : bloc d'eau, ou bloc « waterlogged ».
    pub fn a_de_l_eau(&self) -> bool {
        self.liquide == Some(Liquide::Eau) || self.propriete("waterlogged") == Some("true")
    }

    /// Le modèle d'un bloc fait d'une seule partie sans variante.
    pub fn modele_unique(&self) -> Option<&ModeleCuit> {
        match self.parties.as_slice() {
            [p] if p.len() == 1 => Some(&p[0].modele),
            _ => None,
        }
    }
}

/// Un fichier vanilla (chemin relatif à assets/minecraft/).
pub enum Fichier<'a> {
    Present(&'a Value),
    /// Introuvable partout.
    Absent,
    /// Pas encore téléchargé (ou pas encore demandé).
    EnAttente,
}

pub trait SourceFichiers {
    fn fichier(&self, chemin: &str) -> Fichier<'_>;
}

/// Sépare « oak_stairs[facing=east,half=top] » en nom et propriétés.
pub fn decomposer_etat(etat: &str) -> (String, Vec<(String, String)>) {
    let Some(c) = etat.find('[') else {
        return (etat.to_owned(), Vec::new());
    };
    let corps = etat[c + 1..].trim_end_matches(']');
    let proprietes = corps
        .split(',')
        .filter_map(|p| p.split_once('=').map(|(k, v)| (k.to_owned(), v.to_owned())))
        .collect();
    (etat[..c].to_owned(), proprietes)
}

/// « minecraft:block/stone » → « block/stone ».
pub fn chemin_ressource(r: &str) -> &str {
    r.strip_prefix("minecraft:").unwrap_or(r)
}

/// Cuit chaque état de la palette ; `Err` : fichiers encore à télécharger.
pub fn cuire(etats: &[String], source: &dyn SourceFichiers) -> Result<Vec<BlocCuit>, Vec<String>> {
    let mut c = Cuiseur {
        source,
        modeles: HashMap::new(),
        manquants: BTreeSet::new(),
    };
    let blocs: Vec<BlocCuit> = etats.iter().map(|e| c.cuire_etat(e)).collect();
    if c.manquants.is_empty() {
        Ok(blocs)
    } else {
        Err(c.manquants.into_iter().collect())
    }
}

/// Textures utilisées par des blocs cuits, liquides compris (sans doublon, triées).
pub fn textures(blocs: &[BlocCuit]) -> Vec<String> {
    let mut t = BTreeSet::new();
    for b in blocs {
        for v in b.parties.iter().flatten() {
            for q in &v.modele.quads {
                t.insert(q.texture.clone());
            }
        }
        if b.a_de_l_eau() {
            t.insert(TEXTURE_EAU.to_owned());
        }
        if b.liquide == Some(Liquide::Lave) {
            t.insert(TEXTURE_LAVE.to_owned());
        }
    }
    t.into_iter().collect()
}

struct Cuiseur<'a> {
    source: &'a dyn SourceFichiers,
    modeles: HashMap<String, Option<ModeleResolu>>,
    manquants: BTreeSet<String>,
}

impl Cuiseur<'_> {
    fn json(&mut self, chemin: &str) -> Option<&Value> {
        match self.source.fichier(chemin) {
            Fichier::Present(v) => Some(v),
            Fichier::Absent => None,
            Fichier::EnAttente => {
                self.manquants.insert(chemin.to_owned());
                None
            }
        }
    }

    fn cuire_etat(&mut self, etat: &str) -> BlocCuit {
        let (nom, proprietes) = decomposer_etat(etat);
        let mut b = BlocCuit {
            etat: etat.to_owned(),
            nom,
            proprietes,
            parties: Vec::new(),
            liquide: None,
            inconnu: false,
        };
        match b.nom.as_str() {
            "air" => return b,
            "water" | "bubble_column" => {
                b.liquide = Some(Liquide::Eau);
                return b;
            }
            "lava" => {
                b.liquide = Some(Liquide::Lave);
                return b;
            }
            _ => {}
        }
        if let Some(s) = modele_de_secours(&b.nom) {
            b.parties = vec![vec![Variante {
                modele: cuire_modele(&s, 0, 0, false),
                poids: 1.0,
            }]];
            return b;
        }
        let chemin = format!("blockstates/{}.json", b.nom);
        let Some(json) = self.json(&chemin).cloned() else {
            b.inconnu = true;
            return b;
        };
        let mut appliques: Vec<&Value> = Vec::new();
        if let Some(Value::Object(variantes)) = json.get("variants") {
            // Première variante dont toutes les conditions « clé=valeur » sont remplies
            let choisie = variantes.iter().find(|(cle, _)| {
                cle.is_empty()
                    || cle.split(',').all(|c| {
                        let (k, v) = c.split_once('=').unwrap_or((c, ""));
                        b.propriete(k) == Some(v)
                    })
            });
            appliques.extend(choisie.map(|(_, v)| v));
        }
        if let Some(Value::Array(parties)) = json.get("multipart") {
            for p in parties {
                if p.get("when").is_none_or(|w| condition_remplie(w, &b)) {
                    appliques.extend(p.get("apply"));
                }
            }
        }
        for applique in appliques {
            let liste: Vec<&Value> = match applique {
                Value::Array(l) => l.iter().collect(),
                v => vec![v],
            };
            let mut variantes = Vec::new();
            for a in liste {
                let Some(modele) = a.get("model").and_then(Value::as_str) else {
                    continue;
                };
                let Some(r) = self.resoudre(chemin_ressource(modele)) else {
                    continue;
                };
                let entier = |k: &str| a.get(k).and_then(Value::as_i64).unwrap_or(0) as i32;
                let uvlock = a.get("uvlock").and_then(Value::as_bool).unwrap_or(false);
                variantes.push(Variante {
                    modele: cuire_modele(&r, entier("x"), entier("y"), uvlock),
                    poids: a.get("weight").and_then(Value::as_f64).unwrap_or(1.0) as f32,
                });
            }
            if !variantes.is_empty() {
                b.parties.push(variantes);
            }
        }
        // Bloc à entité (coffre, panneau…) : pas de géométrie dans son modèle
        let vide = b
            .parties
            .iter()
            .flatten()
            .all(|v| v.modele.quads.is_empty());
        b.inconnu = b.parties.is_empty() || (vide && !sans_geometrie(&b.nom));
        b
    }

    /// Modèle avec ses parents fusionnés (textures du plus proche, éléments du premier
    /// qui en a).
    fn resoudre(&mut self, chemin: &str) -> Option<ModeleResolu> {
        if let Some(m) = self.modeles.get(chemin) {
            return m.clone();
        }
        let json = self.json(&format!("models/{chemin}.json")).cloned()?;
        let parent = json
            .get("parent")
            .and_then(Value::as_str)
            .filter(|p| !p.starts_with("builtin/"))
            .map(|p| chemin_ressource(p).to_owned());
        let attente = self.manquants.len();
        let parent = parent.and_then(|p| self.resoudre(&p));
        // Un ancêtre encore à télécharger : rien à garder pour l'instant
        if self.manquants.len() > attente {
            return None;
        }
        let mut textures = parent
            .as_ref()
            .map(|p| p.textures.clone())
            .unwrap_or_default();
        if let Some(Value::Object(t)) = json.get("textures") {
            textures.extend(t.iter().map(|(k, v)| (k.clone(), v.clone())));
        }
        let elements = match json.get("elements") {
            Some(Value::Array(e)) => e.clone(),
            _ => parent.map(|p| p.elements).unwrap_or_default(),
        };
        let r = ModeleResolu { elements, textures };
        self.modeles.insert(chemin.to_owned(), Some(r.clone()));
        Some(r)
    }
}

/// Blocs sans géométrie par nature : leur modèle vide n'est pas une erreur.
fn sans_geometrie(nom: &str) -> bool {
    matches!(
        nom,
        "air" | "light" | "barrier" | "structure_void" | "moving_piston"
    ) || ["sign", "banner", "head", "skull"]
        .iter()
        .any(|s| nom == *s || nom.ends_with(&format!("_{s}")))
}

fn condition_remplie(c: &Value, b: &BlocCuit) -> bool {
    if let Some(Value::Array(ou)) = c.get("OR") {
        return ou.iter().any(|x| condition_remplie(x, b));
    }
    if let Some(Value::Array(et)) = c.get("AND") {
        return et.iter().all(|x| condition_remplie(x, b));
    }
    let Value::Object(m) = c else { return false };
    m.iter().all(|(cle, attendu)| {
        let attendu = match attendu {
            Value::String(s) => s.clone(),
            v => v.to_string(),
        };
        let valeur = b.propriete(cle).unwrap_or("");
        attendu.split('|').any(|a| a == valeur)
    })
}

/// Blocs dont le modèle vanilla est vide (dessinés comme des entités) : une boîte qui
/// en donne l'allure.
fn modele_de_secours(nom: &str) -> Option<ModeleResolu> {
    let boite = |from: [f32; 3], to: [f32; 3], texture: String| {
        let faces: Map<String, Value> = Direction::TOUTES
            .iter()
            .map(|d| (d.nom().to_owned(), serde_json::json!({ "texture": "#t" })))
            .collect();
        ModeleResolu {
            textures: Map::from_iter([("t".to_owned(), Value::String(texture))]),
            elements: vec![serde_json::json!({ "from": from, "to": to, "faces": faces })],
        }
    };
    let coffre = ([1.0, 0.0, 1.0], [15.0, 14.0, 15.0]);
    match nom {
        "chest" | "trapped_chest" => {
            return Some(boite(coffre.0, coffre.1, "block/oak_planks".into()));
        }
        "ender_chest" => return Some(boite(coffre.0, coffre.1, "block/obsidian".into())),
        _ => {}
    }
    if let Some(couleur) = nom.strip_suffix("_bed") {
        return Some(boite(
            [0.0; 3],
            [16.0, 9.0, 16.0],
            format!("block/{couleur}_wool"),
        ));
    }
    if nom.ends_with("shulker_box") {
        return Some(boite([0.0; 3], [16.0; 3], format!("block/{nom}")));
    }
    None
}
