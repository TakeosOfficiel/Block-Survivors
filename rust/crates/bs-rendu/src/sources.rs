//! Armes à dessiner (src/reseau/hote.ts et client.ts, option « Effets des autres joueurs ») :
//! celles du joueur, puis, en coop et si l'option est active, celles de chaque coéquipier
//! encore dans la partie. L'hôte les simule pour de vrai, l'invité pour l'effet
//! (`Partie::simuler_invite`) : les deux n'ont qu'à les montrer.

use bs_regles::Partie;
use bs_regles::combat::invocations::Allie;
use bs_regles::combat::{Arsenal, SystemesArmes};
use bs_regles::etat::EtatJoueur;

/// Les armes d'un combattant, son arsenal (stats, définitions) et sa position.
#[derive(Clone, Copy)]
pub struct Source<'a> {
    pub armes: &'a SystemesArmes,
    pub arsenal: &'a Arsenal,
    pub joueur: &'a EtatJoueur,
}

/// Le joueur, puis les coéquipiers présents si `allies` (hors d'un pas : le combattant 0
/// est bien le joueur de cette machine).
pub fn sources(p: &Partie, allies: bool) -> impl Iterator<Item = Source<'_>> + Clone {
    let moi = Source {
        armes: &p.armes,
        arsenal: &p.arsenal,
        joueur: &p.etat.joueur,
    };
    let autres = p
        .equipe
        .membres
        .iter()
        .filter(move |c| allies && !c.parti)
        .map(|c| Source {
            armes: &c.armes,
            arsenal: &c.arsenal,
            joueur: &c.etat_joueur,
        });
    std::iter::once(moi).chain(autres)
}

/// Créatures invoquées à dessiner, chacune avec une clé unique (chaque combattant numérote
/// les siennes depuis 1 : son rang dans les sources fait la différence).
pub fn invocations(p: &Partie, allies: bool) -> impl Iterator<Item = (u64, &Allie)> + Clone {
    sources(p, allies).enumerate().flat_map(|(n, s)| {
        s.armes
            .invocations
            .pool
            .iter()
            .map(move |a| (((n as u64) << 32) | u64::from(a.id), a))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bs_donnees::Donnees;

    #[test]
    fn coequipiers_selon_l_option() {
        let d: &'static Donnees = Box::leak(Box::new(Donnees::embarquees().unwrap()));
        let mut p = Partie::nouvelle(d, 0, 0, 1, true);
        assert_eq!(sources(&p, true).count(), 1);
        p.ajouter_combattant(1);
        p.ajouter_combattant(2);
        assert_eq!(sources(&p, true).count(), 3);
        assert_eq!(sources(&p, false).count(), 1);
        p.retirer_combattant(1);
        assert_eq!(sources(&p, true).count(), 2);
    }
}
