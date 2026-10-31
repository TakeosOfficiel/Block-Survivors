//! Son de Block Survivors par kira, sur PC comme dans le navigateur. Dans le navigateur,
//! cpal remplit deux tampons WebAudio depuis le fil principal (celui du jeu) : une image
//! plus longue qu'un tampon et le son se hache. D'où `taille_tampon` : plus grand, plus
//! robuste, mais plus de latence (voir docs/PROTOTYPE-RUST.md, pari 2).
//!
//! Prototype : deux sons synthétisés (coup, tir) ; les vrais sons (.ogg) viendront avec
//! les catégories et le son spatial (étape 3 du plan).

use std::sync::Arc;

use kira::backend::cpal::{CpalBackend, CpalBackendSettings, cpal};
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::{AudioManager, AudioManagerSettings, Frame};

/// Fréquence d'échantillonnage du jeu.
pub const FREQUENCE: u32 = 48_000;

pub struct Audio {
    gestionnaire: AudioManager<CpalBackend>,
}

impl Audio {
    /// Ouvre la sortie son ; `taille_tampon` en images (échantillons par voie), `None` :
    /// celle de cpal (2 048 dans le navigateur).
    pub fn ouvrir(taille_tampon: Option<u32>) -> Result<Self, String> {
        let config = taille_tampon.map(|n| cpal::StreamConfig {
            channels: 2,
            sample_rate: FREQUENCE,
            buffer_size: cpal::BufferSize::Fixed(n),
        });
        let reglages = AudioManagerSettings {
            backend_settings: CpalBackendSettings {
                device: None,
                config,
            },
            ..Default::default()
        };
        let gestionnaire = AudioManager::<CpalBackend>::new(reglages).map_err(|e| e.to_string())?;
        Ok(Self { gestionnaire })
    }

    pub fn jouer(&mut self, son: &StaticSoundData) {
        // Un son de plus que la capacité est simplement ignoré (pas de plantage en partie)
        let _ = self.gestionnaire.play(son.clone());
    }
}

/// Son synthétisé : `duree` s, échantillon `f(t)` entre -1 et 1.
fn synthese(duree: f32, f: impl Fn(f32) -> f32) -> StaticSoundData {
    let n = (duree * FREQUENCE as f32) as usize;
    let images: Arc<[Frame]> = (0..n)
        .map(|i| Frame::from_mono(f(i as f32 / FREQUENCE as f32)))
        .collect();
    StaticSoundData {
        sample_rate: FREQUENCE,
        frames: images,
        settings: StaticSoundSettings::default(),
        slice: None,
    }
}

/// Bruit pseudo-aléatoire déterministe (pas de dépendance d'aléa pour un son).
fn bruit(t: f32) -> f32 {
    let x = (t * 48_000.0) as u32;
    let h = x.wrapping_mul(0x9e37_79b9) ^ (x >> 15).wrapping_mul(0x85eb_ca6b);
    (h >> 8) as f32 / (1 << 23) as f32 - 1.0
}

/// Coup sourd (un zombie frappé) : sinus grave qui descend, vite éteint.
pub fn son_coup() -> StaticSoundData {
    synthese(0.18, |t| {
        let f = 140.0 - 300.0 * t;
        (core::f32::consts::TAU * f * t).sin() * (-t * 22.0).exp() * 0.6
            + bruit(t) * (-t * 60.0).exp() * 0.2
    })
}

/// Souffle (boule de feu tirée) : bruit filtré grossièrement, enveloppe montante puis éteinte.
pub fn son_tir() -> StaticSoundData {
    synthese(0.3, |t| {
        let enveloppe = (t * 30.0).min(1.0) * (-t * 9.0).exp();
        (bruit(t) + bruit(t - 1.0 / 48_000.0) + bruit(t - 2.0 / 48_000.0)) / 3.0 * enveloppe * 0.5
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sons_bornes() {
        for s in [son_coup(), son_tir()] {
            assert!(
                s.frames
                    .iter()
                    .all(|f| f.left.abs() <= 1.0 && f.right.abs() <= 1.0)
            );
            assert!(s.frames.len() > 4800);
        }
    }
}
