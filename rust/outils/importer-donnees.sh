#!/usr/bin/env bash
# Régénère rust/donnees/*.ron depuis la version TypeScript (src/donnees) :
# export JSON par Vite (outils/exporter-donnees.mjs), puis importeur Rust qui
# vérifie chaque table contre ses types, écrit le RON et valide les références.
#
# Usage : rust/outils/importer-donnees.sh            (réécrit rust/donnees)
#         rust/outils/importer-donnees.sh --verifier (échoue si rust/donnees n'est pas à jour)
set -euo pipefail
racine="$(cd "$(dirname "$0")/../.." && pwd)"
json="$(mktemp -d)"
trap 'rm -rf "$json" "${ron:-}"' EXIT

(cd "$racine" && node rust/outils/exporter-donnees.mjs "$json" > /dev/null)

if [[ "${1:-}" == "--verifier" ]]; then
  ron="$(mktemp -d)"
  (cd "$racine/rust" && cargo run -q -p bs-donnees --bin importer-donnees -- "$json" "$ron")
  if ! diff -r "$racine/rust/donnees" "$ron" > /dev/null; then
    diff -ru "$racine/rust/donnees" "$ron" | head -40
    echo "rust/donnees n'est pas à jour : lancer rust/outils/importer-donnees.sh" >&2
    exit 1
  fi
  echo "rust/donnees est à jour"
else
  (cd "$racine/rust" && cargo run -q -p bs-donnees --bin importer-donnees -- "$json" donnees)
fi
