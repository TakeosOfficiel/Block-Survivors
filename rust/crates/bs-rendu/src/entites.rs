//! Entités affichées : joueur, ennemis et alliés en modèles Minecraft (`modeles`) ; les
//! autres joueurs de la coop passent par `coequipiers`.
//!
//! `FixedUpdate` : avant chaque pas, les positions courantes deviennent les précédentes
//! (`noter_positions`) ; après le pas, les animations avancent (`animer`, gestes
//! d'attaque lus dans les événements). `Update` : chaque entité est posée entre ses deux
//! dernières positions (`placer_entites`), avec chute à la mort, gonflement du creeper
//! et teintes d'état (src/entites/ennemis.ts, joueur.ts, invocations.ts).
//!
//! Les places des pools bougent (un mort est remplacé par le dernier) : positions et
//! animations sont retrouvées par identifiant, dans des tables dont la capacité est
//! réservée une fois pour toutes.

use std::f32::consts::PI;

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bs_donnees::Donnees;
use bs_donnees::entites::Comportement;
use bs_regles::ennemis::{DUREE_MORT, Ennemi};
use bs_regles::evenements::Evenement;
use bs_regles::ramassage::Coffre;

use crate::coequipiers::{Coequipiers, Suivi};
use crate::effets::{Effets, Ombres};
use crate::modeles::Rendus;
use crate::modeles::animation::{
    EtatAnimation, angle_chute, calculer_rotations, hauteur_flottement,
};
use crate::modeles::rendu::Composants;
use crate::retours::Barres;
use crate::sources::invocations;
use crate::{PAS, PartieEnCours};

/// Durée de la chute du joueur mort (config.ts `joueur.dureeMort`).
const DUREE_MORT_JOUEUR: f32 = 0.6;
/// Barre de vie des mini-boss : longueur × celle des alliés (config.ts).
const BARRE_MINI_BOSS: f32 = 2.0;
/// Coffre retourné : son dessous (le couvercle du modèle) à 14 px du sol.
const HAUTEUR_COFFRE: f32 = 14.0 / 16.0;
/// Teinte rouge d'un coup reçu (config.ts `blessure`).
const BLESSURE: [f32; 3] = [1.0, 0.0, 0.0];
const FORCE_BLESSURE: f32 = 0.35;
/// Fenêtre de teinte rouge du joueur après un coup (s).
const DUREE_BLESSURE: f32 = 0.3;
/// Clignotement blanc du creeper (config.ts `flash`).
const FLASH: [f32; 3] = [1.0, 1.0, 1.0];
const FORCE_FLASH: f32 = 0.7;
/// Gonflement du creeper au bout de sa mèche (config.ts `ennemis`).
const GONFLEMENT_LARGEUR: f32 = 0.4;
const GONFLEMENT_HAUTEUR: f32 = 0.1;
/// Capacité réservée des tables d'alliés.
const CAPACITE_ALLIES: usize = 64;

/// Positions au pas précédent (pour l'interpolation).
#[derive(Resource)]
pub struct Precedentes {
    joueur: Vec2,
    ennemis: HashMap<u32, Vec2>,
    allies: HashMap<u64, Vec2>,
}

/// Teinte d'un état (couleur, force), lue une fois dans les données.
type Teinte = Option<([f32; 3], f32)>;

/// État d'animation de chaque entité.
#[derive(Resource)]
pub struct Animations {
    joueur: EtatAnimation,
    ennemis: HashMap<u32, EtatAnimation>,
    allies: HashMap<u64, EtatAnimation>,
    /// Ravageur de l'Évocation supérieure.
    pub(crate) ravageur: EtatAnimation,
    /// Graine de la partie animée (une nouvelle partie remet tout à zéro).
    graine: u32,
    /// Numéro du pas, pour oublier les entités disparues.
    pas: u32,
    brulure: Teinte,
    poison: Teinte,
    saignement: Teinte,
    ralenti: Teinte,
    invulnerabilite: f32,
}

/// Décalage d'animation stable dérivé de l'identifiant (les foules ne marchent pas au pas).
fn decalage(id: u32) -> f32 {
    (id.wrapping_mul(2_654_435_761) >> 8) as f32 / (1u32 << 24) as f32
}

fn teinte(d: &Donnees, cle: &str) -> Teinte {
    let e = d.regles.etats.get(cle)?;
    Some((e.couleur?, e.force.unwrap_or(0.0)))
}

pub fn creer_entites(mut commandes: Commands, jeu: Res<PartieEnCours>) {
    let p = &jeu.partie;
    let d = p.d;
    let capacite = p.ennemis.pool.len();
    let j = &p.etat.joueur;
    commandes.insert_resource(Precedentes {
        joueur: Vec2::new(j.x, j.z),
        ennemis: HashMap::with_capacity(capacite),
        allies: HashMap::with_capacity(CAPACITE_ALLIES),
    });
    commandes.insert_resource(Animations {
        joueur: EtatAnimation::nouveau(0.0),
        ennemis: HashMap::with_capacity(capacite),
        allies: HashMap::with_capacity(CAPACITE_ALLIES),
        ravageur: EtatAnimation::nouveau(0.0),
        graine: jeu.graine,
        pas: 0,
        brulure: teinte(d, "brulure"),
        poison: teinte(d, "poison"),
        saignement: teinte(d, "saignement"),
        ralenti: teinte(d, "ralenti"),
        invulnerabilite: d.regles.joueur.invulnerabilite,
    });
}

/// Avant chaque pas : positions courantes, qui deviennent les précédentes.
pub fn noter_positions(
    jeu: Res<PartieEnCours>,
    mut prec: ResMut<Precedentes>,
    rendus: Res<Rendus>,
    mut suivi: ResMut<Suivi>,
    options: Res<crate::options::Options>,
) {
    let p = &jeu.partie;
    crate::coequipiers::noter(p, &rendus, &mut suivi);
    prec.joueur = Vec2::new(p.etat.joueur.x, p.etat.joueur.z);
    prec.ennemis.clear();
    for e in p.ennemis.actifs() {
        prec.ennemis.insert(e.id, Vec2::new(e.x, e.z));
    }
    prec.allies.clear();
    for (cle, a) in invocations(p, options.effets_allies) {
        prec.allies.insert(cle, Vec2::new(a.x, a.z));
    }
}

/// Après chaque pas : animations (marche, repos, gestes d'attaque).
pub fn animer(
    jeu: Res<PartieEnCours>,
    rendus: Res<Rendus>,
    mut anim: ResMut<Animations>,
    mut suivi: ResMut<Suivi>,
    options: Res<crate::options::Options>,
) {
    let p = &jeu.partie;
    crate::coequipiers::animer(p, &rendus, &mut suivi, PAS);
    let a = &mut *anim;
    if a.graine != jeu.graine {
        a.graine = jeu.graine;
        a.joueur = EtatAnimation::nouveau(0.0);
        a.ennemis.clear();
        a.allies.clear();
    }
    a.pas = a.pas.wrapping_add(1);
    let pas = a.pas;
    let j = &p.etat.joueur;
    let mj = rendus.modele_joueur();
    a.joueur.avancer(mj, PAS, Vec2::new(j.vx, j.vz).length());
    for e in p.ennemis.actifs() {
        let m = rendus.modele_ennemi(e.index_type);
        let etat = a
            .ennemis
            .entry(e.id)
            .or_insert_with(|| EtatAnimation::nouveau(decalage(e.id)));
        etat.vu = pas;
        etat.avancer(m, PAS, e.vitesse_sol / e.def.echelle.unwrap_or(1.0));
    }
    a.ennemis.retain(|_, e| e.vu == pas);
    for (cle, al) in invocations(p, options.effets_allies) {
        let m = rendus.modele_allie(al.type_allie);
        let etat = a
            .allies
            .entry(cle)
            .or_insert_with(|| EtatAnimation::nouveau(decalage(al.id)));
        etat.vu = pas;
        etat.avancer(m, PAS, al.vitesse_sol);
    }
    a.allies.retain(|_, e| e.vu == pas);
    crate::creatures::animer(p, &rendus, &mut a.ravageur);
    for ev in &p.ev.liste {
        let Evenement::Geste(qui) = ev else { continue };
        match qui {
            None => a.joueur.declencher_attaque(mj),
            Some(id) => {
                let Some(al) = p.armes.invocations.pool.iter().find(|x| x.id == *id) else {
                    continue;
                };
                // Gestes des créatures du joueur (combattant 0 : clé = id)
                if let Some(etat) = a.allies.get_mut(&u64::from(*id)) {
                    etat.declencher_attaque(rendus.modele_allie(al.type_allie));
                }
            }
        }
    }
}

/// Surcouche d'un ennemi vivant : premier état actif (brûlure pulsée, poison,
/// saignement, ralenti), sinon rien.
fn surcouche_etat(a: &Animations, e: &Ennemi, temps: f32) -> [f32; 4] {
    let pulse = 0.75 + 0.25 * (temps * 23.0 + e.id as f32).sin();
    let choix = [
        (e.brulure.cumuls > 0.0, a.brulure, pulse),
        (e.poison.cumuls > 0.0, a.poison, 1.0),
        (e.saignement.cumuls > 0.0, a.saignement, 1.0),
        (e.ralenti_temps > 0.0 || e.enracine > 0.0, a.ralenti, 1.0),
    ];
    for (actif, t, facteur) in choix {
        if let (true, Some((c, f))) = (actif, t) {
            return [c[0], c[1], c[2], f * facteur];
        }
    }
    [0.0; 4]
}

fn rouge() -> [f32; 4] {
    [BLESSURE[0], BLESSURE[1], BLESSURE[2], FORCE_BLESSURE]
}

#[allow(clippy::too_many_arguments)]
pub fn placer_entites(
    mut commandes: Commands,
    jeu: Res<PartieEnCours>,
    prec: Res<Precedentes>,
    anim: Res<Animations>,
    mut rendus: ResMut<Rendus>,
    temps: Res<Time<Fixed>>,
    mut cible: ResMut<crate::camera::Cible>,
    mut ombres: ResMut<Ombres>,
    mut barres: ResMut<Barres>,
    (effets, suivi, coequipiers): (Res<Effets>, Res<Suivi>, Res<Coequipiers>),
    options: Res<crate::options::Options>,
    mut q: Composants,
) {
    ombres.0.clear();
    barres.0.clear();
    let t = temps.overstep_fraction();
    let p = &jeu.partie;
    let temps_jeu = p.etat.temps_jeu as f32;
    for r in &mut rendus.liste {
        r.commencer();
    }

    // Joueur
    let j = &p.etat.joueur;
    let ici = prec.joueur.lerp(Vec2::new(j.x, j.z), t);
    let sol = |x: f32, z: f32| p.carte.hauteur_lissee(x, z);
    cible.0 = ici;
    cible.1 = sol(ici.x, ici.y);
    let r = rendus.joueur();
    calculer_rotations(&r.modele, &anim.joueur, &mut r.pose.rotations);
    let pose = &mut r.pose;
    pose.position = Vec3::new(
        ici.x,
        cible.1 + hauteur_flottement(&r.modele, &anim.joueur),
        ici.y,
    );
    if crate::scene::scene().is_some() {
        // Capture de scène : le héros de l'accueil prendra sa place
        pose.position.y -= 1000.0;
    }
    pose.angle = j.angle;
    pose.inclinaison = if j.mort {
        angle_chute(j.temps_mort, DUREE_MORT_JOUEUR)
    } else {
        0.0
    };
    pose.echelle_xz = 1.0;
    pose.echelle_y = 1.0;
    let touche = j.invulnerabilite > anim.invulnerabilite - DUREE_BLESSURE;
    pose.surcouche = if j.mort || touche { rouge() } else { [0.0; 4] };
    r.ajouter(&mut commandes, &mut q);
    rendus.poser_armure(&mut commandes, &mut q);
    let rayon = p.d.personnages.personnages[p.etat.index_personnage]
        .apparence
        .rayon_ombre;
    // Capture de scène : l'ombre reste, le héros de l'accueil s'y tiendra
    ombres.0.push([ici.x, ici.y, rayon, cible.1]);
    crate::coequipiers::placer(
        p,
        (&suivi, &coequipiers),
        t,
        anim.invulnerabilite,
        &mut rendus,
        &mut commandes,
        &mut q,
        &mut ombres,
    );

    // Ennemis
    for e in p.ennemis.actifs() {
        let Some(etat) = anim.ennemis.get(&e.id) else {
            continue;
        };
        let maintenant = Vec2::new(e.x, e.z);
        let pos = prec
            .ennemis
            .get(&e.id)
            .copied()
            .unwrap_or(maintenant)
            .lerp(maintenant, t);
        let ech = e.def.echelle.unwrap_or(1.0);
        let r = rendus.ennemi(e.index_type);
        calculer_rotations(&r.modele, etat, &mut r.pose.rotations);
        let pose = &mut r.pose;
        let y_sol = sol(pos.x, pos.y);
        pose.position = Vec3::new(
            pos.x,
            y_sol + hauteur_flottement(&r.modele, etat) * ech,
            pos.y,
        );
        pose.angle = e.angle;
        let mort = e.mort >= 0.0;
        pose.inclinaison = if mort {
            angle_chute(e.mort, DUREE_MORT)
        } else {
            0.0
        };
        pose.echelle_xz = ech;
        pose.echelle_y = ech;
        pose.surcouche = if mort {
            [0.0; 4]
        } else {
            surcouche_etat(&anim, e, temps_jeu)
        };
        if let Comportement::Explosif { duree, .. } = &e.def.comportement
            && e.meche > 0.0
        {
            let prog = (e.meche / duree).min(1.0);
            let tremblement = 1.0 + (prog * 100.0).sin() * prog * 0.01;
            let gonf = prog * prog * prog;
            pose.echelle_xz = ech * (1.0 + gonf * GONFLEMENT_LARGEUR) * tremblement;
            pose.echelle_y = ech * (1.0 + gonf * GONFLEMENT_HAUTEUR) / tremblement;
            if (prog * 10.0).floor() as i32 % 2 == 1 {
                let f = FORCE_FLASH * prog.clamp(0.5, 1.0);
                pose.surcouche = [FLASH[0], FLASH[1], FLASH[2], f];
            }
        }
        if e.blessure > 0.0 || mort {
            pose.surcouche = rouge();
        }
        let rayon = e.def.apparence.rayon_ombre * pose.echelle_xz / ech;
        // Mini-boss : sa barre de vie au-dessus de la tête, toujours visible
        if e.def.mini_boss && !mort {
            let y = pose.position.y + r.modele.hauteur * ech + 0.3;
            barres
                .0
                .push([pos.x, y, pos.y, e.pv / e.pv_max, BARRE_MINI_BOSS]);
        }
        r.ajouter(&mut commandes, &mut q);
        ombres.0.push([pos.x, pos.y, rayon, y_sol]);
    }

    // Alliés
    for (cle, a) in invocations(p, options.effets_allies) {
        let Some(etat) = anim.allies.get(&cle) else {
            continue;
        };
        let maintenant = Vec2::new(a.x, a.z);
        let pos = prec
            .allies
            .get(&cle)
            .copied()
            .unwrap_or(maintenant)
            .lerp(maintenant, t);
        let r = rendus.allie(a.type_allie);
        calculer_rotations(&r.modele, etat, &mut r.pose.rotations);
        let pose = &mut r.pose;
        let y_sol = sol(pos.x, pos.y);
        let y = y_sol
            + match a.def.vol {
                Some(v) => v + (a.vie * 4.0).sin() * 0.12,
                None => hauteur_flottement(&r.modele, etat),
            };
        pose.position = Vec3::new(pos.x, y, pos.y);
        pose.angle = a.angle;
        pose.inclinaison = 0.0;
        pose.echelle_xz = a.echelle;
        pose.echelle_y = a.echelle;
        pose.surcouche = if a.blessure > 0.0 { rouge() } else { [0.0; 4] };
        r.ajouter(&mut commandes, &mut q);
        ombres
            .0
            .push([pos.x, pos.y, a.def.apparence.rayon_ombre, y_sol]);
        // Barre de vie dès la première blessure
        if a.pv < a.pv_max {
            let y = y + r.modele.hauteur + 0.35;
            barres.0.push([pos.x, y, pos.y, a.pv / a.pv_max, 1.0]);
        }
    }

    crate::creatures::placer(
        p,
        &anim.ravageur,
        &mut rendus,
        &mut commandes,
        &mut q,
        &mut ombres,
    );

    // Coffres : retournés (loquet en −Z), le couvercle se lève en douceur
    let duree = p.d.coffres.regles.duree_ouverture;
    let n = p.d.coffres.coffres.len();
    // Vitrine : chaque type de coffre, qui s'ouvre et se referme
    let vitrine = (0..if effets.vitrine { n } else { 0 }).map(|k| Coffre {
        index_type: k,
        x: j.x + (k as f32 - (n as f32 - 1.0) / 2.0) * 1.5,
        z: j.z + 7.5,
        angle: 0.0,
        ouvert: Some(temps_jeu % (duree * 4.0)),
    });
    for c in p.ramassables.coffres.pool.iter().copied().chain(vitrine) {
        let r = rendus.coffre(c.index_type);
        let prog = c.ouvert.map_or(0.0, |o| (o / duree).min(1.0));
        let ouverture = -(1.0 - (1.0 - prog) * (1.0 - prog)) * PI * 0.45;
        for (k, partie) in r.modele.parties.iter().enumerate() {
            let x = if matches!(partie.nom, "couvercle" | "loquet") {
                ouverture
            } else {
                0.0
            };
            r.pose.rotations[k] = Vec3::new(x, 0.0, 0.0);
        }
        let pose = &mut r.pose;
        pose.position = Vec3::new(c.x, sol(c.x, c.z) + HAUTEUR_COFFRE, c.z);
        pose.angle = c.angle + PI;
        pose.inclinaison = 0.0;
        pose.echelle_xz = 1.0;
        pose.echelle_y = -1.0;
        pose.surcouche = [0.0; 4];
        r.ajouter(&mut commandes, &mut q);
    }

    for r in &mut rendus.liste {
        r.terminer(&mut q);
    }
}
