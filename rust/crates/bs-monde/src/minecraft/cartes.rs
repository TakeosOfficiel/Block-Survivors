//! Arènes construites dans Minecraft (src/donnees/arenesImportees.ts et
//! src/minecraft/importation.ts) : chaque schématique du dossier cartes/ devient une
//! arène. Le .json du même nom, facultatif, la règle (nom, arène de base dont viennent
//! ennemis, boss et ambiance, ciel, brouillard, lumière…) ; la taille, le sol et le
//! décor viennent de la schématique.

use bs_donnees::Donnees;
use bs_donnees::Intervalle;
use bs_donnees::arenes::{Arene, ChoixEnnemi, Monde, Point, ZoneEnnemis};
use serde::Deserialize;

use super::modeles::BlocCuit;
use super::panneaux::{boss_des_panneaux, zones_des_panneaux};
use super::proprietes::{Physique, physique_de};
use super::schematique::Schematique;
use super::zone::{Zone, construire_carte, trouver_zone};
use crate::carte::CarteMonde;

/// Une map embarquée (fichiers lus à la compilation, voir build.rs).
#[derive(Debug, Clone, Copy)]
pub struct Carte {
    pub id: &'static str,
    pub fichier: &'static [u8],
    pub reglages: Option<&'static str>,
}

/// Toutes les maps du dossier cartes/, par ordre alphabétique.
pub fn cartes() -> &'static [Carte] {
    include!(concat!(env!("OUT_DIR"), "/cartes.rs"))
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReglagesCarte {
    nom: Option<String>,
    description: Option<String>,
    icone: Option<String>,
    base: Option<String>,
    monde: Option<Monde>,
    ciel: Option<Couleur>,
    brouillard: Option<Intervalle>,
    lumiere: Option<ReglagesLumiere>,
    boss: Option<Vec<String>>,
    mini_boss: Option<Vec<String>>,
    biome: Option<String>,
    ennemis: Option<Vec<ChoixEnnemi>>,
    zones: Option<Vec<ZoneEnnemis>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReglagesLumiere {
    ambiante: Option<f32>,
    soleil: Option<f32>,
    couleur_ambiante: Option<Couleur>,
    couleur_soleil: Option<Couleur>,
}

/// « #78a7ff » ou nombre.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Couleur {
    Nombre(u32),
    Texte(String),
}

impl Couleur {
    fn valeur(&self) -> Option<u32> {
        match self {
            Couleur::Nombre(n) => Some(*n),
            Couleur::Texte(t) => u32::from_str_radix(t.trim_start_matches('#'), 16).ok(),
        }
    }
}

fn couleur(c: Option<&Couleur>, defaut: u32) -> u32 {
    c.and_then(Couleur::valeur).unwrap_or(defaut)
}

/// « map_mushroom » → « Map mushroom ».
fn nom_lisible(id: &str) -> String {
    let texte = id.replace(['_', '-'], " ");
    let texte = texte.trim();
    let mut c = texte.chars();
    c.next()
        .map_or_else(String::new, |p| p.to_uppercase().chain(c).collect())
}

/// Définition de l'arène d'une map, avant lecture de la schématique (taille encore nulle).
pub fn definition(d: &Donnees, carte: &Carte) -> Result<Arene, String> {
    let r: ReglagesCarte = match carte.reglages {
        Some(json) => {
            serde_json::from_str(json).map_err(|e| format!("cartes/{}.json : {e}", carte.id))?
        }
        None => ReglagesCarte::default(),
    };
    let bases = &d.arenes.arenes;
    let base = r
        .base
        .as_ref()
        .and_then(|b| bases.iter().find(|a| &a.id == b))
        .or(bases.first())
        .ok_or("Aucune arène de base")?;
    let l = r.lumiere.unwrap_or_default();
    let mut a = base.clone();
    a.id = carte.id.to_owned();
    a.nom = r.nom.unwrap_or_else(|| nom_lisible(carte.id));
    a.description = r
        .description
        .unwrap_or_else(|| "Map construite dans Minecraft.".into());
    a.icone = r.icone.unwrap_or_else(|| "block/grass_block_side".into());
    // Connues une fois la schématique lue
    a.largeur = 0.0;
    a.profondeur = 0.0;
    a.hauteur_mur = 0.0;
    a.sol.clear();
    a.taches.clear();
    a.murs.clear();
    a.decor.clear();
    a.zone_depart = 0.0;
    a.exterieur = None;
    a.depart = None;
    a.ciel = couleur(r.ciel.as_ref(), base.ciel);
    a.brouillard = r.brouillard.unwrap_or(base.brouillard);
    a.lumiere.ambiante = l.ambiante.unwrap_or(base.lumiere.ambiante);
    a.lumiere.soleil = l.soleil.unwrap_or(base.lumiere.soleil);
    a.lumiere.couleur_ambiante =
        couleur(l.couleur_ambiante.as_ref(), base.lumiere.couleur_ambiante);
    a.lumiere.couleur_soleil = couleur(l.couleur_soleil.as_ref(), base.lumiere.couleur_soleil);
    a.monde = r.monde.unwrap_or(base.monde);
    a.boss = r.boss.unwrap_or_else(|| base.boss.clone());
    a.mini_boss = r.mini_boss.unwrap_or_else(|| base.mini_boss.clone());
    a.biome = r.biome;
    a.ennemis = r.ennemis.unwrap_or_else(|| base.ennemis.clone());
    // Les zones de l'arène de base n'ont pas de sens ici : seulement celles du réglage
    a.zones = r.zones.unwrap_or_default();
    Ok(a)
}

/// Map lue : son arène (taille, départ), sa carte de collision et sa zone jouable.
#[derive(Debug, Clone)]
pub struct AreneImportee {
    pub arene: Arene,
    pub carte: CarteMonde,
    pub zone: Zone,
    /// Point de la schématique placé en (0, 0, 0) dans le jeu.
    pub origine: [f32; 3],
    /// Blocs sans modèle connu (affichés en damier rose et noir), puis ennemis inconnus
    /// des panneaux `[ennemis]`.
    pub inconnus: Vec<String>,
}

/// Zone jouable, carte de collision et arène complète d'une schématique aux blocs cuits
/// (zones d'ennemis des panneaux `[ennemis]` ajoutées à celles du `.json`, boss des
/// panneaux `[boss]` et `[mini-boss]` à la place des siens).
pub fn importer(
    d: &Donnees,
    definition: Arene,
    s: &Schematique,
    blocs: &[BlocCuit],
) -> Result<AreneImportee, String> {
    let physiques: Vec<Physique> = blocs.iter().map(physique_de).collect();
    let zone = trouver_zone(s, &physiques)?;
    let carte = construire_carte(s, &physiques, &zone);
    let mut arene = definition;
    arene.largeur = zone.largeur as f32;
    arene.profondeur = zone.profondeur as f32;
    let (dl, dp) = (zone.largeur as f32 / 2.0, zone.profondeur as f32 / 2.0);
    arene.depart = Some(Point {
        x: (zone.depart_x - zone.x0) as f32 - dl + 0.5,
        z: (zone.depart_z - zone.z0) as f32 - dp + 0.5,
    });
    let origine = [zone.x0 as f32 + dl, zone.niveau as f32, zone.z0 as f32 + dp];
    let mut inconnus: Vec<String> = blocs
        .iter()
        .filter(|b| b.inconnu)
        .map(|b| b.etat.clone())
        .collect();
    let zones = zones_des_panneaux(d, s, &zone, &arene.ennemis, &mut inconnus);
    arene.zones.extend(zones);
    let b = boss_des_panneaux(d, s, &zone, &mut inconnus);
    if !b.boss.is_empty() {
        arene.boss = b.boss;
    }
    if !b.mini.is_empty() {
        arene.mini_boss = b.mini;
    }
    arene.arrivee_boss = b.arrivee;
    Ok(AreneImportee {
        arene,
        carte,
        zone,
        origine,
        inconnus,
    })
}
