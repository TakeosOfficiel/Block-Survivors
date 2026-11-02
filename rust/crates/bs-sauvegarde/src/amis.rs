//! Liste d'amis du profil (partie gardée de src/reseau/amis.ts), sous la clé `amis` : le
//! code ami du profil (« ABC-DEF-GHJ », tiré une fois), sa graine de clé ed25519 (tirée
//! une fois, `secret`) et les amis ajoutés avec leur dernier pseudo connu et leur clé
//! publique (retenue à leur première présence, `cle`). Même JSON que le jeu TS (version
//! 1), complété de ces deux champs que le jeu TS ignore.
//!
//! La présence en ligne, les invitations et les demandes passent par le relais
//! (bs-reseau::amis), sur ces mêmes codes.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use serde_json::{Value, json};

use crate::stockage::Stockage;

/// Lettres et chiffres des codes (sans 0, O, 1, I, L : jamais confondus).
pub const ALPHABET: &str = "ABCDEFGHJKMNPQRSTUVWXYZ23456789";
/// Longueur d'un code ami (sans les tirets).
pub const LONGUEUR_CODE_AMI: usize = 9;
/// Amis au plus.
pub const AMIS_MAX: usize = 50;
/// Longueur maximale d'un pseudo d'ami gardé.
const LONGUEUR_PSEUDO: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ami {
    pub code: String,
    /// Dernier pseudo connu (« ? » tant qu'il ne s'est pas manifesté).
    pub pseudo: String,
    /// Clé publique ed25519 retenue à sa première présence (`None` avant).
    pub cle: Option<[u8; 32]>,
}

#[derive(Debug, Clone)]
pub struct Amis {
    stockage: Stockage,
    cle: String,
    /// Code ami du profil.
    pub code: String,
    /// Graine de la clé ed25519 du profil (signe ses présences et ses lettres).
    pub secret: [u8; 32],
    pub amis: Vec<Ami>,
}

/// Groupes de 3 séparés par des tirets.
fn grouper(brut: &str) -> String {
    let c: Vec<char> = brut.chars().collect();
    c.chunks(3)
        .map(|g| g.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("-")
}

/// Code tapé par le joueur, remis en forme (minuscules, espaces, tirets oubliés) ; `None`
/// s'il n'en est pas un.
pub fn normaliser_code(saisie: &str, longueur: usize) -> Option<String> {
    let brut: String = saisie
        .to_uppercase()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect();
    (brut.len() == longueur && brut.chars().all(|c| ALPHABET.contains(c))).then(|| grouper(&brut))
}

pub fn normaliser_code_ami(saisie: &str) -> Option<String> {
    normaliser_code(saisie, LONGUEUR_CODE_AMI)
}

/// Code au hasard (aléa du système), en groupes de 3.
pub fn code_au_hasard(longueur: usize) -> String {
    let mut octets = vec![0u8; longueur];
    getrandom::getrandom(&mut octets).expect("aléa du système");
    let alphabet = ALPHABET.as_bytes();
    let brut: String = octets
        .iter()
        .map(|o| alphabet[usize::from(*o) % alphabet.len()] as char)
        .collect();
    grouper(&brut)
}

/// Refus d'un ajout (message affiché, comme le jeu TS).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusAmi {
    CodeInvalide,
    PropreCode,
    DejaAmi,
    ListePleine,
}

impl RefusAmi {
    pub fn message(self) -> &'static str {
        match self {
            Self::CodeInvalide => "Un code ami a 9 lettres ou chiffres, comme ABC-DEF-GHJ",
            Self::PropreCode => "C'est votre propre code",
            Self::DejaAmi => "Cet ami est déjà dans votre liste",
            Self::ListePleine => "Liste d'amis pleine",
        }
    }
}

/// 32 octets en base64, s'il y en a.
fn octets32(v: &Value) -> Option<[u8; 32]> {
    let o = B64.decode(v.as_str()?).ok()?;
    o.try_into().ok()
}

impl Amis {
    /// Lit la liste (un code ami est tiré et enregistré s'il n'y en a pas encore).
    pub fn charger(stockage: Stockage, cle: String) -> Self {
        let lu: Option<Value> = stockage.lire(&cle);
        let lu = lu.unwrap_or(Value::Null);
        let valide = |v: &Value| {
            v.as_str()
                .filter(|c| normaliser_code_ami(c).as_deref() == Some(*c))
                .map(str::to_owned)
        };
        let code = valide(&lu["code"]).unwrap_or_else(|| code_au_hasard(LONGUEUR_CODE_AMI));
        let amis = lu["amis"]
            .as_array()
            .map(|l| {
                l.iter()
                    .filter_map(|a| {
                        let code = valide(&a["code"])?;
                        let pseudo = match &a["pseudo"] {
                            Value::String(s) => s.clone(),
                            Value::Null => "?".to_owned(),
                            v => v.to_string(),
                        };
                        let pseudo = pseudo.chars().take(LONGUEUR_PSEUDO).collect();
                        let cle = octets32(&a["cle"]);
                        Some(Ami { code, pseudo, cle })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let secret = octets32(&lu["secret"]).unwrap_or_else(|| {
            let mut s = [0u8; 32];
            getrandom::getrandom(&mut s).expect("aléa du système");
            s
        });
        let a = Self {
            stockage,
            cle,
            code,
            secret,
            amis,
        };
        a.enregistrer();
        a
    }

    pub fn enregistrer(&self) {
        let amis: Vec<Value> = self
            .amis
            .iter()
            .map(|a| {
                let mut v = json!({ "code": a.code, "pseudo": a.pseudo });
                if let Some(c) = a.cle {
                    v["cle"] = B64.encode(c).into();
                }
                v
            })
            .collect();
        let v = json!({
            "version": 1,
            "code": self.code,
            "secret": B64.encode(self.secret),
            "amis": amis,
        });
        self.stockage.ecrire(&self.cle, &v);
    }

    /// Ajoute un ami par son code (enregistré).
    pub fn ajouter(&mut self, saisie: &str, pseudo: &str) -> Result<&Ami, RefusAmi> {
        let code = normaliser_code_ami(saisie).ok_or(RefusAmi::CodeInvalide)?;
        if code == self.code {
            return Err(RefusAmi::PropreCode);
        }
        if self.amis.iter().any(|a| a.code == code) {
            return Err(RefusAmi::DejaAmi);
        }
        if self.amis.len() >= AMIS_MAX {
            return Err(RefusAmi::ListePleine);
        }
        let pseudo = pseudo.chars().take(LONGUEUR_PSEUDO).collect();
        self.amis.push(Ami {
            code,
            pseudo,
            cle: None,
        });
        self.enregistrer();
        Ok(self.amis.last().expect("ami ajouté"))
    }

    /// Retire un ami (enregistré) ; faux s'il n'était pas dans la liste.
    pub fn retirer(&mut self, code: &str) -> bool {
        let Some(i) = self.amis.iter().position(|a| a.code == code) else {
            return false;
        };
        self.amis.remove(i);
        self.enregistrer();
        true
    }

    /// Nouveau pseudo connu d'un ami (enregistré s'il a changé).
    pub fn renommer(&mut self, code: &str, pseudo: &str) {
        let pseudo: String = pseudo.chars().take(LONGUEUR_PSEUDO).collect();
        if let Some(a) = self.amis.iter_mut().find(|a| a.code == code)
            && a.pseudo != pseudo
        {
            a.pseudo = pseudo;
            self.enregistrer();
        }
    }

    /// Clé d'un ami vue à sa présence : retenue la première fois (enregistrée). Faux si
    /// l'ami est inconnu ou en avait déjà une autre (quelqu'un d'autre sous son code).
    pub fn lier_cle(&mut self, code: &str, cle: [u8; 32]) -> bool {
        let Some(a) = self.amis.iter_mut().find(|a| a.code == code) else {
            return false;
        };
        match a.cle {
            Some(c) => c == cle,
            None => {
                a.cle = Some(cle);
                self.enregistrer();
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stockage(nom: &str) -> Stockage {
        let d = std::env::temp_dir().join(format!("bs-amis-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        Stockage::nouveau(d)
    }

    #[test]
    fn codes() {
        assert_eq!(
            normaliser_code_ami(" abc def ghj").as_deref(),
            Some("ABC-DEF-GHJ")
        );
        assert_eq!(normaliser_code_ami("ABC-DEF-GH"), None);
        // 0 et 1 ne sont pas dans l'alphabet
        assert_eq!(normaliser_code_ami("ABC-DEF-GH0"), None);
        for _ in 0..20 {
            let c = code_au_hasard(LONGUEUR_CODE_AMI);
            assert_eq!(c.len(), 11);
            assert_eq!(normaliser_code_ami(&c), Some(c));
        }
    }

    #[test]
    fn liste_gardee() {
        let s = stockage("liste");
        let mut a = Amis::charger(s.clone(), "amis".into());
        let code = a.code.clone();
        assert_eq!(a.ajouter("xyz", "?"), Err(RefusAmi::CodeInvalide));
        assert_eq!(a.ajouter(&code, "?"), Err(RefusAmi::PropreCode));
        assert!(a.ajouter("abc-def-ghj", "Steve").is_ok());
        assert_eq!(a.ajouter("ABCDEFGHJ", "?"), Err(RefusAmi::DejaAmi));
        a.ajouter("KMN-PQR-STU", "?").unwrap();
        a.renommer("KMN-PQR-STU", "Alex");
        assert!(a.lier_cle("KMN-PQR-STU", [7; 32]));
        assert!(!a.lier_cle("KMN-PQR-STU", [8; 32]));
        assert!(!a.lier_cle("ZZZ-ZZZ-ZZZ", [7; 32]));
        let mut relu = Amis::charger(s.clone(), "amis".into());
        assert_eq!(relu.code, code);
        assert_eq!(relu.secret, a.secret);
        assert_eq!(relu.amis[1].cle, Some([7; 32]));
        assert_eq!(relu.amis[0].cle, None);
        assert_eq!(relu.amis.len(), 2);
        assert_eq!(relu.amis[1].pseudo, "Alex");
        assert!(relu.retirer("ABC-DEF-GHJ"));
        assert!(!relu.retirer("ABC-DEF-GHJ"));
        // JSON du jeu TS : code invalide écarté, pseudo absent remplacé
        s.ecrire_brut(
            "amis",
            r#"{"version":1,"code":"bad","amis":[{"code":"ABC-DEF-GHJ"},{"code":"x","pseudo":"y"}]}"#,
        );
        let a = Amis::charger(s, "amis".into());
        assert_ne!(a.code, "bad");
        assert_eq!(
            a.amis,
            vec![Ami {
                code: "ABC-DEF-GHJ".into(),
                pseudo: "?".into(),
                cle: None,
            }]
        );
    }
}
