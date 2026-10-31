# Migration Rust : avancement et mesures du prototype

Journal de travail de la migration (plan : `docs/PLAN-RUST.md`). Tenu à jour à chaque
étape, pour ne rien perdre d'une session à l'autre. Branche : `rust` (jamais `main`).

## Où on en est

| Étape | État |
|---|---|
| 0. Socle (espace de travail, fenêtre PC et web, CI) | fait : build web corrigé, page vérifiée (WebGL2, WebGPU) |
| −1. Robot d'équilibrage, aléatoire à graine, pas fixe (TypeScript) | fait |
| 0 bis. Prototype des cinq paris | en cours : rendu, son, relais, sauvegardes mesurés ; mise à jour à faire |

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

---

## Étape 0 bis : les cinq paris du prototype

Prototype : `bs-regles` (simulation sans Bevy : 300 zombies, une arme à boules de feu,
grille spatiale, aléatoire PCG32 à graine, pas fixe de 1/60 s, aucune allocation dans la
boucle, vérifié par un test), `bs-rendu` (Bevy : maillages fusionnés partagés, rendu
interpolé entre deux pas, pilote automatique), `bs-audio`, `bs-protocole`, `bs-relais`,
`bs-sauvegarde`.

### Pari 1 : rendu de 300 zombies à 60 images/s

| Cible | Résultat | Image |
|---|---|---|
| Navigateur, WebGL2 | démarre en 0,6 s, aucune erreur | `docs/captures/proto-webgl2.png` |
| Navigateur, WebGPU | démarre en 0,5 s, aucune erreur | `docs/captures/proto-webgpu.png` |
| PC (Linux, Vulkan) | tourne, aucune erreur | `docs/captures/proto-pc-lavapipe.png` |

- Simulation : **65 µs par pas** (300 zombies, projectiles, grille) en natif, soit 0,4 %
  d'une image à 60 images/s. La simulation ne sera pas le goulot.
- Les images/s ne sont **pas mesurables ici** : la machine de travail n'a pas de carte
  graphique (SwiftShader dans Chromium : 1 à 2 images/s ; lavapipe sur PC : 8 images/s,
  tout est calculé par le processeur). Le pari « 60 images/s » reste à confirmer sur une
  vraie machine : la CI publie le build Windows (`block-survivors-windows`) et le build web
  (`block-survivors-web`, à servir avec `python3 -m http.server`, `?rendu=webgl2` ou
  `?rendu=webgpu`) ; les images/s s'affichent dans le titre de la fenêtre ou de l'onglet.
  **À faire par un humain** sur un PC moyen et un portable : noter les images/s ici.
- Le rendu est déjà économe : un maillage par type d'entité, partagé (Bevy les regroupe en
  un seul appel de dessin), aucune allocation par image.

### Pari 2 : le son (kira) dans le navigateur

Sur PC, cpal remplit le son sur un fil dédié : aucune gêne. Dans le navigateur (sans
threads wasm), cpal enchaîne deux tampons WebAudio remplis **depuis le fil principal**,
celui du jeu : une image plus longue qu'un tampon et le tampon suivant part en retard.

Banc d'essai : `rust/outils/banc-son.sh` (kira seul, sons en rafale, fil principal chargé
de N ms par image, ou une image lente toutes les 2 s). Trou = un tampon qui commence
après la fin du précédent. 10 s par essai, Chromium sans écran.

| Tampon (latence) | Charge par image | Images/s | Trous | Silence total |
|---|---|---|---|---|
| 2048 (43 ms, défaut cpal) | 0 / 16 / 30 ms | 60 / 60 / 33 | 0 | 0 |
| 2048 | 50 ms | 20 | 23 | 119 ms |
| 2048 | 80 ms | 12 | 100 | 2 288 ms |
| 2048 | image de 100 ms / 2 s | 58 | 5 | 184 ms |
| 2048 | image de 250 ms / 2 s | 54 | 22 | 891 ms |
| **4096 (85 ms)** | 0 à 80 ms | 60 à 12 | **0** | 0 |
| 4096 | image de 100 ms / 2 s | 58 | 1 | 3 ms |
| 4096 | image de 250 ms / 2 s | 55 | 7 | 507 ms |
| 8192 (171 ms) | 0 à 80 ms, image de 100 ms | 60 à 12 | 0 | 0 |
| 8192 | image de 250 ms / 2 s | 54 | 1 | 27 ms |

- **Verdict** : kira marche dans le navigateur sans hachures tant que le jeu tient au moins
  12 images/s, avec un tampon de **4096** (`Audio::ouvrir(Some(4096))`). Le prix : 85 ms
  de latence du son en navigateur (sur PC : défaut de cpal, quelques ms). Pour un jeu de
  survie à la souris, c'est acceptable ; un coup d'épée entendu avec 1/12 s de retard reste
  à juger à l'oreille.
- Les longues pauses (chargement d'arène, > 85 ms) hachent encore : couper ou baisser le son
  pendant les chargements, ou les découper.
- Défaut de cpal trouvé : après un retard, son curseur de temps ne se recale pas. Le
  tampon suivant part trop tôt et chevauche le précédent (compté dans le banc : autant de
  chevauchements que de trous). À signaler en amont (un `max(curseur, maintenant)`) ou à
  corriger par un correctif local de cpal.
- Option écartée pour l'instant : le mode `audioworklet` de cpal (son hors du fil
  principal) demande les threads wasm (Rust nightly, en-têtes COOP/COEP sur itch.io) :
  lourd. À reprendre seulement si 4096 déçoit à l'oreille.

### Pari 3 : relais en ligne (axum + tokio) et instantanés

`bs-relais` : salons de 4 (code de 6 lettres), refus d'une version de protocole
différente, l'hôte simule et envoie un instantané 20 fois par seconde (`bs-protocole` :
6 octets par zombie vivant). Mesure : `cargo run --release -p bs-relais --example
aller_retour` (hôte + 3 invités, local, 10 s).

| Mesure | Résultat |
|---|---|
| Instantané de fin de partie (294 zombies, 5 projectiles) | 1 808 octets |
| Débit par invité (20 instantanés/s) | 35,4 Ko/s |
| Décodage d'un instantané | 2 µs |
| Aller-retour invité → relais → hôte → relais → invité | médiane 0,48 ms, 90 % sous 0,7 ms, 99 % sous 1,4 ms (local) |

- Piège corrigé : sans `TCP_NODELAY`, Nagle retenait les petits messages et un aller-retour
  sur dix prenait **+40 ms**. Il faut nodelay **des deux côtés** (relais et client natif ;
  les navigateurs l'activent d'eux-mêmes).
- 35 Ko/s par invité, soit 105 Ko/s pour l'hôte et un relais à 1 Mo/s pour 10 salons pleins :
  tenable sur un petit serveur. Piste si besoin : instantanés différentiels.
- Reste : la mesure depuis la France vers un vrai relais déployé (latence réseau réelle).

### Pari 5 : importer une vraie sauvegarde (.bssave)

`bs-sauvegarde` lit et écrit le format du jeu TS (AES-256-GCM, clé = SHA-256 du secret,
en-tête `BSSAVE1:`, IV de 12 octets + chiffré en base64). Test sur un **vrai** fichier
exporté par le jeu TS 0.50.0 (`scripts/exporter-bssave.mjs` : pseudo « Éloïse », deux
parties) : `rust/crates/bs-sauvegarde/tests/donnees/profil-0.50.0.bssave`, relu champ par
champ (pseudo accentué, records, statistiques), et l'aller-retour Rust → Rust est
identique. Fichier illisible ou vide : erreur propre, pas de plantage. **Pari tenu.**
