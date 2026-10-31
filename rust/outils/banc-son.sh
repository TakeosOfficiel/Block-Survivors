#!/usr/bin/env bash
# Construit le banc d'essai du son (exemple bs-audio/banc_son, wasm) puis le mesure dans
# Chromium sans écran : voir outils/banc-son.mjs. À lancer depuis rust/.
set -euo pipefail
cargo build --target wasm32-unknown-unknown --profile web -p bs-audio --example banc_son
sortie=$(mktemp -d)
wasm-bindgen --target web --no-typescript --out-dir "$sortie" target/wasm32-unknown-unknown/web/examples/banc_son.wasm
node outils/banc-son.mjs "$sortie"
