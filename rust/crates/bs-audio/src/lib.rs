//! Son de Block Survivors par kira (src/audio/sons.ts de la version TypeScript) : les
//! sons vanilla (.ogg décodés une fois), joués par catégorie (joueur, armes, créatures,
//! interface) avec le volume général et celui de la catégorie, une hauteur tirée dans
//! les bornes de chaque son, un délai minimal entre deux lectures et un nombre de voix
//! limité (la plus ancienne s'arrête pour laisser place à la nouvelle).
//! Sans sortie son (machine sans carte, CI), tout reste muet sans erreur.

use std::io::Cursor;

use bs_donnees::effets::{CategorieSon, Son};
use kira::backend::cpal::CpalBackend;
use kira::sound::PlaybackState;
use kira::sound::static_sound::StaticSoundHandle;
use kira::{AudioManager, AudioManagerSettings, Decibels, PlaybackRate, Tween};

pub use kira::sound::static_sound::StaticSoundData;

/// Volumes de 0 à 1 (options du jeu, src/options.ts).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Volumes {
    pub general: f32,
    /// Dans l'ordre de `CategorieSon` : joueur, armes, créatures, interface.
    pub categories: [f32; 4],
}

impl Default for Volumes {
    fn default() -> Self {
        Self {
            general: 0.7,
            categories: [1.0; 4],
        }
    }
}

impl Volumes {
    pub fn de(&self, c: CategorieSon) -> f32 {
        let k = match c {
            CategorieSon::Joueur => 0,
            CategorieSon::Armes => 1,
            CategorieSon::Creatures => 2,
            CategorieSon::Interface => 3,
        };
        self.general * self.categories[k]
    }
}

/// Décode un fichier .ogg.
pub fn decoder(octets: Vec<u8>) -> Result<StaticSoundData, String> {
    StaticSoundData::from_cursor(Cursor::new(octets)).map_err(|e| e.to_string())
}

/// Un son de la table : données (une fois téléchargées), dernière lecture, voix.
struct Canal {
    donnees: Option<StaticSoundData>,
    dernier: f64,
    voix: Vec<StaticSoundHandle>,
}

/// Tous les sons du jeu, repérés par leur index dans la table des données.
pub struct Mixeur {
    sortie: Option<AudioManager<CpalBackend>>,
    canaux: Vec<Canal>,
    pub volumes: Volumes,
    alea: u32,
}

/// Amplitude (0 à 1) en décibels pour kira.
fn decibels(a: f32) -> Decibels {
    if a <= 0.0 {
        Decibels::SILENCE
    } else {
        Decibels(20.0 * a.log10())
    }
}

impl Mixeur {
    /// Ouvre la sortie son pour `n` sons ; `muet` : ne l'ouvre pas.
    pub fn nouveau(n: usize, muet: bool) -> Self {
        let sortie = if muet {
            None
        } else {
            AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).ok()
        };
        Self {
            sortie,
            canaux: (0..n)
                .map(|_| Canal {
                    donnees: None,
                    dernier: f64::NEG_INFINITY,
                    voix: Vec::new(),
                })
                .collect(),
            volumes: Volumes::default(),
            alea: 0x2545_f491,
        }
    }

    /// Une sortie son est ouverte.
    pub fn actif(&self) -> bool {
        self.sortie.is_some()
    }

    /// Le son `k` est prêt (téléchargé et décodé).
    pub fn pret(&self, k: usize) -> bool {
        self.canaux.get(k).is_some_and(|c| c.donnees.is_some())
    }

    /// Donne au son `k` (défini par `def`) ses échantillons.
    pub fn charger(&mut self, k: usize, def: &Son, donnees: StaticSoundData) {
        if let Some(c) = self.canaux.get_mut(k) {
            c.donnees = Some(donnees);
            c.voix = Vec::with_capacity(def.simultanes.max(1) as usize);
        }
    }

    fn hasard(&mut self) -> f32 {
        // xorshift : le son n'a pas besoin de l'aléatoire à graine de la simulation
        let mut x = self.alea;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.alea = x;
        (x >> 8) as f32 / (1 << 24) as f32
    }

    /// Joue le son `k` ; `hauteur` multiplie la hauteur tirée au hasard ; `maintenant`
    /// en secondes (horloge réelle).
    pub fn jouer(&mut self, k: usize, def: &Son, hauteur: f32, maintenant: f64) {
        let volume = self.volumes.de(def.categorie) * def.volume;
        if volume <= 0.0 || self.sortie.is_none() {
            return;
        }
        let [min, max] = def.hauteur;
        let vitesse = (hauteur * (min + self.hasard() * (max - min))).clamp(0.25, 4.0);
        let Some(c) = self.canaux.get_mut(k) else {
            return;
        };
        let Some(donnees) = &c.donnees else {
            return;
        };
        if maintenant - c.dernier < f64::from(def.intervalle_min) {
            return;
        }
        c.dernier = maintenant;
        c.voix.retain(|v| v.state() != PlaybackState::Stopped);
        if c.voix.len() >= def.simultanes.max(1) as usize {
            let mut ancienne = c.voix.remove(0);
            ancienne.stop(Tween::default());
        }
        let son = donnees
            .volume(decibels(volume.min(1.0)))
            .playback_rate(PlaybackRate(f64::from(vitesse)));
        if let Some(sortie) = self.sortie.as_mut()
            && let Ok(v) = sortie.play(son)
        {
            c.voix.push(v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volumes_et_decibels() {
        let v = Volumes {
            general: 0.5,
            categories: [1.0, 0.0, 0.5, 1.0],
        };
        assert_eq!(v.de(CategorieSon::Armes), 0.0);
        assert_eq!(v.de(CategorieSon::Creatures), 0.25);
        assert_eq!(decibels(1.0).0, 0.0);
        assert!((decibels(0.1).0 + 20.0).abs() < 1e-4);
    }

    #[test]
    fn muet_sans_sortie() {
        let mut m = Mixeur::nouveau(1, true);
        assert!(!m.actif());
        let d = bs_donnees::Donnees::embarquees().expect("données");
        let (_, def) = d.sons.get_index(0).expect("un son");
        m.jouer(0, def, 1.0, 0.0);
        assert!(!m.pret(0));
    }
}
