//! Panneaux de réglage posés dans une map (en plus du panneau « depart ») :
//! `[zone] Village [ennemis] zombie squelette [rayon] 15` crée autour du panneau une
//! zone carrée (12 blocs de demi-côté par défaut) peuplée de ces ennemis seulement.
//! Les mots-clés peuvent être sur des lignes différentes du panneau ; `[zone]` et
//! `[rayon]` sont facultatifs. Un ennemi garde le moment d'arrivée et le poids qu'il a
//! dans la population de la map ; absent de celle-ci, il vient dès le début, poids 1.
//! `[boss] wither` et `[mini-boss] ravageur` remplacent les boss de la map, dans l'ordre
//! des noms (puis des panneaux) ; les boss arrivent au premier panneau `[boss]`.

use bs_donnees::Donnees;
use bs_donnees::arenes::{ChoixEnnemi, Point, ZoneEnnemis};

use super::schematique::Schematique;
use super::zone::Zone;

/// Demi-côté (blocs) d'une zone posée par panneau, sans `[rayon]`.
const RAYON_DEFAUT: f32 = 12.0;

/// Mots qui suivent `[cle]` sur le panneau, jusqu'au mot-clé suivant.
fn apres<'a>(texte: &'a str, cle: &str) -> Option<&'a str> {
    let i = texte.find(cle)? + cle.len();
    let reste = &texte[i..];
    Some(reste[..reste.find('[').unwrap_or(reste.len())].trim())
}

/// Zones d'ennemis des panneaux `[ennemis]` de la schématique, en coordonnées du jeu.
/// Les noms d'ennemis inconnus sont ignorés (et signalés dans `ignores`).
pub fn zones_des_panneaux(
    d: &Donnees,
    s: &Schematique,
    zone: &Zone,
    population: &[ChoixEnnemi],
    ignores: &mut Vec<String>,
) -> Vec<ZoneEnnemis> {
    let (dl, dp) = (zone.largeur as f32 / 2.0, zone.profondeur as f32 / 2.0);
    let mut zones = Vec::new();
    for p in &s.panneaux {
        let texte = p.texte.to_lowercase();
        let Some(liste) = apres(&texte, "[ennemis]") else {
            continue;
        };
        let ennemis: Vec<ChoixEnnemi> = liste
            .split([' ', ','])
            .filter(|m| !m.is_empty())
            .filter_map(|id| {
                if !d.ennemis.contains_key(id) {
                    ignores.push(format!("[ennemis] {id}"));
                    return None;
                }
                let connu = population.iter().find(|e| e.ennemi == id);
                Some(connu.cloned().unwrap_or(ChoixEnnemi {
                    ennemi: id.to_owned(),
                    depuis: 0.0,
                    poids: 1.0,
                }))
            })
            .collect();
        if ennemis.is_empty() {
            continue;
        }
        let rayon = apres(&texte, "[rayon]")
            .and_then(|r| r.split(' ').next()?.parse::<f32>().ok())
            .filter(|r| *r > 0.0)
            .unwrap_or(RAYON_DEFAUT);
        let nom = apres(&p.texte, "[zone]")
            .or_else(|| apres(&p.texte, "[Zone]"))
            .filter(|n| !n.is_empty())
            .map_or_else(|| format!("Zone {}", zones.len() + 1), str::to_owned);
        let x = (p.x - zone.x0) as f32 - dl + 0.5;
        let z = (p.z - zone.z0) as f32 - dp + 0.5;
        zones.push(ZoneEnnemis {
            nom,
            rect: [x - rayon, z - rayon, x + rayon, z + rayon],
            ennemis,
        });
    }
    zones
}

/// Boss des panneaux : listes (vides s'il n'y en a pas) et lieu d'arrivée.
pub struct BossDesPanneaux {
    pub boss: Vec<String>,
    pub mini: Vec<String>,
    /// Position du premier panneau `[boss]`, en coordonnées du jeu.
    pub arrivee: Option<Point>,
}

/// Boss et mini-boss des panneaux `[boss]` et `[mini-boss]` ; le premier panneau `[boss]`
/// (même sans nom) fixe le lieu d'arrivée des boss.
/// Les noms inconnus, ou qui ne sont pas des boss de cette sorte, vont dans `ignores`.
pub fn boss_des_panneaux(
    d: &Donnees,
    s: &Schematique,
    zone: &Zone,
    ignores: &mut Vec<String>,
) -> BossDesPanneaux {
    let (dl, dp) = (zone.largeur as f32 / 2.0, zone.profondeur as f32 / 2.0);
    let (mut boss, mut mini, mut arrivee) = (Vec::new(), Vec::new(), None);
    for p in &s.panneaux {
        let texte = p.texte.to_lowercase();
        if arrivee.is_none() && texte.contains("[boss]") {
            arrivee = Some(Point {
                x: (p.x - zone.x0) as f32 - dl + 0.5,
                z: (p.z - zone.z0) as f32 - dp + 0.5,
            });
        }
        let listes = [
            ("[boss]", apres(&texte, "[boss]"), false),
            ("[mini-boss]", apres(&texte, "[mini-boss]"), true),
            ("[miniboss]", apres(&texte, "[miniboss]"), true),
        ];
        for (cle, liste, est_mini) in listes {
            for id in liste.into_iter().flat_map(|l| l.split([' ', ','])) {
                if id.is_empty() {
                    continue;
                }
                let convient = d.ennemis.get(id).is_some_and(|e| {
                    if est_mini {
                        e.mini_boss
                    } else {
                        e.boss.is_some()
                    }
                });
                if !convient {
                    ignores.push(format!("{cle} {id}"));
                } else if est_mini {
                    mini.push(id.to_owned());
                } else {
                    boss.push(id.to_owned());
                }
            }
        }
    }
    BossDesPanneaux {
        boss,
        mini,
        arrivee,
    }
}

#[cfg(test)]
mod essais {
    use super::apres;

    #[test]
    fn mots_cles() {
        let t = "[zone] village [ennemis] zombie squelette [rayon] 8";
        assert_eq!(apres(t, "[zone]"), Some("village"));
        assert_eq!(apres(t, "[ennemis]"), Some("zombie squelette"));
        assert_eq!(apres(t, "[rayon]"), Some("8"));
        assert_eq!(apres(t, "[boss]"), None);
    }
}
