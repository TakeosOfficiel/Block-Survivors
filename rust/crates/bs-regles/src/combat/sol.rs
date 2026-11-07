//! Armes qui frappent le sol : onde de choc (fissures), lame colossale qui tombe du ciel
//! (chutes) et nova de glace (port de fissures.ts, chutes.ts, novas.ts). Les blocs
//! soulevés sont visuels, mais tirent dans l'aléa : on garde leur compte. Talent du Mage
//! lu ici : Zéro absolu (nova gelante, plus forte, qui enracine). La nova s'arrête aux murs
//! pleins (`a_couvert`) ; la lame qui tombe du ciel, non.

use bs_commun::maths::{atan2, cos, sin};
use bs_donnees::armes::Mecanique;

use super::tours;
use crate::classes::{guerrier, valeur};
use crate::evenements::EffetArme;
use crate::partie::Partie;

const FISSURE_CAPACITE: usize = 12;
const FISSURE_BLOCS: usize = 320;
const FISSURE_RAYON_GROUPE: f32 = 2.0;
const FISSURE_DEPART: f32 = 1.0;
const FISSURE_ESPACEMENT: f32 = 0.8;
const FISSURE_TAILLE_BLOC: f32 = 0.9;
pub const FISSURE_DUREE_BLOC: f32 = 0.7;
const FISSURE_IMMUNITE: f32 = 0.45;

const CHUTE_CAPACITE: usize = 6;
pub const CHUTE_DUREE: f32 = 0.4;
pub const CHUTE_TENUE: f32 = 0.4;
pub const CHUTE_FONDU: f32 = 0.3;
const CHUTE_DECALAGE: f32 = 0.25;

const NOVA_CAPACITE: usize = 4;
const NOVA_IMMUNITE: f32 = 0.6;
pub const NOVA_ECLATS_RESTENT: f32 = 0.35;
const NOVA_PERIODE_SOL: f32 = 0.25;
/// Zéro absolu : dégâts multipliés et enracinement (s) de la nova gelante.
const ZERO_ABSOLU: (f32, f32) = (2.0, 1.0);

/// Onde qui avance en ligne droite et soulève le sol tous les `FISSURE_ESPACEMENT` blocs.
#[derive(Debug, Clone, Copy)]
struct Fissure {
    /// Position de l'arme dans l'arsenal.
    arme: usize,
    x: f32,
    z: f32,
    dir_x: f32,
    dir_z: f32,
    avance: f32,
    prochain: f32,
    attente: f32,
    facteur: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Chute {
    pub arme: usize,
    pub x: f32,
    pub z: f32,
    pub t: f32,
    pub tombee: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Nova {
    pub arme: usize,
    pub x: f32,
    pub z: f32,
    pub temps: f32,
    pub sol: bool,
    pub prochain_sol: f32,
    pub facteur: f32,
    /// Nova gelante (Zéro absolu).
    pub gel: bool,
}

#[derive(Debug, Clone)]
pub struct ArmesSol {
    fissures: Vec<Fissure>,
    /// Âge des blocs soulevés (visuels).
    blocs: Vec<f32>,
    pub chutes: Vec<Chute>,
    pub novas: Vec<Nova>,
}

impl Default for ArmesSol {
    fn default() -> Self {
        Self {
            fissures: Vec::with_capacity(FISSURE_CAPACITE),
            blocs: Vec::with_capacity(FISSURE_BLOCS),
            chutes: Vec::with_capacity(CHUTE_CAPACITE),
            novas: Vec::with_capacity(NOVA_CAPACITE),
        }
    }
}

impl Partie {
    /// Vrai si l'ennemi `e` est encore immunisé contre l'arme ; sinon l'immunise `delai` s.
    pub(crate) fn immunise(&mut self, e: usize, arme: usize, delai: f32) -> bool {
        let temps = self.etat.temps_jeu;
        let imm = &mut self.ennemis.pool[e].immunites[arme];
        if f64::from(*imm) > temps {
            return true;
        }
        *imm = (temps + f64::from(delai)) as f32;
        false
    }

    pub(super) fn frapper_fissure(&mut self, i: usize, ecart: f32) -> bool {
        let s = self.arsenal.armes[i].stats;
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let Some((gx, gz)) = self.plus_gros_groupe(jx, jz, s.portee, FISSURE_RAYON_GROUPE) else {
            return false;
        };
        let angle = atan2(gx - jx, gz - jz);
        // Séisme : 6 fissures en étoile au lieu de l'éventail
        let circulaire = self.talents.classes.onde_circulaire;
        let nombre = if circulaire {
            guerrier::DIRECTIONS_SEISME
        } else {
            s.nombre
        };
        let replique = valeur(self.talents.rang("repliques"), &guerrier::REPLIQUES);
        for k in 0..tours(nombre) {
            let a = if circulaire {
                angle + k as f32 / nombre * std::f32::consts::TAU
            } else {
                angle + (k as f32 - (nombre - 1.0) / 2.0) * ecart
            };
            self.lancer_fissure(i, (jx, jz), a, 0.0, 1.0);
            if replique > 0.0 {
                self.lancer_fissure(i, (jx, jz), a, guerrier::REPLIQUE_DELAI, replique);
            }
        }
        self.etat.joueur.angle = angle;
        self.ev.geste(None);
        self.ev.secousse(0.15);
        true
    }

    fn lancer_fissure(
        &mut self,
        arme: usize,
        (x, z): (f32, f32),
        a: f32,
        attente: f32,
        facteur: f32,
    ) {
        if self.armes.sol.fissures.len() < FISSURE_CAPACITE {
            self.armes.sol.fissures.push(Fissure {
                arme,
                x,
                z,
                dir_x: sin(a),
                dir_z: cos(a),
                avance: 0.0,
                prochain: FISSURE_DEPART,
                attente,
                facteur,
            });
        }
    }

    pub fn maj_fissures(&mut self, dt: f32) {
        let mut i = self.armes.sol.fissures.len();
        while i > 0 {
            i -= 1;
            let mut f = self.armes.sol.fissures[i];
            if f.attente > 0.0 {
                self.armes.sol.fissures[i].attente -= dt;
                continue;
            }
            let arme = self.arsenal.armes[f.arme];
            let Mecanique::Fissure { vitesse, .. } = arme.def.mecanique else {
                self.armes.sol.fissures.swap_remove(i);
                continue;
            };
            f.avance += vitesse * dt;
            let mut finie = false;
            while f.prochain <= f.avance {
                if f.prochain > arme.stats.portee {
                    finie = true;
                    break;
                }
                let x = f.x + f.dir_x * f.prochain;
                let z = f.z + f.dir_z * f.prochain;
                if self.carte.bloque_marche(x, z) {
                    finie = true;
                    break;
                }
                self.soulever(f.arme, x, z, f.facteur);
                f.prochain += FISSURE_ESPACEMENT;
            }
            if finie {
                self.armes.sol.fissures.swap_remove(i);
            } else {
                self.armes.sol.fissures[i] = f;
            }
        }
        let blocs = &mut self.armes.sol.blocs;
        let mut i = blocs.len();
        while i > 0 {
            i -= 1;
            blocs[i] += dt;
            if blocs[i] >= FISSURE_DUREE_BLOC {
                blocs.swap_remove(i);
            }
        }
    }

    fn soulever(&mut self, i: usize, x: f32, z: f32, facteur: f32) {
        let arme = self.arsenal.armes[i];
        let s = arme.stats;
        let largeur = libm::floorf(s.zone / FISSURE_TAILLE_BLOC + 0.5).max(1.0) as u32;
        for _ in 0..largeur {
            if self.armes.sol.blocs.len() >= FISSURE_BLOCS {
                break;
            }
            self.armes.sol.blocs.push(0.0);
            // Position, rotation et inclinaison du bloc
            for _ in 0..4 {
                self.alea.tirer();
            }
        }
        self.ev
            .gerbe_icone("eboulis", self.texture_sol(), x, 0.4, z);
        self.ev
            .effet(arme.index, EffetArme::Soulevement { zone: s.zone }, x, z);
        let mut tampon = [0u32; 64];
        let n = self.ennemis.chercher(x, z, s.zone / 2.0, &mut tampon);
        for &e in &tampon[..n] {
            let e = e as usize;
            if self.immunise(e, arme.index, FISSURE_IMMUNITE) {
                continue;
            }
            self.toucher(e, arme.index, &s, x, z, s.recul, facteur);
        }
    }

    pub(super) fn frapper_chute(&mut self, i: usize) -> bool {
        let s = self.arsenal.armes[i].stats;
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let mut lancee = false;
        for k in 0..tours(s.nombre) {
            let Some((x, z)) = self.plus_gros_groupe(jx, jz, s.portee, s.zone) else {
                break;
            };
            if self.armes.sol.chutes.len() >= CHUTE_CAPACITE {
                break;
            }
            self.armes.sol.chutes.push(Chute {
                arme: i,
                x,
                z,
                t: -(k as f32) * CHUTE_DECALAGE,
                tombee: false,
            });
            lancee = true;
        }
        lancee
    }

    pub fn maj_chutes(&mut self, dt: f32) {
        let mut i = self.armes.sol.chutes.len();
        let mut tampon = [0u32; 128];
        while i > 0 {
            i -= 1;
            let mut c = self.armes.sol.chutes[i];
            c.t += dt;
            if !c.tombee && c.t >= CHUTE_DUREE {
                c.tombee = true;
                let arme = self.arsenal.armes[c.arme];
                let s = arme.stats;
                let n = self.ennemis.chercher(c.x, c.z, s.zone, &mut tampon);
                for &e in &tampon[..n] {
                    self.toucher(e as usize, arme.index, &s, c.x, c.z, s.recul, 1.0);
                }
                self.ev.gerbe("impactLame", c.x, 0.3, c.z);
                self.ev
                    .gerbe_icone("eboulis", self.texture_sol(), c.x, 0.3, c.z);
                self.ev.son("lame_colossale");
                self.ev.secousse(0.35);
            }
            if c.t >= CHUTE_DUREE + CHUTE_TENUE + CHUTE_FONDU {
                self.armes.sol.chutes.swap_remove(i);
            } else {
                self.armes.sol.chutes[i] = c;
            }
        }
    }

    pub(super) fn frapper_nova(&mut self, i: usize, sol_au_rang: f32) -> bool {
        let arme = self.arsenal.armes[i];
        let (jx, jz) = (self.etat.joueur.x, self.etat.joueur.z);
        let mut tampon = [0u32; 256];
        if self.ennemis.chercher(jx, jz, arme.stats.zone, &mut tampon) == 0 {
            return false;
        }
        if self.armes.sol.novas.len() >= NOVA_CAPACITE {
            return false;
        }
        // Graine de l'ondulation des éclats (visuelle)
        self.alea.tirer();
        let gel = self.talents.classes.nova_gel;
        self.armes.sol.novas.push(Nova {
            arme: i,
            x: jx,
            z: jz,
            temps: 0.0,
            sol: arme.rang as f32 >= sol_au_rang,
            prochain_sol: 0.0,
            facteur: self.talent_facteur_declenchement() * if gel { ZERO_ABSOLU.0 } else { 1.0 },
            gel,
        });
        true
    }

    pub fn maj_novas(&mut self, dt: f32) {
        let mut i = self.armes.sol.novas.len();
        let mut tampon = [0u32; 256];
        while i > 0 {
            i -= 1;
            let mut n = self.armes.sol.novas[i];
            n.temps += dt;
            let arme = self.arsenal.armes[n.arme];
            let Mecanique::Nova {
                expansion,
                duree_sol,
                ..
            } = arme.def.mecanique
            else {
                self.armes.sol.novas.swap_remove(i);
                continue;
            };
            let s = arme.stats;
            if n.temps <= expansion + dt {
                let rayon = s.zone * (n.temps / expansion).min(1.0);
                let k = self.ennemis.chercher(n.x, n.z, rayon, &mut tampon);
                for &e in &tampon[..k] {
                    let e = e as usize;
                    if self.a_couvert(n.x, n.z, e) || self.immunise(e, arme.index, NOVA_IMMUNITE) {
                        continue;
                    }
                    self.toucher(e, arme.index, &s, n.x, n.z, s.recul, n.facteur);
                    if n.gel {
                        self.enraciner(e, ZERO_ABSOLU.1);
                    }
                    let (ex, ez) = (self.ennemis.pool[e].x, self.ennemis.pool[e].z);
                    let texture = match &arme.def.mecanique {
                        Mecanique::Nova { texture, .. } => texture.as_str(),
                        _ => "block/packed_ice",
                    };
                    self.ev.gerbe_icone("gel", texture, ex, 0.8, ez);
                }
            } else if n.sol {
                n.prochain_sol -= dt;
                if n.prochain_sol <= 0.0 {
                    n.prochain_sol = NOVA_PERIODE_SOL;
                    let k = self.ennemis.chercher(n.x, n.z, s.zone, &mut tampon);
                    for &e in &tampon[..k] {
                        if self.a_couvert(n.x, n.z, e as usize) {
                            continue;
                        }
                        self.ralentir(e as usize, s.ralenti, NOVA_PERIODE_SOL * 2.0);
                    }
                }
            }
            let fin = expansion + NOVA_ECLATS_RESTENT + if n.sol { duree_sol } else { 0.0 };
            if n.temps >= fin {
                self.armes.sol.novas.swap_remove(i);
            } else {
                self.armes.sol.novas[i] = n;
            }
        }
    }
}
