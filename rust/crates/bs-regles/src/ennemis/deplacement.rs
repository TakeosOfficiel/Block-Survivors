//! Déplacement générique des ennemis (comportement, séparation, recul, collisions) et
//! leurs comportements : corps à corps, tireur, explosif (src/entites/comportements.ts).

use bs_commun::maths::{atan2, cos, ecart_angle, facteur_lissage, longueur, racine, sin};
use bs_donnees::entites::Comportement;

use super::{Ennemi, PERIODE_VUE};
use crate::partie::Partie;

const SEPARATION: f32 = 6.0;
const AMORTISSEMENT_RECUL: f32 = 6.0;
const VITESSE_ROTATION: f32 = 10.0;
/// Recul minimal (blocs/s) pour un choc contre un mur (Lame lourde).
const CHOC_MUR: f32 = 4.0;
/// Rayon des projectiles contre le décor (lignes de tir).
pub const RAYON_DECOR_TIR: f32 = 0.1;

/// Direction et distance vers la cible, et chemin à suivre pour l'atteindre.
#[derive(Debug, Clone, Copy)]
struct Vers {
    /// Direction unitaire en ligne droite (pour viser).
    dx: f32,
    dz: f32,
    distance: f32,
    /// Aucun bloc n'arrête un tir vers elle.
    ligne_de_tir: bool,
    /// Direction unitaire à suivre en contournant le décor.
    chemin_x: f32,
    chemin_z: f32,
}

/// Qui l'ennemi frappe au contact.
#[derive(Debug, Clone, Copy)]
enum Cible {
    Joueur,
    Allie(usize),
}

/// Tir décidé par un comportement (exécuté hors de l'emprunt de l'ennemi).
struct Tir {
    projectile: &'static str,
    x: f32,
    z: f32,
    base: f32,
    nombre: f32,
    dispersion: f32,
    degats: f32,
    boss: bool,
}

/// Ce que décide le comportement : un coup au contact, un tir, ou une explosion.
#[derive(Default)]
struct Action {
    coup: Option<f32>,
    tir: Option<Tir>,
    explose: bool,
}

/// Écrit e.dir_x / e.dir_z (direction voulue, 0 = immobile) et l'attaque décidée.
fn appliquer_comportement(e: &mut Ennemi, dt: f32, v: &Vers, rayon_joueur: f32) -> Action {
    let mut action = Action::default();
    let def = e.def;
    match &def.comportement {
        Comportement::Melee {
            degats,
            portee,
            cadence,
        } => {
            e.dir_x = v.chemin_x;
            e.dir_z = v.chemin_z;
            let contact = e.def.rayon + rayon_joueur + portee;
            if v.distance <= contact && e.minuterie <= 0.0 {
                action.coup = Some(*degats);
                e.minuterie = *cadence;
            }
        }
        // Se place à sa distance idéale et tire quand la cible est à portée ; cachée derrière
        // un bloc, il la contourne au lieu de tirer
        Comportement::Archer {
            projectile,
            degats,
            distance_ideale,
            portee,
            cadence,
            nombre,
            dispersion,
        } => {
            if !v.ligne_de_tir || v.distance > distance_ideale + 1.0 {
                e.dir_x = v.chemin_x;
                e.dir_z = v.chemin_z;
            } else if v.distance < distance_ideale - 1.5 {
                e.dir_x = -v.dx;
                e.dir_z = -v.dz;
            } else {
                e.dir_x = 0.0;
                e.dir_z = 0.0;
            }
            if v.ligne_de_tir && v.distance <= *portee && e.minuterie <= 0.0 {
                action.tir = Some(Tir {
                    projectile,
                    x: e.x + v.dx * 0.5,
                    z: e.z + v.dz * 0.5,
                    base: atan2(v.dx, v.dz),
                    nombre: nombre.unwrap_or(1.0),
                    dispersion: dispersion.unwrap_or(0.0),
                    degats: *degats,
                    boss: e.boss(),
                });
                e.minuterie = *cadence;
            }
        }
        // S'approche, puis s'arrête et gonfle ; la mèche redescend si la cible s'éloigne
        Comportement::Explosif {
            distance_amorcage,
            distance_annulation,
            duree,
            ..
        } => {
            let amorce = (e.meche > 0.0 || v.distance < *distance_amorcage)
                && v.distance < *distance_annulation;
            if amorce {
                e.meche += dt;
                e.dir_x = 0.0;
                e.dir_z = 0.0;
            } else {
                e.meche = (e.meche - dt).max(0.0);
                e.dir_x = v.chemin_x;
                e.dir_z = v.chemin_z;
            }
            action.explose = e.meche >= *duree;
        }
    }
    action
}

impl Partie {
    pub(super) fn deplacer(&mut self, index: usize, dt: f32) {
        let rayon_joueur = self.d.regles.joueur.rayon;
        let (jx, jz, j_mort) = (
            self.etat.joueur.x,
            self.etat.joueur.z,
            self.etat.joueur.mort,
        );
        let e = &mut self.ennemis.pool[index];
        let def = e.def;
        if e.minuterie > 0.0 {
            e.minuterie -= dt;
        }
        if e.blessure > 0.0 {
            e.blessure -= dt;
        }

        // Vers le joueur : en ligne droite s'il est en vue, sinon par le champ de flux
        let (vers_x, vers_z) = (jx - e.x, jz - e.z);
        let distance = longueur(vers_x, vers_z);
        let distance = if distance == 0.0 { 1e-6 } else { distance };
        let (dx, dz) = (vers_x / distance, vers_z / distance);
        let au_contact = matches!(def.comportement, Comportement::Melee { .. });
        e.minuterie_vue -= dt;
        let mut verifier_provocation = false;
        if e.minuterie_vue <= 0.0 {
            e.minuterie_vue = PERIODE_VUE;
            e.voit_joueur = def.ignore_obstacles || self.carte.ligne_de_vue(e.x, e.z, jx, jz);
            if matches!(def.comportement, Comportement::Archer { .. }) {
                e.ligne_de_tir = self.carte.ligne_de_tir(e.x, e.z, jx, jz, RAYON_DECOR_TIR);
            }
            verifier_provocation = au_contact;
        }
        if verifier_provocation {
            // Un allié proche (plus que le joueur) attire les ennemis au corps à corps
            let (ex, ez) = (self.ennemis.pool[index].x, self.ennemis.pool[index].z);
            let a = self.allie_provocateur(ex, ez, distance);
            self.ennemis.pool[index].cible_allie = a;
        }
        let e = &self.ennemis.pool[index];
        let mut v = Vers {
            dx,
            dz,
            distance,
            ligne_de_tir: e.ligne_de_tir,
            chemin_x: dx,
            chemin_z: dz,
        };
        if !e.voit_joueur
            && let Some((cx, cz)) = self.flux.direction(&self.carte, e.x, e.z)
        {
            v.chemin_x = cx;
            v.chemin_z = cz;
        }

        // Cible : l'allié qui l'a attiré s'il vit encore, sinon le joueur
        let mut cible = Cible::Joueur;
        if let Some((a, id)) = e.cible_allie {
            if let Some((ax, az, rayon)) = self.allie_vivant(a, id) {
                let (ex, ez) = (ax - e.x, az - e.z);
                let d = longueur(ex, ez);
                let d = if d == 0.0 { 1e-6 } else { d };
                v.dx = ex / d;
                v.dz = ez / d;
                v.chemin_x = v.dx;
                v.chemin_z = v.dz;
                // La portée de contact est calculée pour le joueur : on compense la taille de l'allié
                v.distance = (d - (rayon - rayon_joueur)).max(0.0);
                cible = Cible::Allie(a);
            } else {
                self.ennemis.pool[index].cible_allie = None;
            }
        }

        // Comportement (immobile si le joueur est mort ou s'il est étourdi ; enraciné, il attaque encore)
        let e = &mut self.ennemis.pool[index];
        if j_mort || e.etourdi > 0.0 {
            e.dir_x = 0.0;
            e.dir_z = 0.0;
        } else {
            let meche_avant = e.meche;
            let action = appliquer_comportement(e, dt, &v, rayon_joueur);
            e.a_exploser = action.explose;
            let meche = e.meche;
            if e.enracine > 0.0 {
                e.dir_x = 0.0;
                e.dir_z = 0.0;
            }
            if meche_avant == 0.0 && meche > 0.0 {
                self.ev.son("meche");
            }
            if let Some(degats) = action.coup {
                let boss = def.boss.is_some();
                match cible {
                    Cible::Joueur => {
                        self.blesser_joueur(degats, boss);
                    }
                    Cible::Allie(a) => self.blesser_allie(a, degats),
                }
            }
            if let Some(t) = action.tir {
                for k in 0..t.nombre as u32 {
                    let a = t.base + (k as f32 - (t.nombre - 1.0) / 2.0) * t.dispersion;
                    self.tirer_ennemi(t.projectile, t.x, t.z, sin(a), cos(a), t.degats, t.boss);
                }
            }
        }

        // Séparation : repousse doucement les ennemis qui se chevauchent
        let e = &self.ennemis.pool[index];
        let (mut sep_x, mut sep_z) = (0.0, 0.0);
        let rayon_max = self.ennemis.rayon_max;
        let candidats = self.ennemis.grille.requeter(
            e.x,
            e.z,
            def.rayon + rayon_max,
            &mut self.ennemis.voisins,
        );
        for k in 0..candidats {
            let autre = self.ennemis.voisins[k] as usize;
            if autre == index {
                continue;
            }
            let o = &self.ennemis.pool[autre];
            if !o.vivant() {
                continue;
            }
            let (ox, oz) = (e.x - o.x, e.z - o.z);
            let minimum = def.rayon + o.def.rayon;
            let d2 = ox * ox + oz * oz;
            if d2 >= minimum * minimum || d2 < 1e-8 {
                continue;
            }
            let d = racine(d2);
            sep_x += ox / d * (minimum - d);
            sep_z += oz / d * (minimum - d);
        }
        let k_separation = (SEPARATION * dt).min(1.0);

        // Intégration : marche voulue (ralentie par le sol) + recul amorti + séparation
        let sol = if def.ignore_obstacles {
            1.0
        } else {
            self.carte.vitesse_sol(e.x, e.z, def.traverse_toiles)
        };
        let e = &mut self.ennemis.pool[index];
        let vitesse = def.vitesse * (1.0 - e.ralenti) * sol;
        e.x += (e.dir_x * vitesse + e.vx) * dt + sep_x * k_separation;
        e.z += (e.dir_z * vitesse + e.vz) * dt + sep_z * k_separation;
        let amorti = 1.0 - facteur_lissage(AMORTISSEMENT_RECUL, dt);
        e.vx *= amorti;
        e.vz *= amorti;

        // Jamais dans le joueur
        let (px, pz) = (e.x - jx, e.z - jz);
        let contact = def.rayon + rayon_joueur;
        let dj2 = px * px + pz * pz;
        if dj2 < contact * contact && dj2 > 1e-8 {
            let dj = racine(dj2);
            e.x = jx + px / dj * contact;
            e.z = jz + pz / dj * contact;
        }

        // Décor et bords de l'arène
        if def.ignore_obstacles {
            let limite_x = self.etat.arene.largeur / 2.0 - def.rayon;
            let limite_z = self.etat.arene.profondeur / 2.0 - def.rayon;
            e.x = e.x.clamp(-limite_x, limite_x);
            e.z = e.z.clamp(-limite_z, limite_z);
        } else {
            let (cx, cz) =
                self.carte
                    .resoudre_collision(e.x, e.z, def.rayon_decor.unwrap_or(def.rayon));
            // Projeté contre un mur : le recul s'arrête net (Lame lourde)
            let choc = (cx - e.x) * (cx - e.x) + (cz - e.z) * (cz - e.z) > 1e-4
                && e.vx * e.vx + e.vz * e.vz > CHOC_MUR * CHOC_MUR;
            e.x = cx;
            e.z = cz;
            if choc {
                e.vx = 0.0;
                e.vz = 0.0;
                self.talent_sur_impact_mur(index);
            }
        }

        // Toujours tourné vers sa cible
        let e = &mut self.ennemis.pool[index];
        e.angle += ecart_angle(e.angle, atan2(v.dx, v.dz)) * facteur_lissage(VITESSE_ROTATION, dt);
        e.vitesse_sol = longueur(e.dir_x, e.dir_z) * vitesse;
    }
}
