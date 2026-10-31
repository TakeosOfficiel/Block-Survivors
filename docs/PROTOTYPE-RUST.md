# Migration Rust : avancement et mesures du prototype

Journal de travail de la migration (plan : `docs/PLAN-RUST.md`). Tenu à jour à chaque
étape, pour ne rien perdre d'une session à l'autre. Branche : `rust` (jamais `main`).

## Où on en est

| Étape | État |
|---|---|
| 0. Socle (espace de travail, fenêtre PC et web, CI) | fait : build web corrigé, page vérifiée (WebGL2, WebGPU) |
| −1. Robot d'équilibrage, aléatoire à graine, pas fixe (TypeScript) | fait |
| 0 bis. Prototype des cinq paris | en cours : sauvegardes et relais faits, rendu en mesure |

---

## Étape 0 : socle

### Poids du build web (profil `web` : `opt-level = "z"`, LTO, `wasm-opt -Oz`)

Une fenêtre Bevy avec un bloc qui tourne (fonctionnalités : rendu 2D et 3D, UI) :

| Module | Brut | gzip -9 |
|---|---|---|
| `jeu-webgpu_bg.wasm` | 18 302 Ko | 6 111 Ko |
| `jeu-webgl2_bg.wasm` | 19 155 Ko | 6 451 Ko |

Premier build : le module WebGPU pesait autant que le WebGL2 (19 257 Ko) parce que la
fonctionnalité Bevy `default_platform` active `webgl2` : le module WebGPU embarquait les
deux. Corrigé en recopiant la liste de `default_platform` sans `webgl2` (ni Android) :
−955 Ko bruts, −358 Ko en gzip pour WebGPU.

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

Outils : `outils/page-web.mjs` (chargement, erreurs, capture) et `outils/pixels-webgpu.mjs`
(capture du rendu WebGPU). Chromium sans GPU (SwiftShader, rendu sur le processeur).

| Rendu | Démarrage | Erreurs | Image |
|---|---|---|---|
| WebGL2 | 1,7 s | aucune | `docs/captures/web-webgl2.png` : le bloc vert sur le ciel |
| WebGPU | 0,6 s | aucune (adaptateur `BrowserWebGpu`) | `docs/captures/web-webgpu.png` : identique |

- Les images/s mesurées (8 à 10) ne valent rien : c'est SwiftShader, sans carte graphique.
  La mesure à 60 images/s se fera sur une vraie machine (pari 1).
- Piège : la capture d'écran d'un canevas WebGPU sort **blanche** dans Chromium sans écran
  (même sous Xvfb), y compris pour une page WebGPU de dix lignes qui efface en rouge ; ce
  n'est pas Bevy. La capture WebGPU est donc lue dans la page (texture copiée juste après
  l'envoi de l'image). Couleurs vérifiées : bloc (110, 169, 83), ciel (120, 166, 255),
  comme en WebGL2.
- Bevy signale en WebGPU logiciel : pas d'occlusion ambiante (SSAO, faute de textures de
  stockage) ni de calcul pour les cartes d'environnement. Sans effet pour nous.

### CI GitHub (branche `rust`)

Premier passage (commit 62f77fe) : vert sur les trois jobs (vérifications 37 min, dont
30 min de tests en mode debug ; Windows 26 min ; web 21 min).

---

## Étape −1 : préparer la version TypeScript

### Aléatoire à graine
`src/utils/aleatoire.ts` : `alea()` (mulberry32) et `semerSimulation(graine)`. Tous les
tirages de la simulation (combat, entités, progression, `jeu.ts`) passent par `alea()` ;
les effets purement visuels (particules, secousses, sons, interface, pseudo) gardent
`Math.random` et ne changent donc rien à la partie. Vérifié : même graine, même
personnage, même arène → `parties.json` identique d'une exécution à l'autre.

### Pas fixe
Le robot fait avancer la simulation au pas fixe de 1/60 s. La boucle normale du jeu TS
garde son pas variable : son rendu n'interpole pas, un pas fixe y saccaderait sur les
écrans à plus de 60 Hz. La version Rust, elle, a le pas fixe partout (interpolation dans
`bs-rendu`).

### Le robot (`src/robot/robot.ts`, `npm run robot`)
`?jouer=1&robot=1&graine=N&personnage=X&arene=Y&duree=S[&invincible=1][&voir=1]` : le robot
joue une partie entière sans écran (≈ 1 s de calcul par 2 min de jeu) :
- déplacement : 16 directions notées (ennemis proches à 2 blocs d'anticipation, creepers
  fuis de loin, trajets des projectiles ennemis, murs, bords), attiré par l'orbe d'XP la
  plus proche où qu'elle soit ;
- cartes : barème fixe (soin si PV < 40 %, éveil, arme possédée, nouvelle arme, renfort,
  objet, passif).
`scripts/robot.mjs` joue N graines par personnage dans Chromium (4 en parallèle) et écrit
`parties.json` (chaque partie) et `reference.json` (moyenne, écart type, min, max).

### Références (prairie, profil vierge)

`docs/robot/` : 5 graines, joueur mortel. `docs/robot-invincible/` : 3 graines, joueur
invincible (puissance des armes sur toute la partie, jusqu'au boss).

| Personnage | Survie | Niveau | Tués | Invincible : victoires | Niveau | Tués | Dégâts/s à la fin |
|---|---|---|---|---|---|---|---|
| archer | 149.6 s (±96) | 4.4 | 97 | 3/3 | 31.3 | 3956 | 796 |
| guerrier | 152.3 s (±35) | 3.8 | 61 | 3/3 | 29 | 3567 | 837 |
| invocateur | 139.5 s (±37) | 4.2 | 65 | 0/3 | 10.7 | 509 | 34 |
| mage | 112.6 s (±31) | 4 | 55 | 3/3 | 27.3 | 3179 | 783 |

- Le robot meurt vite (2 à 3 min) : il esquive moins bien qu'un joueur. Ces chiffres
  servent de **référence de comparaison** pour la version Rust (même robot, mêmes
  graines, tolérance), pas de mesure de la difficulté réelle.
- **À surveiller** : l'invocateur invincible plafonne (niveau 11 en 20 min, contre 27 à 31
  pour les autres). Le robot prend volontiers ses invocations de soutien (totem, esprit
  protecteur : 0 dégât) et la partie s'enlise. Avant, il ne ramassait les orbes qu'à
  14 blocs (les invocations tuent loin) : corrigé, mieux (9 → 10,7) mais pas assez. Soit
  le barème des cartes du robot, soit un vrai défaut d'équilibrage : à trancher en
  regardant une partie (`&voir=1`).
