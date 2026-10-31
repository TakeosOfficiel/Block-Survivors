//! Point d'entrée du navigateur : la page (web/index.html) charge ce module WebAssembly,
//! qui lance le jeu dans le canevas `#jeu`.

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn demarrer() {
    block_survivors::lancer();
}
