//! Mode « scène » des captures (`BS_SCENE=lacet,tangage,distance,champ`, angles en
//! degrés) : la caméra se fige derrière ces réglages autour du joueur, qui reste
//! immobile, invisible et sans dégâts ; l'interface est masquée. Sert à photographier
//! l'arène vue du sol pour le fond de l'accueil (`BS_CAPTURE`).

use std::sync::OnceLock;

use bevy::prelude::*;

/// Réglages de la caméra de scène (radians, blocs).
pub struct Scene {
    pub lacet: f32,
    pub tangage: f32,
    pub distance: f32,
    pub champ: f32,
}

/// La scène demandée, lue une fois.
pub fn scene() -> Option<&'static Scene> {
    static SCENE: OnceLock<Option<Scene>> = OnceLock::new();
    SCENE
        .get_or_init(|| {
            let v = std::env::var("BS_SCENE").ok()?;
            let n: Vec<f32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
            let [lacet, tangage, distance, champ] = n[..] else {
                return None;
            };
            Some(Scene {
                lacet: lacet.to_radians(),
                tangage: tangage.to_radians(),
                distance,
                champ: champ.to_radians(),
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
