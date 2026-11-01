//! Maillages provisoires, faits de pavés fusionnés en un seul maillage par modèle (un
//! seul appel de dessin par lot) : personnage cubique (jambes, corps, bras, tête).

use bevy::prelude::*;

fn pave(taille: Vec3, centre: Vec3) -> Mesh {
    Cuboid::from_size(taille)
        .mesh()
        .build()
        .translated_by(centre)
}

fn fusion(pieces: impl IntoIterator<Item = Mesh>) -> Mesh {
    let mut pieces = pieces.into_iter();
    let mut m = pieces.next().expect("au moins une pièce");
    for p in pieces {
        m.merge(&p).expect("pièces de même format");
    }
    m
}

/// Personnage de 1,8 bloc de haut, le devant vers +Z.
pub fn personnage() -> Mesh {
    fusion([
        pave(Vec3::new(0.24, 0.7, 0.24), Vec3::new(-0.13, 0.35, 0.0)),
        pave(Vec3::new(0.24, 0.7, 0.24), Vec3::new(0.13, 0.35, 0.0)),
        pave(Vec3::new(0.5, 0.7, 0.26), Vec3::new(0.0, 1.05, 0.0)),
        pave(Vec3::new(0.2, 0.65, 0.22), Vec3::new(-0.36, 1.05, 0.12)),
        pave(Vec3::new(0.2, 0.65, 0.22), Vec3::new(0.36, 1.05, 0.12)),
        pave(Vec3::new(0.46, 0.46, 0.46), Vec3::new(0.0, 1.63, 0.0)),
    ])
}
