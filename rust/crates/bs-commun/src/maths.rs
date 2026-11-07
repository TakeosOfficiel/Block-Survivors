//! Fonctions mathématiques de la simulation, par libm : mêmes résultats en natif et en
//! WebAssembly (les fonctions de la bibliothèque standard dépendent de la plateforme).
//! Reprend aussi utils/maths.ts (lissage, bornes, écart d'angle).

pub use core::f32::consts::{PI, TAU};

#[inline]
pub fn sin(x: f32) -> f32 {
    libm::sinf(x)
}

#[inline]
pub fn cos(x: f32) -> f32 {
    libm::cosf(x)
}

#[inline]
pub fn atan2(y: f32, x: f32) -> f32 {
    libm::atan2f(y, x)
}

#[inline]
pub fn exp(x: f32) -> f32 {
    libm::expf(x)
}

#[inline]
pub fn pow(x: f32, y: f32) -> f32 {
    libm::powf(x, y)
}

#[inline]
pub fn ln(x: f32) -> f32 {
    libm::logf(x)
}

/// Racine carrée (exacte en IEEE 754 : identique partout).
#[inline]
pub fn racine(x: f32) -> f32 {
    x.sqrt()
}

/// Longueur du vecteur (x, z).
#[inline]
pub fn longueur(x: f32, z: f32) -> f32 {
    (x * x + z * z).sqrt()
}

/// Facteur d'approche exponentielle indépendant du pas : 0 = immobile, 1 = cible atteinte.
#[inline]
pub fn facteur_lissage(raideur: f32, dt: f32) -> f32 {
    1.0 - exp(-raideur * dt)
}

#[inline]
pub fn borner(valeur: f32, min: f32, max: f32) -> f32 {
    if valeur < min {
        min
    } else if valeur > max {
        max
    } else {
        valeur
    }
}

/// Écart angulaire signé le plus court pour aller de `depuis` à `vers` (rad).
#[inline]
pub fn ecart_angle(depuis: f32, vers: f32) -> f32 {
    atan2(sin(vers - depuis), cos(vers - depuis))
}
