//! Objets du jeu hors modèles animés (src/effets/renduProjectiles.ts, ombres.ts,
//! src/entites/orbes.ts, objets.ts) : projectiles du joueur et des ennemis, flèches
//! ricocheuses, armes en orbite, orbes d'XP, objets au sol et ombres rondes.
//!
//! Chaque sorte d'objet est un `Lot` : un maillage et un matériau partagés par toutes
//! ses instances (Bevy les dessine en un appel). Chaque image, les instances sont
//! reposées ; celles en trop sont cachées, jamais détruites.

mod construction;
pub mod geometrie;
mod lot;

use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use bs_donnees::armes::Mecanique;
use bs_donnees::entites::{FormeProjectile, Projectile};
use bs_regles::combat::tours;

use crate::materiau_entite::tag_surcouche;
use crate::modeles::rendu::Composants;
use crate::{PAS, PartieEnCours};
pub use construction::{construire, demander_textures};
pub use lot::Lot;

/// Orbes d'XP (config.ts `orbes`) : atlas 4 × 4 d'images de 16 px.
const TEXTURE_ORBES: &str = "entity/experience/experience_orb";
const TAILLE_ORBE: f32 = 0.4;
const HAUTEUR_ORBE: f32 = 0.35;
/// Ombres (config.ts `ombre`).
const TEXTURE_OMBRE: &str = "misc/shadow";
const OPACITE_OMBRE: f32 = 0.45;
const HAUTEUR_OMBRE: f32 = 0.01;
/// Un tas d'émeraudes grossit avec sa valeur (objets.ts).
const GROSSISSEMENT_TAS: f32 = 0.15;
/// Icônes posées à plat : on les tourne pour pointer vers +Z (renduProjectiles.ts).
const ORIENTATION_ICONE: f32 = FRAC_PI_4;

/// Ombres à dessiner cette image : (x, z, rayon), remplies par `entites`.
#[derive(Resource, Default)]
pub struct Ombres(pub Vec<[f32; 3]>);

struct TypeProjectile {
    pub(super) def: &'static Projectile,
    pub(super) lot: Lot,
}

#[derive(Resource)]
pub struct Effets {
    projectiles: Vec<TypeProjectile>,
    orbes: Vec<Lot>,
    objets: Vec<Lot>,
    ombres: Lot,
    /// `BS_VITRINE` : un exemplaire de chaque objet près du joueur (vérification
    /// visuelle, captures).
    pub vitrine: bool,
}

/// Pose toutes les instances de l'image (après la caméra : orientation des plans de face).
pub fn placer(
    mut commandes: Commands,
    jeu: Res<PartieEnCours>,
    mut effets: ResMut<Effets>,
    ombres: Res<Ombres>,
    temps: Res<Time<Fixed>>,
    camera: Query<&GlobalTransform, With<Camera3d>>,
    mut q: Composants,
) {
    let p = &jeu.partie;
    let t = temps.overstep_fraction() * PAS;
    let temps_jeu = p.etat.temps_jeu as f32 + t;
    let face = camera
        .single()
        .map_or(Quat::IDENTITY, |c| c.compute_transform().rotation);
    let neutre = tag_surcouche([0.0; 4]);
    let Effets {
        projectiles,
        orbes,
        objets,
        ombres: lot_ombres,
        vitrine,
    } = &mut *effets;
    let j = &p.etat.joueur;
    // Vitrine : rangées devant le joueur (projectiles, orbes, objets)
    let rangee = |ligne: f32, k: usize, n: usize| {
        Vec3::new(
            j.x + (k as f32 - (n as f32 - 1.0) / 2.0) * 1.2,
            0.0,
            j.z + ligne,
        )
    };

    for tp in projectiles.iter_mut() {
        tp.lot.commencer();
    }
    // Projectiles : prolongés de la fraction de pas écoulée (mouvement fluide)
    let mut poser = |def: &'static Projectile, x: f32, z: f32, angle: f32| {
        let Some(tp) = projectiles.iter_mut().find(|tp| std::ptr::eq(tp.def, def)) else {
            return;
        };
        let rotation = match def.forme {
            FormeProjectile::FaceCamera => face,
            _ => Quat::from_rotation_y(angle),
        };
        let pos = Vec3::new(x, def.hauteur, z);
        tp.lot.ajouter(
            &mut commandes,
            &mut q,
            Transform::from_translation(pos).with_rotation(rotation),
            &neutre,
        );
    };
    for pr in &p.armes.projectiles.pool {
        let (vx, vz) = (pr.dir_x * pr.vitesse, pr.dir_z * pr.vitesse);
        let angle = match pr.def.rotation_propre {
            Some(r) => temps_jeu * r,
            None => pr.dir_x.atan2(pr.dir_z),
        };
        poser(pr.def, pr.x + vx * t, pr.z + vz * t, angle);
    }
    for pr in &p.projectiles_ennemis.pool {
        poser(
            pr.def,
            pr.x + pr.vx * t,
            pr.z + pr.vz * t,
            pr.vx.atan2(pr.vz),
        );
    }
    if let Some(fleche) = p.d.projectiles.get("fleche_spectrale") {
        for [x, z, dx, dz] in p.armes.traque.ricochets() {
            poser(fleche, x, z, dx.atan2(dz));
        }
    }
    if *vitrine {
        let n = p.d.projectiles.len();
        for (k, def) in p.d.projectiles.values().enumerate() {
            let v = rangee(2.5, k, n);
            poser(def, v.x, v.z, temps_jeu);
        }
    }
    if !j.mort {
        for arme in &p.arsenal.armes {
            let Mecanique::Orbite { projectile } = &arme.def.mecanique else {
                continue;
            };
            let Some(def) = p.d.projectiles.get(projectile) else {
                continue;
            };
            let s = arme.stats;
            let rotation = temps_jeu * def.rotation_propre.unwrap_or(0.0);
            let a0 = p.armes.angles_orbite[arme.index] + s.vitesse * t;
            for k in 0..tours(s.nombre) {
                let a = a0 + k as f32 * std::f32::consts::TAU / s.nombre;
                poser(
                    def,
                    j.x + a.sin() * s.zone,
                    j.z + a.cos() * s.zone,
                    rotation,
                );
            }
        }
    }
    for tp in projectiles.iter_mut() {
        tp.lot.terminer(&mut q);
    }

    // Orbes d'XP : pulsation du jeu (rouge et bleu oscillent, vert plein)
    for l in orbes.iter_mut() {
        l.commencer();
    }
    let paliers = &p.d.regles.progression.orbes;
    for o in &p.ramassables.orbes {
        let palier = (1..paliers.len())
            .rev()
            .find(|&k| o.valeur >= paliers[k].valeur_min)
            .unwrap_or(0);
        let ph = temps_jeu * 10.0 + o.phase;
        let pos = Vec3::new(o.x, HAUTEUR_ORBE + (ph * 0.3).sin() * 0.06, o.z);
        let c = Color::srgb(
            ((ph / 2.0).sin() + 1.0) * 0.5,
            1.0,
            ((ph / 2.0 + 4.18879).sin() + 1.0) * 0.1,
        )
        .to_linear();
        let tag = tag_surcouche([c.red, c.green, c.blue, 1.0]);
        let tf = Transform::from_translation(pos).with_rotation(face);
        orbes[palier].ajouter(&mut commandes, &mut q, tf, &tag);
    }
    if *vitrine {
        let n = orbes.len();
        for (k, l) in orbes.iter_mut().enumerate() {
            let tf = Transform::from_translation(rangee(4.0, k, n) + Vec3::Y * HAUTEUR_ORBE)
                .with_rotation(face);
            l.ajouter(
                &mut commandes,
                &mut q,
                tf,
                &tag_surcouche([0.5, 1.0, 0.1, 1.0]),
            );
        }
    }
    for l in orbes.iter_mut() {
        l.terminer(&mut q);
    }

    // Objets au sol : balancement et rotation (un tour toutes les ~6 s)
    for l in objets.iter_mut() {
        l.commencer();
    }
    for o in &p.ramassables.objets {
        let pos = Vec3::new(o.x, 0.55 + (temps_jeu * 2.0 + o.phase).sin() * 0.1, o.z);
        let echelle = if o.valeur > 1.0 {
            1.0 + GROSSISSEMENT_TAS * (o.valeur - 1.0)
        } else {
            1.0
        };
        let tf = Transform {
            translation: pos,
            rotation: Quat::from_rotation_y(temps_jeu + o.phase),
            scale: Vec3::splat(echelle),
        };
        if let Some(l) = objets.get_mut(o.index_type) {
            l.ajouter(&mut commandes, &mut q, tf, &neutre);
        }
    }
    if *vitrine {
        let n = objets.len();
        for (k, l) in objets.iter_mut().enumerate() {
            let tf = Transform::from_translation(rangee(5.5, k, n) + Vec3::Y * 0.55)
                .with_rotation(Quat::from_rotation_y(temps_jeu));
            l.ajouter(&mut commandes, &mut q, tf, &neutre);
        }
    }
    for l in objets.iter_mut() {
        l.terminer(&mut q);
    }

    // Ombres rondes au sol
    lot_ombres.commencer();
    for &[x, z, rayon] in &ombres.0 {
        let tf = Transform {
            translation: Vec3::new(x, HAUTEUR_OMBRE, z),
            scale: Vec3::new(rayon * 2.0, 1.0, rayon * 2.0),
            ..default()
        };
        lot_ombres.ajouter(&mut commandes, &mut q, tf, &neutre);
    }
    lot_ombres.terminer(&mut q);
}
