//! Entités affichées (provisoires) : le joueur et les ennemis en personnages cubiques,
//! en attendant les modèles Minecraft (étape 2b). Une entité Bevy par place du pool
//! d'ennemis, cachée quand la place est libre : aucune création pendant la partie.
//!
//! Les places du pool bougent (un mort est remplacé par le dernier) : la position
//! précédente de chaque ennemi est retrouvée par son identifiant, dans une table dont la
//! capacité est réservée une fois pour toutes.

use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::PartieEnCours;
use crate::modeles::personnage;

/// Couleurs des ennemis selon leur type (provisoire).
const PALETTE: [u32; 6] = [0x4c8a3a, 0x8f8f8f, 0x3a5f8a, 0x7a3a8a, 0xa8743a, 0x2e2e2e];

#[derive(Component)]
pub struct JoueurAffiche;

/// Place du pool d'ennemis affichée par cette entité.
#[derive(Component)]
pub struct EnnemiAffiche(usize);

/// Positions au pas précédent (pour l'interpolation).
#[derive(Resource)]
pub struct Precedentes {
    joueur: Vec2,
    ennemis: HashMap<u32, Vec2>,
    /// Matériau de chaque couleur de la palette.
    couleurs: Vec<Handle<StandardMaterial>>,
}

pub fn creer_entites(
    mut commandes: Commands,
    jeu: Res<PartieEnCours>,
    mut maillages: ResMut<Assets<Mesh>>,
    mut materiaux: ResMut<Assets<StandardMaterial>>,
) {
    let p = &jeu.partie;
    let corps = maillages.add(personnage());
    let couleurs: Vec<_> = PALETTE
        .iter()
        .map(|&hex| {
            materiaux.add(Color::srgb_u8(
                (hex >> 16) as u8,
                (hex >> 8) as u8,
                hex as u8,
            ))
        })
        .collect();
    commandes.spawn((
        JoueurAffiche,
        Mesh3d(corps.clone()),
        MeshMaterial3d(materiaux.add(Color::srgb(0.2, 0.55, 0.85))),
        Transform::default(),
    ));
    let capacite = p.ennemis.pool.len();
    for k in 0..capacite {
        commandes.spawn((
            EnnemiAffiche(k),
            Mesh3d(corps.clone()),
            MeshMaterial3d(couleurs[k % couleurs.len()].clone()),
            Transform::default(),
            Visibility::Hidden,
        ));
    }
    let j = &p.etat.joueur;
    commandes.insert_resource(Precedentes {
        joueur: Vec2::new(j.x, j.z),
        ennemis: HashMap::with_capacity(capacite),
        couleurs,
    });
}

/// Avant chaque pas : positions courantes, qui deviennent les précédentes.
pub fn noter_positions(jeu: Res<PartieEnCours>, mut prec: ResMut<Precedentes>) {
    let p = &jeu.partie;
    prec.joueur = Vec2::new(p.etat.joueur.x, p.etat.joueur.z);
    prec.ennemis.clear();
    for e in p.ennemis.actifs() {
        prec.ennemis.insert(e.id, Vec2::new(e.x, e.z));
    }
}

pub fn placer_entites(
    jeu: Res<PartieEnCours>,
    prec: Res<Precedentes>,
    temps: Res<Time<Fixed>>,
    mut cible: ResMut<crate::camera::Cible>,
    mut joueur: Query<&mut Transform, (With<JoueurAffiche>, Without<EnnemiAffiche>)>,
    mut ennemis: Query<(
        &EnnemiAffiche,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let t = temps.overstep_fraction();
    let p = &jeu.partie;
    let j = &p.etat.joueur;
    let ici = prec.joueur.lerp(Vec2::new(j.x, j.z), t);
    cible.0 = ici;
    for mut tr in &mut joueur {
        tr.translation = Vec3::new(ici.x, 0.0, ici.y);
        tr.rotation = Quat::from_rotation_y(j.angle);
    }
    let actifs = p.ennemis.actifs();
    for (EnnemiAffiche(k), mut tr, mut vis, mut materiau) in &mut ennemis {
        let Some(e) = actifs.get(*k).filter(|e| e.mort < 0.0) else {
            vis.set_if_neq(Visibility::Hidden);
            continue;
        };
        vis.set_if_neq(Visibility::Inherited);
        let maintenant = Vec2::new(e.x, e.z);
        let avant = prec.ennemis.get(&e.id).copied().unwrap_or(maintenant);
        let pos = avant.lerp(maintenant, t);
        tr.translation = Vec3::new(pos.x, 0.0, pos.y);
        tr.rotation = Quat::from_rotation_y(e.angle);
        tr.scale = Vec3::splat(e.def.echelle.unwrap_or(1.0));
        let couleur = &prec.couleurs[e.index_type % prec.couleurs.len()];
        if materiau.0 != *couleur {
            materiau.0 = couleur.clone();
        }
    }
}
