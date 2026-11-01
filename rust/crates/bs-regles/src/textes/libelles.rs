//! Libellés et mise en forme des nombres des cartes (port des tables de
//! src/progression/ameliorations.ts) : « Dégâts : 10 → 15 », « Vitesse : +20 % ».

use bs_donnees::armes::{Mecanique, TypeEtat};
use bs_donnees::{Modifs, StatArme, StatsArme};

/// Toutes les statistiques d'arme, dans l'ordre des cartes.
pub const STATS: [StatArme; 11] = [
    StatArme::Degats,
    StatArme::Cadence,
    StatArme::Nombre,
    StatArme::Portee,
    StatArme::Vitesse,
    StatArme::Zone,
    StatArme::Duree,
    StatArme::Perforation,
    StatArme::Recul,
    StatArme::Ralenti,
    StatArme::Etourdi,
];

/// Nom et unité d'une statistique d'arme.
pub fn libelle_stat(s: StatArme) -> (&'static str, &'static str) {
    match s {
        StatArme::Degats => ("Dégâts", ""),
        StatArme::Cadence => ("Délai", " s"),
        StatArme::Nombre => ("Nombre", ""),
        StatArme::Portee => ("Portée", " blocs"),
        StatArme::Vitesse => ("Vitesse", ""),
        StatArme::Zone => ("Zone", " blocs"),
        StatArme::Duree => ("Durée", " s"),
        StatArme::Perforation => ("Perforation", ""),
        StatArme::Recul => ("Recul", ""),
        StatArme::Ralenti => ("Ralentissement", ""),
        StatArme::Etourdi => ("Étourdissement", " s"),
    }
}

pub fn valeur_stat(s: &StatsArme, cle: StatArme) -> f32 {
    match cle {
        StatArme::Degats => s.degats,
        StatArme::Cadence => s.cadence,
        StatArme::Nombre => s.nombre,
        StatArme::Portee => s.portee,
        StatArme::Vitesse => s.vitesse,
        StatArme::Zone => s.zone,
        StatArme::Duree => s.duree,
        StatArme::Perforation => s.perforation,
        StatArme::Recul => s.recul,
        StatArme::Ralenti => s.ralenti,
        StatArme::Etourdi => s.etourdi,
    }
}

/// Ligne d'une carte d'éveil pour l'état qu'il inflige.
pub fn libelle_etat(t: TypeEtat) -> &'static str {
    match t {
        TypeEtat::Brulure => "Enflamme à chaque coup",
        TypeEtat::Poison => "Empoisonne à chaque coup",
        TypeEtat::Ralenti => "Ralentit à chaque coup",
        TypeEtat::Etourdi => "Étourdit à chaque coup",
    }
}

/// Statistiques utiles à chaque mécanique (les autres ne sont pas affichées).
pub fn stats_utiles(m: &Mecanique) -> &'static [StatArme] {
    use StatArme::*;
    match m {
        Mecanique::Balayage { .. } => &[Degats, Cadence, Zone, Recul],
        Mecanique::Tir { .. } => &[Degats, Cadence, Nombre, Portee, Perforation],
        Mecanique::Retour { .. } => &[Degats, Cadence, Nombre, Portee],
        Mecanique::Orbite { .. } => &[Degats, Nombre, Zone, Vitesse],
        Mecanique::Explosif { .. } | Mecanique::Foudre { .. } => &[Degats, Cadence, Nombre, Zone],
        Mecanique::Invocation { .. } | Mecanique::Orbes { .. } => &[Degats, Cadence, Nombre, Duree],
        Mecanique::Fissure { .. } => &[Degats, Cadence, Nombre, Portee, Ralenti],
        Mecanique::Chute { .. } => &[Degats, Cadence, Nombre, Zone, Ralenti],
        Mecanique::Tourbillon { .. } => &[Degats, Cadence, Zone, Duree],
        Mecanique::Heurt { .. } => &[Degats, Cadence, Recul, Etourdi],
        Mecanique::Tornade { .. } => &[Degats, Cadence, Nombre, Duree, Zone],
        Mecanique::Nova { .. } => &[Degats, Cadence, Zone, Ralenti],
        Mecanique::Meteore { .. } | Mecanique::Pluie { .. } => {
            &[Degats, Cadence, Nombre, Zone, Duree]
        }
        Mecanique::Ricochet { .. } => &[Degats, Cadence, Nombre, Perforation],
        Mecanique::Piege { .. } => &[Degats, Cadence, Nombre, Zone, Duree, Ralenti],
    }
}

pub fn formater_stat(cle: StatArme, valeur: f32) -> String {
    if cle == StatArme::Ralenti {
        return format!("{} %", arrondi_js(valeur * 100.0));
    }
    format!("{}{}", nombre(valeur), libelle_stat(cle).1)
}

/// Façon d'afficher un modificateur.
#[derive(Clone, Copy, PartialEq)]
enum Forme {
    /// Multiplicateur (1,2 : +20 %).
    Pourcentage,
    /// Part (0,2 : +20 %).
    Part,
    /// Ajout, avec une unité.
    Ajout(&'static str),
    Armure,
    Critique,
    DegatsCritiques,
}

type Lecteur = fn(&Modifs) -> f32;

/// Modificateurs dans l'ordre des cartes : nom, lecture, forme.
const MODIFICATEURS: [(&str, Lecteur, Forme); 40] = {
    use Forme::*;
    [
        ("Dégâts des armes", |m| m.degats, Pourcentage),
        ("Délai des armes", |m| m.cadence, Pourcentage),
        ("Zone des armes", |m| m.zone, Pourcentage),
        ("Portée des armes", |m| m.portee, Pourcentage),
        ("Projectiles en plus", |m| m.projectiles, Ajout("")),
        ("Alliés par invocation", |m| m.allies, Ajout("")),
        ("Vitesse", |m| m.vitesse, Pourcentage),
        ("Santé maximale", |m| m.pv_max, Ajout(" PV")),
        ("Dégâts reçus", |m| m.armure, Armure),
        ("Régénération", |m| m.regeneration, Ajout(" PV/s")),
        ("Vies en plus", |m| m.vies, Ajout("")),
        ("Rayon de collecte", |m| m.collecte, Pourcentage),
        ("Coups critiques", |m| m.critique, Critique),
        ("Dégâts critiques", |m| m.degats_critiques, DegatsCritiques),
        ("Dégâts de mêlée", |m| m.degats_melee, Part),
        ("Rayon de mêlée", |m| m.zone_melee, Part),
        ("Dégâts des projectiles", |m| m.degats_projectiles, Part),
        ("Vitesse des projectiles", |m| m.vitesse_projectiles, Part),
        ("Dégâts de feu", |m| m.degats_feu, Part),
        ("Délai des armes de feu", |m| m.cadence_feu, Ajout("")),
        ("Dégâts des alliés", |m| m.degats_allies, Part),
        ("Vitesse des alliés", |m| m.vitesse_allies, Part),
        ("PV des alliés", |m| m.pv_allies, Part),
        ("Durée des alliés", |m| m.duree_allies, Ajout(" s")),
        ("Recul", |m| m.recul, Pourcentage),
        ("Dégâts de brûlure", |m| m.degats_brulure, Part),
        ("Durée des brûlures", |m| m.duree_brulure, Part),
        ("Cumuls de brûlure", |m| m.cumuls_brulure, Ajout("")),
        ("Dégâts du poison", |m| m.degats_poison, Part),
        ("Durée des ralentissements", |m| m.duree_ralenti, Part),
        ("Dégâts contre les ralentis", |m| m.degats_ralentis, Part),
        ("Régénération", |m| m.regeneration_part, Ajout("")),
        ("Soins des pommes", |m| m.soins_objets, Part),
        (
            "Dégâts contre les morts-vivants",
            |m| m.degats_morts_vivants,
            Part,
        ),
        (
            "Dégâts contre les arthropodes",
            |m| m.degats_arthropodes,
            Part,
        ),
        ("Dégâts contre le Nether", |m| m.degats_nether, Part),
        ("Dégâts reçus", |m| m.degats_recus, Part),
        ("Émeraudes ramassées", |m| m.emeraudes, Part),
        ("Coups qui enflamment", |m| m.chance_brulure, Part),
        ("Dégâts du magma", |m| m.degats_sol, Part),
    ]
};

fn formater_modificateur(forme: Forme, v: f32) -> String {
    let signe = |p: i64| format!("{}{} %", if p >= 0 { '+' } else { '−' }, p.abs());
    match forme {
        Forme::Armure if v == 0.0 => "normaux".into(),
        Forme::Armure => format!("−{} par coup", nombre(v)),
        Forme::Critique => format!("{} % de chance", arrondi_js(v * 100.0)),
        Forme::DegatsCritiques => format!("×{}", nombre(v)),
        Forme::Part => signe(arrondi_js(v * 100.0)),
        Forme::Pourcentage => signe(arrondi_js((v - 1.0) * 100.0)),
        Forme::Ajout(unite) => format!("+{}{unite}", nombre(v)),
    }
}

/// Lignes « Nom : avant → après » de chaque modificateur qui change.
pub fn lignes_modificateurs(avant: &Modifs, apres: &Modifs, sortie: &mut Vec<String>) {
    for (nom, lire, forme) in MODIFICATEURS {
        let (a, b) = (lire(avant), lire(apres));
        if (b - a).abs() > 1e-6 {
            sortie.push(format!(
                "{nom} : {} → {}",
                formater_modificateur(forme, a),
                formater_modificateur(forme, b)
            ));
        }
    }
}

/// Arrondi comme `Math.round` (moitiés vers +∞).
fn arrondi_js(v: f32) -> i64 {
    (f64::from(v) + 0.5).floor() as i64
}

/// Nombre à la française, deux décimales au plus (`Intl.NumberFormat('fr-FR')`).
pub fn nombre(v: f32) -> String {
    let centiemes = (f64::from(v) * 100.0).round() as i64;
    let entier = centiemes.abs() / 100;
    let decimales = centiemes.abs() % 100;
    let mut s = String::new();
    if centiemes < 0 {
        s.push('-');
    }
    let chiffres = entier.to_string();
    for (i, c) in chiffres.chars().enumerate() {
        if i > 0 && (chiffres.len() - i).is_multiple_of(3) {
            s.push(' ');
        }
        s.push(c);
    }
    if decimales > 0 {
        s.push(',');
        if decimales % 10 == 0 {
            s.push_str(&(decimales / 10).to_string());
        } else {
            s.push_str(&format!("{decimales:02}"));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nombres_a_la_francaise() {
        assert_eq!(nombre(10.0), "10");
        assert_eq!(nombre(1.5), "1,5");
        assert_eq!(nombre(0.333), "0,33");
        assert_eq!(nombre(0.05), "0,05");
        assert_eq!(nombre(12345.0), "12 345");
        assert_eq!(formater_stat(StatArme::Ralenti, 0.25), "25 %");
        assert_eq!(formater_modificateur(Forme::Pourcentage, 1.2), "+20 %");
        assert_eq!(formater_modificateur(Forme::Part, -0.1), "−10 %");
    }
}
