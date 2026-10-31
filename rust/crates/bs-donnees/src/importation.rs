//! Importation des données exportées par la version TypeScript (JSON de
//! `outils/exporter-donnees.mjs`) : remise en forme (variantes étiquetées,
//! apparence et gains rangés à part), vérification par les types, écriture en RON.
//!
//! Utilisé tant que les deux versions coexistent ; ensuite, les fichiers RON
//! deviennent la seule source.

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::communs::CHAMPS_APPARENCE;
use crate::progression::CHAMPS_GAINS;
use crate::{arenes, armes, blocs, effets, entites, equipement, progression, regles};

const EN_TETE: &str = "// Généré depuis la version TypeScript (src/donnees) par outils/importer-donnees.sh.\n\
// Tant que les deux versions coexistent, modifier src/donnees puis relancer l'import.\n";

/// `{ "type": "tir", … }` (TypeScript) → `{ "tir": { … } }` (variante Rust).
fn etiqueter(v: &mut Value) {
    if let Value::Object(objet) = v
        && let Some(Value::String(genre)) = objet.remove("type")
    {
        let reste = std::mem::take(objet);
        objet.insert(genre, Value::Object(reste));
    }
}

/// Range les champs `champs` d'un objet dans un sous-objet `nom`.
fn ranger(v: &mut Value, nom: &str, champs: &[&str]) {
    if let Value::Object(objet) = v {
        let mut sous = Map::new();
        for &c in champs {
            if let Some(x) = objet.remove(c) {
                sous.insert(c.to_owned(), x);
            }
        }
        if !sous.is_empty() {
            objet.insert(nom.to_owned(), Value::Object(sous));
        }
    }
}

fn valeurs(v: &mut Value) -> Box<dyn Iterator<Item = &mut Value> + '_> {
    match v {
        Value::Object(o) => Box::new(o.values_mut()),
        Value::Array(a) => Box::new(a.iter_mut()),
        _ => Box::new(std::iter::empty()),
    }
}

/// Applique `f` à chaque élément du champ `nom` (objet ou liste), s'il existe.
fn chaque(v: &mut Value, nom: &str, f: impl FnMut(&mut Value)) {
    if let Some(x) = v.get_mut(nom) {
        valeurs(x).for_each(f);
    }
}

fn etiqueter_champ(v: &mut Value, nom: &str) {
    if let Some(x) = v.get_mut(nom) {
        etiqueter(x);
    }
}

/// Remise en forme propre à chaque fichier.
fn remettre_en_forme(nom: &str, v: &mut Value) {
    let apparence = |x: &mut Value| ranger(x, "apparence", &CHAMPS_APPARENCE);
    match nom {
        "ennemis" => valeurs(v).for_each(|e| {
            apparence(e);
            etiqueter_champ(e, "comportement");
        }),
        "personnages" => chaque(v, "personnages", apparence),
        "allies" => chaque(v, "allies", apparence),
        "armes" => valeurs(v).for_each(|a| etiqueter_champ(a, "mecanique")),
        "arenes" => chaque(v, "arenes", |a| {
            chaque(a, "taches", etiqueter);
            chaque(a, "decor", etiqueter);
        }),
        "talents" => {
            chaque(v, "arbres", |paire| {
                valeurs(paire).for_each(|arbre| {
                    chaque(arbre, "noeuds", |noeud| {
                        ranger(noeud, "gains", &CHAMPS_GAINS);
                        chaque(noeud, "options", |o| ranger(o, "gains", &CHAMPS_GAINS));
                    })
                })
            });
            chaque(v, "deblocages", |d| etiqueter_champ(d, "condition"));
        }
        "equipement" => chaque(v, "objets", |o| etiqueter_champ(o, "effet")),
        _ => {}
    }
}

fn en_ron<T: DeserializeOwned + Serialize>(nom: &str, v: Value) -> Result<String, String> {
    let typee: T = serde_json::from_value(v).map_err(|e| format!("{nom}.json : {e}"))?;
    let config = ron::ser::PrettyConfig::new()
        .indentor("  ")
        .struct_names(false)
        // Les structures restent dessinées ligne à ligne
        .compact_arrays(nom != "structures")
        .escape_strings(false)
        .extensions(ron::extensions::Extensions::IMPLICIT_SOME);
    let texte = ron::ser::to_string_pretty(&typee, config).map_err(|e| format!("{nom} : {e}"))?;
    Ok(format!("{EN_TETE}{texte}\n"))
}

/// Convertit le JSON exporté d'une table en texte RON (vérifié par les types).
pub fn convertir(nom: &str, json: &str) -> Result<String, String> {
    let mut v: Value = serde_json::from_str(json).map_err(|e| format!("{nom}.json : {e}"))?;
    remettre_en_forme(nom, &mut v);
    match nom {
        "blocs" => en_ron::<crate::Table<blocs::Bloc>>(nom, v),
        "modeles" => en_ron::<crate::Table<blocs::Modele>>(nom, v),
        "ennemis" => en_ron::<crate::Table<entites::Ennemi>>(nom, v),
        "projectiles" => en_ron::<crate::Table<entites::Projectile>>(nom, v),
        "armes" => en_ron::<crate::Table<armes::Arme>>(nom, v),
        "allies" => en_ron::<entites::Allies>(nom, v),
        "personnages" => en_ron::<entites::Personnages>(nom, v),
        "arenes" => en_ron::<arenes::Arenes>(nom, v),
        "structures" => en_ron::<arenes::Structures>(nom, v),
        "passifs" => en_ron::<progression::Passifs>(nom, v),
        "objets" => en_ron::<crate::Table<entites::Objet>>(nom, v),
        "coffres" => en_ron::<entites::Coffres>(nom, v),
        "equipement" => en_ron::<equipement::Equipement>(nom, v),
        "armures" => en_ron::<equipement::Armures>(nom, v),
        "talents" => en_ron::<progression::Talents>(nom, v),
        "sons" => en_ron::<effets::Sons>(nom, v),
        "particules" => en_ron::<effets::Particules>(nom, v),
        "regles" => en_ron::<regles::Regles>(nom, v),
        "textes" => en_ron::<regles::Textes>(nom, v),
        _ => Err(format!("table inconnue : {nom}")),
    }
}
