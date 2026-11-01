//! Profils (src/profils.ts, src/progression/transfert.ts) : plusieurs joueurs (ou départs)
//! sur la même installation. Chaque profil range ses données (`DONNEES_PROFIL` : pseudo
//! et apparence, progression, records à plusieurs, Collection, dernier choix, amis) sous
//! « block-survivors.p.<id>.<nom> » ; le registre « block-survivors.profils » donne les
//! identifiants et l'actif. Au premier lancement, une sauvegarde d'avant les profils
//! (« block-survivors.<nom> ») devient le profil 1. Import et export en `.bssave`.

use serde::{Deserialize, Serialize};

use crate::bssave::{self, DONNEES_PROFIL, Erreur, Profil};
use crate::stockage::Stockage;

const CLE_REGISTRE: &str = "block-survivors.profils";
const PREFIXE_ANCIEN: &str = "block-survivors.";
pub const MAX_PROFILS: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Registre {
    version: u32,
    actif: String,
    ids: Vec<String>,
}

/// Pourquoi un import est refusé (messages du jeu TS).
#[derive(Debug, PartialEq, Eq)]
pub enum RefusImport {
    Complet,
    Illisible,
    Vide,
}

impl RefusImport {
    pub fn message(&self) -> &'static str {
        match self {
            Self::Complet => "Il y a déjà le nombre maximal de profils : supprimez-en un d'abord.",
            Self::Illisible => "Ce fichier n'est pas une sauvegarde de Block Survivors.",
            Self::Vide => "Cette sauvegarde est vide.",
        }
    }
}

/// Registre des profils de l'installation.
#[derive(Debug, Clone)]
pub struct Profils {
    stockage: Stockage,
    registre: Registre,
}

/// Clé du stockage d'une donnée d'un profil.
pub fn cle(id: &str, nom: &str) -> String {
    format!("block-survivors.p.{id}.{nom}")
}

fn identifiant_valide(id: &str) -> bool {
    !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())
}

impl Profils {
    /// Lit le registre ; au premier lancement, la sauvegarde d'avant les profils devient
    /// le profil 1.
    pub fn charger(stockage: Stockage) -> Self {
        let lu: Option<serde_json::Value> = stockage.lire(CLE_REGISTRE);
        let ids: Vec<String> = lu
            .as_ref()
            .and_then(|v| v["ids"].as_array())
            .map(|ids| {
                let ids = ids.iter().filter_map(|i| i.as_str());
                ids.filter(|i| identifiant_valide(i))
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();
        if !ids.is_empty() {
            let actif = lu.as_ref().and_then(|v| v["actif"].as_str());
            let actif = actif
                .filter(|a| ids.iter().any(|i| i == a))
                .unwrap_or(&ids[0]);
            let registre = Registre {
                version: 1,
                actif: actif.to_owned(),
                ids,
            };
            return Self { stockage, registre };
        }
        let registre = Registre {
            version: 1,
            actif: "1".into(),
            ids: vec!["1".into()],
        };
        for nom in DONNEES_PROFIL {
            let ancienne = format!("{PREFIXE_ANCIEN}{nom}");
            if let Some(v) = stockage.lire_brut(&ancienne) {
                stockage.ecrire_brut(&cle("1", nom), &v);
                stockage.retirer(&ancienne);
            }
        }
        stockage.ecrire(CLE_REGISTRE, &registre);
        Self { stockage, registre }
    }

    pub fn stockage(&self) -> &Stockage {
        &self.stockage
    }

    pub fn actif(&self) -> &str {
        &self.registre.actif
    }

    /// Identifiants, dans l'ordre de création.
    pub fn ids(&self) -> &[String] {
        &self.registre.ids
    }

    /// Clé d'une donnée du profil actif.
    pub fn cle(&self, nom: &str) -> String {
        cle(self.actif(), nom)
    }

    pub fn peut_creer(&self) -> bool {
        self.registre.ids.len() < MAX_PROFILS
    }

    fn enregistrer(&self) {
        self.stockage.ecrire(CLE_REGISTRE, &self.registre);
    }

    /// Crée un profil (ses données déjà écrites par `remplir`) et le rend actif.
    pub fn creer(&mut self, remplir: impl FnOnce(&Stockage, &str)) -> String {
        let r = &mut self.registre;
        let mut n = r.ids.len() + 1;
        while r.ids.contains(&n.to_string()) {
            n += 1;
        }
        let id = n.to_string();
        remplir(&self.stockage, &id);
        r.ids.push(id.clone());
        r.actif.clone_from(&id);
        self.enregistrer();
        id
    }

    pub fn changer(&mut self, id: &str) {
        if self.registre.ids.iter().any(|i| i == id) {
            self.registre.actif = id.to_owned();
            self.enregistrer();
        }
    }

    /// Supprime un profil et ses données (jamais le dernier) ; l'actif passe au premier.
    pub fn supprimer(&mut self, id: &str) {
        let r = &mut self.registre;
        if r.ids.len() <= 1 || !r.ids.iter().any(|i| i == id) {
            return;
        }
        for nom in DONNEES_PROFIL {
            self.stockage.retirer(&cle(id, nom));
        }
        r.ids.retain(|i| i != id);
        if r.actif == id {
            r.actif = r.ids[0].clone();
        }
        self.enregistrer();
    }

    /// Nouveau profil (devenu l'actif) lu d'un `.bssave`, même d'avant les profils.
    pub fn importer(&mut self, contenu: &str) -> Result<String, RefusImport> {
        if !self.peut_creer() {
            return Err(RefusImport::Complet);
        }
        let profil = bssave::importer(contenu).map_err(|e| match e {
            Erreur::Illisible => RefusImport::Illisible,
            Erreur::Vide => RefusImport::Vide,
        })?;
        Ok(self.creer(|s, id| {
            for (nom, valeur) in &profil.donnees {
                s.ecrire_brut(&cle(id, nom), valeur);
            }
        }))
    }

    /// Le profil actif en `.bssave` (`version` du jeu, `date` ISO 8601).
    pub fn exporter(&self, version: &str, date: &str) -> String {
        let mut profil = Profil::default();
        for nom in DONNEES_PROFIL {
            if let Some(v) = self.stockage.lire_brut(&self.cle(nom)) {
                profil.donnees.insert(nom.to_owned(), v);
            }
        }
        bssave::exporter(&profil, version, date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stockage::essai;

    #[test]
    fn ancienne_sauvegarde_creation_suppression() {
        let s = essai("profils");
        s.ecrire_brut("block-survivors.progression", r#"{"parties":3}"#);
        let mut p = Profils::charger(s.clone());
        assert_eq!((p.actif(), p.ids().len()), ("1", 1));
        assert_eq!(
            s.lire_brut("block-survivors.p.1.progression").as_deref(),
            Some(r#"{"parties":3}"#)
        );
        assert!(s.lire_brut("block-survivors.progression").is_none());

        let deux = p.creer(|_, _| {});
        assert_eq!((deux.as_str(), p.actif()), ("2", "2"));
        let fichier = {
            p.changer("1");
            p.exporter("0.60.0", "2026-11-01T00:00:00Z")
        };
        let trois = p.importer(&fichier).unwrap();
        assert_eq!(
            s.lire_brut(&cle(&trois, "progression")).as_deref(),
            Some(r#"{"parties":3}"#)
        );
        p.supprimer("1");
        let relu = Profils::charger(s);
        assert_eq!(relu.ids(), ["2", "3"]);
        assert_eq!(relu.actif(), "3");
        assert_eq!(p.clone().importer("rien"), Err(RefusImport::Illisible));
    }
}
