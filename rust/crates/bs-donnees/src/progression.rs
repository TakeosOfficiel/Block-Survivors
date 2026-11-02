//! Progression : bonus passifs et courbe d'XP pendant la partie ; entre les parties,
//! arbres de talents, maîtrise des classes et déblocages.

use serde::{Deserialize, Serialize};

use crate::communs::{ModifArme, Modifs, Table, Texture, faux};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NiveauPassif {
    pub description: String,
    /// Ajouts aux modificateurs (cumulés rang après rang).
    pub modif: Modifs,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Consommable {
    pub reserve_max: f32,
    pub libelle: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Passif {
    pub nom: String,
    pub description: String,
    pub icone: Texture,
    /// Réservé aux personnages qui le listent dans `cartesExclusives`.
    #[serde(default, skip_serializing_if = "faux")]
    pub exclusif: bool,
    /// Objet à usage unique (totem), sans rangs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consommable: Option<Consommable>,
    /// Effets des rangs 1, 2… ; rang maximal = longueur.
    pub niveaux: Vec<NiveauPassif>,
}

pub type Passifs = Table<Passif>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PalierXp {
    /// Dernier niveau de la tranche (aucun : tous les suivants).
    pub jusqu_au_niveau: Option<u32>,
    pub increment: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CarteSoin {
    pub nom: String,
    pub description: String,
    pub icone: Texture,
    pub pv: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageOrbe {
    pub valeur_min: f32,
    pub image: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SansFin {
    pub pv_par_minute: f32,
    pub degats_par_minute: f32,
    pub groupe_par_minute: f32,
    pub groupe_max: f32,
}

/// Courbe d'expérience, cartes d'amélioration et orbes d'XP.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Progression {
    pub xp_premier_niveau: f32,
    /// Durée d'une partie (s) : survivre jusque-là, c'est gagner.
    pub duree_partie: f32,
    pub paliers: Vec<PalierXp>,
    pub cartes_par_niveau: u32,
    pub relances: u32,
    pub emplacements_armes: u32,
    pub emplacements_passifs: u32,
    pub soin: CarteSoin,
    pub orbes: Vec<ImageOrbe>,
    pub sans_fin: SansFin,
}

// ---------------------------------------------------------------------------
// Talents
// ---------------------------------------------------------------------------

/// Bonus d'un talent pour une arme précise, cumulé à chaque rang.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BonusArmeTalent {
    pub arme: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ajout: Option<ModifArme>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<ModifArme>,
    /// Alliés invoqués par l'arme : PV et vitesse en plus (parts).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pv: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vitesse: Option<f32>,
}

/// Ce que fait un talent (ou une option de talent à choix), cumulé à chaque rang.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Gains {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modif: Option<Modifs>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub armes: Vec<BonusArmeTalent>,
    /// Rangs offerts de cette arme au départ.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rangs_arme: Option<String>,
    /// Commence avec cette arme, au rang du talent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arme_depart: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relances: Option<f32>,
    /// Effet propre, géré par le combat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mecanique: Option<String>,
}

/// Champs de `Gains`, rangés à part par l'importeur (à plat en TypeScript).
pub const CHAMPS_GAINS: [&str; 6] = [
    "modif",
    "armes",
    "rangsArme",
    "armeDepart",
    "relances",
    "mecanique",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionTalent {
    pub nom: String,
    pub icone: Texture,
    pub effet: String,
    #[serde(default, skip_serializing_if = "est_vide")]
    pub gains: Gains,
}

fn est_vide(g: &Gains) -> bool {
    *g == Gains::default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeNoeud {
    Bonus,
    Capacite,
    Choix,
    Final,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Branche {
    Tronc,
    Gauche,
    Centre,
    Droite,
    Libre,
}

/// Un talent de l'arbre, à sa place (ligne 0 en haut, colonnes 0 à 6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Noeud {
    /// Identifiant stable (sauvegarde).
    pub id: String,
    pub ligne: u32,
    pub colonne: u32,
    #[serde(rename = "type")]
    pub genre: TypeNoeud,
    pub branche: Branche,
    pub nom: String,
    pub icone: Texture,
    /// Effet affiché (tous rangs).
    pub effet: String,
    pub rang_max: u32,
    /// Talents reliés au-dessus : il faut qu'un d'eux soit complet.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prerequis: Vec<String>,
    #[serde(default, skip_serializing_if = "faux")]
    pub libre: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<[OptionTalent; 2]>,
    #[serde(default, skip_serializing_if = "est_vide")]
    pub gains: Gains,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Arbre {
    pub id: String,
    pub nom: String,
    pub description: String,
    pub icone: Texture,
    pub branches: indexmap::IndexMap<Branche, String>,
    pub noeuds: Vec<Noeud>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Palier {
    pub ligne: u32,
    pub rangs: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Maitrise {
    pub icone: Texture,
    pub premier_niveau: u32,
    pub increment: u32,
}

impl Maitrise {
    /// XP de maîtrise totale pour atteindre ce niveau.
    pub fn xp_pour_niveau(&self, niveau: u32) -> u64 {
        let n = u64::from(niveau);
        n * u64::from(self.premier_niveau)
            + u64::from(self.increment) * (n * n.saturating_sub(1)) / 2
    }

    /// Niveau de maîtrise atteint avec cette XP.
    pub fn niveau(&self, xp: u64) -> u32 {
        let mut niveau = 0;
        while self.xp_pour_niveau(niveau + 1) <= xp {
            niveau += 1;
        }
        niveau
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Condition {
    Survie {
        duree: f32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        arene: Option<String>,
    },
    Boss {
        nombre: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeDeblocage {
    Personnage,
    Arene,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Deblocage {
    #[serde(rename = "type")]
    pub genre: TypeDeblocage,
    pub id: String,
    pub nom: String,
    pub condition: Condition,
    pub texte: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Talents {
    pub paliers: Vec<Palier>,
    /// Mécaniques propres déjà en jeu (les autres sont « en préparation »).
    pub mecaniques_pretes: Vec<String>,
    /// Deux arbres par personnage.
    pub arbres: Table<[Arbre; 2]>,
    pub maitrise: Maitrise,
    pub deblocages: Vec<Deblocage>,
    /// Succès de coop, gagnés une fois pour toutes.
    #[serde(default)]
    pub succes: Vec<Succes>,
}

/// Succès : gagné à la fin d'une partie qui remplit toute sa condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Succes {
    pub id: String,
    pub nom: String,
    pub texte: String,
    pub condition: ConditionSucces,
}

/// Ce que la partie doit remplir ; un champ laissé à zéro (ou faux) n'est pas exigé.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConditionSucces {
    /// Joueurs au moins dans la partie, soi compris.
    pub joueurs: u8,
    /// Partie gagnée (arène finie).
    pub victoire: bool,
    /// Boss vaincus au moins.
    pub boss: u32,
    /// Tous les joueurs ont la même classe.
    pub meme_classe: bool,
    /// Temps survécu au moins (s).
    pub survie: f32,
}

/// Effets en jeu de l'arbre actif d'une classe (src/progression/sauvegarde.ts `bonus`),
/// cumulés rang après rang.
#[derive(Debug, Clone, Default)]
pub struct BonusTalents {
    pub modif: Modifs,
    /// Rangs d'armes offerts au départ (une entrée par rang).
    pub armes: Vec<&'static str>,
    pub relances: u32,
    /// Bonus d'armes précises (une entrée par rang).
    pub bonus_armes: Vec<&'static BonusArmeTalent>,
    /// Mécaniques propres actives et leur rang.
    pub mecaniques: Vec<(&'static str, u32)>,
}

/// Talent actif d'un arbre : identifiant, rang et option choisie (ce qu'un joueur envoie
/// à l'hôte en coop pour qu'il recalcule ses bonus).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoeudActif {
    pub noeud: String,
    pub rang: u32,
    #[serde(default)]
    pub option: usize,
}

impl BonusTalents {
    /// Cumule les effets des talents `actifs` de l'arbre `arbre` du personnage (inconnus
    /// ignorés, rangs bornés au maximum).
    pub fn depuis_noeuds<'a>(
        talents: &'static Talents,
        personnage: &str,
        arbre: usize,
        actifs: impl IntoIterator<Item = (&'a str, u32, usize)>,
    ) -> Self {
        let mut b = Self::default();
        let Some(arbre) = talents.arbres.get(personnage).and_then(|a| a.get(arbre)) else {
            return b;
        };
        for (id, rang, option) in actifs {
            let Some(noeud) = arbre.noeuds.iter().find(|n| n.id == id) else {
                continue;
            };
            let rang = rang.min(noeud.rang_max);
            let effet: &'static Gains = match &noeud.options {
                Some(options) => &options[option.min(1)].gains,
                None => &noeud.gains,
            };
            for _ in 0..rang {
                if let Some(m) = &effet.modif {
                    b.modif.cumuler(m);
                }
                b.bonus_armes.extend(effet.armes.iter());
                b.armes.extend(effet.rangs_arme.as_deref());
                b.armes.extend(effet.arme_depart.as_deref());
                b.relances += effet.relances.unwrap_or(0.0) as u32;
            }
            if let Some(m) = &effet.mecanique {
                // Comme un dictionnaire : la dernière entrée l'emporte
                b.mecaniques.retain(|(n, _)| n != m);
                b.mecaniques.push((m.as_str(), rang));
            }
        }
        b
    }
}
