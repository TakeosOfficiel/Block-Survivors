#!/usr/bin/env bash
# Construit la version navigateur dans web/dist : deux modules WebAssembly (WebGPU et
# WebGL2, que Bevy ne sait pas réunir), choisis par la page selon `navigator.gpu`.
#   outils/build-web.sh          profil « web » (taille minimale, pour publier)
#   outils/build-web.sh --dev    profil de développement (compilation rapide)
set -euo pipefail
cd "$(dirname "$0")/.."

profil=web
[ "${1:-}" = "--dev" ] && profil=dev
dossier=$([ "$profil" = dev ] && echo debug || echo "$profil")
sortie=web/dist
rm -rf "$sortie"
mkdir -p "$sortie"

for rendu in webgpu webgl2; do
  cargo build -q --profile "$profil" --target wasm32-unknown-unknown -p bs-web \
    --no-default-features --features "$rendu"
  wasm-bindgen --target web --no-typescript --out-dir "$sortie" --out-name "jeu-$rendu" \
    "target/wasm32-unknown-unknown/$dossier/bs_web.wasm"
  # Taille minimale pour la version publiée (si binaryen est installé)
  if [ "$profil" = web ] && command -v wasm-opt >/dev/null; then
    wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
      -o "$sortie/jeu-$rendu.tmp.wasm" "$sortie/jeu-${rendu}_bg.wasm"
    mv "$sortie/jeu-$rendu.tmp.wasm" "$sortie/jeu-${rendu}_bg.wasm"
  fi
done
cp web/index.html "$sortie/"

# Poids (brut et compressé) : suivi de près, Bevy en WebAssembly est lourd
for f in "$sortie"/*.wasm; do
  printf '%s : %s Ko (%s Ko en gzip)\n' "$(basename "$f")" "$(( $(stat -c %s "$f") / 1024 ))" "$(( $(gzip -9c "$f" | wc -c) / 1024 ))"
done
