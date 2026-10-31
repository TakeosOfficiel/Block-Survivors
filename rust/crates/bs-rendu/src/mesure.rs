//! Mesure des performances : images/s (moyenne, pire image) et temps d'un pas de
//! simulation, écrits toutes les 5 s dans la console (« MESURE … », lu par les scripts) et
//! dans le titre de la fenêtre (onglet du navigateur) : lisible sans console sur un vrai PC.
//! Sur PC, pour les mesures sans personne : `BS_DUREE=s` quitte au bout de s secondes,
//! `BS_CAPTURE=fichier.png` enregistre une capture à mi-parcours.

use std::time::Duration;

use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Mesure {
    images: u32,
    pire_image: f32,
    duree: f32,
    pas: u32,
    duree_pas: Duration,
}

impl Mesure {
    pub fn ajouter_pas(&mut self, d: Duration) {
        self.pas += 1;
        self.duree_pas += d;
    }
}

pub struct PluginMesure;

impl Plugin for PluginMesure {
    fn build(&self, app: &mut App) {
        app.init_resource::<Mesure>().add_systems(Last, compter);
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(duree) = std::env::var("BS_DUREE") {
            let duree: f32 = duree.parse().expect("BS_DUREE en secondes");
            app.insert_resource(Automatique {
                duree,
                capture: std::env::var("BS_CAPTURE").ok(),
            })
            .add_systems(Update, automatique);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource)]
struct Automatique {
    duree: f32,
    capture: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
fn automatique(
    mut commandes: Commands,
    temps: Res<Time<Real>>,
    mut a: ResMut<Automatique>,
    mut sortie: MessageWriter<AppExit>,
) {
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    let t = temps.elapsed_secs();
    if t >= a.duree / 2.0
        && let Some(chemin) = a.capture.take()
    {
        commandes
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(chemin));
    }
    if t >= a.duree {
        sortie.write(AppExit::Success);
    }
}

fn compter(temps: Res<Time<Real>>, mut m: ResMut<Mesure>, mut fenetres: Query<&mut Window>) {
    let dt = temps.delta_secs();
    m.images += 1;
    m.duree += dt;
    m.pire_image = m.pire_image.max(dt);
    if m.duree >= 5.0 {
        let (ips, pire) = (m.images as f32 / m.duree, m.pire_image * 1000.0);
        let pas = m.duree_pas.as_secs_f64() * 1e6 / m.pas.max(1) as f64;
        info!("MESURE images/s {ips:.1} pire image {pire:.1} ms pas de simulation {pas:.1} µs");
        for mut f in &mut fenetres {
            f.title = format!(
                "Block Survivors {} · {ips:.0} images/s (pire {pire:.0} ms) · simulation {pas:.0} µs",
                bs_commun::VERSION
            );
        }
        *m = Mesure::default();
    }
}
