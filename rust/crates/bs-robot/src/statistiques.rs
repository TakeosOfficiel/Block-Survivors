//! Statistiques de référence tirées des bilans (mêmes calculs que scripts/robot.mjs) et
//! comparaison avec celles de la version TypeScript, à une tolérance près : les deux
//! simulations ne tombent pas exactement sur les mêmes parties (f32 en Rust, f64 en TS),
//! seules les moyennes doivent concorder.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::Bilan;

/// Écart relatif toléré entre les moyennes, sauf si l'écart type de référence est plus large.
pub const TOLERANCE_RELATIVE: f64 = 0.25;
/// Erreurs types admises : une trentaine de mesures sont comparées à chaque passage ; à
/// deux, l'une d'elles sortait presque toujours par hasard, à trois rarement.
pub const ERREURS_TYPES: f64 = 3.0;

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
    /// Mesure indicative : hors tolérance, elle ne fait pas échouer la comparaison.
    pub indicatif: bool,
}

/// Dans la tolérance : écart relatif d'au plus `TOLERANCE_RELATIVE`, ou d'au plus
/// `ERREURS_TYPES` erreurs types de la différence des moyennes (les parties sont très
/// dispersées).
fn comparer_valeurs(
    mesure: String,
    ts: &Resume,
    n_ts: usize,
    rust: &Resume,
    n_rust: usize,
) -> Ecart {
    let variance =
        ts.ecart.powi(2) / n_ts.max(1) as f64 + rust.ecart.powi(2) / n_rust.max(1) as f64;
    let marge = (ts.moyenne.abs() * TOLERANCE_RELATIVE)
        .max(ERREURS_TYPES * variance.sqrt())
        .max(1.0);
    Ecart {
        ok: (rust.moyenne - ts.moyenne).abs() <= marge,
        mesure,
        ts: ts.moyenne,
        rust: rust.moyenne,
        indicatif: false,
    }
}

/// Puissance de la minute `k` des parties qui l'ont jouée (None : moins de la moitié).
fn puissance_minute(bilans: &[&Bilan], k: usize) -> Option<Resume> {
    let v: Vec<f64> = bilans
        .iter()
        .filter_map(|b| b.puissance_par_minute.get(k).copied())
        .collect();
    (!v.is_empty() && 2 * v.len() >= bilans.len()).then(|| {
        let mut r = resumer(&v);
        r.parties = Some(v.len());
        r
    })
}

/// Compare une référence Rust à celle du jeu TS. Mesures qui comptent : survie, niveau,
/// tués, dégâts subis, puissance des minutes jouées par au moins la moitié des parties
/// des deux côtés (`bilans_ts`, `bilans_rust` : les parties). Les dégâts par seconde des
/// armes obtenues dans au moins deux parties de chaque côté sont indicatifs (`indicatif`) :
/// trop peu de parties, et l'arme arrive à des moments très différents.
pub fn comparer(
    ts: &Reference,
    rust: &Reference,
    bilans_ts: &[&Bilan],
    bilans_rust: &[&Bilan],
) -> Vec<Ecart> {
    let (n, m) = (ts.parties, rust.parties);
    let mut ecarts = vec![
        comparer_valeurs("survie".into(), &ts.survie, n, &rust.survie, m),
        comparer_valeurs("niveau".into(), &ts.niveau, n, &rust.niveau, m),
        comparer_valeurs("tués".into(), &ts.tues, n, &rust.tues, m),
        comparer_valeurs(
            "dégâts subis".into(),
            &ts.degats_subis,
            n,
            &rust.degats_subis,
            m,
        ),
    ];
    for k in 0.. {
        let (Some(a), Some(b)) = (
            puissance_minute(bilans_ts, k),
            puissance_minute(bilans_rust, k),
        ) else {
            break;
        };
        let (pa, pb) = (a.parties.unwrap_or(0), b.parties.unwrap_or(0));
        ecarts.push(comparer_valeurs(
            format!("puissance minute {}", k + 1),
            &a,
            pa,
            &b,
            pb,
        ));
    }
    for (id, a) in &ts.dps_par_arme {
        let Some(b) = rust.dps_par_arme.get(id) else {
            continue;
        };
        let (pa, pb) = (a.parties.unwrap_or(0), b.parties.unwrap_or(0));
        if pa >= 2 && pb >= 2 {
            let mut e = comparer_valeurs(format!("dps {id}"), a, pa, b, pb);
            e.indicatif = true;
            ecarts.push(e);
        }
    }
    ecarts
}
