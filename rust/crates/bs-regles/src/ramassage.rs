//! Ce qui se ramasse : butin des ennemis vaincus, orbes d'XP, objets au sol (soins, tas
//! d'émeraudes) et coffres (port de src/entites/butin.ts, orbes.ts, objets.ts et
//! coffres.ts, sans le rendu). Pools à capacité fixe, libérés par échange avec le dernier
//! comme les pools TS (même ordre de parcours).

use std::f64::consts::PI;

use bs_commun::maths::{atan2, cos, longueur, sin};
use bs_donnees::entites::Ennemi as DefinitionEnnemi;

use crate::etat::gagner_xp;
use crate::partie::Partie;

/// Orbes d'XP simultanées ; au-delà, la valeur rejoint l'orbe la plus proche.
const CAPACITE_ORBES: usize = 1200;
const CAPACITE_OBJETS: usize = 192;
/// Accélération (blocs/s²), vitesse maximale (blocs/s) et distance de ramassage des orbes.
const ATTRACTION: f32 = 30.0;
const VITESSE_MAX: f32 = 14.0;
const DISTANCE_COLLECTE: f32 = 0.6;
/// Distance maximale (cases) où chercher une case accessible pour le butin.
const RAYON_RECHERCHE: i32 = 8;
/// Essais pour placer un coffre à quelques blocs.
const ESSAIS_CASE: u32 = 24;

#[derive(Debug, Clone, Copy)]
pub struct Orbe {
    pub x: f32,
    pub z: f32,
    /// blocs/s, vers le joueur.
    pub vitesse: f32,
    /// Entrée une fois dans le rayon de collecte : suit le joueur jusqu'au bout.
    pub aimantee: bool,
    pub valeur: f32,
    /// Décale la pulsation et le flottement (visuel).
    pub phase: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Objet {
    pub index_type: usize,
    pub x: f32,
    pub z: f32,
    pub phase: f32,
    /// Émeraudes du tas (0 pour un soin).
    pub valeur: f32,
    /// Tas d'émeraudes aspiré vers le joueur, et sa vitesse (blocs/s).
    pub aimante: bool,
    pub vitesse: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Coffre {
    pub index_type: usize,
    pub x: f32,
    pub z: f32,
    pub angle: f32,
    /// None : fermé ; sinon s depuis l'ouverture.
    pub ouvert: Option<f32>,
}

/// Coffres posés sur la carte et le rythme d'apparition des coffres ordinaires.
#[derive(Debug, Clone)]
pub struct Coffres {
    pub pool: Vec<Coffre>,
    /// Ennemis tués à atteindre pour le prochain coffre, et l'écart suivant.
    prochain_seuil: f32,
    intervalle: f32,
    /// Coffres payants déjà ouverts : le prix monte, la rareté des objets aussi.
    pub payants_ouverts: u32,
}

#[derive(Debug, Clone)]
pub struct Ramassables {
    pub orbes: Vec<Orbe>,
    pub objets: Vec<Objet>,
    pub coffres: Coffres,
}

impl Ramassables {
    pub fn nouveaux(d: &bs_donnees::Donnees) -> Self {
        let r = &d.coffres.regles;
        Self {
            orbes: Vec::with_capacity(CAPACITE_ORBES),
            objets: Vec::with_capacity(CAPACITE_OBJETS),
            coffres: Coffres {
                pool: Vec::with_capacity(r.capacite as usize),
                prochain_seuil: r.tues.premier,
                intervalle: r.tues.intervalle,
                payants_ouverts: 0,
            },
        }
    }
}

impl Partie {
    // --- Butin ---

    /// Ce que lâche un ennemi vaincu : son XP, son butin garanti, et parfois un objet.
    /// Un mini-boss lâche aussi un coffre, un boss un coffre de l'Ender (gratuit). Mort
    /// au-dessus de la lave, dans un mur ou hors d'atteinte, il lâche tout sur la case
    /// accessible la plus proche.
    pub fn lacher_butin(&mut self, def: &'static DefinitionEnnemi, x_mort: f32, z_mort: f32) {
        let (x, z) = self.placer_butin(x_mort, z_mort);
        self.deposer_xp(x, z, def.xp);
        if def.mini_boss {
            self.poser_coffre(self.c.coffre, x, z);
        }
        if def.boss.is_some() {
            self.poser_coffre(self.c.coffre_ender, x, z);
        }
        if let Some(butin) = &def.butin {
            if let Some(i) = self.d.objets.get_index_of(butin) {
                self.poser_objet(i, x, z);
            }
            return;
        }
        // Un seul tirage : chaque objet occupe sa part de la probabilité
        let mut tirage = self.alea.tirer();
        for (i, objet) in self.d.objets.values().enumerate() {
            tirage -= f64::from(objet.chance);
            if tirage < 0.0 {
                self.poser_objet(i, x, z);
                return;
            }
        }
    }

    /// Case où le joueur peut marcher sans se blesser et qu'il peut atteindre : là, ou la plus proche autour.
    fn placer_butin(&self, x: f32, z: f32) -> (f32, f32) {
        if self.accessible(x, z) {
            return (x, z);
        }
        let (cx, cz) = (libm::floorf(x) + 0.5, libm::floorf(z) + 0.5);
        let mut meilleure = f32::INFINITY;
        let mut place = (x, z);
        let mut r = 1;
        while r <= RAYON_RECHERCHE && meilleure == f32::INFINITY {
            for dx in -r..=r {
                for dz in -r..=r {
                    // Seulement l'anneau
                    if dx.abs().max(dz.abs()) != r {
                        continue;
                    }
                    let (px, pz) = (cx + dx as f32, cz + dz as f32);
                    if !self.accessible(px, pz) {
                        continue;
                    }
                    let d = (px - x) * (px - x) + (pz - z) * (pz - z);
                    if d < meilleure {
                        meilleure = d;
                        place = (px, pz);
                    }
                }
            }
            r += 1;
        }
        place
    }

    fn accessible(&self, x: f32, z: f32) -> bool {
        !self.carte.bloque_marche(x, z)
            && self.carte.degats_sol(x, z) == 0.0
            && self.flux.atteignable(&self.carte, x, z)
    }

    // --- Orbes d'XP ---

    pub fn deposer_xp(&mut self, x: f32, z: f32, valeur: f32) {
        let orbes = &mut self.ramassables.orbes;
        if orbes.len() >= CAPACITE_ORBES {
            // Pool plein : la valeur rejoint l'orbe la plus proche
            let mut meilleure: Option<usize> = None;
            let mut meilleure_d2 = f32::INFINITY;
            for (i, o) in orbes.iter().enumerate() {
                let d2 = (o.x - x) * (o.x - x) + (o.z - z) * (o.z - z);
                if d2 < meilleure_d2 {
                    meilleure_d2 = d2;
                    meilleure = Some(i);
                }
            }
            if let Some(i) = meilleure {
                orbes[i].valeur += valeur;
            }
            return;
        }
        let phase = (self.alea.tirer() * 100.0) as f32;
        self.ramassables.orbes.push(Orbe {
            x,
            z,
            vitesse: 0.0,
            aimantee: false,
            valeur,
            phase,
        });
    }

    /// Attire vers le joueur les orbes à moins de `rayon` de (x, z) (Allay chanteur).
    pub fn aimanter_orbes(&mut self, x: f32, z: f32, rayon: f32) {
        for o in &mut self.ramassables.orbes {
            if (o.x - x) * (o.x - x) + (o.z - z) * (o.z - z) <= rayon * rayon {
                o.aimantee = true;
            }
        }
    }

    pub fn maj_orbes(&mut self, dt: f32) {
        let j = &self.etat.joueur;
        if j.mort {
            return;
        }
        let (jx, jz) = (j.x, j.z);
        let rayon = self.d.regles.joueur.collecte * self.etat.modificateurs.collecte;
        let mut i = self.ramassables.orbes.len();
        while i > 0 {
            i -= 1;
            let o = &mut self.ramassables.orbes[i];
            let (dx, dz) = (jx - o.x, jz - o.z);
            let d = longueur(dx, dz);
            if d < rayon {
                o.aimantee = true;
            }
            if !o.aimantee {
                continue;
            }
            // Guidage direct, de plus en plus rapide : jamais en orbite autour du joueur
            o.vitesse = VITESSE_MAX.min(o.vitesse + ATTRACTION * dt);
            let pas = o.vitesse * dt;
            if d < DISTANCE_COLLECTE || pas >= d {
                let valeur = o.valeur;
                self.ramassables.orbes.swap_remove(i);
                gagner_xp(self.d, &mut self.etat, valeur);
                self.ev.son("orbe");
                continue;
            }
            o.x += dx / d * pas;
            o.z += dz / d * pas;
        }
    }

    // --- Objets au sol ---

    pub fn poser_objet(&mut self, index_type: usize, x: f32, z: f32) {
        if self.ramassables.objets.len() >= CAPACITE_OBJETS {
            return;
        }
        let phase = (self.alea.tirer() * 10.0) as f32;
        let valeur = match self.d.objets[index_type].emeraudes {
            Some([min, max]) => {
                min + libm::floor(self.alea.tirer() * f64::from(max - min + 1.0)) as f32
            }
            None => 0.0,
        };
        self.ramassables.objets.push(Objet {
            index_type,
            x,
            z,
            phase,
            valeur,
            aimante: false,
            vitesse: 0.0,
        });
    }

    pub fn maj_objets(&mut self, dt: f32) {
        if self.etat.joueur.mort {
            return;
        }
        let portee = self.d.regles.joueur.rayon_ramassage;
        let collecte = self.d.regles.joueur.collecte * self.etat.modificateurs.collecte;
        let mut i = self.ramassables.objets.len();
        while i > 0 {
            i -= 1;
            let o = &mut self.ramassables.objets[i];
            let def = &self.d.objets[o.index_type];
            let j = &mut self.etat.joueur;
            let (dx, dz) = (j.x - o.x, j.z - o.z);
            let d = longueur(dx, dz);
            let m = &self.etat.modificateurs;
            if def.emeraudes.is_some() {
                // Aspiré comme une orbe d'XP : droit sur le joueur, de plus en plus vite
                if d < collecte {
                    o.aimante = true;
                }
                if !o.aimante {
                    continue;
                }
                o.vitesse = VITESSE_MAX.min(o.vitesse + ATTRACTION * dt);
                let pas = o.vitesse * dt;
                if d > DISTANCE_COLLECTE && pas < d {
                    o.x += dx / d * pas;
                    o.z += dz / d * pas;
                    continue;
                }
                let emeraudes = libm::floorf(o.valeur * (1.0 + m.emeraudes) + 0.5) as u32;
                self.etat.emeraudes += emeraudes;
                self.etat.stats.emeraudes_ramassees += emeraudes;
                self.ramassables.objets.swap_remove(i);
                self.ev.son("emeraude");
                continue;
            }
            if d > portee {
                continue;
            }
            let avant = j.pv;
            j.pv = j.pv_max.min(j.pv + def.soin * (1.0 + m.soins_objets));
            self.etat.stats.pv_soignes += j.pv - avant;
            let (ox, oz) = (o.x, o.z);
            self.ramassables.objets.swap_remove(i);
            self.ev.son("manger");
            self.ev.gerbe_icone("ramassage", &def.icone, ox, 0.8, oz);
        }
    }

    // --- Coffres ---

    /// Pose un coffre à cet endroit (ou sur une case accessible proche).
    pub fn poser_coffre(&mut self, index_type: usize, x: f32, z: f32) {
        if self.ramassables.coffres.pool.len() >= self.d.coffres.regles.capacite as usize {
            return;
        }
        let (mut cx, mut cz) = (libm::floorf(x) + 0.5, libm::floorf(z) + 0.5);
        if !self.accessible_coffre(cx, cz)
            && let Some((px, pz)) = self.trouver_case(x, z, 0.0, 6.0)
        {
            cx = px;
            cz = pz;
        }
        // Le loquet face au joueur
        let j = &self.etat.joueur;
        let quart = std::f32::consts::FRAC_PI_2;
        let angle = libm::floorf(atan2(j.x - cx, j.z - cz) / quart + 0.5) * quart;
        self.ramassables.coffres.pool.push(Coffre {
            index_type,
            x: cx,
            z: cz,
            angle,
            ouvert: None,
        });
        self.ev.son("coffre_apparition");
        self.ev.gerbe("invocation", cx, 0.6, cz);
    }

    pub fn maj_coffres(&mut self, dt: f32) {
        let r = &self.d.coffres.regles;
        let (jx, jz, mort) = (
            self.etat.joueur.x,
            self.etat.joueur.z,
            self.etat.joueur.mort,
        );

        // Un coffre tous les tant d'ennemis tués, à quelques blocs du joueur
        let c = &mut self.ramassables.coffres;
        if self.etat.ennemis_tues as f32 >= c.prochain_seuil {
            c.prochain_seuil += c.intervalle;
            c.intervalle += r.tues.croissance;
            let [min, max] = r.distance;
            if let Some((x, z)) = self.trouver_case(jx, jz, min, max) {
                self.poser_coffre(self.c.coffre, x, z);
            }
        }

        let mut i = self.ramassables.coffres.pool.len();
        while i > 0 {
            i -= 1;
            let coffres = &mut self.ramassables.coffres;
            let c = &mut coffres.pool[i];
            let def = &self.d.coffres.coffres[c.index_type];
            if let Some(ouvert) = c.ouvert {
                // Les cartes viennent une fois le couvercle levé
                if ouvert < r.duree_ouverture && ouvert + dt >= r.duree_ouverture {
                    let choix = def.choix as u32;
                    self.etat.choix_coffre.restants += choix;
                    if !def.payant {
                        self.etat.choix_coffre.ender += choix;
                    }
                }
                c.ouvert = Some(ouvert + dt);
                if ouvert + dt >= r.duree_ouverture + r.reste_ouvert {
                    let (x, z) = (c.x, c.z);
                    coffres.pool.swap_remove(i);
                    self.ev.gerbe("invocation", x, 0.5, z);
                }
                continue;
            }
            let a_portee = (jx - c.x) * (jx - c.x) + (jz - c.z) * (jz - c.z)
                <= r.rayon_ouverture * r.rayon_ouverture;
            if mort || !a_portee {
                continue;
            }
            // Le prix est le même pour tous les coffres payants, il monte à chaque ouverture
            let prix = if def.payant {
                (r.prix.base + r.prix.increment * coffres.payants_ouverts as f32) as u32
            } else {
                0
            };
            if self.etat.emeraudes < prix {
                continue;
            }
            // Ouverture : on paie, le couvercle se lève, les cartes suivent
            self.etat.emeraudes -= prix;
            if def.payant {
                coffres.payants_ouverts += 1;
            }
            c.ouvert = Some(0.0);
            self.etat.stats.coffres_ouverts += 1;
            let (x, z) = (c.x, c.z);
            self.ev.son(&def.son);
            self.ev.gerbe("niveau", x, 1.0, z);
        }
    }

    /// Case accessible au hasard entre `min` et `max` blocs de (x, z).
    fn trouver_case(&mut self, x: f32, z: f32, min: f32, max: f32) -> Option<(f32, f32)> {
        for _ in 0..ESSAIS_CASE {
            let angle = (self.alea.tirer() * PI * 2.0) as f32;
            let d = min + (self.alea.tirer() * f64::from(max - min)) as f32;
            let cx = libm::floorf(x + sin(angle) * d) + 0.5;
            let cz = libm::floorf(z + cos(angle) * d) + 0.5;
            if self.accessible_coffre(cx, cz) {
                return Some((cx, cz));
            }
        }
        None
    }

    fn accessible_coffre(&self, x: f32, z: f32) -> bool {
        let demi_x = self.etat.arene.largeur / 2.0 - 1.0;
        let demi_z = self.etat.arene.profondeur / 2.0 - 1.0;
        x.abs() <= demi_x && z.abs() <= demi_z && self.accessible(x, z)
    }
}
