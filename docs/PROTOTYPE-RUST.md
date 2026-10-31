# Migration Rust : avancement et mesures du prototype

Journal de travail de la migration (plan : `docs/PLAN-RUST.md`). Tenu à jour à chaque
étape, pour ne rien perdre d'une session à l'autre. Branche : `rust` (jamais `main`).

## Où on en est

| Étape | État |
|---|---|
| 0. Socle (espace de travail, fenêtre PC et web, CI) | fait (commit 62f77fe), build web corrigé |
| −1. Robot d'équilibrage, aléatoire à graine, pas fixe (TypeScript) | en cours |
| 0 bis. Prototype des cinq paris | à faire |

---

## Étape 0 : socle

### Poids du build web (profil `web` : `opt-level = "z"`, LTO, `wasm-opt -Oz`)

Une fenêtre Bevy avec un bloc qui tourne (fonctionnalités : rendu 2D et 3D, UI) :

| Module | Brut | gzip -9 |
|---|---|---|
| `jeu-webgpu_bg.wasm` | 19 257 Ko | 6 469 Ko |
| `jeu-webgl2_bg.wasm` | 19 186 Ko | 6 443 Ko |

Sans `wasm-opt`, la sortie de wasm-bindgen pèse 75 Mo : `wasm-opt` est indispensable.
C'est déjà le socle de Bevy (rendu, UI, ECS) ; le jeu complet ajoutera surtout du code
de jeu, plus léger. Objectif du plan : moins de 20 Mo compressés, tenable à ce rythme.
Pistes si ça grossit : retirer les fonctionnalités Bevy inutiles (2D ?), `wasm-opt -O3`
contre `-Oz` à comparer, compression Brotli côté serveur (≈ 20 % de mieux que gzip).

Durée du build web complet (deux modules, machine à 4 cœurs, sans cache) : environ 15 min.

### Problème trouvé et corrigé : wasm-opt d'Ubuntu trop ancien

Le binaryen d'Ubuntu (`apt install binaryen`, version 108) casse la table des références
de wasm-bindgen : la page affichait « WebAssembly.Table.grow(): failed to grow table by 4 »
(en WebGPU comme en WebGL2), alors que la CI était verte (elle ne charge pas la page).
Correction : `cargo install wasm-opt --version 0.116.1 --locked` (binaryen 116), plus
`--enable-reference-types`. `outils/build-web.sh` refuse désormais un wasm-opt < 116.

### La page dans Chromium (sans écran)

À compléter (capture, rendu choisi, erreurs).

### CI GitHub (branche `rust`)

Premier passage (commit 62f77fe) : vert sur les trois jobs (vérifications 37 min, dont
30 min de tests en mode debug ; Windows 26 min ; web 21 min).

---

## Étape −1 : préparer la version TypeScript

À compléter.
