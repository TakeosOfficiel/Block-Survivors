//! Mode « scène » des captures (`BS_SCENE=lacet,tangage,distance,champ[,decalage]`,
//! angles en degrés) : la caméra se fige derrière ces réglages autour du joueur (visée
//! décalée de `decalage` blocs vers la gauche : il paraît à droite de l'image), qui
//! reste immobile, désarmé, invisible (son ombre reste) et sans dégâts, les mobs en
//! cercle autour de lui ; l'interface est masquée. Le héros de l'accueil prend sa place,
//! vu par la même caméra (`menu::apercu`). Sert à photographier
//! l'arène vue du sol pour le fond de l'accueil (`BS_CAPTURE`).

use std::sync::OnceLock;

use bevy::prelude::*;

/// Réglages de la caméra de scène (radians, blocs).
pub struct Scene {
    pub lacet: f32,
    pub tangage: f32,
    pub distance: f32,
    pub champ: f32,
    /// Visée décalée vers la gauche (blocs).
    pub decalage: f32,
}

/// Vrai en capture de scène.
pub fn immobile() -> bool {
    scene().is_some()
}

/// La scène demandée, lue une fois.
pub fn scene() -> Option<&'static Scene> {
    static SCENE: OnceLock<Option<Scene>> = OnceLock::new();
    SCENE
        .get_or_init(|| {
            let v = std::env::var("BS_SCENE").ok()?;
            let n: Vec<f32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
            let (&[lacet, tangage, distance, champ], reste) = n.split_first_chunk::<4>()?;
            Some(Scene {
                lacet: lacet.to_radians(),
                tangage: tangage.to_radians(),
                distance,
                champ: champ.to_radians(),
                decalage: reste.first().copied().unwrap_or(0.0),
            })
        })
        .as_ref()
}

/// Masque toute l'interface (racines des nœuds).
pub fn masquer_interface(mut racines: Query<&mut Node, Without<ChildOf>>) {
    if scene().is_none() {
        return;
    }
    for mut n in &mut racines {
        if n.display != Display::None {
            n.display = Display::None;
        }
    }
}
