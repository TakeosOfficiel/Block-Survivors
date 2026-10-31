#!/usr/bin/env bash
# Essai de bout en bout de la chaîne de mise à jour (pari 4), sous Linux : empaquette
# l'appli de démonstration (bs-maj/examples/demo_maj) avec Velopack en versions successives,
# signe chaque manifeste, installe N et vérifie qu'elle se met à jour seule ; puis que les
# attaques sont refusées (manifeste falsifié, rejoué, expiré, mauvaise clé, paquet corrompu).
# Demande : vpk (dotnet tool install -g vpk), mksquashfs, zstd, FUSE. À lancer depuis rust/.
set -euo pipefail
T=$(mktemp -d)
export BS_JOURNAL=$T/journal.log BS_FICHIER_SERIE=$T/serie
rm -rf /var/tmp/velopack/BSDemo
cargo build -q --release -p bs-maj --example demo_maj --bin bs-signer
S=$PWD/target/release/bs-signer
mkdir -p "$T/app" "$T/depot" "$T/install"
cp target/release/examples/demo_maj "$T/app/"
$S cles > "$T/cles.txt"
export BS_CLE_PRIVEE=$(sed -n 's/^privée *: //p' "$T/cles.txt") BS_CLE_PUBLIQUE=$(sed -n 's/^publique : //p' "$T/cles.txt")

empaqueter() { # version série
  echo "données $1" > "$T/app/donnees.txt"
  (cd "$T" && vpk pack -u BSDemo -v "$1" -p "$T/app" -e demo_maj -o "$T/depot" -c stable > /dev/null)
  $S manifeste "$T/depot" stable "$2" 30 0.1.0 > /dev/null
}
lancer() { (cd "$T/install" && timeout 120 ./BSDemo-stable.AppImage "$@" > /dev/null 2>&1 || true); sleep 3; }
attendu() { # texte attendu dans la dernière ligne du journal
  local derniere; derniere=$(tail -1 "$BS_JOURNAL")
  if [[ "$derniere" == *"$1"* ]]; then echo "ok    : $derniere"; else echo "ÉCHEC : attendu « $1 », journal : $derniere"; exit 1; fi
}

echo "== installation de N puis mises à jour"
empaqueter 0.1.0 1
cp "$T/depot/BSDemo-stable.AppImage" "$T/install/"
lancer; attendu "démarré : version 0.1.0"
empaqueter 0.2.0 2
lancer --maj "$T/depot"; attendu "démarré : version 0.2.0"
grep -q "0.2.0 téléchargée (paquet complet)" "$BS_JOURNAL" && echo "ok    : N → N+1 par le paquet complet (pas encore de base locale)"
empaqueter 0.3.0 3
lancer --maj "$T/depot"; attendu "démarré : version 0.3.0"
grep -q "0.3.0 téléchargée (delta)" "$BS_JOURNAL" && echo "ok    : N+1 → N+2 par le delta ($(stat -c %s "$T"/depot/BSDemo-0.3.0-stable-delta.nupkg) octets, complet : $(stat -c %s "$T"/depot/BSDemo-0.3.0-stable-full.nupkg))"
lancer --maj "$T/depot"; attendu "à jour"

echo "== attaques"
empaqueter 0.4.0 4
cp "$T/depot/version.json" "$T/bon.json"
# Manifeste falsifié : une taille changée, signature d'origine
sed -i 's/\\"taille\\": \([0-9]\)/\\"taille\\": 9\1/' "$T/depot/version.json"
lancer --maj "$T/depot"; attendu "refusé : signature du manifeste invalide"
# Manifeste signé par une autre clé
BS_CLE_PRIVEE=$($S cles | sed -n 's/^privée *: //p') $S manifeste "$T/depot" stable 5 30 0.1.0 > /dev/null
lancer --maj "$T/depot"; attendu "refusé : signature du manifeste invalide"
# Manifeste expiré (0 jour de validité)
$S manifeste "$T/depot" stable 5 0 0.1.0 > /dev/null
lancer --maj "$T/depot"; attendu "refusé : manifeste expiré"
# Vieux manifeste rejoué (série 2 < 3, déjà vue)
$S manifeste "$T/depot" stable 2 30 0.1.0 > /dev/null
lancer --maj "$T/depot"; attendu "refusé : manifeste ancien"
# Paquets corrompus (delta et complet), manifeste valide
cp "$T/bon.json" "$T/depot/version.json"
for f in "$T"/depot/BSDemo-0.4.0-stable-*.nupkg; do printf 'X' | dd of="$f" bs=1 seek=100 conv=notrunc status=none; done
lancer --maj "$T/depot"; attendu "téléchargement refusé"
lancer; attendu "démarré : version 0.3.0"
echo "== tout est bon (dossier : $T)"
