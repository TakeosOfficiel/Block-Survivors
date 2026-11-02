//! Joueur : déplacement au clavier (ou au robot), collisions, régénération, coups reçus,
//! totem d'immortalité (port de src/entites/joueur.ts, sans le rendu).

use bs_commun::maths::{atan2, cos, ecart_angle, facteur_lissage, longueur, sin};

use crate::partie::Partie;

/// Totem : part des PV maximum rendue, s d'invincibilité, recul des ennemis proches.
const TOTEM_PV: f32 = 0.5;
const TOTEM_PROTECTION: f32 = 3.0;
pub const TOTEM_RAYON_RECUL: f32 = 6.0;
pub const TOTEM_RECUL: f32 = 14.0;

/// Touches de déplacement enfoncées.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Commandes {
    pub haut: bool,
    pub bas: bool,
    pub gauche: bool,
    pub droite: bool,
    /// Lacet de la caméra (rad) qui oriente les touches ; 0 dans la vue de dessus.
    pub lacet: f32,
}

/// Ce qui pilote le joueur en dehors de son état (src/entites/joueur.ts).
#[derive(Debug, Clone, Default)]
pub struct Joueur {
    pub commandes: Commandes,
    /// Ignore tous les dégâts (debug, robot « invincible »).
    pub invincible: bool,
}

/// Coup reçu : l'armure retire ses points (au plus la moitié du coup), au moins 1 dégât.
pub fn degats_apres_defense(degats: f32, armure: f32, plafond: f32) -> f32 {
    (degats - armure.min(degats * plafond)).max(1.0)
}

impl Partie {
    /// Un pas de simulation du joueur.
    pub fn maj_joueur(&mut self, dt: f32) {
        let base = &self.d.regles.joueur;
        let plafond_regen = self.d.regles.plafonds.regeneration;
        let j = &mut self.etat.joueur;
        if j.mort {
            j.temps_mort += dt;
            return;
        }
        if j.invulnerabilite > 0.0 {
            j.invulnerabilite -= dt;
        }
        if j.protection > 0.0 {
            j.protection -= dt;
        }
        let m = &self.etat.modificateurs;
        if m.regeneration > 0.0 || m.regeneration_part > 0.0 {
            let avant = j.pv;
            // PV fixes des passifs et part des PV max des talents, dans le même plafond
            let regeneration =
                (m.regeneration + m.regeneration_part * j.pv_max).min(plafond_regen * j.pv_max);
            j.pv = j.pv_max.min(j.pv + regeneration * dt);
            self.etat.stats.pv_soignes += j.pv - avant;
        }

        // Direction voulue d'après les touches
        let c = self.joueur.commandes;
        let mut dx = f32::from(u8::from(c.droite)) - f32::from(u8::from(c.gauche));
        let mut dz = f32::from(u8::from(c.bas)) - f32::from(u8::from(c.haut));
        let norme = longueur(dx, dz);
        if norme > 0.0 {
            dx /= norme;
            dz /= norme;
        }
        // Vue à la 3e personne : « haut » avance dans le sens où regarde la caméra
        if c.lacet != 0.0 {
            let (sin, cos) = (sin(c.lacet), cos(c.lacet));
            let x = dx * cos + dz * sin;
            dz = dz * cos - dx * sin;
            dx = x;
        }

        // Vitesse : approche exponentielle de la vitesse cible
        let k = facteur_lissage(base.reactivite, dt);
        let vitesse_talents = self.talent_vitesse();
        let j = &mut self.etat.joueur;
        let vitesse = self.etat.personnage.vitesse
            * self.etat.modificateurs.vitesse
            * vitesse_talents
            * self.carte.vitesse_sol(j.x, j.z, false);
        j.vx += (dx * vitesse - j.vx) * k;
        j.vz += (dz * vitesse - j.vz) * k;
        j.x += j.vx * dt;
        j.z += j.vz * dt;

        // Décor et bords : on glisse le long des obstacles, la vitesse bloquée est annulée
        let (cx, cz) = self.carte.resoudre_collision(j.x, j.z, base.rayon);
        if cx != j.x {
            j.vx = 0.0;
        }
        if cz != j.z {
            j.vz = 0.0;
        }
        j.x = cx;
        j.z = cz;

        // Sol brûlant (magma)
        let degats_sol =
            self.carte.degats_sol(j.x, j.z) * (1.0 + self.etat.modificateurs.degats_sol).max(0.0);
        if degats_sol > 0.0 {
            self.subir_degats(degats_sol, false);
        }

        // Orientation vers la direction de marche
        if norme > 0.0 {
            let j = &mut self.etat.joueur;
            j.angle +=
                ecart_angle(j.angle, atan2(dx, dz)) * facteur_lissage(base.vitesse_rotation, dt);
        }
    }

    /// Coup d'un ennemi : de plus en plus fort en mode sans fin.
    pub fn blesser_joueur(&mut self, degats: f32, boss: bool) -> bool {
        let p = &self.d.regles.progression;
        let facteur =
            1.0 + self.etat.minutes_sans_fin(p.duree_partie) * p.sans_fin.degats_par_minute;
        self.subir_degats(degats * facteur, boss)
    }

    /// Applique des dégâts (ignorés pendant l'invulnérabilité) ; vrai si le coup porte.
    pub fn subir_degats(&mut self, mut degats: f32, boss: bool) -> bool {
        let j = &self.etat.joueur;
        if j.mort
            || self.joueur.invincible
            || j.invulnerabilite > 0.0
            || j.protection > 0.0
            || self.etat.issue.is_some()
        {
            return false;
        }
        degats = self.talent_degats_recus(degats, boss);
        // Équipement : dégâts reçus réduits (au mieux −60 %)
        degats *= (1.0 + self.etat.modificateurs.degats_recus).max(0.4);
        // Le bouclier absorbe d'abord ; s'il prend tout, le coup ne coûte aucun PV
        let armure = self.etat.modificateurs.armure + self.talent_armure();
        let mut reste = degats_apres_defense(degats, armure, self.d.regles.plafonds.armure);
        let bloque = degats - reste;
        let j = &mut self.etat.joueur;
        let absorbe = j.bouclier.min(reste);
        j.bouclier -= absorbe;
        reste -= absorbe;
        let subis = j.pv.min(reste);
        j.pv -= subis;
        j.invulnerabilite = self.d.regles.joueur.invulnerabilite;
        self.etat.stats.degats_subis += subis;
        self.ev.son("blessure_joueur");
        self.ev.secousse(0.18 + subis.min(10.0) * 0.03);
        if self.etat.joueur.pv == 0.0 && self.talent_eviter_mort() {
            return true;
        }
        if self.etat.joueur.pv > 0.0 {
            self.talent_apres_coup_recu(subis, bloque + absorbe);
        }
        let j = &mut self.etat.joueur;
        if j.pv == 0.0 && self.etat.modificateurs.vies > j.vies_utilisees as f32 {
            // Totem d'immortalité : la moitié des PV et quelques secondes de protection
            j.vies_utilisees += 1;
            j.pv = libm::ceilf(j.pv_max * TOTEM_PV).max(1.0);
            j.protection = TOTEM_PROTECTION;
            self.ev.son("totem");
            self.ev.secousse(0.6);
        } else if j.pv == 0.0 {
            j.mort = true;
            j.vx = 0.0;
            j.vz = 0.0;
        }
        true
    }

    /// Ajoute des PV de bouclier, dans la limite de la réserve (part des PV max).
    pub fn ajouter_bouclier(&mut self, pv: f32) {
        let j = &mut self.etat.joueur;
        j.bouclier = (self.d.regles.plafonds.bouclier * j.pv_max).min(j.bouclier + pv);
    }
}
