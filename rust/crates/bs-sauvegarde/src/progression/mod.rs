//! Progression entre les parties (src/progression/sauvegarde.ts, version 3), même JSON
//! que la version TypeScript : XP de maîtrise de chaque classe, talents achetés dans les
//! deux arbres de chaque classe, réglages de chaque arbre (option des talents à choix,
//! talent final actif, arme de départ active), arbre actif, records qui débloquent
//! personnages et arènes.
//!
//! Chaque niveau de maîtrise donne un point de talent ; un rang coûte un point. La
//! maîtrise plafonne (`niveau_max`) bien en dessous des rangs d'un arbre : on ne prend
//! pas tout, il faut choisir ses branches. Un seul talent final par arbre, et l'option
//! d'un talent à choix est fixée à l'achat (réinitialiser pour changer). Les deux
//! arbres d'une classe ont chacun tous les points (double spécialisation). Une sauvegarde
//! d'une version plus ancienne perd ses talents (records et arbre actif gardés), comme
//! un arbre qui dépasse ces règles (ancienne maîtrise sans plafond).
//! `bonus` donne les effets en jeu de l'arbre actif (bonus.rs).

use std::collections::BTreeMap;

use bs_donnees::Donnees;
use bs_donnees::progression::{Condition, ConditionSucces, Deblocage, TypeDeblocage};
use serde_json::{Map, Value};

use crate::stockage::Stockage;

mod bonus;
mod talents;

const VERSION: u32 = 3;

type Nombres = BTreeMap<String, f64>;
type Textes = BTreeMap<String, String>;

/// Où en est la maîtrise d'une classe.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EtatMaitrise {
    pub niveau: u32,
    pub xp: u64,
    /// XP du début du niveau actuel et du suivant.
    pub debut_niveau: u64,
    pub fin_niveau: u64,
}

impl EtatMaitrise {
    /// Niveau plafond atteint : plus de point à gagner.
    pub fn au_max(&self) -> bool {
        self.fin_niveau == self.debut_niveau
    }
}

/// Bilan d'une partie terminée.
#[derive(Debug, Clone)]
pub struct Bilan<'a> {
    pub personnage: &'a str,
    pub arene: &'a str,
    /// Temps survécu (s) ; temps total pour la suite d'une partie gagnée.
    pub temps: f64,
    /// XP ramassée : elle va à la maîtrise de la classe.
    pub xp: f64,
    pub boss: u32,
    /// Suite en mode sans fin d'une partie gagnée : la victoire a déjà compté la partie,
    /// le record et l'XP d'avant ; `xp` et `boss` ne comptent que la suite.
    pub sans_fin: bool,
    /// Joueurs dans la partie, soi compris : à plusieurs, seuls la maîtrise et les
    /// succès progressent.
    pub joueurs: u8,
    /// Tous les joueurs avaient la même classe.
    pub meme_classe: bool,
    /// Partie gagnée (arène finie) ; faux pour la suite sans fin.
    pub victoire: bool,
}

impl Bilan<'_> {
    pub fn en_equipe(&self) -> bool {
        self.joueurs > 1
    }

    /// Le bilan remplit `c`.
    pub fn remplit(&self, c: &ConditionSucces) -> bool {
        self.joueurs >= c.joueurs
            && (self.victoire || !c.victoire)
            && self.boss >= c.boss
            && (self.meme_classe || !c.meme_classe)
            && self.temps >= f64::from(c.survie)
    }
}

/// Ce que la partie a rapporté.
#[derive(Debug, Clone, PartialEq)]
pub struct Recompense {
    pub xp: u64,
    pub avant: EtatMaitrise,
    pub apres: EtatMaitrise,
    /// Index des déblocages obtenus (`d.talents.deblocages`).
    pub deblocages: Vec<usize>,
    /// Mode sans fin : le temps bat le record sans fin de l'arène.
    pub record_sans_fin: bool,
    /// Index des succès gagnés par cette partie (`d.succes`).
    pub succes: Vec<usize>,
}

/// Pourquoi un rang ne peut pas être acheté.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusAchat {
    Max,
    Verrou,
    Points,
    /// Un autre talent final est déjà appris dans cet arbre.
    Exclusif,
}

#[derive(Debug, Clone, Default)]
struct Champs {
    maitrise: Nombres,
    rangs: Nombres,
    options: Nombres,
    finals: Textes,
    armes_depart: Textes,
    arbres: Nombres,
    records: Nombres,
    records_sans_fin: Nombres,
    /// Succès gagnés : identifiant → 1.
    succes: Nombres,
    boss_vaincus: f64,
    parties: f64,
    reinitialise: bool,
}

/// Progression d'un profil.
#[derive(Debug, Clone)]
pub struct Progression {
    d: &'static Donnees,
    stockage: Stockage,
    cle: String,
    c: Champs,
}

fn nombre(v: &Value) -> f64 {
    v.as_f64()
        .filter(|n| n.is_finite())
        .map_or(0.0, |n| n.max(0.0))
}

fn table(v: &Value) -> Nombres {
    let o = v.as_object().into_iter().flatten();
    o.map(|(k, v)| (k.clone(), nombre(v))).collect()
}

fn textes(v: &Value) -> Textes {
    let o = v.as_object().into_iter().flatten();
    o.filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_owned())))
        .collect()
}

/// Nombre JSON comme l'écrit JavaScript (entier sans « .0 »).
fn json(n: f64) -> Value {
    if n.fract() == 0.0 && n.abs() < 9e15 {
        Value::from(n as i64)
    } else {
        Value::from(n)
    }
}

fn json_table(t: &Nombres) -> Value {
    Value::Object(t.iter().map(|(k, v)| (k.clone(), json(*v))).collect())
}

fn json_textes(t: &Textes) -> Value {
    Value::Object(
        t.iter()
            .map(|(k, v)| (k.clone(), Value::from(v.as_str())))
            .collect(),
    )
}

impl Progression {
    /// Lit la progression rangée sous `cle` (« block-survivors.p.1.progression »).
    pub fn charger(d: &'static Donnees, stockage: Stockage, cle: String) -> Self {
        let lu: Option<Value> = stockage.lire(&cle);
        let existe = lu.is_some();
        let v = lu.unwrap_or(Value::Null);
        let actuelle = v["version"].as_f64() == Some(f64::from(VERSION));
        let si = |t: Nombres| if actuelle { t } else { Nombres::new() };
        let c = Champs {
            maitrise: si(table(&v["maitrise"])),
            rangs: si(table(&v["rangs"])),
            options: si(table(&v["options"])),
            finals: if actuelle {
                textes(&v["finals"])
            } else {
                Textes::new()
            },
            armes_depart: if actuelle {
                textes(&v["armesDepart"])
            } else {
                Textes::new()
            },
            arbres: table(&v["arbres"]),
            records: table(&v["records"]),
            records_sans_fin: table(&v["recordsSansFin"]),
            succes: table(&v["succes"]),
            boss_vaincus: nombre(&v["bossVaincus"]),
            parties: nombre(&v["parties"]),
            // Une ancienne sauvegarde existait : ses talents sont remis à zéro
            reinitialise: if actuelle {
                v["reinitialise"] == Value::Bool(true)
            } else {
                existe
            },
        };
        let mut p = Self {
            d,
            stockage,
            cle,
            c,
        };
        let assaini = p.assainir();
        if (existe && !actuelle) || assaini {
            p.enregistrer();
        }
        p
    }

    fn enregistrer(&self) {
        let c = &self.c;
        let mut o = Map::new();
        o.insert("version".into(), Value::from(VERSION));
        o.insert("maitrise".into(), json_table(&c.maitrise));
        o.insert("rangs".into(), json_table(&c.rangs));
        o.insert("options".into(), json_table(&c.options));
        o.insert("finals".into(), json_textes(&c.finals));
        o.insert("armesDepart".into(), json_textes(&c.armes_depart));
        o.insert("arbres".into(), json_table(&c.arbres));
        o.insert("records".into(), json_table(&c.records));
        o.insert("recordsSansFin".into(), json_table(&c.records_sans_fin));
        o.insert("succes".into(), json_table(&c.succes));
        o.insert("bossVaincus".into(), json(c.boss_vaincus));
        o.insert("parties".into(), json(c.parties));
        o.insert("reinitialise".into(), Value::Bool(c.reinitialise));
        self.stockage.ecrire(&self.cle, &Value::Object(o));
    }

    /// Vrai une seule fois si les talents viennent d'être remis à zéro.
    pub fn annoncer_reinitialisation(&mut self) -> bool {
        if !std::mem::take(&mut self.c.reinitialise) {
            return false;
        }
        self.enregistrer();
        true
    }

    pub fn parties(&self) -> u32 {
        self.c.parties as u32
    }

    // --- Maîtrise ---

    fn etat_maitrise(&self, xp: u64) -> EtatMaitrise {
        let m = &self.d.talents.maitrise;
        let niveau = m.niveau(xp);
        let debut_niveau = m.xp_pour_niveau(niveau);
        EtatMaitrise {
            niveau,
            xp,
            debut_niveau,
            // Au plafond, plus de niveau suivant (barre pleine)
            fin_niveau: if niveau >= m.niveau_max {
                debut_niveau
            } else {
                m.xp_pour_niveau(niveau + 1)
            },
        }
    }

    pub fn maitrise(&self, personnage: &str) -> EtatMaitrise {
        let xp = self.c.maitrise.get(personnage).copied().unwrap_or(0.0);
        self.etat_maitrise(xp as u64)
    }

    /// Points de talent encore libres dans cet arbre.
    pub fn points_libres(&self, personnage: &str, arbre: usize) -> u32 {
        let niveau = self.maitrise(personnage).niveau;
        niveau.saturating_sub(self.points(personnage, arbre, None))
    }

    // --- Records et déblocages ---

    /// Meilleur temps survécu dans l'arène (s), 0 si jamais jouée.
    pub fn record(&self, arene: &str) -> f64 {
        self.c.records.get(arene).copied().unwrap_or(0.0)
    }

    /// Le succès `id` est gagné.
    pub fn succes(&self, id: &str) -> bool {
        self.c.succes.contains_key(id)
    }

    pub fn record_sans_fin(&self, arene: &str) -> f64 {
        self.c.records_sans_fin.get(arene).copied().unwrap_or(0.0)
    }

    /// Condition à remplir, ou `None` si c'est déjà disponible.
    pub fn verrou(&self, genre: TypeDeblocage, id: &str) -> Option<&'static Deblocage> {
        let d = self.d.talents.deblocages.iter();
        d.into_iter()
            .find(|b| b.genre == genre && b.id == id)
            .filter(|b| !self.rempli(b))
    }

    fn rempli(&self, b: &Deblocage) -> bool {
        match &b.condition {
            Condition::Survie { duree, arene } => {
                let records = &self.c.records;
                let meilleur = match arene {
                    Some(a) => records.get(a).copied().unwrap_or(0.0),
                    None => records.values().copied().fold(0.0, f64::max),
                };
                meilleur >= f64::from(*duree)
            }
            Condition::Boss { nombre } => self.c.boss_vaincus >= f64::from(*nombre),
        }
    }

    // --- Fin de partie ---

    /// Ajoute l'XP de la partie à la maîtrise de la classe et enregistre ses records.
    pub fn terminer_partie(&mut self, b: &Bilan) -> Recompense {
        let deblocages = &self.d.talents.deblocages;
        let verrouilles: Vec<usize> = (0..deblocages.len())
            .filter(|&i| !self.rempli(&deblocages[i]))
            .collect();
        let avant = self.maitrise(b.personnage);
        let xp = b.xp.max(0.0).floor() as u64;
        self.c
            .maitrise
            .insert(b.personnage.into(), (avant.xp + xp) as f64);
        let mut record_sans_fin = false;
        let c = &mut self.c;
        let succes: Vec<usize> = (self.d.succes.iter().enumerate())
            .filter(|(_, s)| !c.succes.contains_key(&s.id) && b.remplit(&s.condition))
            .map(|(i, _)| i)
            .collect();
        for &i in &succes {
            c.succes.insert(self.d.succes[i].id.clone(), 1.0);
        }
        if b.en_equipe() {
            // Une partie à plusieurs ne compte pas pour les records du solo
        } else if b.sans_fin {
            c.boss_vaincus += f64::from(b.boss);
            let ancien = c.records_sans_fin.get(b.arene).copied().unwrap_or(0.0);
            record_sans_fin = b.temps > ancien;
            if record_sans_fin {
                c.records_sans_fin.insert(b.arene.into(), b.temps);
            }
        } else {
            c.boss_vaincus += f64::from(b.boss);
            let r = c.records.entry(b.arene.into()).or_insert(0.0);
            *r = r.max(b.temps);
            c.parties += 1.0;
        }
        self.enregistrer();
        Recompense {
            xp,
            avant,
            apres: self.maitrise(b.personnage),
            deblocages: verrouilles
                .into_iter()
                .filter(|&i| self.rempli(&deblocages[i]))
                .collect(),
            record_sans_fin,
            succes,
        }
    }

    /// Essai : un niveau de maîtrise offert à la classe.
    pub fn offrir_niveau(&mut self, personnage: &str) -> EtatMaitrise {
        let fin = self.maitrise(personnage).fin_niveau;
        self.c.maitrise.insert(personnage.into(), fin as f64);
        self.enregistrer();
        self.maitrise(personnage)
    }
}
