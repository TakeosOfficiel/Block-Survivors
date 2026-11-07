//! Profil du joueur (port de src/profil.ts), gardé dans le profil actif sous la clé
//! `profil` : pseudo (étiquette au-dessus du personnage) et apparence. L'apparence ne
//! change que le modèle et la skin ; la classe garde ses statistiques, ses armes et son
//! objet en main.
//!
//! Skin : celle de la classe, la skin importée (PNG en data URL, voir
//! `bs-rendu/src/skins.rs`) ou une des skins par défaut du jeu, bras larges ou fins.

use serde_json::{Map, Value};

use crate::stockage::Stockage;

/// Skins par défaut du jeu (`entity/player/wide|slim/<nom>`).
pub const SKINS_VANILLA: [&str; 9] = [
    "steve", "alex", "ari", "efe", "kai", "makena", "noor", "sunny", "zuri",
];

/// Nom affiché de chaque skin par défaut (même ordre).
pub const NOMS_SKINS: [&str; 9] = [
    "Steve", "Alex", "Ari", "Efe", "Kai", "Makena", "Noor", "Sunny", "Zuri",
];

pub const LONGUEUR_PSEUDO: usize = 16;

/// Chemin de texture de la skin importée.
pub const CHEMIN_SKIN_IMPORTE: &str = "profil/skin";

/// Skin choisie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChoixSkin {
    Classe,
    Importe,
    /// Index dans `SKINS_VANILLA`.
    Vanilla(usize),
}

impl ChoixSkin {
    /// Identifiant gardé dans le profil et envoyé aux autres joueurs de la coop.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Classe => "classe",
            Self::Importe => "importe",
            Self::Vanilla(k) => SKINS_VANILLA[*k],
        }
    }

    /// Choix d'après son identifiant (inconnu : celle de la classe).
    pub fn depuis_id(id: &str) -> Self {
        match id {
            "importe" => Self::Importe,
            _ => SKINS_VANILLA
                .iter()
                .position(|s| *s == id)
                .map_or(Self::Classe, Self::Vanilla),
        }
    }
}

/// Modèle et skin du personnage joué quand le profil remplace ceux de la classe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApparenceProfil {
    pub modele: &'static str,
    pub skin: String,
}

#[derive(Debug, Clone)]
pub struct ProfilJoueur {
    stockage: Stockage,
    cle: String,
    pub pseudo: String,
    pub skin: ChoixSkin,
    /// Modèle aux bras fins (3 pixels, comme Alex).
    pub bras_fins: bool,
    /// Skin importée (PNG en data URL).
    pub skin_importe: Option<String>,
}

/// Pseudo sans espaces superflus, limité à `LONGUEUR_PSEUDO` caractères.
pub fn nettoyer_pseudo(pseudo: &str) -> String {
    let mots: Vec<&str> = pseudo.split_whitespace().collect();
    mots.join(" ").chars().take(LONGUEUR_PSEUDO).collect()
}

/// Skin d'une des skins par défaut, au gabarit choisi.
pub fn chemin_skin_vanilla(index: usize, bras_fins: bool) -> String {
    let gabarit = if bras_fins { "slim" } else { "wide" };
    format!("entity/player/{gabarit}/{}", SKINS_VANILLA[index])
}

const DEBUTS: [&str; 12] = [
    "Creeper", "Diamant", "Redstone", "Ender", "Pixel", "Blaze", "Golem", "Nether", "Obsi",
    "Lapis", "Silex", "Cubo",
];
const FINS: [&str; 10] = [
    "Chasseur", "Furtif", "Doré", "Sombre", "Rapide", "Mineur", "Lunaire", "Royal", "Sauvage",
    "Brave",
];

/// Pseudo au hasard dans l'esprit du jeu (« RedstoneFurtif42 ») ; `tirer` donne un
/// nombre dans [0, 1[.
pub fn pseudo_aleatoire(mut tirer: impl FnMut() -> f32) -> String {
    let mut choisir =
        |l: &[&'static str]| l[((tirer() * l.len() as f32) as usize).min(l.len() - 1)];
    let debut = choisir(&DEBUTS);
    let fin = choisir(&FINS);
    let nombre = if tirer() < 0.6 {
        ((tirer() * 100.0) as u32).min(99).to_string()
    } else {
        String::new()
    };
    nettoyer_pseudo(&format!("{debut}{fin}{nombre}"))
}

impl ProfilJoueur {
    pub fn charger(stockage: Stockage, cle: String) -> Self {
        let mut p = Self {
            stockage,
            cle,
            pseudo: "Joueur".into(),
            skin: ChoixSkin::Classe,
            bras_fins: false,
            skin_importe: None,
        };
        let Some(v) = p.stockage.lire::<Value>(&p.cle) else {
            return p;
        };
        if let Some(s) = v["pseudo"].as_str() {
            p.pseudo = nettoyer_pseudo(s);
        }
        if let Some(b) = v["brasFins"].as_bool() {
            p.bras_fins = b;
        }
        if let Some(s) = v["skinImporte"].as_str()
            && s.starts_with("data:image/")
        {
            p.skin_importe = Some(s.to_owned());
        }
        // L'ancien personnage de l'éditeur (« perso ») n'existe plus : skin de la classe
        p.skin = match v["skin"].as_str() {
            Some("importe") if p.skin_importe.is_some() => ChoixSkin::Importe,
            Some(s) => SKINS_VANILLA
                .iter()
                .position(|n| *n == s)
                .map_or(ChoixSkin::Classe, ChoixSkin::Vanilla),
            None => ChoixSkin::Classe,
        };
        p
    }

    /// Enregistre le profil (même JSON que le jeu TS).
    pub fn enregistrer(&self) {
        let mut o = Map::new();
        o.insert("pseudo".into(), Value::from(self.pseudo.clone()));
        o.insert("skin".into(), Value::from(self.skin.id()));
        o.insert("brasFins".into(), Value::from(self.bras_fins));
        o.insert(
            "skinImporte".into(),
            self.skin_importe.clone().map_or(Value::Null, Value::from),
        );
        self.stockage.ecrire(&self.cle, &Value::Object(o));
    }

    /// Modèle et skin du personnage joué, ou `None` : ceux de la classe.
    pub fn apparence(&self) -> Option<ApparenceProfil> {
        let modele = if self.bras_fins {
            "humanoide_fin"
        } else {
            "humanoide"
        };
        let skin = match self.skin {
            ChoixSkin::Classe => return None,
            ChoixSkin::Importe => CHEMIN_SKIN_IMPORTE.to_owned(),
            ChoixSkin::Vanilla(k) => chemin_skin_vanilla(k, self.bras_fins),
        };
        Some(ApparenceProfil { modele, skin })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stockage(nom: &str) -> Stockage {
        let d = std::env::temp_dir().join(format!("bs-profil-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        Stockage::nouveau(d)
    }

    #[test]
    fn pseudo_nettoye() {
        assert_eq!(
            nettoyer_pseudo("  Le   grand\tCreeper  "),
            "Le grand Creeper"
        );
        assert_eq!(nettoyer_pseudo("ÉÉÉÉÉÉÉÉÉÉÉÉÉÉÉÉÉÉÉÉ").chars().count(), 16);
        let mut k = 0.0;
        let p = pseudo_aleatoire(|| {
            k += 0.37;
            k % 1.0
        });
        assert!(!p.is_empty() && p.chars().count() <= LONGUEUR_PSEUDO, "{p}");
    }

    #[test]
    fn profil_relu_et_apparence() {
        let s = stockage("relu");
        let mut p = ProfilJoueur::charger(s.clone(), "profil".into());
        assert_eq!(p.pseudo, "Joueur");
        assert_eq!(p.apparence(), None);
        p.pseudo = "Alex".into();
        p.skin = ChoixSkin::Vanilla(1);
        p.bras_fins = true;
        p.enregistrer();
        let relu = ProfilJoueur::charger(s.clone(), "profil".into());
        assert_eq!(relu.skin, ChoixSkin::Vanilla(1));
        assert_eq!(
            relu.apparence(),
            Some(ApparenceProfil {
                modele: "humanoide_fin",
                skin: "entity/player/slim/alex".into()
            })
        );
        // Skin importée absente : on revient à celle de la classe ; « perso » aussi
        s.ecrire_brut(
            "profil",
            r#"{"pseudo":"x","skin":"importe","skinImporte":null}"#,
        );
        assert_eq!(
            ProfilJoueur::charger(s.clone(), "profil".into()).skin,
            ChoixSkin::Classe
        );
        s.ecrire_brut("profil", r#"{"skin":"perso"}"#);
        assert_eq!(
            ProfilJoueur::charger(s.clone(), "profil".into()).skin,
            ChoixSkin::Classe
        );
        s.ecrire_brut(
            "profil",
            r#"{"skin":"importe","skinImporte":"data:image/png;base64,AAAA"}"#,
        );
        let p = ProfilJoueur::charger(s, "profil".into());
        assert_eq!(p.apparence().unwrap().skin, CHEMIN_SKIN_IMPORTE);
    }
}
