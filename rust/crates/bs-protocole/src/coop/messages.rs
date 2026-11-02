//! Messages fiables de la coop (src/reseau/protocole.ts `MessageFiable`), en JSON : rares
//! (salon, lancement, cartes, coffres, bilans, fin), ils peuvent allouer. L'hôte n'a qu'un
//! envoi vers tous les invités : un message pour un seul porte sa `place`. Les cartes et
//! les objets passent par leur identifiant (pas leur index), pour rester lisibles et
//! rejeter proprement une donnée inconnue.

use bs_donnees::arenes::Monde;
use bs_donnees::equipement::Piece;
use bs_donnees::progression::{BonusTalents, NoeudActif};
use bs_regles::Partie;
use bs_regles::cartes::Amelioration;
use bs_regles::equipement::ChoixEquipement;
use bs_regles::etat::{Issue, Statistiques};
use serde::{Deserialize, Serialize};

/// Qui est un joueur : envoyé à l'hôte en arrivant, puis à tous dans le salon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
    /// Version du jeu : l'hôte refuse une version différente.
    pub version: String,
    pub pseudo: String,
    /// Identifiant du personnage.
    pub personnage: String,
    /// Skin vanilla, ou « importe » (PNG dans `skin_importe`, en base64).
    pub skin: String,
    pub bras_fins: bool,
    pub skin_importe: Option<String>,
    /// Arbre de talents actif du personnage et ses talents.
    pub arbre: usize,
    pub talents: Vec<NoeudActif>,
    /// Pièces d'armure de la Collection (bonus dans l'arène de leur monde).
    pub armures: Vec<(Monde, Piece)>,
}

impl Presentation {
    pub fn bonus(&self, d: &'static bs_donnees::Donnees) -> BonusTalents {
        let actifs = self
            .talents
            .iter()
            .map(|n| (n.noeud.as_str(), n.rang, n.option));
        BonusTalents::depuis_noeuds(&d.talents, &self.personnage, self.arbre, actifs)
    }
}

/// Un joueur du salon (index 0 : l'hôte).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InfoJoueur {
    pub index: u8,
    #[serde(flatten)]
    pub presentation: Presentation,
}

/// Carte choisie : amélioration d'un niveau, ou objet d'un coffre (jamais « retour »).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "genre", rename_all = "camelCase")]
pub enum CarteJouee {
    Arme { arme: String },
    Eveil { arme: String, variante: usize },
    Passif { passif: String },
    Soin,
    Objet { objet: String },
    Renfort { objet: String },
    Remplacer { objet: String, index: usize },
    Emeraudes { nombre: u32 },
}

impl CarteJouee {
    pub fn depuis_amelioration(d: &bs_donnees::Donnees, a: Amelioration) -> Self {
        let arme = |i: usize| d.armes.get_index(i).map_or("", |(id, _)| id).to_owned();
        match a {
            Amelioration::Arme(i) => Self::Arme { arme: arme(i) },
            Amelioration::Eveil { arme: i, variante } => Self::Eveil {
                arme: arme(i),
                variante,
            },
            Amelioration::Passif(i) => Self::Passif {
                passif: d.passifs.get_index(i).map_or("", |(id, _)| id).to_owned(),
            },
            Amelioration::Soin => Self::Soin,
        }
    }

    /// `None` pour « retour », qui reste chez le joueur.
    pub fn depuis_equipement(d: &bs_donnees::Donnees, c: ChoixEquipement) -> Option<Self> {
        let objet = |i: usize| {
            d.equipement
                .objets
                .get_index(i)
                .map_or("", |(id, _)| id)
                .to_owned()
        };
        Some(match c {
            ChoixEquipement::Objet(i) => Self::Objet { objet: objet(i) },
            ChoixEquipement::Renfort(i) => Self::Renfort { objet: objet(i) },
            ChoixEquipement::Remplacer { objet: i, index } => Self::Remplacer {
                objet: objet(i),
                index,
            },
            ChoixEquipement::Emeraudes(nombre) => Self::Emeraudes { nombre },
            ChoixEquipement::Retour => return None,
        })
    }

    /// Joue la carte pour le joueur actif de `p` ; faux si elle désigne une donnée inconnue.
    pub fn jouer(&self, p: &mut Partie) -> bool {
        let d = p.d;
        let arme = |id: &str| d.armes.get_index_of(id);
        let objet = |id: &str| d.equipement.objets.get_index_of(id);
        let amelioration = match self {
            Self::Arme { arme: a } => arme(a).map(Amelioration::Arme),
            Self::Eveil { arme: a, variante } => arme(a).map(|i| Amelioration::Eveil {
                arme: i,
                variante: (*variante).min(1),
            }),
            Self::Passif { passif } => d
                .passifs
                .get_index_of(passif.as_str())
                .map(Amelioration::Passif),
            Self::Soin => Some(Amelioration::Soin),
            _ => None,
        };
        if let Some(a) = amelioration {
            p.appliquer_amelioration(a);
            return true;
        }
        let choix = match self {
            Self::Objet { objet: o } => objet(o).map(ChoixEquipement::Objet),
            Self::Renfort { objet: o } => objet(o).map(ChoixEquipement::Renfort),
            Self::Remplacer { objet: o, index } => {
                objet(o).map(|objet| ChoixEquipement::Remplacer {
                    objet,
                    index: *index,
                })
            }
            Self::Emeraudes { nombre } => Some(ChoixEquipement::Emeraudes(*nombre)),
            _ => None,
        };
        choix.map(|c| p.appliquer_equipement(c)).is_some()
    }

    /// Carte d'un coffre (et non d'une montée de niveau).
    pub fn de_coffre(&self) -> bool {
        matches!(
            self,
            Self::Objet { .. }
                | Self::Renfort { .. }
                | Self::Remplacer { .. }
                | Self::Emeraudes { .. }
        )
    }
}

/// Bilan d'un joueur (compteur de dégâts, écran de fin).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsReseau {
    pub tues_par_type: Vec<u32>,
    pub degats_par_arme: Vec<f64>,
    pub coups_par_arme: Vec<u32>,
    pub critiques_par_arme: Vec<u32>,
    pub plus_gros_coup_par_arme: Vec<f32>,
    pub tues_par_arme: Vec<u32>,
    pub acquisition_arme: Vec<f64>,
    pub degats_subis: f32,
    pub pv_soignes: f32,
    pub xp_totale: f32,
    pub emeraudes_ramassees: u32,
    pub coffres_ouverts: u32,
    pub boss_vaincus: u32,
}

impl From<&Statistiques> for StatsReseau {
    fn from(s: &Statistiques) -> Self {
        Self {
            tues_par_type: s.tues_par_type.clone(),
            degats_par_arme: s.degats_par_arme.clone(),
            coups_par_arme: s.coups_par_arme.clone(),
            critiques_par_arme: s.critiques_par_arme.clone(),
            plus_gros_coup_par_arme: s.plus_gros_coup_par_arme.clone(),
            tues_par_arme: s.tues_par_arme.clone(),
            acquisition_arme: s.acquisition_arme.clone(),
            degats_subis: s.degats_subis,
            pv_soignes: s.pv_soignes,
            xp_totale: s.xp_totale,
            emeraudes_ramassees: s.emeraudes_ramassees,
            coffres_ouverts: s.coffres_ouverts,
            boss_vaincus: s.boss_vaincus,
        }
    }
}

impl StatsReseau {
    /// Recopie dans `s` (les tableaux gardent leur taille : données de même version).
    pub fn copier_dans(&self, s: &mut Statistiques) {
        fn copier<T: Copy>(de: &[T], vers: &mut [T]) {
            for (v, d) in vers.iter_mut().zip(de) {
                *v = *d;
            }
        }
        copier(&self.tues_par_type, &mut s.tues_par_type);
        copier(&self.degats_par_arme, &mut s.degats_par_arme);
        copier(&self.coups_par_arme, &mut s.coups_par_arme);
        copier(&self.critiques_par_arme, &mut s.critiques_par_arme);
        copier(
            &self.plus_gros_coup_par_arme,
            &mut s.plus_gros_coup_par_arme,
        );
        copier(&self.tues_par_arme, &mut s.tues_par_arme);
        copier(&self.acquisition_arme, &mut s.acquisition_arme);
        s.degats_subis = self.degats_subis;
        s.pv_soignes = self.pv_soignes;
        s.xp_totale = self.xp_totale;
        s.emeraudes_ramassees = self.emeraudes_ramassees;
        s.coffres_ouverts = self.coffres_ouverts;
        s.boss_vaincus = self.boss_vaincus;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IssueReseau {
    Victoire,
    Defaite,
}

impl From<Issue> for IssueReseau {
    fn from(i: Issue) -> Self {
        match i {
            Issue::Victoire => Self::Victoire,
            Issue::Defaite => Self::Defaite,
        }
    }
}

impl From<IssueReseau> for Issue {
    fn from(i: IssueReseau) -> Self {
        match i {
            IssueReseau::Victoire => Self::Victoire,
            IssueReseau::Defaite => Self::Defaite,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MessageFiable {
    /// Joueur → hôte, dès l'arrivée dans le salon.
    Bonjour(Presentation),
    /// Hôte → le joueur de la place `place` : refusé (version, partie en cours).
    Refus {
        place: u8,
        raison: String,
    },
    /// Hôte → tous : composition du salon et arène choisie.
    Salon {
        arene: String,
        joueurs: Vec<InfoJoueur>,
    },
    /// Hôte → tous : chacun charge l'arène, puis répond `Pret`.
    Lancer {
        arene: String,
        joueurs: Vec<InfoJoueur>,
    },
    Pret,
    /// Hôte → tous : tout le monde est prêt, la partie commence.
    Depart,
    /// Joueur → hôte : carte choisie.
    Carte {
        carte: CarteJouee,
    },
    /// Hôte → tous : carte choisie par le joueur `index`.
    CarteJouee {
        index: u8,
        carte: CarteJouee,
    },
    /// Hôte → tous : coffres ouverts (`ender` : dont ceux d'un coffre de l'Ender).
    Coffre {
        nombre: u32,
        ender: u32,
    },
    /// Hôte → un joueur : son bilan, régulièrement et à la fin.
    Stats {
        place: u8,
        stats: StatsReseau,
    },
    /// Hôte → un joueur : fin de partie au temps de jeu `temps` (s).
    Fin {
        place: u8,
        issue: IssueReseau,
        temps: f64,
        stats: StatsReseau,
    },
    /// Joueur → hôte : « Continuer sans fin » ; hôte → tous : la partie reprend.
    Continuer,
    Reprise,
}

impl MessageFiable {
    pub fn json(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    pub fn lire(octets: &[u8]) -> Option<Self> {
        serde_json::from_slice(octets).ok()
    }
}
