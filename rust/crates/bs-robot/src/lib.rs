//! Robot d'équilibrage (port de src/robot/robot.ts) : joue une partie seul, sans rendu,
//! à pas fixe (1/60 s) et avec l'aléa semé. Même graine, même personnage, même arène :
//! même partie. Il fuit les ennemis, ramasse l'XP, choisit ses cartes par une règle
//! fixe et rend un bilan chiffré (`main.rs` le lance sur N graines et compare les
//! statistiques à celles de la version TypeScript, docs/robot/reference.json).

pub mod statistiques;

use bs_donnees::entites::Comportement;
use bs_regles::Partie;
use bs_regles::cartes::Amelioration;
use bs_regles::equipement::ChoixEquipement;
use bs_regles::etat::Issue;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Pas de simulation du robot TS (le jeu TS ne tourne pas au pas Rust).
pub const PAS: f32 = 1.0 / 60.0;
const RAYON_DANGER: f32 = 9.0;
const DIRECTIONS: u32 = 16;
/// Distance (blocs) à laquelle le danger d'une direction est évalué.
const ANTICIPATION: f64 = 2.0;
/// Durée (s) du trajet des projectiles ennemis pris en compte.
const HORIZON_TIRS: f64 = 0.6;
/// Attrait de l'orbe d'XP la plus proche face au danger, où qu'elle soit dans l'arène.
const POIDS_XP: f64 = 0.08;
/// ≈ sin(22,5°) : huit directions au clavier.
const SEUIL_TOUCHE: f64 = 0.38;

/// Dégâts et dégâts par seconde (depuis son obtention) d'une arme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BilanArme {
    pub degats: f64,
    pub dps: f64,
}

/// Bilan d'une partie (mêmes champs que BilanRobot en TS, sans le temps de calcul).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bilan {
    pub graine: u32,
    pub personnage: String,
    pub arene: String,
    pub issue: String,
    pub temps: f64,
    pub niveau: u32,
    pub xp: f64,
    pub tues: u32,
    pub boss: u32,
    pub degats_subis: f64,
    /// Dégâts infligés par seconde, minute par minute (la « puissance »).
    pub puissance_par_minute: Vec<f64>,
    pub armes: IndexMap<String, BilanArme>,
}

pub struct Robot {
    proches: [u32; 128],
    /// Direction suivie (rad).
    cap: f64,
    degats_minute: f64,
    puissance: Vec<f64>,
}

impl Default for Robot {
    fn default() -> Self {
        Self {
            proches: [0; 128],
            cap: 0.0,
            degats_minute: 0.0,
            puissance: Vec::new(),
        }
    }
}

/// Arrondi de JavaScript (Math.round : moitiés vers +∞).
fn arrondir(v: f64) -> f64 {
    (v + 0.5).floor()
}

impl Robot {
    /// Joue jusqu'à la fin de la partie (ou `duree` s de jeu).
    pub fn jouer(&mut self, p: &mut Partie, graine: u32, duree: f64, invincible: bool) -> Bilan {
        p.joueur.invincible = invincible;
        let mut pas: u64 = 0;
        while p.etat.issue.is_none() && p.etat.temps_jeu < duree {
            self.avancer(p);
            pas += 1;
            if pas.is_multiple_of(3600) {
                self.noter_minute(p, 60.0);
            }
        }
        let reste = (pas % 3600) as f64 / 60.0;
        if reste >= 1.0 {
            self.noter_minute(p, reste);
        }
        self.bilan(p, graine)
    }

    /// Un pas : cartes en attente, direction, puis la simulation.
    pub fn avancer(&mut self, p: &mut Partie) {
        self.choisir_cartes(p);
        self.diriger(p);
        p.pas(PAS);
    }

    fn noter_minute(&mut self, p: &Partie, secondes: f64) {
        let total = degats_totaux(p);
        let dps = arrondir((total - self.degats_minute) / secondes);
        self.puissance.push(dps);
        self.degats_minute = total;
    }

    // --- Déplacement -------------------------------------------------------

    /// Note 16 directions et prend la meilleure : loin des ennemis proches (là où le joueur
    /// sera dans un instant), jamais dans un mur ni vers les bords, vers l'XP quand la voie
    /// est libre, et sans changer de cap pour un rien.
    fn diriger(&mut self, p: &mut Partie) {
        let (jx, jz) = (p.etat.joueur.x, p.etat.joueur.z);
        let n = p.ennemis.chercher(jx, jz, RAYON_DANGER, &mut self.proches);
        let (jx, jz) = (f64::from(jx), f64::from(jz));
        // Orbe d'XP la plus proche
        let mut orbe = None;
        let mut meilleure = f64::INFINITY;
        for o in &p.ramassables.orbes {
            let (ox, oz) = (f64::from(o.x), f64::from(o.z));
            let d2 = (ox - jx).powi(2) + (oz - jz).powi(2);
            if d2 < meilleure {
                meilleure = d2;
                orbe = Some((ox, oz));
            }
        }
        let marge = 5.0;
        let arene = p.etat.arene;
        let demi_x = f64::from(arene.largeur) / 2.0 - marge;
        let demi_z = f64::from(arene.profondeur) / 2.0 - marge;
        let bloque = |x: f64, z: f64| p.carte.bloque_marche(x as f32, z as f32);
        let mut meilleur_angle = self.cap;
        let mut meilleure_note = f64::INFINITY;
        for d in 0..DIRECTIONS {
            let angle = f64::from(d) / f64::from(DIRECTIONS) * std::f64::consts::PI * 2.0;
            let (cx, cz) = (angle.cos(), angle.sin());
            // Murs sur le chemin : direction exclue
            if bloque(jx + cx, jz + cz) || bloque(jx + cx * 2.5, jz + cz * 2.5) {
                continue;
            }
            let qx = jx + cx * ANTICIPATION;
            let qz = jz + cz * ANTICIPATION;
            let mut note = 0.0;
            for &k in &self.proches[..n] {
                let e = &p.ennemis.pool[k as usize];
                let d2 = (qx - f64::from(e.x)).powi(2) + (qz - f64::from(e.z)).powi(2);
                // Creeper : son explosion fait la moitié des PV, on le fuit de loin
                note += if matches!(e.def.comportement, Comportement::Explosif { .. }) {
                    12.0 / d2.max(1.0)
                } else {
                    1.0 / d2.max(0.2)
                };
            }
            // Projectiles ennemis : distance au trajet qu'ils feront dans l'instant qui vient
            for t in &p.projectiles_ennemis.pool {
                if t.renvoye {
                    continue;
                }
                let (tx, tz) = (f64::from(t.x), f64::from(t.z));
                let vx = f64::from(t.vx) * HORIZON_TIRS;
                let vz = f64::from(t.vz) * HORIZON_TIRS;
                let l2 = vx * vx + vz * vz + 1e-6;
                let s = (((qx - tx) * vx + (qz - tz) * vz) / l2).clamp(0.0, 1.0);
                let d2 = (qx - tx - vx * s).powi(2) + (qz - tz - vz * s).powi(2);
                note += 1.5 / d2.max(0.2);
            }
            // Bords de l'arène et obstacles un peu plus loin
            let hors_x = (qx.abs() - demi_x).max(0.0);
            let hors_z = (qz.abs() - demi_z).max(0.0);
            note += 0.05 * (hors_x * hors_x + hors_z * hors_z);
            if bloque(jx + cx * 5.0, jz + cz * 5.0) {
                note += 0.15;
            }
            if let Some((ox, oz)) = orbe {
                note -= POIDS_XP * ((ox - jx) * cx + (oz - jz) * cz) / (meilleure + 0.01).sqrt();
            }
            note -= 0.02 * (angle - self.cap).cos();
            if note < meilleure_note {
                meilleure_note = note;
                meilleur_angle = angle;
            }
        }
        self.cap = meilleur_angle;
        let (fx, fz) = (meilleur_angle.cos(), meilleur_angle.sin());
        let c = &mut p.joueur.commandes;
        c.droite = fx > SEUIL_TOUCHE;
        c.gauche = fx < -SEUIL_TOUCHE;
        c.bas = fz > SEUIL_TOUCHE;
        c.haut = fz < -SEUIL_TOUCHE;
    }

    // --- Cartes ------------------------------------------------------------

    /// Cartes de niveau puis objets des coffres, choisis par une règle fixe.
    pub fn choisir_cartes(&mut self, p: &mut Partie) {
        if p.etat.issue.is_some() || p.etat.joueur.mort {
            return;
        }
        while p.etat.niveaux_en_attente > 0 {
            p.tirer_cartes();
            let carte = meilleure(&p.cartes.proposees, |c| noter_amelioration(p, c));
            p.appliquer_amelioration(carte);
            p.etat.niveaux_en_attente -= 1;
        }
        while p.etat.choix_coffre.restants > 0 {
            let ender = p.etat.choix_coffre.ender > 0;
            let ouverts = p.ramassables.coffres.payants_ouverts;
            let cartes = p.tirer_equipement(ender, ouverts);
            let mut choix = meilleure(cartes, noter_equipement);
            if let ChoixEquipement::Objet(objet) = choix
                && p.remplacement_necessaire(objet)
            {
                choix = p.cartes_remplacement(objet)[0];
            }
            p.appliquer_equipement(choix);
            p.etat.choix_coffre.restants -= 1;
            if ender {
                p.etat.choix_coffre.ender -= 1;
            }
        }
    }

    // --- Bilan -------------------------------------------------------------

    fn bilan(&self, p: &Partie, graine: u32) -> Bilan {
        let e = &p.etat;
        let s = &e.stats;
        let mut armes = IndexMap::new();
        for (i, id) in p.d.armes.keys().enumerate() {
            if s.acquisition_arme[i] < 0.0 {
                continue;
            }
            let duree = (e.temps_jeu - s.acquisition_arme[i]).max(1.0);
            let degats = s.degats_par_arme[i];
            let dps = arrondir(degats / duree);
            let degats = arrondir(degats);
            armes.insert(id.clone(), BilanArme { degats, dps });
        }
        let personnages = &p.d.personnages.personnages;
        let personnage = personnages
            .get_index(e.index_personnage)
            .map(|(id, _)| id.clone());
        Bilan {
            graine,
            personnage: personnage.unwrap_or_default(),
            arene: e.arene.id.clone(),
            issue: match e.issue {
                Some(Issue::Victoire) => "victoire",
                Some(Issue::Defaite) => "defaite",
                None => "limite",
            }
            .into(),
            temps: arrondir(e.temps_jeu * 10.0) / 10.0,
            niveau: e.joueur.niveau,
            xp: arrondir(f64::from(s.xp_totale)),
            tues: e.ennemis_tues,
            boss: s.boss_vaincus,
            degats_subis: arrondir(f64::from(s.degats_subis)),
            puissance_par_minute: self.puissance.clone(),
            armes,
        }
    }
}

fn degats_totaux(p: &Partie) -> f64 {
    p.etat.stats.degats_par_arme.iter().sum()
}

/// Première carte de meilleure note.
fn meilleure<T: Copy>(cartes: &[T], noter: impl Fn(&T) -> i32) -> T {
    let mut choix = cartes[0];
    let mut note = i32::MIN;
    for c in cartes {
        let n = noter(c);
        if n > note {
            note = n;
            choix = *c;
        }
    }
    choix
}

/// Éveil > arme déjà portée > nouvelle arme > passif ; soin si les PV sont bas.
fn noter_amelioration(p: &Partie, c: &Amelioration) -> i32 {
    let j = &p.etat.joueur;
    match *c {
        Amelioration::Eveil { .. } => 6,
        Amelioration::Arme(i) if p.etat.stats.acquisition_arme[i] >= 0.0 => 5,
        Amelioration::Arme(_) => 4,
        Amelioration::Passif(_) => 2,
        Amelioration::Soin if j.pv < j.pv_max * 0.4 => 7,
        Amelioration::Soin => 0,
    }
}

/// Renfort > objet > remplacement > émeraudes.
fn noter_equipement(c: &ChoixEquipement) -> i32 {
    match c {
        ChoixEquipement::Renfort(_) => 4,
        ChoixEquipement::Objet(_) => 3,
        ChoixEquipement::Remplacer { .. } => 1,
        ChoixEquipement::Niveau => 2,
        ChoixEquipement::Soin => 1,
        ChoixEquipement::Emeraudes(_) => 0,
        ChoixEquipement::Retour => -1,
    }
}
