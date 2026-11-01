//! Statistiques de référence tirées des bilans (mêmes calculs que scripts/robot.mjs) et
//! comparaison avec celles de la version TypeScript, à une tolérance près : les deux
//! simulations ne tombent pas exactement sur les mêmes parties (f32 en Rust, f64 en TS),
//! seules les moyennes doivent concorder.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::Bilan;

/// Écart relatif toléré entre les moyennes, sauf si l'écart type de référence est plus large.
pub const TOLERANCE_RELATIVE: f64 = 0.25;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resume {
    pub moyenne: f64,
    pub ecart: f64,
    pub min: f64,
    pub max: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parties: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    pub parties: usize,
    pub victoires: usize,
    pub survie: Resume,
    pub niveau: Resume,
    pub tues: Resume,
    pub degats_subis: Resume,
    pub puissance_par_minute: Vec<f64>,
    pub dps_par_arme: IndexMap<String, Resume>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FichierReference {
    pub graines: u32,
    pub duree: f64,
    pub invincible: bool,
    pub reference: IndexMap<String, Reference>,
}

fn arrondi(v: f64) -> f64 {
    (v * 10.0 + 0.5).floor() / 10.0
}

pub fn resumer(valeurs: &[f64]) -> Resume {
    let n = valeurs.len() as f64;
    let moyenne = valeurs.iter().sum::<f64>() / n;
    let ecart = (valeurs.iter().map(|v| (v - moyenne).powi(2)).sum::<f64>() / n).sqrt();
    Resume {
        moyenne: arrondi(moyenne),
        ecart: arrondi(ecart),
        min: valeurs.iter().copied().fold(f64::INFINITY, f64::min),
        max: valeurs.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        parties: None,
    }
}

/// Références par « personnage/arène », triées comme dans le script TS.
pub fn references(bilans: &[Bilan]) -> IndexMap<String, Reference> {
    let mut groupes: IndexMap<String, Vec<&Bilan>> = IndexMap::new();
    for b in bilans {
        let cle = format!("{}/{}", b.personnage, b.arene);
        groupes.entry(cle).or_default().push(b);
    }
    groupes.sort_keys();
    let mut sortie = IndexMap::new();
    for (cle, liste) in groupes {
        let mut armes: IndexMap<String, Vec<f64>> = IndexMap::new();
        for b in &liste {
            for (id, a) in &b.armes {
                armes.entry(id.clone()).or_default().push(a.dps);
            }
        }
        armes.sort_keys();
        let minutes = liste
            .iter()
            .map(|b| b.puissance_par_minute.len())
            .max()
            .unwrap_or(0);
        let puissance = (0..minutes)
            .map(|m| {
                let v: Vec<f64> = liste
                    .iter()
                    .filter_map(|b| b.puissance_par_minute.get(m).copied())
                    .collect();
                resumer(&v).moyenne
            })
            .collect();
        let champ = |f: fn(&Bilan) -> f64| resumer(&liste.iter().map(|b| f(b)).collect::<Vec<_>>());
        let reference = Reference {
            parties: liste.len(),
            victoires: liste.iter().filter(|b| b.issue == "victoire").count(),
            survie: champ(|b| b.temps),
            niveau: champ(|b| f64::from(b.niveau)),
            tues: champ(|b| f64::from(b.tues)),
            degats_subis: champ(|b| b.degats_subis),
            puissance_par_minute: puissance,
            dps_par_arme: armes
                .into_iter()
                .map(|(id, v)| {
                    let mut r = resumer(&v);
                    r.parties = Some(v.len());
                    (id, r)
                })
                .collect(),
        };
        sortie.insert(cle, reference);
    }
    sortie
}

/// Une mesure comparée : nom, valeur TS, valeur Rust, dans la tolérance ?
pub struct Ecart {
    pub mesure: String,
    pub ts: f64,
    pub rust: f64,
    pub ok: bool,
}

fn comparer_valeurs(mesure: String, ts: &Resume, rust: f64) -> Ecart {
    let marge = (ts.moyenne.abs() * TOLERANCE_RELATIVE)
        .max(ts.ecart)
        .max(1.0);
    Ecart {
        ok: (rust - ts.moyenne).abs() <= marge,
        mesure,
        ts: ts.moyenne,
        rust,
    }
}

/// Compare une référence Rust à celle du jeu TS : survie, niveau, tués, dégâts subis,
/// puissance moyenne et dégâts par seconde des armes vues des deux côtés.
pub fn comparer(ts: &Reference, rust: &Reference) -> Vec<Ecart> {
    let mut ecarts = vec![
        comparer_valeurs("survie".into(), &ts.survie, rust.survie.moyenne),
        comparer_valeurs("niveau".into(), &ts.niveau, rust.niveau.moyenne),
        comparer_valeurs("tués".into(), &ts.tues, rust.tues.moyenne),
        comparer_valeurs(
            "dégâts subis".into(),
            &ts.degats_subis,
            rust.degats_subis.moyenne,
        ),
    ];
    let moyenne = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
    let p_ts = moyenne(&ts.puissance_par_minute);
    let p_rust = moyenne(&rust.puissance_par_minute);
    let r = resumer(&[p_ts]);
    ecarts.push(comparer_valeurs("puissance".into(), &r, p_rust));
    for (id, a) in &ts.dps_par_arme {
        if let Some(b) = rust.dps_par_arme.get(id) {
            ecarts.push(comparer_valeurs(format!("dps {id}"), a, b.moyenne));
        }
    }
    ecarts
}
