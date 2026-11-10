//! Armes posées au sol : pluie de flèches sur le plus gros groupe et pièges de ronces sous
//! le joueur (port de pluies.ts, pieges.ts, sans les talents). Les flèches qui tombent
//! sont tirées dans l'aléa (position, orientation) : on les garde toutes.

use bs_commun::maths::{cos, sin};
use bs_donnees::armes::Mecanique;

use super::tours;
use crate::classes::valeur;
use crate::ennemis::Nature;
use crate::evenements::EffetArme;
use crate::partie::Partie;

const PLUIE_ZONES: usize = 8;
const PLUIE_FLECHES: usize = 320;
pub const PLUIE_CHUTE: f32 = 0.3;
pub const PLUIE_PLANTEE: f32 = 2.2;
pub const PLUIE_ENFONCEMENT: f32 = 0.4;
const PLUIE_RAYON_IMPACT: f32 = 0.9;
/// Le cercle au sol reste puis s'efface (s) : la zone vit jusque-là.
pub const PLUIE_CERCLE_RESTE: f32 = 1.2;
pub const PLUIE_CERCLE_FONDU: f32 = 0.6;

const PIEGE_CAPACITE: usize = 12;
const PIEGE_RECHERCHE: f32 = 12.0;
const PIEGE_ENRACINEMENT: f32 = 0.5;
const PIEGE_DELAI_ENRACINEMENT: f32 = 3.0;
/// Talents : enracinement à l'entrée (s, par rang), propagation du poison (s).
const RONCES_COLLANTES: [f32; 2] = [0.5, 1.0];
const PERIODE_PROPAGATION: f32 = 1.0;

#[derive(Debug, Clone, Copy)]
pub struct ZonePluie {
    /// Position de l'arme dans l'arsenal.
    pub arme: usize,
    pub x: f32,
    pub z: f32,
    pub rayon: f32,
    pub duree: f32,
    pub temps: f32,
    pub prochaine_fleche: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct FlecheTombante {
    pub arme: usize,
    pub x: f32,
    pub z: f32,
    pub age: f32,
    pub touche: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Piege {
    pub arme: usize,
    pub x: f32,
    pub z: f32,
    pub temps: f32,
    pub prochain_tic: f32,
    /// s avant la prochaine propagation du poison (Ronces venimeuses).
    pub propagation: f32,
}

#[derive(Debug, Clone)]
pub struct ArmesPosees {
    pub zones: Vec<ZonePluie>,
    pub fleches: Vec<FlecheTombante>,
    pub pieges: Vec<Piege>,
}

impl Default for ArmesPosees {
    fn default() -> Self {
        Self {
            zones: Vec::with_capacity(PLUIE_ZONES),
            fleches: Vec::with_capacity(PLUIE_FLECHES),
            pieges: Vec::with_capacity(PIEGE_CAPACITE),
        }
    }
}

impl Partie {
    pub(super) fn frapper_pluie(&mut self, i: usize) -> bool {
        let s = self.arsenal.armes[i].stats;
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let mut lancee = false;
        for _ in 0..tours(s.nombre) {
            let Some((x, z)) = self.plus_gros_groupe(jx, jz, s.portee, s.zone) else {
                break;
            };
            if self.armes.posees.zones.len() >= PLUIE_ZONES {
                break;
            }
            self.armes.posees.zones.push(ZonePluie {
                arme: i,
                x,
                z,
                rayon: s.zone,
                duree: s.duree,
                temps: 0.0,
                prochaine_fleche: 0.0,
            });
            lancee = true;
        }
        lancee
    }

    pub fn maj_pluies(&mut self, dt: f32) {
        let mut i = self.armes.posees.zones.len();
        while i > 0 {
            i -= 1;
            let mut z = self.armes.posees.zones[i];
            let arme = self.arsenal.armes[z.arme];
            z.temps += dt;
            let Mecanique::Pluie {
                impacts_par_seconde,
                ..
            } = arme.def.mecanique
            else {
                self.armes.posees.zones.swap_remove(i);
                continue;
            };
            if z.temps > z.duree + PLUIE_CHUTE + PLUIE_CERCLE_RESTE + PLUIE_CERCLE_FONDU {
                self.armes.posees.zones.swap_remove(i);
                continue;
            }
            if z.temps <= z.duree {
                z.prochaine_fleche -= dt;
                while z.prochaine_fleche <= 0.0 {
                    z.prochaine_fleche += 1.0 / impacts_par_seconde;
                    if self.armes.posees.fleches.len() >= PLUIE_FLECHES {
                        break;
                    }
                    let a = (self.alea.tirer() * std::f64::consts::PI * 2.0) as f32;
                    let r = z.rayon * self.alea.tirer().sqrt() as f32;
                    // Azimut et inclinaison (visuels)
                    self.alea.tirer();
                    self.alea.tirer();
                    self.armes.posees.fleches.push(FlecheTombante {
                        arme: z.arme,
                        x: z.x + sin(a) * r,
                        z: z.z + cos(a) * r,
                        age: 0.0,
                        touche: false,
                    });
                }
            }
            self.armes.posees.zones[i] = z;
        }

        let mut tampon = [0u32; 64];
        let mut i = self.armes.posees.fleches.len();
        while i > 0 {
            i -= 1;
            let mut f = self.armes.posees.fleches[i];
            f.age += dt;
            if !f.touche && f.age >= PLUIE_CHUTE {
                f.touche = true;
                let arme = self.arsenal.armes[f.arme];
                let s = arme.stats;
                let n = self
                    .ennemis
                    .chercher(f.x, f.z, PLUIE_RAYON_IMPACT, &mut tampon);
                for &e in &tampon[..n] {
                    self.toucher(e as usize, arme.index, &s, f.x, f.z, s.recul, 1.0);
                }
                self.ev.gerbe("impactFleche", f.x, 0.2, f.z);
                self.ev
                    .effet(arme.index, EffetArme::Impact { zone: 0.0 }, f.x, f.z);
                if n > 0 {
                    self.ev.son("impact_fleche");
                }
            }
            if f.age >= PLUIE_CHUTE + PLUIE_PLANTEE + PLUIE_ENFONCEMENT {
                self.armes.posees.fleches.swap_remove(i);
            } else {
                self.armes.posees.fleches[i] = f;
            }
        }
    }

    pub(super) fn frapper_piege(&mut self, i: usize) -> bool {
        let s = self.arsenal.armes[i].stats;
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        if self.ennemis.plus_proche(jx, jz, PIEGE_RECHERCHE).is_none() {
            return false;
        }
        // Au-delà du nombre de pièges, le plus ancien de cette arme disparaît
        let pieges = &mut self.armes.posees.pieges;
        let mut poses = 0;
        let mut ancien: Option<usize> = None;
        for (k, p) in pieges.iter().enumerate() {
            if p.arme != i {
                continue;
            }
            poses += 1;
            if ancien.is_none_or(|a| p.temps > pieges[a].temps) {
                ancien = Some(k);
            }
        }
        if poses as f32 >= s.nombre
            && let Some(a) = ancien
        {
            pieges.swap_remove(a);
        }
        if pieges.len() >= PIEGE_CAPACITE {
            return false;
        }
        pieges.push(Piege {
            arme: i,
            x: jx,
            z: jz,
            temps: 0.0,
            prochain_tic: 0.0,
            propagation: 0.0,
        });
        // Graine des buissons (visuelle)
        self.alea.tirer();
        let image = match &self.arsenal.armes[i].def.mecanique {
            Mecanique::Piege { image, .. } => image.as_str(),
            _ => "block/sweet_berry_bush_stage3",
        };
        self.ev.gerbe_icone("ronces", image, jx, 0.3, jz);
        true
    }

    pub fn maj_pieges(&mut self, dt: f32) {
        let temps = self.etat.temps_jeu;
        let mut tampon = [0u32; 128];
        let collantes = valeur(self.talents.rang("ronces_collantes"), &RONCES_COLLANTES);
        let venimeuses = self.talents.rang("ronces_venimeuses") > 0;
        let max_poison = self.talent_cumuls_poison();
        let mut i = self.armes.posees.pieges.len();
        while i > 0 {
            i -= 1;
            let mut p = self.armes.posees.pieges[i];
            let arme = self.arsenal.armes[p.arme];
            let s = arme.stats;
            p.temps += dt;
            p.propagation -= dt;
            let Mecanique::Piege {
                intervalle,
                enracine_au_rang,
                ..
            } = arme.def.mecanique
            else {
                self.armes.posees.pieges.swap_remove(i);
                continue;
            };
            if p.temps >= s.duree {
                self.armes.posees.pieges.swap_remove(i);
                continue;
            }
            p.prochain_tic -= dt;
            if p.prochain_tic > 0.0 {
                self.armes.posees.pieges[i] = p;
                continue;
            }
            p.prochain_tic += intervalle;
            self.armes.posees.pieges[i] = p;
            let n = self.ennemis.chercher(p.x, p.z, s.zone, &mut tampon);
            let mut source: Option<usize> = None;
            for &e in &tampon[..n] {
                let e = e as usize;
                let en = &mut self.ennemis.pool[e];
                // Pris jusqu'au prochain passage ; à l'entrée, les Ronces collantes enracinent
                en.piege = temps + f64::from(intervalle * 1.5);
                if !en.dans_piege {
                    en.dans_piege = true;
                    if collantes > 0.0 {
                        self.enraciner(e, collantes);
                    }
                }
                let cumuls = self.ennemis.pool[e].poison.cumuls;
                if cumuls > 0.0
                    && source.is_none_or(|k| cumuls > self.ennemis.pool[k].poison.cumuls)
                {
                    source = Some(e);
                }
                self.blesser(e, s.degats, p.x, p.z, 0.0, Some(arme.index), Nature::Arme);
                self.ralentir(e, s.ralenti, intervalle * 1.5);
                if arme.rang as f32 >= enracine_au_rang
                    && f64::from(self.ennemis.pool[e].immunites[arme.index]) <= temps
                {
                    self.enraciner(e, PIEGE_ENRACINEMENT);
                    self.ennemis.pool[e].immunites[arme.index] =
                        (temps + f64::from(PIEGE_DELAI_ENRACINEMENT)) as f32;
                }
            }
            // Ronces venimeuses : une fois par seconde, le poison passe aux autres
            if venimeuses
                && p.propagation <= 0.0
                && let Some(k) = source
            {
                p.propagation = PERIODE_PROPAGATION;
                self.armes.posees.pieges[i] = p;
                let poison = self.ennemis.pool[k].poison;
                let dps = poison.degats_par_seconde / (1.0 + self.etat.modificateurs.degats_poison);
                for &e in &tampon[..n] {
                    let e = e as usize;
                    if e != k && self.ennemis.pool[e].vivant() {
                        self.empoisonner_max(e, 1.0, dps, poison.temps, poison.arme, max_poison);
                    }
                }
            }
        }
    }
}
