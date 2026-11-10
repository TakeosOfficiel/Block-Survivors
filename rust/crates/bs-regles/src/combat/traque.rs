//! Armes qui poursuivent leur cible : orbes arcaniques (tournent, puis foncent) et
//! flèches ricocheuses (port de orbesArcaniques.ts, ricochets.ts), dont les flèches
//! spectrales de l'Œil de l'aigle (réactions, sans rebond). Talents du Mage lus ici :
//! Orbes givrées (ralentissent) et Flux arcanique (l'orbe repart une fois).
//! Le jeu TS garde une référence vers l'ennemi visé ; ici on garde son index et son
//! identifiant, et on le retrouve s'il a changé de place dans le pool.
//! Différence avec le TS : les flèches et les orbes lancées s'arrêtent sur le décor (comme
//! les projectiles), et une flèche ne rebondit pas vers un ennemi caché par un mur.

use bs_commun::maths::{cos, longueur, sin};
use bs_donnees::armes::Mecanique;

use super::tours;
use crate::ennemis::Nature;
use crate::evenements::EffetArme;
use crate::partie::Partie;

const ORBES_CAPACITE: usize = 32;
const ORBES_VOL_MAX: f32 = 3.0;
const ORBES_VITESSE_ORBITE: f32 = 3.0;
const ORBES_RAYON_CONTACT: f32 = 0.45;
const ORBES_DELAI_CONTACT: f32 = 0.5;
/// Orbes givrées : ralenti (part) et durée (s) ; Flux arcanique : part des dégâts par rang.
const ORBES_GIVREES: (f32, f32) = (0.3, 1.0);
const FLUX_ARCANIQUE: [f32; 2] = [0.5, 0.8];

const RICOCHET_CAPACITE: usize = 24;
const RICOCHET_REBONDS_MAX: f32 = 6.0;
const RICOCHET_VIE_MAX: f32 = 4.0;
const RICOCHET_TOUCHES: usize = 8;
/// Distance de rebond d'une flèche de talent, et bonus du Tir rebondissant (blocs).
const RICOCHET_REBOND_TALENT: f32 = 4.0;
const TIR_REBONDISSANT: f32 = 2.0;
/// Rayon des flèches et des orbes lancées contre le décor (comme les projectiles).
const RAYON_DECOR: f32 = 0.1;

/// Ennemi visé : index dans le pool et identifiant.
type Cible = Option<(usize, u32)>;

#[derive(Debug, Clone, Copy)]
pub struct Orbe {
    /// Position de l'arme dans l'arsenal.
    pub arme: usize,
    pub x: f32,
    pub z: f32,
    pub angle: f32,
    pub temps: f32,
    pub lance: bool,
    cible: Cible,
    pub facteur: f32,
    /// Déjà repartie vers un autre ennemi (Flux arcanique).
    revenu: bool,
}

#[derive(Debug, Clone, Copy)]
struct Ricochet {
    arme: usize,
    x: f32,
    z: f32,
    cible: Cible,
    degats: f32,
    rebonds: f32,
    touches: [u32; RICOCHET_TOUCHES],
    nb_touches: usize,
    vie: f32,
    /// Direction du dernier déplacement (affichage seulement).
    dir_x: f32,
    dir_z: f32,
    /// Flèche d'un talent (Œil de l'aigle) : réaction, vitesse de la flèche spectrale.
    reaction: bool,
}

#[derive(Debug, Clone)]
pub struct ArmesTraque {
    pub orbes: Vec<Orbe>,
    ricochets: Vec<Ricochet>,
}

impl Orbe {
    /// Orbe sans cible, pour l'affichage (vitrine).
    pub fn figee(arme: usize, x: f32, z: f32, angle: f32, temps: f32, lance: bool) -> Self {
        Self {
            arme,
            x,
            z,
            angle,
            temps,
            lance,
            cible: None,
            facteur: 1.0,
            revenu: false,
        }
    }
}

impl ArmesTraque {
    /// Flèches ricocheuses en vol, pour l'affichage : (x, z, direction x, direction z).
    pub fn ricochets(&self) -> impl Iterator<Item = [f32; 4]> + '_ {
        self.ricochets.iter().map(|f| [f.x, f.z, f.dir_x, f.dir_z])
    }

    /// Flèches ricocheuses avec l'index (dans l'arsenal) de leur arme.
    pub fn ricochets_armes(&self) -> impl Iterator<Item = (usize, [f32; 4])> + '_ {
        self.ricochets
            .iter()
            .map(|f| (f.arme, [f.x, f.z, f.dir_x, f.dir_z]))
    }
}

impl Default for ArmesTraque {
    fn default() -> Self {
        Self {
            orbes: Vec::with_capacity(ORBES_CAPACITE),
            ricochets: Vec::with_capacity(RICOCHET_CAPACITE),
        }
    }
}

impl Partie {
    /// Index actuel de l'ennemi visé s'il vit encore (il a pu changer de place dans le pool).
    fn retrouver(&self, cible: Cible) -> Option<usize> {
        let (i, id) = cible?;
        let actifs = self.ennemis.actifs();
        let i = if actifs.get(i).is_some_and(|e| e.id == id) {
            i
        } else {
            actifs.iter().position(|e| e.id == id)?
        };
        actifs[i].vivant().then_some(i)
    }

    fn viser(&self, e: Option<usize>) -> Cible {
        e.map(|i| (i, self.ennemis.pool[i].id))
    }

    pub(super) fn frapper_orbes(&mut self, i: usize) -> bool {
        let s = self.arsenal.armes[i].stats;
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        if self.ennemis.plus_proche(jx, jz, s.portee).is_none() {
            return false;
        }
        let depart = (self.alea.tirer() * std::f64::consts::PI * 2.0) as f32;
        for k in 0..tours(s.nombre) {
            if self.armes.traque.orbes.len() >= ORBES_CAPACITE {
                break;
            }
            let angle = depart + (k as f32 / s.nombre) * std::f32::consts::TAU;
            self.armes.traque.orbes.push(Orbe {
                arme: i,
                x: jx + sin(angle) * s.zone,
                z: jz + cos(angle) * s.zone,
                angle,
                temps: 0.0,
                lance: false,
                cible: None,
                facteur: self.talent_facteur_declenchement(),
                revenu: false,
            });
        }
        true
    }

    pub fn maj_orbes_arcaniques(&mut self, dt: f32) {
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let mut tampon = [0u32; 64];
        let mut i = self.armes.traque.orbes.len();
        while i > 0 {
            i -= 1;
            let mut o = self.armes.traque.orbes[i];
            let arme = self.arsenal.armes[o.arme];
            let s = arme.stats;
            o.temps += dt;
            let Mecanique::Orbes {
                contact_au_rang,
                contact,
                ..
            } = arme.def.mecanique
            else {
                self.armes.traque.orbes.swap_remove(i);
                continue;
            };
            if o.temps > s.duree + ORBES_VOL_MAX {
                self.armes.traque.orbes.swap_remove(i);
                continue;
            }

            // Autour du joueur, puis lancée à la fin de sa durée
            if !o.lance {
                let a = o.angle + o.temps * ORBES_VITESSE_ORBITE;
                o.x = jx + sin(a) * s.zone;
                o.z = jz + cos(a) * s.zone;
                if arme.rang as f32 >= contact_au_rang {
                    let n = self
                        .ennemis
                        .chercher(o.x, o.z, ORBES_RAYON_CONTACT, &mut tampon);
                    for &e in &tampon[..n] {
                        let degats = s.degats * contact;
                        self.blesser_avec_delai(
                            e as usize,
                            arme.index,
                            ORBES_DELAI_CONTACT,
                            degats,
                            o.x,
                            o.z,
                            1.0,
                        );
                    }
                }
                if o.temps >= s.duree {
                    o.lance = true;
                }
                self.armes.traque.orbes[i] = o;
                continue;
            }

            let c = match self.retrouver(o.cible) {
                Some(c) => c,
                None => {
                    let e = self
                        .ennemi_au_hasard(o.x, o.z, s.portee)
                        .or_else(|| self.ennemis.plus_proche(o.x, o.z, s.portee * 2.0));
                    let Some(c) = e else {
                        self.armes.traque.orbes.swap_remove(i);
                        continue;
                    };
                    o.cible = self.viser(Some(c));
                    c
                }
            };
            let e = &self.ennemis.pool[c];
            let (dx, dz) = (e.x - o.x, e.z - o.z);
            let d = longueur(dx, dz);
            if d <= e.def.rayon + ORBES_RAYON_CONTACT {
                self.toucher(c, arme.index, &s, o.x, o.z, s.recul, o.facteur);
                if self.talents.rang("orbes_givrees") > 0 {
                    self.ralentir(c, ORBES_GIVREES.0, ORBES_GIVREES.1);
                }
                self.ev.gerbe("impactArcane", o.x, 1.0, o.z);
                self.ev
                    .effet(arme.index, EffetArme::Impact { zone: 0.0 }, o.x, o.z);
                // Flux arcanique : l'orbe repart une fois vers un autre ennemi, plus faible
                let flux =
                    crate::classes::valeur(self.talents.rang("flux_arcanique"), &FLUX_ARCANIQUE);
                if flux > 0.0 && !o.revenu {
                    o.revenu = true;
                    o.facteur *= flux;
                    if let Some(c) = self.ennemi_au_hasard(o.x, o.z, s.portee) {
                        o.cible = self.viser(Some(c));
                        self.armes.traque.orbes[i] = o;
                        continue;
                    }
                }
                self.armes.traque.orbes.swap_remove(i);
                continue;
            }
            let pas = d.min(s.vitesse * dt);
            o.x += dx / d * pas;
            o.z += dz / d * pas;
            // Une orbe lancée se brise sur le décor
            if self
                .carte
                .touche_tir(o.x, o.z, RAYON_DECOR, self.carte.hauteur(o.x, o.z))
            {
                self.ev.gerbe("impactArcane", o.x, 1.0, o.z);
                self.armes.traque.orbes.swap_remove(i);
                continue;
            }
            self.armes.traque.orbes[i] = o;
        }
    }

    pub(super) fn frapper_ricochet(&mut self, i: usize) -> bool {
        let s = self.arsenal.armes[i].stats;
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        if self.ennemis.plus_proche(jx, jz, s.portee).is_none() {
            return false;
        }
        for k in 0..tours(s.nombre) {
            if self.armes.traque.ricochets.len() >= RICOCHET_CAPACITE {
                break;
            }
            let cible = if k == 0 {
                self.ennemis.plus_proche(jx, jz, s.portee)
            } else {
                self.ennemi_au_hasard(jx, jz, s.portee)
            };
            let cible = self.viser(cible);
            self.armes.traque.ricochets.push(Ricochet {
                arme: i,
                x: jx,
                z: jz,
                cible,
                degats: s.degats,
                rebonds: RICOCHET_REBONDS_MAX.min(s.perforation),
                touches: [0; RICOCHET_TOUCHES],
                nb_touches: 0,
                vie: RICOCHET_VIE_MAX,
                dir_x: 0.0,
                dir_z: 1.0,
                reaction: false,
            });
        }
        true
    }

    /// Flèche spectrale d'un talent qui part de (x, z) vers l'ennemi `cible` (Œil de
    /// l'aigle) ; `arme` : position de l'arme dans l'arsenal.
    pub(crate) fn ricocher(&mut self, arme: usize, x: f32, z: f32, cible: usize, degats: f32) {
        if self.armes.traque.ricochets.len() >= RICOCHET_CAPACITE {
            return;
        }
        let cible = self.viser(Some(cible));
        self.armes.traque.ricochets.push(Ricochet {
            arme,
            x,
            z,
            cible,
            degats,
            rebonds: 0.0,
            touches: [0; RICOCHET_TOUCHES],
            nb_touches: 0,
            vie: RICOCHET_VIE_MAX,
            dir_x: 0.0,
            dir_z: 1.0,
            reaction: true,
        });
    }

    /// Ennemi le plus proche à moins de `rayon` que la flèche n'a pas encore touché.
    fn prochaine_cible(&mut self, f: &Ricochet, rayon: f32) -> Cible {
        let mut tampon = [0u32; 64];
        let n = self.ennemis.chercher(f.x, f.z, rayon, &mut tampon);
        let mut meilleur = None;
        let mut meilleure = f32::INFINITY;
        for &k in &tampon[..n] {
            let e = &self.ennemis.pool[k as usize];
            if f.touches[..f.nb_touches].contains(&e.id) {
                continue;
            }
            // Pas de rebond vers un ennemi caché derrière un mur
            if !self.carte.ligne_de_tir(f.x, f.z, e.x, e.z, RAYON_DECOR) {
                continue;
            }
            let d2 = (e.x - f.x) * (e.x - f.x) + (e.z - f.z) * (e.z - f.z);
            if d2 < meilleure {
                meilleure = d2;
                meilleur = Some(k as usize);
            }
        }
        self.viser(meilleur)
    }

    pub fn maj_ricochets(&mut self, dt: f32) {
        let mut i = self.armes.traque.ricochets.len();
        while i > 0 {
            i -= 1;
            let mut f = self.armes.traque.ricochets[i];
            let arme = self.arsenal.armes[f.arme];
            f.vie -= dt;
            let ricochet = match arme.def.mecanique {
                Mecanique::Ricochet {
                    perte,
                    distance_rebond,
                    ..
                } => Some((perte, distance_rebond)),
                _ => None,
            };
            if (ricochet.is_none() && !f.reaction) || f.vie <= 0.0 {
                self.armes.traque.ricochets.swap_remove(i);
                continue;
            }
            let (perte, distance_rebond) = ricochet.unwrap_or((0.0, RICOCHET_REBOND_TALENT));
            let distance_rebond = distance_rebond
                + if self.talents.rang("tir_rebondissant") > 0 {
                    TIR_REBONDISSANT
                } else {
                    0.0
                };
            let c = match self.retrouver(f.cible) {
                Some(c) => c,
                None => {
                    f.cible = self.prochaine_cible(&f, distance_rebond * 2.0);
                    let Some((c, _)) = f.cible else {
                        self.armes.traque.ricochets.swap_remove(i);
                        continue;
                    };
                    c
                }
            };
            let e = &self.ennemis.pool[c];
            let (dx, dz) = (e.x - f.x, e.z - f.z);
            let d = longueur(dx, dz);
            let d = if d == 0.0 { 1e-6 } else { d };
            let (dir_x, dir_z) = (dx / d, dz / d);
            let vitesse = if ricochet.is_some() {
                arme.stats.vitesse
            } else {
                self.d
                    .projectiles
                    .get("fleche_spectrale")
                    .map_or(arme.stats.vitesse, |p| p.vitesse)
            };
            let pas = vitesse * dt;
            if d <= pas + e.def.rayon {
                f.x = e.x;
                f.z = e.z;
                let id = e.id;
                let recul = arme.stats.recul;
                let (sx, sz) = (f.x - dir_x, f.z - dir_z);
                if f.reaction {
                    self.blesser_reaction(
                        c,
                        f.degats,
                        sx,
                        sz,
                        recul,
                        Some(arme.index),
                        Nature::Arme,
                    );
                } else {
                    self.blesser(c, f.degats, sx, sz, recul, Some(arme.index), Nature::Arme);
                }
                if f.nb_touches < RICOCHET_TOUCHES {
                    f.touches[f.nb_touches] = id;
                    f.nb_touches += 1;
                }
                self.ev.gerbe("impactFleche", f.x, 1.2, f.z);
                self.ev
                    .effet(arme.index, EffetArme::Impact { zone: 0.0 }, f.x, f.z);
                self.ev.son("impact_fleche");
                if f.rebonds <= 0.0 {
                    self.armes.traque.ricochets.swap_remove(i);
                    continue;
                }
                f.rebonds -= 1.0;
                f.degats *= 1.0 - perte;
                f.cible = self.prochaine_cible(&f, distance_rebond);
                if f.cible.is_none() {
                    self.armes.traque.ricochets.swap_remove(i);
                    continue;
                }
                self.armes.traque.ricochets[i] = f;
                continue;
            }
            f.x += dir_x * pas;
            f.z += dir_z * pas;
            // Arrêtée par le décor, comme les flèches tirées
            if self
                .carte
                .touche_tir(f.x, f.z, RAYON_DECOR, self.carte.hauteur(f.x, f.z))
            {
                self.ev.gerbe("impactFleche", f.x, 1.2, f.z);
                self.armes.traque.ricochets.swap_remove(i);
                continue;
            }
            f.dir_x = dir_x;
            f.dir_z = dir_z;
            self.armes.traque.ricochets[i] = f;
        }
    }
}
