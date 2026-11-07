//! Arme possédée par le joueur : rang, statistiques calculées, minuterie
//! (port de src/combat/armeEquipee.ts), avec les bonus des talents propres à l'arme.

use bs_donnees::armes::{Arme, Eveil, FamilleArme, Mecanique};
use bs_donnees::progression::BonusArmeTalent;
use bs_donnees::{Donnees, ModifArme, StatsArme};

use crate::etat::Modificateurs;

/// Bonus de talents d'une arme précise, cumulés rang après rang (nuls sans talent).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BonusArmeCumule {
    pub ajout: ModifArme,
    pub part: ModifArme,
    /// Alliés invoqués : PV et vitesse en plus (parts).
    pub pv: f32,
    pub vitesse: f32,
}

impl BonusArmeCumule {
    pub fn cumuler(&mut self, b: &BonusArmeTalent) {
        if let Some(a) = &b.ajout {
            self.ajout.cumuler(a);
        }
        if let Some(p) = &b.part {
            self.part.cumuler(p);
        }
        self.pv += b.pv.unwrap_or(0.0);
        self.vitesse += b.vitesse.unwrap_or(0.0);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ArmeEquipee {
    /// Index dans la table des armes (immunités des ennemis, statistiques).
    pub index: usize,
    pub def: &'static Arme,
    pub rang: u32,
    pub stats: StatsArme,
    /// Bonus des talents propres à cette arme.
    pub talent: BonusArmeCumule,
    /// Éveil choisi (index dans `def.eveils`).
    pub eveil: Option<usize>,
    /// s avant le prochain déclenchement.
    pub minuterie: f32,
}

impl ArmeEquipee {
    pub fn nouvelle(
        d: &'static Donnees,
        index: usize,
        m: &Modificateurs,
        talent: BonusArmeCumule,
    ) -> Self {
        let def = &d.armes[index];
        let mut arme = Self {
            index,
            def,
            rang: 1,
            stats: def.stats,
            talent,
            eveil: None,
            minuterie: 0.0,
        };
        arme.recalculer(d, m);
        arme
    }

    pub fn rang_max(&self) -> u32 {
        rang_max(self.def)
    }

    /// Éveil choisi, ou aucun.
    pub fn eveil_choisi(&self) -> Option<&'static Eveil> {
        let def: &'static Arme = self.def;
        self.eveil.and_then(|i| def.eveils.as_ref().map(|e| &e[i]))
    }

    pub fn recalculer(&mut self, d: &Donnees, m: &Modificateurs) {
        self.stats = calculer_stats(d, self.def, self.rang, m, &self.talent, self.eveil_choisi());
    }
}

pub fn rang_max(def: &Arme) -> u32 {
    1 + def.niveaux.len() as u32
}

fn ajouter(s: &mut StatsArme, m: &ModifArme) {
    s.degats += m.degats;
    s.cadence += m.cadence;
    s.nombre += m.nombre;
    s.portee += m.portee;
    s.vitesse += m.vitesse;
    s.zone += m.zone;
    s.duree += m.duree;
    s.perforation += m.perforation;
    s.recul += m.recul;
    s.ralenti += m.ralenti;
    s.etourdi += m.etourdi;
}

fn multiplier_parts(s: &mut StatsArme, m: &ModifArme) {
    s.degats *= 1.0 + m.degats;
    s.cadence *= 1.0 + m.cadence;
    s.nombre *= 1.0 + m.nombre;
    s.portee *= 1.0 + m.portee;
    s.vitesse *= 1.0 + m.vitesse;
    s.zone *= 1.0 + m.zone;
    s.duree *= 1.0 + m.duree;
    s.perforation *= 1.0 + m.perforation;
    s.recul *= 1.0 + m.recul;
    s.ralenti *= 1.0 + m.ralenti;
    s.etourdi *= 1.0 + m.etourdi;
}

/// Statistiques d'une arme au rang donné : base + améliorations des rangs atteints et
/// ajouts des talents, puis modificateurs globaux et de sa famille, puis parts des
/// talents et de l'éveil, dans leurs plafonds.
pub fn calculer_stats(
    d: &Donnees,
    def: &Arme,
    rang: u32,
    m: &Modificateurs,
    talent: &BonusArmeCumule,
    eveil: Option<&Eveil>,
) -> StatsArme {
    let mut s = def.stats;
    for niveau in def.niveaux.iter().take(rang.saturating_sub(1) as usize) {
        ajouter(&mut s, &niveau.modif);
    }
    ajouter(&mut s, &talent.ajout);
    if let Some(a) = eveil.and_then(|e| e.ajout.as_ref()) {
        ajouter(&mut s, a);
    }
    let (mut degats, mut cadence, mut zone) = (m.degats, m.cadence, m.zone);
    match def.famille {
        Some(FamilleArme::Melee) => {
            degats += m.degats_melee;
            zone += m.zone_melee;
        }
        Some(FamilleArme::Projectile) => {
            degats += m.degats_projectiles;
            s.vitesse *= 1.0 + m.vitesse_projectiles;
        }
        Some(FamilleArme::Feu) => {
            degats += m.degats_feu;
            cadence += m.cadence_feu;
        }
        Some(FamilleArme::Allie) => {
            degats += m.degats_allies;
            s.duree += m.duree_allies;
        }
        None => {}
    }
    let p = &d.regles.plafonds;
    s.degats *= degats;
    s.cadence *= p.delai_min.max(cadence);
    s.zone *= zone;
    s.portee *= m.portee;
    s.recul *= m.recul;
    multiplier_parts(&mut s, &talent.part);
    if let Some(part) = eveil.and_then(|e| e.part.as_ref()) {
        multiplier_parts(&mut s, part);
    }
    if d.regles
        .profite_projectiles
        .get(def.mecanique.nom())
        .copied()
        .unwrap_or(false)
    {
        s.nombre += p.projectiles_bonus.min(m.projectiles);
    }
    if let Mecanique::Invocation { allie, .. } = &def.mecanique
        && let Some(allie) = d.allies.allies.get(allie)
    {
        // Les alliés en plus qui dépasseraient le quota deviennent des dégâts
        if allie.renforts {
            s.nombre += m.allies;
        }
        let surplus = (s.nombre - allie.quota).max(0.0);
        s.nombre -= surplus;
        s.degats *= 1.0 + surplus * p.surplus_allie;
    }
    s
}
