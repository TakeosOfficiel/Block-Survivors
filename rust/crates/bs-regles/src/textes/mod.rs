//! Textes des cartes (port de `decrire` dans src/progression/ameliorations.ts et des
//! cartes de src/progression/equipement.ts) : nom, famille (couleur), icône, étiquette,
//! description et lignes « avant → après ». Calculés seulement à l'ouverture d'un écran
//! de cartes, jamais dans la boucle de jeu.

mod coffres;
pub mod libelles;

use bs_donnees::equipement::Rarete;

use crate::cartes::Amelioration;
use crate::combat::arme::calculer_stats;
use crate::equipement::ChoixEquipement;
use crate::partie::Partie;
use libelles::{
    STATS, formater_stat, libelle_etat, libelle_stat, lignes_modificateurs, stats_utiles,
    valeur_stat,
};

/// Ce que la carte fait une fois choisie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choix {
    Niveau(Amelioration),
    Coffre(ChoixEquipement),
}

/// Famille de la carte : couleur et bandeau (rareté pour les objets de coffre).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Famille {
    NouvelleArme,
    Arme,
    Eveil,
    Passif,
    Exclusif,
    Soin,
    Objet(Rarete),
    Action,
}

impl Famille {
    /// Nom du bandeau en haut de la carte.
    pub fn nom(self) -> &'static str {
        match self {
            Self::Arme => "Amélioration",
            Self::Passif => "Bonus",
            Self::NouvelleArme => "Nouvelle arme",
            Self::Exclusif => "Exclusif",
            Self::Eveil => "Éveil",
            Self::Soin => "Soin",
            Self::Objet(Rarete::Commune) => "Commune",
            Self::Objet(Rarete::Rare) => "Rare",
            Self::Objet(Rarete::Epique) => "Épique",
            Self::Objet(Rarete::Legendaire) => "Légendaire",
            Self::Action => "Choix",
        }
    }
}

/// Icône : texture du jeu, ou icône dessinée d'un objet d'équipement (index).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icone {
    Texture(&'static str),
    Objet(usize),
}

/// Tout ce qu'une carte affiche.
#[derive(Debug, Clone, PartialEq)]
pub struct Carte {
    pub choix: Choix,
    pub famille: Famille,
    pub nom: String,
    pub icone: Icone,
    /// « Nouveau », « Rang 2 → 3 »…
    pub etiquette: String,
    pub description: String,
    /// Statistiques modifiées, ex. « Dégâts : 10 → 15 ».
    pub lignes: Vec<String>,
}

impl Partie {
    /// Une carte de niveau décrite.
    pub fn decrire_amelioration(&self, a: Amelioration) -> Carte {
        match a {
            Amelioration::Arme(index) => self.decrire_arme(index),
            Amelioration::Eveil { arme, variante } => self.decrire_eveil(arme, variante),
            Amelioration::Passif(index) => self.decrire_passif(index),
            Amelioration::Soin => {
                let s = &self.d.regles.progression.soin;
                Carte {
                    choix: Choix::Niveau(a),
                    famille: Famille::Soin,
                    nom: s.nom.clone(),
                    icone: Icone::Texture(&s.icone),
                    etiquette: "Soin".into(),
                    description: s.description.clone(),
                    lignes: vec![format!("Santé : +{} PV", libelles::nombre(s.pv))],
                }
            }
        }
    }

    fn decrire_arme(&self, index: usize) -> Carte {
        let d = self.d;
        let def = &d.armes[index];
        let rang = self.arsenal.rang(index);
        let m = &self.etat.modificateurs;
        let t = &self.talents.armes[index];
        let apres = calculer_stats(d, def, rang + 1, m, t, None);
        let nom_stat = |cle| {
            def.libelles
                .as_ref()
                .and_then(|l| l.get(&cle))
                .map_or(libelle_stat(cle).0, String::as_str)
        };
        let choix = Choix::Niveau(Amelioration::Arme(index));
        if rang == 0 {
            let utiles = def
                .stats_affichees
                .as_deref()
                .unwrap_or_else(|| stats_utiles(&def.mecanique));
            return Carte {
                choix,
                famille: Famille::NouvelleArme,
                nom: def.nom.clone(),
                icone: Icone::Texture(&def.icone),
                etiquette: "Nouvelle arme".into(),
                description: def.description.clone(),
                lignes: utiles
                    .iter()
                    .map(|&c| {
                        let v = formater_stat(c, valeur_stat(&apres, c));
                        format!("{} : {v}", nom_stat(c))
                    })
                    .collect(),
            };
        }
        let avant = calculer_stats(d, def, rang, m, t, None);
        let mut lignes = lignes_stats(&avant, &apres, nom_stat);
        if let bs_donnees::armes::Mecanique::Retour { loyaute, .. } = def.mecanique
            && (rang + 1) as f32 == loyaute
        {
            lignes.push("Revient : non → oui".into());
        }
        Carte {
            choix,
            famille: Famille::Arme,
            nom: def.nom.clone(),
            icone: Icone::Texture(&def.icone),
            etiquette: format!("Rang {rang} → {}", rang + 1),
            description: def.niveaux[rang as usize - 1].description.clone(),
            lignes,
        }
    }

    fn decrire_eveil(&self, index: usize, variante: usize) -> Carte {
        let d = self.d;
        let def = &d.armes[index];
        let eveils = def.eveils.as_ref();
        let arme = self.arsenal.armes.iter().find(|a| a.index == index);
        let (Some(eveil), Some(arme)) = (eveils.and_then(|e| e.get(variante)), arme) else {
            return self.decrire_amelioration(Amelioration::Soin);
        };
        let m = &self.etat.modificateurs;
        let t = &arme.talent;
        let avant = calculer_stats(d, def, arme.rang, m, t, None);
        let apres = calculer_stats(d, def, arme.rang, m, t, Some(eveil));
        let nom_stat = |cle| {
            def.libelles
                .as_ref()
                .and_then(|l| l.get(&cle))
                .map_or(libelle_stat(cle).0, String::as_str)
        };
        let mut lignes = lignes_stats(&avant, &apres, nom_stat);
        if let Some(e) = &eveil.etat {
            lignes.push(libelle_etat(e.genre).into());
        }
        let description = match eveils.and_then(|e| e.get(1 - variante)) {
            Some(autre) => format!("{} (Exclut {}.)", eveil.description, autre.nom),
            None => eveil.description.clone(),
        };
        Carte {
            choix: Choix::Niveau(Amelioration::Eveil {
                arme: index,
                variante,
            }),
            famille: Famille::Eveil,
            nom: format!("{} : {}", def.nom, eveil.nom),
            icone: Icone::Texture(&eveil.icone),
            etiquette: "Éveil".into(),
            description,
            lignes,
        }
    }

    fn decrire_passif(&self, index: usize) -> Carte {
        let d = self.d;
        let def = &d.passifs[index];
        let famille = if def.exclusif {
            Famille::Exclusif
        } else {
            Famille::Passif
        };
        let choix = Choix::Niveau(Amelioration::Passif(index));
        let premier = || format!("{} {}.", def.description, def.niveaux[0].description);
        if let Some(c) = &def.consommable {
            // Pas de rangs : un exemplaire de plus dans la réserve
            let reserve = self.etat.modificateurs.vies - self.etat.joueur.vies_utilisees as f32;
            return Carte {
                choix,
                famille,
                nom: def.nom.clone(),
                icone: Icone::Texture(&def.icone),
                etiquette: "Consommable".into(),
                description: premier(),
                lignes: vec![format!(
                    "{} : {} → {}",
                    c.libelle,
                    libelles::nombre(reserve),
                    libelles::nombre(reserve + 1.0)
                )],
            };
        }
        let rang = self.passifs.rang(index);
        let avant = self.passifs.calculer(d, None);
        let apres = self.passifs.calculer(d, Some(index));
        let mut lignes = Vec::new();
        lignes_modificateurs(&avant, &apres, &mut lignes);
        let etiquette = match (rang, def.exclusif) {
            (0, true) => "Nouveau bonus exclusif".into(),
            (0, false) => "Nouveau bonus".into(),
            _ => format!("Rang {rang} → {}", rang + 1),
        };
        Carte {
            choix,
            famille,
            nom: def.nom.clone(),
            icone: Icone::Texture(&def.icone),
            etiquette,
            description: if rang == 0 {
                premier()
            } else {
                def.niveaux[rang as usize].description.clone()
            },
            lignes,
        }
    }
}

/// Lignes « Nom : avant → après » des statistiques d'arme qui changent.
fn lignes_stats<'a>(
    avant: &bs_donnees::StatsArme,
    apres: &bs_donnees::StatsArme,
    nom: impl Fn(bs_donnees::StatArme) -> &'a str,
) -> Vec<String> {
    STATS
        .iter()
        .filter(|&&c| (valeur_stat(apres, c) - valeur_stat(avant, c)).abs() > 1e-6)
        .map(|&c| {
            format!(
                "{} : {} → {}",
                nom(c),
                formater_stat(c, valeur_stat(avant, c)),
                formater_stat(c, valeur_stat(apres, c))
            )
        })
        .collect()
}
