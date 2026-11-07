//! Valeurs des talents du Guerrier (V de guerrier.ts, fissures de config.ts) et rangs
//! lus une fois, au début de la partie.

use crate::talents::Talents;

pub(super) const ENCHAINEMENT_TOUCHES: u32 = 3;
pub(super) const ENCHAINEMENT_DELAI: f32 = 0.85;
pub(super) const FRENESIE_ELIMINATIONS: u32 = 10;
pub(super) const FRENESIE_FENETRE: f64 = 3.0;
pub(super) const FRENESIE_DUREE: f32 = 4.0;
pub(super) const FRENESIE_DELAI: [f32; 2] = [0.85, 0.75];
pub(super) const FRENESIE_SOIN: f32 = 1.0;
pub(super) const CHAMPION_ELIMINATIONS: u32 = 6;
pub(super) const CHAMPION_DUREE: f32 = 6.0;
pub(super) const FUREUR_DUREE: f32 = 5.0;
pub(super) const FUREUR_DEGATS: [f32; 2] = [0.05, 0.1];
pub(super) const BOURREAU_SEUIL: f32 = 0.3;
pub(super) const BOURREAU_DEGATS: [f32; 2] = [0.2, 0.35];
pub(super) const ELAN_ZONE: f32 = 1.4;
pub(super) const ELAN_DEGATS: f32 = 1.3;
pub(super) const ELAN_DELAI: f32 = 1.5;
/// Saignée : cumuls, dégâts par seconde, durée.
pub(super) const SAIGNEE: (f32, f32, f32) = (1.0, 4.0, 3.0);
pub(super) const LAME_LOURDE: [f32; 2] = [10.0, 20.0];
pub(super) const SOIF: [f32; 2] = [0.1, 0.2];
pub(super) const FUREUR_ATTAQUES: u32 = 5;
pub(super) const FUREUR_RAYON: f32 = 3.5;
pub(super) const FUREUR_PART: f32 = 0.6;
pub(super) const FUREUR_DELAI: f32 = 3.0;
pub(super) const EXECUTION_SEUIL: f32 = 0.15;
pub(super) const EXECUTION_DELAI: f32 = 2.0;
pub(super) const EXECUTION_BOSS: f32 = 0.3;
/// Posture défensive : rayon, force, délai interne.
pub(super) const POSTURE: (f32, f32, f32) = (3.0, 12.0, 3.0);
pub(super) const EPINES: [f32; 2] = [10.0, 18.0];
pub(super) const EPINES_RAYON: f32 = 2.5;
pub(super) const EPINES_DELAI: f32 = 2.0;
pub(super) const RONCES_RAYON: [f32; 2] = [0.5, 1.0];
pub(super) const RONCES_RALENTI: f32 = 0.3;
pub(super) const RONCES_DUREE: f32 = 1.0;
pub(super) const REMPART_PERIODE: f32 = 12.0;
pub(super) const REMPART_BOUCLIER: f32 = 4.0;
pub(super) const INEBRANLABLE: f32 = 0.9;
pub(super) const PEAU_SEUIL: f32 = 0.5;
pub(super) const PEAU_ARMURE: [f32; 2] = [1.0, 2.0];
pub(super) const TENIR_IMMOBILE: f32 = 1.0;
pub(super) const TENIR_REDUCTION: [f32; 2] = [0.1, 0.2];
pub(super) const BASTION_CHARGE: f32 = 30.0;
pub(super) const BASTION_RAYON: f32 = 4.0;
pub(super) const BASTION_DEGATS: f32 = 40.0;
pub(super) const BASTION_MAX: f32 = 80.0;
pub(super) const BASTION_DELAI: f32 = 6.0;
pub(super) const REMPART_FINAL_SEUIL: f32 = 0.25;
pub(super) const REMPART_FINAL_PROTECTION: f32 = 3.0;
pub(super) const REMPART_FINAL_RAYON: f32 = 5.0;
pub(super) const REMPART_FINAL_FORCE: f32 = 16.0;
pub(super) const SEISME_ONDES: u32 = 3;
/// Onde de choc : délai des répliques (s), part de leurs dégâts, directions du Séisme.
pub const REPLIQUE_DELAI: f32 = 0.5;
pub const REPLIQUES: [f32; 2] = [0.4, 0.6];
pub const DIRECTIONS_SEISME: f32 = 6.0;

/// Rangs des talents à mécanique de l'arbre actif.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rangs {
    pub enchainement: u32,
    pub frenesie: u32,
    pub champion: u32,
    pub fureur: u32,
    pub bourreau: u32,
    pub elan: u32,
    pub saignee: u32,
    pub lame_lourde: u32,
    pub soif: u32,
    pub fureur_gladiateur: u32,
    pub execution: u32,
    pub posture: u32,
    pub epines: u32,
    pub ronces_acier: u32,
    pub rempart: u32,
    pub inebranlable: u32,
    pub peau_de_fer: u32,
    pub tenir: u32,
    pub bastion: u32,
    pub dernier_rempart: u32,
    pub seisme: u32,
    pub repliques: u32,
}

impl Rangs {
    pub fn lire(t: &Talents) -> Self {
        Self {
            enchainement: t.rang("enchainement"),
            frenesie: t.rang("frenesie"),
            champion: t.rang("champion_arene"),
            fureur: t.rang("fureur"),
            bourreau: t.rang("bourreau"),
            elan: t.rang("elan_critique"),
            saignee: t.rang("saignee"),
            lame_lourde: t.rang("lame_lourde"),
            soif: t.rang("soif_combat"),
            fureur_gladiateur: t.rang("fureur_gladiateur"),
            execution: t.rang("execution"),
            posture: t.rang("posture_defensive"),
            epines: t.rang("epines"),
            ronces_acier: t.rang("ronces_acier"),
            rempart: t.rang("rempart"),
            inebranlable: t.rang("inebranlable"),
            peau_de_fer: t.rang("peau_de_fer"),
            tenir: t.rang("tenir_la_ligne"),
            bastion: t.rang("contre_attaque_bastion"),
            dernier_rempart: t.rang("dernier_rempart"),
            seisme: t.rang("seisme"),
            repliques: t.rang("repliques"),
        }
    }
}
