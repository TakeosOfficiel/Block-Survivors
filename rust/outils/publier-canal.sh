#!/usr/bin/env bash
# Publie un canal Velopack (« win-stable », « linux-beta »…) sur le dépôt public de versions,
# dans sa release roulante « rust-<canal> » (pré-version : la dernière version du dépôt
# reste celle d'Electron). Ordre : paquets et installeurs, puis retrait des fichiers de la
# version précédente, puis le manifeste signé (version.json) en dernier : une version n'est
# annoncée qu'une fois tous ses fichiers en ligne.
#   publier-canal.sh <dépôt> <canal> <version> <version min coop> <dossier de vpk>
# Demande gh et GH_TOKEN (droit d'écrire sur le dépôt).
set -euo pipefail
DEPOT=$1 CANAL=$2 VERSION=$3 MIN_COOP=$4 DOSSIER=$5
TAG="rust-$CANAL"
NOTES="Block Survivors $VERSION (version Rust, canal $CANAL). Coop dès la version $MIN_COOP."

if ! gh release view "$TAG" --repo "$DEPOT" > /dev/null 2>&1; then
  gh release create "$TAG" --repo "$DEPOT" --prerelease --latest=false \
    --title "Block Survivors Rust ($CANAL)" --notes "$NOTES"
else
  gh release edit "$TAG" --repo "$DEPOT" --notes "$NOTES"
fi

mapfile -t FICHIERS < <(find "$DOSSIER" -maxdepth 1 -type f ! -name version.json -printf '%f\n' | sort)
for f in "${FICHIERS[@]}"; do
  gh release upload "$TAG" "$DOSSIER/$f" --repo "$DEPOT" --clobber
done

# Fichiers de la version précédente (paquets, installeurs renommés) : retirés
gh release view "$TAG" --repo "$DEPOT" --json assets --jq '.assets[].name' | while read -r nom; do
  if [ "$nom" != version.json ] && ! printf '%s\n' "${FICHIERS[@]}" | grep -qxF "$nom"; then
    gh release delete-asset "$TAG" "$nom" --repo "$DEPOT" --yes
  fi
done

gh release upload "$TAG" "$DOSSIER/version.json" --repo "$DEPOT" --clobber
echo "$CANAL : version $VERSION publiée (rust-$CANAL)"
