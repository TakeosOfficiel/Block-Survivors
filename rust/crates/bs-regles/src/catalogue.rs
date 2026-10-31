//! Références vers les données souvent lues pendant la partie, résolues une fois au
//! lancement (pas de recherche par identifiant dans la boucle de jeu).

use bs_donnees::Donnees;
use bs_donnees::effets::Etat;

/// États altérés appliqués aux ennemis.
#[derive(Debug, Clone, Copy)]
pub struct EtatsRef {
    pub brulure: &'static Etat,
    pub poison: &'static Etat,
    pub saignement: &'static Etat,
    pub ralenti: &'static Etat,
    pub etourdi: &'static Etat,
    pub marque: &'static Etat,
}

#[derive(Debug, Clone, Copy)]
pub struct Catalogue {
    pub etats: EtatsRef,
    /// Index des coffres ordinaire et de l'Ender.
    pub coffre: usize,
    pub coffre_ender: usize,
}

impl Catalogue {
    pub fn nouveau(d: &'static Donnees) -> Self {
        let etat = |id: &str| {
            d.regles
                .etats
                .get(id)
                .unwrap_or_else(|| panic!("état « {id} » absent des règles"))
        };
        let coffre = |id: &str| d.coffres.coffres.get_index_of(id).unwrap_or_else(|| panic!("coffre « {id} » absent"));
        Self {
            etats: EtatsRef {
                brulure: etat("brulure"),
                poison: etat("poison"),
                saignement: etat("saignement"),
                ralenti: etat("ralenti"),
                etourdi: etat("etourdi"),
                marque: etat("marque"),
            },
            coffre: coffre("coffre"),
            coffre_ender: coffre("coffre_ender"),
        }
    }
}
