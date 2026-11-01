# Migration Rust : avancement et mesures du prototype

Journal de travail de la migration (plan : `docs/PLAN-RUST.md`). Tenu à jour à chaque
étape, pour ne rien perdre d'une session à l'autre. Branche : `rust` (jamais `main`).

## Où on en est

| Étape | État |
|---|---|
| 0. Socle (espace de travail, fenêtre PC et web, CI) | fait : build web corrigé, page vérifiée (WebGL2, WebGPU) |
| −1. Robot d'équilibrage, aléatoire à graine, pas fixe (TypeScript) | fait |
| 0 bis. Prototype des cinq paris | fait : on continue, avec ajustements (voir la décision) ; 94 images/s mesurées sur le PC du joueur |
| 1. Données et règles, sans rendu | fait : données, monde, simulation complète et robot (`bs-robot`) ; ses moyennes tombent dans la tolérance de celles du jeu TS (voir 1b) |
| 2. Monde et rendu | en cours : 2a (textures vanilla, arène affichée, vraie partie), 2b (modèles Minecraft des entités), 2c (maps .schem importées), 2d-1 (projectiles, orbes, objets, coffres, ombres) et 2d-2 (particules, chiffres de dégâts, barres de vie, secousse) faits ; 2d-3a (lueurs, mêlée, foudre) fait ; 2d-3b, 2d-3c et 2d-4 à faire |

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

`docs/robot/` : 5 graines, joueur mortel. `docs/robot-15/` : 15 graines, joueur mortel
(celle que vérifie le robot Rust). `docs/robot-invincible/` : 3 graines, joueur
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

- **Mesure sur un vrai PC Windows** (build `ship` de la CI, 31/10/2026) : **94 images/s**,
  pire image 28 ms, simulation 95 µs par pas. Le pari « 60 images/s » est **tenu** avec de la
  marge. La pire image à 28 ms est un pic isolé (démarrage ou chargement probable) ; à
  surveiller quand le vrai rendu arrivera.
- Simulation : **65 µs par pas** (300 zombies, projectiles, grille) en natif, soit 0,4 %
  d'une image à 60 images/s. La simulation ne sera pas le goulot.
- Les images/s ne sont **pas mesurables ici** : la machine de travail n'a pas de carte
  graphique (SwiftShader dans Chromium : 1 à 2 images/s ; lavapipe sur PC : 8 images/s,
  tout est calculé par le processeur). Le pari « 60 images/s » reste à confirmer sur une
  vraie machine : la CI publie à chaque push sur `rust` le build Windows
  (`block-survivors-windows.exe`) et le build web (`block-survivors-web.zip`, à dézipper et
  servir avec `python3 -m http.server`, `?rendu=webgl2` ou `?rendu=webgpu`) dans la
  pré-version **« prototype-rust »** (onglet Releases du dépôt, privé ; le quota des
  artefacts d'Actions du compte était plein) ; les images/s s'affichent dans le titre de
  la fenêtre ou de l'onglet.
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

### Pari 4 : mise à jour Velopack de N à N+1, manifeste signé

`bs-maj` : `SourceSignee`, une source de mises à jour Velopack qui lit d'abord
`version.json` (manifeste signé en ed25519), le vérifie (signature par la clé de la CI **ou**
la clé de secours, numéro de série jamais en arrière, date d'expiration, voie) et ne
propose à Velopack **que** les paquets qu'il liste ; chaque fichier téléchargé est revérifié
contre son SHA-256 signé (Velopack revérifie SHA-1 et SHA-256 à son tour). `bs-signer` :
outil de la CI (paire de clés, manifeste écrit depuis `releases.<voie>.json` de `vpk`,
signature). Velopack 1.2.161 (crate et `vpk` à la même version ; la 1.3 est une préversion).

Essai de bout en bout, `rust/outils/essai-maj.sh` (appli de démonstration empaquetée par
`vpk` en AppImage, versions successives) :

| Cas | Résultat (Linux) |
|---|---|
| Installer N (0.1.0), lancer | démarre en 0.1.0 |
| N → N+1 | téléchargé, appliqué, redémarré tout seul en 0.2.0 (paquet complet) |
| N+1 → N+2 | **delta** : 6 Ko au lieu de 5 Mo, appliqué en 0,3 s, redémarré en 0.3.0 |
| Déjà à jour | « à jour » |
| Manifeste falsifié (une taille changée) | refusé : signature invalide |
| Manifeste signé par une autre clé | refusé : signature invalide |
| Manifeste expiré | refusé |
| Vieux manifeste rejoué (série 2 après 3) | refusé : manifeste ancien |
| Paquets corrompus, manifeste valide | téléchargement refusé (empreinte), le jeu reste en 0.3.0 |

- La première mise à jour après l'installation prend le **paquet complet** : Velopack n'a
  pas encore de paquet local de la version installée pour lui appliquer un delta (l'AppImage
  seule ne suffit pas). Ensuite, tout passe par les deltas. À garder en tête pour le passage
  depuis Electron : la première mise à jour sera complète.
- Mêmes vérifications en tests unitaires (`bs-maj/tests/manifeste.rs`, sans Velopack) et
  dans la CI : jobs `mise-a-jour-linux` (le script ci-dessus) et `mise-a-jour-windows`
  (`outils/essai-maj.ps1` : installeur Velopack silencieux, N → N+1 → N+2, manifeste
  falsifié). Les deux sont **verts** (CI du 31/10, commit 4d7c48d).
- **Deltas sur le vrai jeu : décevants.** Deux builds `ship` du prototype Bevy qui ne
  diffèrent que d'une constante (position de la caméra) : 60 Mo d'octets changent sur 71
  (LTO : toutes les adresses bougent). Mesures (Linux) :

  | Paquet | Taille |
  |---|---|
  | complet Velopack (exécutable de 71 Mo avec symboles) | 24,3 Mo |
  | delta Velopack, mode par défaut (`BestSpeed`) | 19,7 Mo |
  | delta Velopack, `--delta BestSize` | 18,9 Mo |
  | `zstd -19 --long=27 --patch-from` (même paire) | **1,2 Mo** |
  | exécutable sans symboles (`strip`), compressé seul | 11,3 Mo |
  | `zstd --patch-from` entre exécutables sans symboles | **1,1 Mo** |

  Le delta de Velopack ne gagne presque rien sur un gros exécutable unique ; un vrai
  différentiel binaire (zstd) fait 16 fois mieux. Ajusté tout de suite : `strip = true`
  dans le profil `ship` (49 Mo au lieu de 71). À décider à l'étape des mises à jour :
  ajouter notre propre delta zstd (téléchargé, vérifié par le manifeste signé, appliqué pour
  reconstruire le paquet complet que Velopack installe), ou s'en tenir aux paquets complets
  (≈ 11 Mo par version, acceptable). Les données du jeu (sons, modèles) dans des fichiers à
  part de l'exécutable ne seront, elles, retéléchargées que si elles changent.
- Pas encore fait (étapes suivantes du plan, pas des paris) : dépôt HTTPS (aujourd'hui un
  dossier), retour arrière après deux plantages, révocation de la clé de CI par la clé de
  secours, voie bêta, signature Authenticode.

### Pari 5 : importer une vraie sauvegarde (.bssave)

`bs-sauvegarde` lit et écrit le format du jeu TS (AES-256-GCM, clé = SHA-256 du secret,
en-tête `BSSAVE1:`, IV de 12 octets + chiffré en base64). Test sur un **vrai** fichier
exporté par le jeu TS 0.50.0 (`scripts/exporter-bssave.mjs` : pseudo « Éloïse », deux
parties) : `rust/crates/bs-sauvegarde/tests/donnees/profil-0.50.0.bssave`, relu champ par
champ (pseudo accentué, records, statistiques), et l'aller-retour Rust → Rust est
identique. Fichier illisible ou vide : erreur propre, pas de plantage. **Pari tenu.**

### Décision : on continue (avec trois ajustements)

| Pari | Verdict |
|---|---|
| 1. Rendu de 300 zombies | **tenu** : 94 images/s sur un vrai PC Windows (pire image 28 ms), simulation 65 à 95 µs/pas ; fonctionne aussi en WebGPU et WebGL2 |
| 2. kira dans le navigateur | **tenu avec ajustement** : tampon de 4096 en navigateur (85 ms de latence) |
| 3. Relais axum/tokio | **tenu** : 0,5 ms d'aller-retour en local, 35 Ko/s par invité ; mesure depuis la France à faire au déploiement |
| 4. Velopack N → N+1, manifeste signé | **tenu avec ajustement** sous Linux et **Windows** (CI : installeur silencieux, N → N+1, delta N+1 → N+2, manifeste falsifié refusé) ; deltas de Velopack inefficaces sur l'exécutable (19 Mo sur 24) |
| 5. Import d'un vrai .bssave | **tenu** |

**Continuer** la migration (étape 1 du plan). Ajustements retenus :
1. **Son en navigateur** : `Audio::ouvrir(Some(4096))` sur le web, défaut de cpal sur PC ;
   corriger (ou faire corriger en amont) le curseur de cpal qui ne se recale pas après un
   retard ; couper le son pendant les chargements longs.
2. **Réseau** : `TCP_NODELAY` partout (relais et client natif), à garder dans les tests.
3. **Mises à jour** : Velopack et `vpk` épinglés à la même version (1.2.161) ; la première
   mise à jour après l'installation est un paquet complet (pas de delta) ; les deltas de
   Velopack ne valent rien sur l'exécutable : exécutable sans symboles (fait), puis delta
   zstd maison ou paquets complets (≈ 11 Mo), à trancher à l'étape des mises à jour.

Point ouvert levé le 31/10/2026 : 94 images/s sur le PC Windows du joueur (pari 1). Une
mesure sur un portable reste bienvenue.

---

## Étape 1 : données et règles, sans rendu

### 1a. Les données (`bs-donnees`, `rust/donnees/*.ron`) : fait

Tout `src/donnees` (environ 5 000 lignes) est dans 19 fichiers RON (10 800 lignes), un par
table : blocs, modèles, ennemis, projectiles, armes, alliés, personnages, arènes,
structures, passifs, objets, coffres, équipement, armures, talents, sons, particules,
règles (joueur, apparitions, boss, progression, plafonds, états, multijoueur) et textes.

**Comment.** Rien n'est recopié à la main : `rust/outils/importer-donnees.sh` charge les
modules TypeScript par Vite (`outils/exporter-donnees.mjs`, export JSON), puis
l'importeur Rust (`importer-donnees`) les fait passer par les types Rust et écrit le RON.
Tant que la version TypeScript reste la référence, on modifie `src/donnees` puis on relance
l'import ; la CI échoue si `rust/donnees` n'est plus la copie exacte
(`importer-donnees.sh --verifier`).

**Garde-fous.**
- Types stricts (`deny_unknown_fields`) : un champ en trop ou mal écrit est refusé avec le
  fichier, la ligne et la colonne. Toutes les données de la version 0.50.0 sont passées du
  premier coup.
- Validation des références au chargement : arme → projectile, allié, gerbe, son ; arène →
  blocs, structures, ennemis, boss ; talent → armes, prérequis ; modèle → parties animées ;
  structure → légende ; etc. Une erreur dit « fichier.ron, entrée : … inconnu(e) « x » ».
- L'ordre des tables est gardé (`IndexMap`) : c'est celui des listes et des tirages du jeu.
- Données lues au lancement (`Donnees::charger`, pour le rechargement à chaud) ou
  embarquées dans l'exécutable (`Donnees::embarquees`, navigateur et tests) ; un test
  vérifie que les deux sont identiques.

**Choix de forme.**
- Noms des champs identiques à la version TypeScript (camelCase) : les identifiants du
  contenu (parties de modèles comme `brasGauche`, gerbes comme `traineeFeu`) ont la même
  forme, et on retrouve chaque champ des deux côtés.
- Variantes écrites à la RON : `mecanique: tir(projectile: "fleche", dispersion: 0.18)`
  au lieu de `{ type: 'tir', … }` (RON ne lit pas les variantes « étiquetées par un champ »).
- L'apparence des entités (modèle, skin, ombre…) et les gains des talents sont rangés dans
  un sous-bloc (`apparence: (…)`, `gains: (…)`) au lieu d'être à plat.
- Descriptions de l'équipement : en TypeScript, des fonctions du niveau ; ici, le texte de
  chaque niveau (`description: ["… 8 %", "… 12 %", "… 16 %"]`), prêt pour la traduction.
- Arènes importées (`cartes/*.schem`) : pas dans les données ; elles seront lues au
  lancement avec le monde (`bs-monde`, étape 2), comme aujourd'hui.
- `Infinity` (dernier palier d'XP) devient « aucun » (`jusquAuNiveau: None`).

Repéré en passant (à corriger dans la version TypeScript, pas ici) : « fait 2 dégât de
moins » (Collier du forgeron, niveau 3) devrait être au pluriel.

### 1b. La simulation : faite

Combat et armes (environ 5 600 lignes), ennemis et directeurs, progression (XP, cartes,
coffres, équipement, talents, maîtrise, Collection), puis le robot, dont les statistiques
doivent tomber dans la tolérance de celles de la version TypeScript (étape −1).

| Morceau | État |
|---|---|
| 1b-1 Monde (`bs-monde` : arènes, collisions, champ de flux) | fait |
| 1b-2 Partie, joueur, niveaux, passifs | fait |
| 1b-3 Ennemis, boss et directeurs | fait |
| 1b-4 Orbes d'XP, objets, coffres, butin | fait |
| 1b-5 Arsenal et les 25 armes, alliés invoqués compris | fait |
| 1b-6 Talents de classe et d'objets | fait (sauvegarde vierge : talents de classe au rang 0) |
| 1b-7 Cartes d'amélioration et équipement | fait (`cartes.rs`, `equipement.rs`) |
| 1b-8 Robot (`bs-robot`) et comparaison aux références | fait (vérifié en CI) |

Choix de portage (`bs-regles/src/combat/`) :

- Le jeu TS garde des références d'objets ; le port garde des index. Une arme est
  désignée par sa place dans l'arsenal (ses stats restent à jour quand elle monte de
  rang). Un ennemi ou un allié visé est désigné par (index, identifiant) et retrouvé
  par son identifiant s'il a changé de place dans le pool.
- Les pools font comme `Pool.liberer` en TS (on échange avec le dernier) et sont
  parcourus à l'envers, pour garder le même ordre et donc le même aléa.
- Les effets purement visuels qui tirent dans l'aléa (éclairs, blocs soulevés, traces
  des tornades, flèches de la pluie…) sont comptés, sans être affichés, pour que la
  suite de nombres tirés reste celle du jeu TS.
- Test `chaque_arme_frappe` : chaque arme seule, au rang maximum, pendant 60 s,
  blesse des ennemis (l'esprit protecteur, qui ne fait que protéger, doit invoquer).
- Test `cartes_et_coffres` : cinq minutes en prenant toujours la première carte de
  niveau et de coffre (avec remplacement d'objet si les emplacements sont pleins).

### Le robot Rust (`rust/crates/bs-robot`)

Port de `robot.ts` (mêmes règles de déplacement et de choix des cartes, même bilan).
`cargo run --release -p bs-robot` joue 15 graines × 4 personnages en 2 s (contre une
vingtaine de minutes pour le robot TS dans Chromium), puis compare aux références TS.

- **Même partie au départ, puis divergence.** L'exemple `trace` sort la partie pas à pas
  (tirages d'aléa, positions, PV), comme une trace équivalente du jeu TS : les deux sont
  identiques au millième de bloc pendant les premières secondes, jusqu'à ce qu'une
  minuterie en f32 tombe à 0 un pas plus tôt que la même en f64. La suite est
  chaotique : on ne compare que des moyennes.
- **Tolérance.** Une mesure passe si l'écart des moyennes est d'au plus 25 %, ou d'au
  plus deux erreurs types de la différence (les parties sont très dispersées : survie de
  50 à 650 s pour l'archer). Mesures vérifiées : survie, niveau, tués, dégâts subis,
  puissance minute par minute (tant que la moitié des parties jouent encore). Les dégâts
  par seconde de chaque arme sont indicatifs : l'arme arrive à des moments très
  différents d'une partie à l'autre, et dans peu de parties.
- **Résultat (15 graines, joueur mortel).** Toutes les mesures vérifiées passent. Survie
  moyenne TS / Rust : archer 208 / 245 s, guerrier 175 / 201 s, invocateur 174 / 149 s,
  mage 117 / 151 s. Puissance des trois premières minutes à ±15 % près (mage : 7,5 /
  12,1 / 25 en TS ; 7,3 / 12,1 / 23,7 en Rust). Les armes obtenues se répartissent de
  la même façon.
- **Joueur invincible (3 et 5 graines).** Survie, niveau, tués et puissance par minute
  dans la tolérance (par exemple guerrier : niveau 29 / 28,4, tués 3567 / 3502).
- Butin d'armure des boss : la Collection n'agit que sur les parties suivantes, mais ses
  tirages dans l'aléa sont reproduits (`butin.rs`).

---

## Étape 2 : monde et rendu

### 2a : textures vanilla et arène affichée

- **Textures.** Jamais dans le dépôt : téléchargées au lancement depuis les mêmes
  adresses que le jeu TS (GitHub, puis mcasset.cloud en secours), par une source
  d'assets Bevy `mc://` (`bs-rendu/src/textures.rs`). Sur PC elles sont gardées en
  cache (`~/.cache/block-survivors/textures/26.3` sous Linux, `%LOCALAPPDATA%` sous
  Windows) ; le navigateur garde son cache HTTP. Une texture introuvable donne la
  couleur de secours du bloc, en damier, comme en TS.
- **Arène.** Un seul maillage par (bloc, groupe de faces : côtés, dessus, dessous) :
  les faces collées à un bloc plein opaque, et celles entre deux cases du même liquide,
  ne sont pas créées. Formes : cube, croix (fleurs, herbes), liquide (14/16 de haut).
  Rotation aléatoire par quarts de tour, comme en TS. Terrain extérieur autour de
  l'arène, texture répétée et assombrie.
- **Matériau des blocs** (`shaders/blocs.wgsl`) : le même éclairage que le Lambert de
  three.js (ambiante + soleil, sans reflets), brouillard linéaire lissé comme
  `THREE.Fog`, découpe nette (feuilles, fleurs), blocs lumineux, eau et lave animées
  (bande d'images, aller-retour). Pas de tonemapping : couleurs identiques au TS.
- **Caméra** vue de dessus inclinée à 55°, suivi lissé, zoom à la molette (réglages de
  `src/config.ts`).
- **Partie.** La vraie simulation (`bs-regles`) tourne à 60 pas/s ; ZQSD/WASD ou les
  flèches conduisent le joueur, sinon le robot joue (et choisit les cartes, en
  attendant l'interface de l'étape 3). Relance automatique 3 s après la fin.
- **Capture** (PC sans carte graphique, lavapipe) : `docs/captures/arene-prairie-pc.png`.
  Simulation : environ 30 µs par pas.
- **Tests** : faces cachées entre blocs et dans un liquide, hauteur de l'eau, damier
  de secours (sans carte graphique).
- **Développement local** : `CARGO_PROFILE_DEV_DEBUG=0` évite de remplir le disque
  (les infos de débogage de Bevy dépassaient 20 Go).


### 2b : modèles Minecraft des entités

- **Modèles** (`bs-rendu/src/modeles/`) : compilés depuis `donnees/modeles.ron`
  (`compilation.rs`), une boîte par partie avec le patron de skin Minecraft
  (`geometrie.rs`, faces et UV comme `THREE.BoxGeometry`, membres en miroir,
  gonflement), skins vanilla téléchargés comme les blocs ; sans skin, couleurs de
  secours par partie.
- **Objet tenu** : sprite de l'objet extrudé d'un pixel d'épaisseur (rebords colorés
  pixel par pixel), accroché à la main du modèle.
- **Animation** (`animation.rs`, comme `src/modeles/animation.ts`) : balancement des
  membres selon la vitesse au sol, bras au repos, tournoiement, flottement, chute sur
  le côté à la mort. Le geste d'attaque est déclenché par un nouvel événement de la
  simulation, `Evenement::Geste` (joueur ou allié de mêlée), lu après chaque pas.
- **Instanciation** : une entité Bevy par partie et par instance, enfants d'une
  racine ; toutes les instances d'un modèle partagent maillages et matériau, donc Bevy
  les dessine en un appel par partie. Les instances ne sont créées qu'au premier
  dépassement du record, puis cachées ou réutilisées : rien n'est détruit en partie.
- **Matériau des entités** (`shaders/entites.wgsl`) : même éclairage et brouillard
  que les blocs ; la surcouche (rouge des coups, clignotement du creeper, teintes de
  brûlure, poison, saignement, ralenti) est passée par instance dans le `MeshTag`
  (r, g, b, force sur 4 octets), sans matériau par entité.
- **Poses** (`entites.rs`) : positions interpolées entre deux pas, gonflement et
  tremblement du creeper au bout de sa mèche, alliés volants qui ondulent, rouge du
  joueur touché ou mort.
- **Reste à faire** : armures et capes (attendent les profils, étape 4) ; ombres,
  particules et chiffres de dégâts (2d).
- **Capture** : `docs/captures/modeles-prairie-pc.png` (Steve en marche, épée en main).
- **Tests** : boîtes de tous les modèles (24 sommets, UV dans la texture, triangles
  orientés vers leur normale), extrusion d'un sprite, balancement à la marche, chute.

### 2c : maps Minecraft importées (`cartes/*.schem`)

- **Lecture** (`bs-monde/src/minecraft/`, sans carte graphique) : NBT compressé
  (gzip ou zlib), schématiques Sponge 1 à 3 et structures `.nbt`, panneaux ; noms de
  blocs normalisés comme en TS. Les maps de `cartes/` (et leur `.json` de réglages)
  sont embarquées à la compilation (`build.rs`).
- **Modèles de blocs** : états et modèles vanilla (variantes, multipart, parents,
  rotations d'éléments et d'état, uvlock) « cuits » en quadrilatères. Les fichiers
  JSON sont téléchargés au fil des besoins par une seconde source d'assets, `mcj://`
  (mêmes adresses et même cache disque que les textures, dossier `fichiers/`).
  Coffres, lits et boîtes de shulker ont une boîte de secours.
- **Zone jouable** : depuis le panneau « départ », remplissage des cases praticables ;
  carte de collision (plein, liquide, toiles, blocs qui ralentissent ou blessent).
  La simulation prend l'arène telle quelle : `Partie::avec_arene` (le générateur reste
  pour les arènes classiques).
- **Rendu** (`bs-rendu/src/carte_mc.rs`) : un maillage par texture, faces cachées par
  un voisin plein et opaque ; chaque texture est classée d'après ses pixels (opaque,
  découpée, translucide). Le matériau des blocs gagne le mélange alpha (verre teinté,
  eau). Eau et lave animées comme les blocs du jeu ; les autres bandes montrent leur
  première image, comme en TS.
- **Choix** (provisoire, jusqu'au menu) : `BS_ARENE=map_mushroom` (ou l'id d'une arène
  générée). La relance après la fin reprend la même map.
- **Reste à faire** : balancement des plantes au vent (2d).
- **Capture** : `docs/captures/map-champignons-pc.png`.
- **Tests** : lecture de la map embarquée, cuisson en plusieurs passes avec des
  fichiers factices, zone et collisions d'un bassin construit à la main.

### 2d : effets (en quatre parties)

2d-1 projectiles, orbes, objets, coffres, ombres ; 2d-2 particules, chiffres de
dégâts, barres de vie, secousse ; 2d-3 visuels des armes (météores, tornades, pluies,
failles, foudre, orbes arcaniques, pièges, novas, invocations) ; 2d-4 lumières, vent,
herbe. La caméra F5 est abandonnée (décision du propriétaire).

#### 2d-1 : projectiles, orbes, objets, coffres, ombres (fait)

- **Lots** (`bs-rendu/src/effets/`) : un maillage et un matériau par type d'objet,
  une entité par instance, réutilisée et cachée quand elle ne sert pas (l'équivalent
  d'un `InstancedMesh`). Aucune allocation par image une fois le record atteint.
- **Projectiles** : les 8 types, avec leurs formes (flèche en croix, icône en relief,
  modèle couché dans le sens du vol, boule de feu face à la caméra). La fusée reçoit
  sa skin peinte d'après les pixels de l'item, comme en TS. Projectiles du joueur et
  des ennemis, flèches ricocheuses, armes en orbite ; position prolongée de la
  fraction de pas écoulée (mouvement fluide).
- **Orbes d'XP** : atlas vanilla, une taille par palier, couleur qui pulse (teinte
  multipliée par instance), toujours face à la caméra, non éclairées.
- **Objets au sol** : icône en relief qui tourne et flotte ; un tas d'émeraudes
  grossit avec sa valeur.
- **Coffres** : modèle retourné, couvercle et loquet qui se lèvent à l'ouverture.
  Les prix au-dessus viendront avec l'interface (étape 3).
- **Ombres** : disque `misc/shadow` translucide (opacité 0,45) sous le joueur, les
  ennemis (suivent leur gonflement) et les alliés.
- **Matériau des entités** : gagne le mode non éclairé, la teinte multipliée et la
  transparence (mélange alpha).
- **Vitrine** (vérification visuelle) : `BS_VITRINE=1` pose un exemplaire de chaque
  projectile, palier d'orbe, objet et coffre devant le joueur. `BS_PERSONNAGE=n`
  choisit le personnage (provisoire, jusqu'au menu).
- **Capture** : `docs/captures/vitrine-objets-pc.png` (map champignons).
- **Tests** : rectangles d'UV, disque de secours, skin de la fusée, flèche en croix.

#### 2d-2 : particules, chiffres de dégâts, barres de vie, secousse (fait)

- **Retours** (`bs-rendu/src/retours/`) : après chaque pas, les événements de la
  simulation (gerbes, dégâts, secousses) sont recueillis ; ils vivent et se dessinent
  à chaque image. Particules, chiffres et barres sont chacun un seul maillage réécrit
  à chaque image (un appel de dessin, tampons réutilisés, aucune allocation une fois
  le record atteint).
- **Particules** : toutes les sortes et gerbes de `particules.ts`, pool de 1024,
  carrés face à la caméra lus dans un atlas construit à l'entrée en partie. Les éclats
  (« morceau ») prennent l'icône donnée par la simulation : objet ramassé, bloc du sol
  pour les éboulis, texture du météore, de la nova ou du piège.
- **Chiffres de dégâts** : blancs, dorés et plus gros avec un « pop » (critique),
  orange (feu) ou verts (poison) ; police pixel du jeu avec son ombre portée ; montée,
  dérive et fondu ; toujours devant le décor.
- **Barres de vie** : mini-boss et alliés blessés, du rouge au vert, devant le décor.
- **Secousse** de la caméra : bruit fait de sinus, qui décroît.
- **Matériau des entités** : nouveau mode « devant » (toujours visible).
- **Vitrine** : `BS_VITRINE=1` montre aussi chaque gerbe, chaque style de chiffre et
  deux barres.
- **Capture** : `docs/captures/vitrine-retours-pc.png`.
- **Tests** : atlas de la police, nombre de chiffres.

#### 2d-3 : visuels des armes (en trois parties)

Décision du propriétaire : les compétences restent celles du jeu, mais leurs visuels
sont refaits, plus propres et plus poussés, sans recopier la version TS (images
vanilla collées). 2d-3a lueurs, mêlée, foudre ; 2d-3b feu (tornade, météore, feu au
sol) ; 2d-3c sol (fissures, chute, nova), pluie, pièges, orbes arcaniques,
invocations.

##### 2d-3a : lueurs, mêlée, foudre (fait)

- **Simulation** : nouvel événement `Evenement::Effet { arme, effet, x, z }`
  (`EffetArme` : balayage, heurt, tourbillon, éclair, soulèvement, brûlure). La
  simulation dit quoi et où ; l'affichage choisit l'allure. Rien ne change l'état ni
  l'aléa.
- **Matériau « lueur »** (`bs-rendu/src/armes/lueur.rs`, `shaders/lueur.wgsl`) :
  formes lumineuses calculées par le shader (disque doux, faisceau à cœur blanc,
  anneau, flamme en bruit animé, traînée d'arc, étincelle), mélange additif,
  extinction dans le brouillard. Toutes les lueurs des armes forment un seul maillage
  réécrit à chaque image : un appel de dessin, pools de taille fixe.
- **Balayage** : croissant bleu acier qui balaie l'arc d'un bord à l'autre (aller
  rapide puis fondu), fil blanc sur le tranchant ; sens alterné d'un coup à l'autre.
- **Tourbillon** : lame qui fait le tour du joueur à chaque coup.
- **Coup de bouclier** : onde dorée en arc qui s'étend jusqu'à la portée, éventail
  lumineux derrière elle.
- **Foudre** : tracé brisé tombé du ciel, deux ramifications, scintillement, éclair
  de lumière au sol, onde jusqu'au bord de la zone touchée, étincelle ; couleur de
  l'arme.
- **Vitrine** : `BS_VITRINE=1` montre aussi un exemplaire figé de chaque effet.
- **Capture** : `docs/captures/vitrine-armes-melee-foudre-pc.png`.
