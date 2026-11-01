//! Collection (gardée d'une partie à l'autre, port de src/progression/collection.ts) :
//! pièces d'armure obtenues, pièce portée par emplacement, objets de coffre déjà trouvés.
//! Clé à part (`collection`), même JSON que la version TypeScript ; chaque changement
//! est enregistré aussitôt. Lecture tolérante : ce qui est illisible est ignoré.

use bs_donnees::Donnees;
use bs_donnees::Modifs;
use bs_donnees::arenes::Monde;
use bs_donnees::equipement::Piece;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::stockage::Stockage;

const VERSION: u32 = 1;
/// Les emplacements d'armure, dans l'ordre du jeu.
pub const PIECES: [Piece; 7] = [
    Piece::Casque,
    Piece::Plastron,
    Piece::Jambieres,
    Piece::Bottes,
    Piece::Bouclier,
    Piece::Cape,
    Piece::Talisman,
];
pub const MONDES: [Monde; 4] = [
    Monde::Prairie,
    Monde::Mine,
    Monde::Nether,
    Monde::Champignons,
];

/// Où en est un monde de la Collection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AvanceeMonde {
    pub pieces: usize,
    pub objets: usize,
    pub total_objets: usize,
    /// 0 à 1.
    pub part: f32,
}

#[derive(Debug, Clone)]
pub struct Collection {
    stockage: Stockage,
    cle: String,
    /// Pièces obtenues, « monde:pièce ».
    pieces: Vec<String>,
    /// Monde de la pièce portée à chaque emplacement (ordre de `PIECES`).
    portees: [Option<Monde>; 7],
    /// Objets de coffre déjà trouvés (identifiants).
    decouverts: Vec<String>,
}

/// Identifiant JSON d'une valeur sérialisée en texte (« prairie », « casque »).
fn id<T: Serialize>(v: T) -> String {
    match serde_json::to_value(v) {
        Ok(Value::String(s)) => s,
        _ => String::new(),
    }
}

fn depuis<T: DeserializeOwned>(s: &str) -> Option<T> {
    serde_json::from_value(Value::String(s.to_owned())).ok()
}

fn place(piece: Piece) -> usize {
    PIECES.iter().position(|&p| p == piece).unwrap_or(0)
}

fn textes(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|l| {
            l.iter()
                .filter_map(|x| x.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

impl Collection {
    pub fn charger(stockage: Stockage, cle: String) -> Self {
        let v: Value = stockage.lire(&cle).unwrap_or(Value::Null);
        let mut portees = [None; 7];
        for (k, piece) in PIECES.iter().enumerate() {
            portees[k] = v["portees"][id(piece)].as_str().and_then(depuis);
        }
        let mut c = Self {
            stockage,
            cle,
            pieces: textes(&v["pieces"]),
            portees,
            decouverts: textes(&v["decouverts"]),
        };
        // Une pièce portée doit être possédée
        for (k, piece) in PIECES.iter().enumerate() {
            if let Some(m) = c.portees[k]
                && !c.possede(m, *piece)
            {
                c.portees[k] = None;
            }
        }
        c
    }

    pub fn possede(&self, monde: Monde, piece: Piece) -> bool {
        let cle = format!("{}:{}", id(monde), id(piece));
        self.pieces.contains(&cle)
    }

    /// Pièces obtenues de ce monde.
    pub fn pieces_du_monde(&self, monde: Monde) -> Vec<Piece> {
        PIECES
            .into_iter()
            .filter(|&p| self.possede(monde, p))
            .collect()
    }

    /// Monde de la pièce portée à cet emplacement (None : rien).
    /// Monde de la pièce portée à chaque emplacement (ordre de `PIECES`).
    pub fn portees(&self) -> [Option<Monde>; 7] {
        self.portees
    }

    pub fn portee(&self, piece: Piece) -> Option<Monde> {
        self.portees[place(piece)]
    }

    /// Porte la pièce de ce monde (None : retire). Sans effet si elle n'est pas possédée.
    pub fn porter(&mut self, piece: Piece, monde: Option<Monde>) {
        if monde.is_some_and(|m| !self.possede(m, piece)) {
            return;
        }
        self.portees[place(piece)] = monde;
        self.enregistrer();
    }

    /// Pièce gagnée (butin d'un boss) : gardée aussitôt, portée si l'emplacement est libre.
    pub fn ajouter(&mut self, monde: Monde, piece: Piece) {
        if self.possede(monde, piece) {
            return;
        }
        self.pieces.push(format!("{}:{}", id(monde), id(piece)));
        let k = place(piece);
        if self.portees[k].is_none() {
            self.portees[k] = Some(monde);
        }
        self.enregistrer();
    }

    pub fn decouvert(&self, objet: &str) -> bool {
        self.decouverts.iter().any(|o| o == objet)
    }

    /// Premier objet de ce type trouvé : inscrit dans la Collection.
    pub fn decouvrir(&mut self, objet: &str) {
        if self.decouvert(objet) {
            return;
        }
        self.decouverts.push(objet.to_owned());
        self.enregistrer();
    }

    /// Bonus des pièces portées, plus celui de l'ensemble si les 7 sont du même monde.
    pub fn bonus(&self, d: &Donnees) -> Modifs {
        let mut m = Modifs::default();
        for piece in PIECES {
            if let Some(monde) = self.portee(piece)
                && let Some(p) = d
                    .armures
                    .armures
                    .get(&monde)
                    .and_then(|a| a.pieces.get(&piece))
            {
                m.cumuler(&p.modif);
            }
        }
        if let Some(monde) = self.ensemble_complet()
            && let Some(a) = d.armures.armures.get(&monde)
        {
            m.cumuler(&a.ensemble.modif);
        }
        m
    }

    /// Monde dont toutes les pièces sont portées.
    pub fn ensemble_complet(&self) -> Option<Monde> {
        let premier = self.portees[0]?;
        self.portees
            .iter()
            .all(|&m| m == Some(premier))
            .then_some(premier)
    }

    /// Pièces et objets trouvés dans ce monde (objets de tous les mondes non comptés).
    pub fn avancee(&self, d: &Donnees, monde: Monde) -> AvanceeMonde {
        let pieces = self.pieces_du_monde(monde).len();
        let ids: Vec<&str> = d
            .equipement
            .objets
            .iter()
            .filter(|(_, o)| o.monde == Some(monde))
            .map(|(id, _)| id.as_str())
            .collect();
        let objets = ids.iter().filter(|id| self.decouvert(id)).count();
        AvanceeMonde {
            pieces,
            objets,
            total_objets: ids.len(),
            part: (pieces + objets) as f32 / (PIECES.len() + ids.len()) as f32,
        }
    }

    fn enregistrer(&self) {
        let mut portees = Map::new();
        for (k, piece) in PIECES.iter().enumerate() {
            if let Some(m) = self.portees[k] {
                portees.insert(id(piece), Value::from(id(m)));
            }
        }
        let mut o = Map::new();
        o.insert("version".into(), Value::from(VERSION));
        o.insert("pieces".into(), Value::from(self.pieces.clone()));
        o.insert("portees".into(), Value::Object(portees));
        o.insert("decouverts".into(), Value::from(self.decouverts.clone()));
        self.stockage.ecrire(&self.cle, &Value::Object(o));
    }
}
