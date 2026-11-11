//! Textes des cartes de coffre (port de src/progression/equipement.ts) : objet, renfort,
//! remplacement, retour, émeraudes et trésors de l'équipement complet, avec les bonus « avant → après ».

use bs_donnees::equipement::Emplacement;

use super::libelles::lignes_modificateurs;
use super::{Carte, Choix, Famille, Icone};
use crate::equipement::{ChoixEquipement, ObjetPorte};
use crate::etat::Modificateurs;
use crate::partie::Partie;

fn nom_emplacement(e: Emplacement) -> &'static str {
    match e {
        Emplacement::Bague => "Bague",
        Emplacement::Amulette => "Amulette",
    }
}

impl Partie {
    /// Une carte de coffre décrite.
    pub fn decrire_equipement(&self, c: ChoixEquipement) -> Carte {
        let d = self.d;
        let objets = &d.equipement.objets;
        let emeraude = d.objets.get("emeraude").map_or("", |o| o.icone.as_str());
        let carte = |famille, nom: &str, icone, etiquette: String, description: String| Carte {
            choix: Choix::Coffre(c),
            famille,
            nom: nom.into(),
            icone,
            etiquette,
            description,
            lignes: Vec::new(),
        };
        match c {
            ChoixEquipement::Objet(objet) => {
                let def = &objets[objet];
                let plein = !self.emplacement_libre(def.emplacement);
                let place = nom_emplacement(def.emplacement);
                let etiquette = if plein {
                    format!("{place} · remplace un objet")
                } else {
                    format!("Nouvelle {}", place.to_lowercase())
                };
                let mut k = carte(
                    Famille::Objet(def.rarete),
                    &def.nom,
                    Icone::Objet(objet),
                    etiquette,
                    def.description[0].clone(),
                );
                if !plein {
                    let mut portes: Vec<ObjetPorte> = self.equipement.portes().collect();
                    portes.push(ObjetPorte { objet, niveau: 1 });
                    k.lignes = self.lignes_equipement(&portes);
                }
                k
            }
            ChoixEquipement::Renfort(objet) => {
                let def = &objets[objet];
                let niveau = self.equipement.portes().find(|o| o.objet == objet);
                let niveau = niveau.map_or(1, |o| o.niveau);
                let mut k = carte(
                    Famille::Objet(def.rarete),
                    &def.nom,
                    Icone::Objet(objet),
                    format!("Renforcer · niveau {niveau} → {}", niveau + 1),
                    def.description[niveau as usize].clone(),
                );
                let portes: Vec<ObjetPorte> = self
                    .equipement
                    .portes()
                    .map(|mut o| {
                        o.niveau += u32::from(o.objet == objet);
                        o
                    })
                    .collect();
                k.lignes = self.lignes_equipement(&portes);
                k
            }
            ChoixEquipement::Remplacer { objet, index } => {
                let def = &objets[objet];
                let places = match def.emplacement {
                    Emplacement::Bague => &self.equipement.bagues,
                    Emplacement::Amulette => &self.equipement.amulettes,
                };
                let Some(ancien) = places.get(index).copied().flatten() else {
                    return carte(
                        Famille::Action,
                        "",
                        Icone::Objet(objet),
                        String::new(),
                        String::new(),
                    );
                };
                let vieux = &objets[ancien.objet];
                let mut k = carte(
                    Famille::Objet(vieux.rarete),
                    &vieux.nom,
                    Icone::Objet(ancien.objet),
                    format!("Remplacer par {}", def.nom),
                    vieux.description[ancien.niveau as usize - 1].clone(),
                );
                let portes: Vec<ObjetPorte> = self
                    .equipement
                    .portes()
                    .map(|o| {
                        if o == ancien {
                            ObjetPorte { objet, niveau: 1 }
                        } else {
                            o
                        }
                    })
                    .collect();
                k.lignes = self.lignes_equipement(&portes);
                k
            }
            ChoixEquipement::Retour => {
                // L'objet refusé est le premier remplacement proposé
                let objet = self.equipement.proposees.iter().find_map(|p| match p {
                    ChoixEquipement::Remplacer { objet, .. } => Some(*objet),
                    _ => None,
                });
                let nom = objet.map_or("", |o| objets[o].nom.as_str());
                carte(
                    Famille::Action,
                    "Garder mes objets",
                    Icone::Objet(objet.unwrap_or(0)),
                    "Retour".into(),
                    format!("Ne pas prendre {nom} : revenir aux objets du coffre."),
                )
            }
            ChoixEquipement::Niveau => carte(
                Famille::Action,
                "Fiole d'expérience",
                Icone::Texture("item/experience_bottle"),
                "Équipement au maximum".into(),
                "Passer tout de suite au niveau suivant.".into(),
            ),
            ChoixEquipement::Soin => carte(
                Famille::Action,
                "Pomme d'or enchantée",
                Icone::Texture("item/golden_apple"),
                "Équipement au maximum".into(),
                "Rendre tous vos PV.".into(),
            ),
            ChoixEquipement::Emeraudes(n) => {
                // Seule carte du coffre : consolation ; sinon, refus
                let seule = self.equipement.proposees.len() == 1;
                let (nom, etiquette, description) = if seule {
                    (
                        "Sac d'émeraudes",
                        "Équipement au maximum",
                        format!("Repartir avec {n} émeraudes."),
                    )
                } else {
                    (
                        "Laisser les objets",
                        "Refuser",
                        format!(
                            "Laisser ces objets dans le coffre et repartir avec {n} émeraudes."
                        ),
                    )
                };
                let nom = if seule {
                    nom.to_owned()
                } else {
                    format!("{nom} · recevoir {n} émeraudes")
                };
                carte(
                    Famille::Action,
                    &nom,
                    Icone::Texture(emeraude),
                    etiquette.into(),
                    description,
                )
            }
        }
    }

    fn emplacement_libre(&self, e: Emplacement) -> bool {
        match e {
            Emplacement::Bague => self.equipement.bagues.contains(&None),
            Emplacement::Amulette => self.equipement.amulettes.contains(&None),
        }
    }

    /// Lignes « avant → après » si l'équipement porté devenait `portes`.
    fn lignes_equipement(&self, portes: &[ObjetPorte]) -> Vec<String> {
        let d = self.d;
        let avant: Modificateurs = self.passifs.calculer(d, None);
        let mut passifs = self.passifs.clone();
        let objets = &d.equipement.objets;
        passifs.definir_equipement(portes.iter().filter_map(|o| {
            let f = d.equipement.regles.niveaux[o.niveau as usize - 1];
            objets[o.objet].modif.as_ref().map(|m| m.fois(f))
        }));
        let apres = passifs.calculer(d, None);
        let mut lignes = Vec::new();
        lignes_modificateurs(&avant, &apres, &mut lignes);
        lignes
    }
}
