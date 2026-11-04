//! Tests des maps Minecraft : la map embarquée se lit, la cuisson suit les fichiers
//! vanilla (variantes, parents, rotations), la zone jouable et la carte de collision
//! sur une petite schématique faite à la main.

use std::collections::HashMap;

use serde_json::{Value, json};

use super::cartes::{cartes, definition, importer};
use super::cuisson::Direction;
use super::maillage::{RenduTexture, construire_maillages};
use super::modeles::{Fichier, SourceFichiers, cuire, textures};
use super::schematique::{Panneau, Schematique, lire_schematique};
use super::vent::{VENT_AUCUN, VENT_PLANTE};
use crate::carte::{BLOQUE_MARCHE, BLOQUE_TIR};

/// Fichiers vanilla minimaux : pierre (cube plein), escalier (variantes tournées),
/// herbe courte (croix plate).
struct Fichiers(HashMap<&'static str, Value>);

impl SourceFichiers for Fichiers {
    fn fichier(&self, chemin: &str) -> Fichier<'_> {
        match self.0.get(chemin) {
            Some(v) => Fichier::Present(v),
            None if chemin.contains("inexistant") => Fichier::Absent,
            None => Fichier::EnAttente,
        }
    }
}

fn faces(t: &str) -> Value {
    let f: serde_json::Map<String, Value> = Direction::TOUTES
        .iter()
        .map(|d| {
            (
                d.nom().to_owned(),
                json!({ "texture": t, "cullface": d.nom() }),
            )
        })
        .collect();
    Value::Object(f)
}

fn fichiers() -> Fichiers {
    Fichiers(HashMap::from([
        (
            "blockstates/stone.json",
            json!({ "variants": { "": { "model": "minecraft:block/stone" } } }),
        ),
        (
            "models/block/stone.json",
            json!({ "parent": "block/cube_all", "textures": { "all": "block/stone" } }),
        ),
        (
            "models/block/cube_all.json",
            json!({
                "textures": { "particle": "#all" },
                "elements": [{ "from": [0, 0, 0], "to": [16, 16, 16], "faces": faces("#all") }]
            }),
        ),
        (
            "blockstates/short_grass.json",
            json!({ "variants": { "": { "model": "block/short_grass" } } }),
        ),
        (
            "models/block/short_grass.json",
            json!({
                "textures": { "cross": "block/short_grass" },
                "elements": [{ "from": [0.8, 0, 8], "to": [15.2, 16, 8], "shade": false,
                    "faces": { "north": { "texture": "#cross", "tintindex": 0 }, "south": { "texture": "#cross", "tintindex": 0 } } }]
            }),
        ),
        (
            "blockstates/oak_stairs.json",
            json!({ "variants": {
            "facing=east": { "model": "block/oak_stairs" },
            "facing=south": { "model": "block/oak_stairs", "y": 90, "uvlock": true }
        } }),
        ),
        (
            "models/block/oak_stairs.json",
            json!({
                "textures": { "t": "block/oak_planks" },
                "elements": [
                    { "from": [0, 0, 0], "to": [16, 8, 16], "faces": faces("#t") },
                    { "from": [8, 8, 0], "to": [16, 16, 16], "faces": faces("#t") }
                ]
            }),
        ),
    ]))
}

#[test]
fn la_map_embarquee_se_lit() {
    let c = cartes()
        .iter()
        .find(|c| c.id == "map_mushroom")
        .expect("map_mushroom embarquée");
    let s = lire_schematique(c.fichier).expect("schématique");
    assert!(s.largeur > 10 && s.hauteur > 3 && s.profondeur > 10);
    assert_eq!(s.blocs.len(), s.largeur * s.hauteur * s.profondeur);
    assert_eq!(s.palette[0], "air");
    assert!(s.palette.len() > 5);
    assert!(s.blocs.iter().all(|&b| (b as usize) < s.palette.len()));
    let d = bs_donnees::Donnees::embarquees().expect("données");
    let a = definition(&d, c).expect("réglages");
    assert_eq!(a.nom, "Jardin des champignons");
}

#[test]
fn cuisson_par_etapes_puis_complete() {
    let f = fichiers();
    let etats = [
        "air",
        "stone",
        "oak_stairs[facing=south,half=bottom]",
        "short_grass",
        "inexistant",
    ]
    .map(String::from);
    // Une source vide réclame d'abord les états
    let vide = Fichiers(HashMap::new());
    let manquants = cuire(&etats, &vide).expect_err("rien n'est encore là");
    assert!(manquants.contains(&"blockstates/stone.json".to_owned()));
    let blocs = cuire(&etats, &f).expect("tout est là");
    let pierre = blocs[1].modele_unique().expect("un modèle");
    assert!(pierre.cube_plein);
    assert_eq!(pierre.quads.len(), 6);
    assert!(pierre.quads.iter().all(|q| q.texture == "block/stone"));
    // Escalier tourné d'un quart de tour : la marche haute passe au sud
    let esc = blocs[2].modele_unique().expect("escalier");
    let (_, max) = esc.boite.expect("boîte");
    assert_eq!(max, [16.0; 3]);
    let marche = esc
        .quads
        .iter()
        .filter(|q| q.normale[1] > 0.5)
        .map(|q| q.positions[0][2])
        .fold(0.0, f32::max);
    assert!(marche >= 0.5);
    assert!(blocs[3].modele_unique().expect("herbe").plat);
    assert!(
        blocs[3]
            .modele_unique()
            .expect("herbe")
            .quads
            .iter()
            .all(|q| q.teinte && !q.ombre)
    );
    assert!(blocs[4].inconnu);
    assert_eq!(
        textures(&blocs),
        ["block/oak_planks", "block/short_grass", "block/stone"]
    );
}

/// Une cuvette 7 × 7 : sol de pierre, muret au bord, herbe au milieu, départ au centre.
fn cuvette() -> Schematique {
    let (l, h, p) = (7, 3, 7);
    let mut blocs = vec![0u16; l * h * p];
    for z in 0..p {
        for x in 0..l {
            blocs[z * l + x] = 1; // sol
            if x == 0 || z == 0 || x == l - 1 || z == p - 1 {
                blocs[(p + z) * l + x] = 1;
                blocs[(2 * p + z) * l + x] = 1;
            }
        }
    }
    blocs[(p + 3) * l + 4] = 2; // herbe à côté du départ
    Schematique {
        largeur: l,
        hauteur: h,
        profondeur: p,
        palette: ["air", "stone", "short_grass"].map(String::from).to_vec(),
        blocs,
        panneaux: vec![Panneau {
            x: 3,
            y: 1,
            z: 3,
            texte: "Départ".into(),
        }],
        version_donnees: 0,
    }
}

#[test]
fn zone_jouable_et_carte_de_collision() {
    let s = cuvette();
    let blocs = cuire(&s.palette, &fichiers()).expect("cuisson");
    let d = bs_donnees::Donnees::embarquees().expect("données");
    let c = cartes()[0];
    let a = importer(&d, definition(&d, &c).expect("définition"), &s, &blocs).expect("zone");
    assert_eq!(
        (a.zone.largeur, a.zone.profondeur, a.zone.niveau),
        (5, 5, 1)
    );
    assert_eq!(a.arene.largeur, 5.0);
    let depart = a.arene.depart.as_ref().expect("départ");
    assert_eq!((depart.x, depart.z), (0.0, 0.0));
    assert!(
        a.carte
            .drapeaux
            .iter()
            .all(|f| f & (BLOQUE_MARCHE | BLOQUE_TIR) == 0)
    );
    // Maillage : murs et sol visibles seulement de l'extérieur ou du dessus
    let tex = textures(&blocs);
    let rendus = vec![
        RenduTexture::Opaque,
        RenduTexture::Decoupe,
        RenduTexture::Opaque,
    ];
    let m = construire_maillages(&s, &blocs, &tex, &rendus, a.origine);
    let pierre = m
        .iter()
        .find(|m| tex[m.texture] == "block/stone")
        .expect("pierre");
    assert!(
        pierre.normales.iter().all(|n| n[1] > -0.5),
        "le dessous n'est jamais vu"
    );
    // Vent : l'herbe se balance depuis le pied de son bloc, la pierre ne bouge pas
    let herbe = m
        .iter()
        .find(|m| tex[m.texture] == "block/short_grass")
        .expect("herbe");
    assert_eq!(herbe.vents.len(), herbe.positions.len());
    assert!(herbe.vents.iter().all(|v| v[3] == VENT_PLANTE));
    assert!(
        herbe
            .vents
            .iter()
            .zip(&herbe.positions)
            .all(|(v, p)| p[1] >= v[1] && (p[0] - v[0]).abs() <= 0.5)
    );
    assert!(pierre.vents.iter().all(|v| v[3] == VENT_AUCUN));
}

/// Panneaux `[boss]` et `[mini-boss]` : remplacent les boss de la map, les noms qui ne sont
/// pas des boss de cette sorte sont signalés.
#[test]
fn boss_des_panneaux() {
    let d = bs_donnees::Donnees::embarquees().expect("données");
    let boss = d
        .ennemis
        .iter()
        .find(|(_, e)| e.boss.is_some())
        .expect("boss")
        .0;
    let mini = d.ennemis.iter().find(|(_, e)| e.mini_boss).expect("mini").0;
    let mut s = cuvette();
    s.panneaux.push(Panneau {
        x: 2,
        y: 1,
        z: 2,
        texte: format!("[boss] {boss} zombie [mini-boss] {mini}"),
    });
    let blocs = cuire(&s.palette, &fichiers()).expect("cuisson");
    let c = cartes()[0];
    let a = importer(&d, definition(&d, &c).expect("définition"), &s, &blocs).expect("zone");
    assert_eq!(a.arene.boss, std::slice::from_ref(boss));
    assert_eq!(a.arene.mini_boss, std::slice::from_ref(mini));
    assert!(a.inconnus.iter().any(|i| i == "[boss] zombie"));
    let arrivee = a.arene.arrivee_boss.as_ref().expect("arrivée");
    assert_eq!((arrivee.x, arrivee.z), (-1.0, -1.0));
}
