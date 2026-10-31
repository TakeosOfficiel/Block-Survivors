//! Banc d'essai du son dans le navigateur (pari 2 du prototype) : la page (outils/banc-son.html)
//! ouvre kira avec une taille de tampon donnée, joue des sons en rafale et charge le fil
//! principal comme le ferait une image de jeu lente ; elle compte les silences.

#[cfg(target_arch = "wasm32")]
mod web {
    use std::cell::RefCell;

    use bs_audio::{Audio, son_coup, son_tir};
    use kira::sound::static_sound::StaticSoundData;
    use wasm_bindgen::prelude::*;

    thread_local! {
        static ETAT: RefCell<Option<(Audio, StaticSoundData, StaticSoundData)>> = const { RefCell::new(None) };
    }

    /// Ouvre le son ; `tampon` = 0 : taille par défaut de cpal.
    #[wasm_bindgen]
    pub fn demarrer(tampon: u32) -> Result<(), JsValue> {
        let audio =
            Audio::ouvrir((tampon > 0).then_some(tampon)).map_err(|e| JsValue::from_str(&e))?;
        ETAT.with(|e| *e.borrow_mut() = Some((audio, son_coup(), son_tir())));
        Ok(())
    }

    #[wasm_bindgen]
    pub fn jouer(numero: u32) {
        ETAT.with(|e| {
            if let Some((audio, coup, tir)) = e.borrow_mut().as_mut() {
                audio.jouer(if numero.is_multiple_of(2) { coup } else { tir });
            }
        });
    }
}
