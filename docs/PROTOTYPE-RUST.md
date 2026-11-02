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
| 2. Monde et rendu | fait : 2a (textures vanilla, arène affichée, vraie partie), 2b (modèles Minecraft des entités), 2c (maps .schem importées), 2d-1 (projectiles, orbes, objets, coffres, ombres), 2d-2 (particules, chiffres de dégâts, barres de vie, secousse), 2d-3 (visuels des armes), 2d-4 (vent, ombres du soleil, lumière des blocs) |
| 3. Jouable en solo | fait : 3a (sons), 3b (HUD), 3c (écrans de cartes), 3d (pause, fin, bilan, mode sans fin) |
| 4. Sauvegardes et profils | fait : 4a à 4i (stockage, profils, maîtrise, records, `.bssave`, talents appliqués, mécaniques des quatre classes, Collection, profil du joueur, amis) |
| 5. Menus | en cours : 5a (menu principal, lancement, retour au menu), 5b (Talents), 5c (Collection), 5d (Profil), 5e (Profils), 5f (Options), 5g (Nouveautés), 5h (chargement), 5i (héros au menu), 5j (compteur de dégâts) |

**01/11/2026 : version navigateur abandonnée** (décision du joueur). `bs-web`, `rust/web`,
les scripts web et la tâche web de la CI sont retirés ; les mesures web plus bas restent
pour l'historique.

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

##### 2d-3b : feu (fait)

- **Simulation** : les tornades, météores et feux au sol sont lisibles par
  l'affichage (champs publics) ; chaque trace de brûlure émet `EffetArme::Brulure`.
- **Tornade** : colonne de chaleur, sept brins de feu en spirale qui tournent,
  anneaux qui montent en s'élargissant, flammes à la base, lueur au sol ; elle laisse
  des braises (lueur au sol et escarbille qui s'élève).
- **Météore** : vrai bloc texturé (texture de l'arme, première image si elle est
  animée) qui tombe en accélérant et en tournoyant, halo et traînée ; cible marquée
  au sol, de plus en plus vive jusqu'à l'impact.
- **Feu au sol** : flammes en goutte, ondulantes, rongées par un bruit qui monte
  (shader), posées à des places fixes tirées de la position ; lueur qui vacille ;
  à l'apparition, éclair de lumière et onde.
- **Vitrine** : l'étalage suit le joueur ; capture
  `docs/captures/vitrine-armes-feu-pc.png`.

##### 2d-3c : sol, armes posées, orbes, invocations (fait)

- **Simulation** : lames, novas, zones de pluie, flèches, pièges et orbes lisibles par
  l'affichage (champs publics, `Orbe::figee` pour la vitrine). Rien ne change l'état.
- **Pièces solides** (`armes/solides.rs`) : un lot par texture et par genre, blocs
  (météores, sol de l'arène, éclats de glace, améthyste) ou images dressées (lame,
  flèches, buissons) ; textures lues dans les définitions des armes.
- **Onde de choc** : vrais blocs du sol qui jaillissent en basculant, rebondissent
  puis rentrent ; onde de poussière.
- **Lame colossale** : cible qui se resserre, l'épée (image de l'arme) tombe pointe
  en bas avec un sillage, se plante ; éclair, onde, fissures lumineuses en étoile qui
  refroidissent, puis elle s'enfonce.
- **Nova de glace** : anneau qui s'étend, éclats de glace (blocs de la texture de
  l'arme) qui jaillissent au passage de l'anneau, reflets ; sol gelé pâle et
  scintillant au rang voulu.
- **Pluie de flèches** : cercle de visée cerné de tirets qui tournent ; flèches qui
  tombent en traits de lumière, éclat à l'impact, puis plantées et qui s'enfoncent.
- **Piège de ronces** : buissons en croix qui poussent, cercle d'épines qui pulse et
  jaillit à chaque morsure, baies qui luisent.
- **Orbes arcaniques** : bloc d'améthyste qui tournoie dans un halo, cœur
  scintillant, sillage qui suit le trajet récent (orbite puis course).
- **Invocations** : un allié arrive dans une colonne de lumière sur un cercle runique
  (étoile à six branches qui tourne, étincelles en spirale), part dans une bouffée.
- **Vitrine** : `BS_ETALAGE=melee|foudre|feu|sol|pose|arcanes` n'étale qu'un groupe ;
  captures `docs/captures/vitrine-armes-sol-pc.png`,
  `docs/captures/vitrine-armes-pose-pc.png`, `docs/captures/vitrine-armes-arcanes-pc.png`.

#### 2d-4 : lumières, vent, herbe (fait)

##### 2d-4a : vent (fait)

- **Shader des blocs** : il a maintenant son propre shader de sommets. Chaque sommet
  porte son vent (`ATTRIBUTE_VENT` : centre du bloc, pied, mode). Le feuillage frémit,
  le sommet des herbes et des fleurs se balance depuis leur pied, comme
  src/monde/vegetation.ts (mêmes direction, vitesse et amplitudes). En plus : des
  rafales lentes balaient l'arène en larges bandes, et le sommet d'une plante qui
  penche s'abaisse un peu.
- **Maps importées** : même vent, bloc par bloc (`bs_monde::minecraft::vent` :
  feuilles, herbes, fleurs, pousses, cultures, lianes ; moitié haute d'une grande
  plante balancée depuis le pied du bloc du dessous). C'était une demande de
  A_FAIRE.md, absente de la version TS.
- **Test** : l'herbe d'une petite schématique ondule depuis son pied, la pierre non.

##### 2d-4b : ombres du soleil (fait)

- **Soleil** (`bs-rendu/src/soleil.rs`) : une lumière directionnelle Bevy, utilisée
  seulement pour sa carte d'ombres (4096², deux cascades qui suivent la caméra : nette
  près du joueur, plus large au loin). Tout le décor visible a ses ombres, y compris
  hors de l'arène (la version TS s'arrêtait à 6 blocs autour, voir A_FAIRE.md).
- **Shaders** : l'éclairage façon Minecraft ne change pas ; le terme du soleil est
  retiré là où il est caché (`shaders/soleil.wgsl`), bords adoucis (filtre gaussien).
- **Passe d'ombre maison** (`shaders/ombres.wgsl`) : les feuilles, fleurs et skins
  découpées projettent leur vraie silhouette, les plantes gardent leur balancement
  dans l'ombre ; l'eau, le verre, les lueurs des armes et les effets non éclairés
  (chiffres, barres de vie, orbes) n'en projettent pas.
- **Ombres rondes** : cachées quand le soleil est actif, comme en TS.
  `BS_OMBRES=0` coupe le soleil (en attendant les options, étape 4).
- **Captures** : `docs/captures/ombres-soleil-pc.png`,
  `docs/captures/ombres-soleil-map-pc.png`.

##### 2d-4c : lumière des blocs (fait)

- **Propagation** (`bs-monde/src/lumiere.rs`) : comme dans Minecraft, chaque source
  (torche 14, lanterne, lave, glowstone 15, lanterne des âmes 10, magma 3, bougies et
  lampes allumées…) éclaire bloc par bloc, un niveau de moins par pas ; les cubes
  pleins opaques l'arrêtent (pas de lumière à travers les murs : les caves restent
  sombres). Calculée une fois au chargement, pour l'arène générée comme pour une map
  importée (`maillage::opaques` dit ce qui arrête la lumière).
- **Rendu** (`bs-rendu/src/lumiere.rs`, `shaders/lumiere.wgsl`) : une texture 3D d'un
  texel par bloc, lissée entre blocs, donnée à chaque matériau de blocs et d'entités à
  sa création. Les shaders lisent le niveau de l'air devant la surface, suivent la
  courbe de luminosité de Minecraft et ajoutent une lumière chaude : le sol rougeoie
  autour de la lave, les monstres s'éclairent en passant près d'une torche.
- Sans source dans l'arène (plaine…), rien n'est envoyé ni calculé dans les shaders.
- **Capture** : `docs/captures/lumiere-blocs-nether-pc.png`.


## Étape 3 : jouable en solo

En quatre parties : 3a sons, 3b HUD, 3c écrans de cartes, 3d pause, fin, bilan et
mode sans fin.

### 3a : sons (fait)

- `bs-audio` (réécrit) : `Mixeur` sur kira (sortie cpal, décodage ogg/vorbis). Il reprend
  src/audio/sons.ts : volume = général × catégorie × son ; hauteur tirée au hasard dans
  l'intervalle du son, multipliée par celle de l'événement et bornée à 0,25 à 4 ;
  `intervalle_min` entre deux lectures ; `simultanes` voix au plus (la plus ancienne est
  coupée). Volumes par défaut : 0,7 général et 1 par catégorie.
- Source d'assets « mcs » (`textures.rs`) : serveur de ressources de Mojang
  (`<2 caractères>/<empreinte>`), avec un cache disque dans `~/.cache/block-survivors/sons`.
- `bs-rendu/src/sons.rs` : chargeur `.ogg` qui décode une seule fois, puis demande de tous
  les sons au démarrage. Après chaque pas, `Evenement::Son` est joué. Un son pas encore
  arrivé reste muet.
- Sans sortie son (CI, xvfb), rien n'est téléchargé et la partie reste muette.
  `BS_SON=0` coupe le son. Vérifié : les 71 sons se téléchargent et se décodent sans
  erreur.

### 3b : HUD (fait)

- `bs-rendu/src/hud/` reprend src/ui/ avec bevy_ui. `UiScale` part d'une fenêtre de
  référence de 1280 × 820, par quarts au-dessus de 1.
- `police.rs` : police pixel du jeu (planches `font/*.png` et table
  `font/include/default.json`). Chaque `TextePixel` est rasterisé dans sa propre image, et
  seulement quand il change. Bord au choix : ombre (couleur / 4, décalée de 1, 1) ou
  contour noir.
- `affichage.rs` :
  - en haut : chrono (doré dans la dernière minute) et « Prochain boss dans » (rouge à
    10 s) ;
  - à droite : émeraudes et tués ;
  - totems de vie, avec une animation quand un totem est consommé ;
  - alerte clignotante et voile rouge.
- `coeurs.rs` : cœurs vanilla × 3 rognés colonne par colonne, cœurs dorés du bouclier,
  tremblement à 4 PV ou moins, et PV au dixième.
- `barres.rs` : barre d'XP avec le niveau (contour noir) et barre du boss sous le chrono,
  à la couleur de sa définition.
- `capacites.rs` : cases d'armes, de passifs et d'équipement, avec le niveau, la rareté
  et l'éveil. Les icônes d'équipement sont dessinées au lancement (voir 3c).
- `bs-regles` : nouvel événement `AnnonceBoss { nom, duree }`, émis à l'annonce d'un vrai
  boss, qui affiche l'alerte « Un boss approche : <nom> ! ».
- `BS_DEBUT=<s>` (captures) : le robot joue ce nombre de secondes avant l'affichage, avec
  la vie gardée pleine.

### 3c : écrans de cartes (fait)

- `bs-regles/src/textes/` : description des cartes, calculée seulement à l'ouverture d'un
  écran. On y trouve le nom, la famille (couleur et bandeau ; la rareté pour les objets),
  l'icône, l'étiquette, la description et les lignes « avant → après ». Ce module porte
  `decrire` (ameliorations.ts) et les cartes d'equipement.ts. Les nombres suivent le format
  français, avec une virgule et au plus 2 décimales. Le test `cartes_et_coffres` décrit
  chaque carte tirée.
- `bs-rendu/src/dessins.rs` : images en pixel art dessinées au lancement :
  - les icônes 16 × 16 des bagues et amulettes (iconesObjets.ts), aussi utilisées dans la
    barre des capacités ;
  - le losange d'or serti d'une émeraude (ornements.ts).
- `bs-rendu/src/ecran_cartes/` :
  - `mod.rs` : un niveau en attente à la fois, puis les coffres (titre tiré des données du
    coffre ou du coffre de l'Ender).
    - Choix avec les touches 1 à 4 (rangée du haut ou pavé numérique) ou à la souris,
      après un délai de 350 ms.
    - R relance le tirage ; les relances sont communes aux niveaux et aux coffres.
    - Si les emplacements sont pleins, une 2e étape « Remplacer quel objet ? » s'ouvre,
      sans relance ; « Retour » rouvre les mêmes propositions.
    - Un test vérifie le niveau, puis le remplacement, le retour et le remplacement.
  - `vue.rs` : mise en page de styles.css :
    - voile sombre, titre d'or entre deux traits ;
    - cartes au liseré d'or bordé de sombre, avec un liseré intérieur et un bandeau de la
      couleur de la famille, un fond en dégradé et le losange en haut ;
    - la carte survolée se soulève de 8 px ;
    - la nouvelle valeur des statistiques est en vert.
  - Ne sont pas portés : la pulsation des cartes rares et l'éclaircissement au survol.
- `police.rs` : texte coupé entre les mots (`TextePixel::bloc`), avec les lignes centrées,
  et texte sans bord (bandeau).
- Pause et démonstration (`simulation.rs`) :
  - tant que des cartes attendent, la partie ne fait plus de pas ;
  - le joueur choisit dès qu'il a touché une touche de déplacement, ou d'emblée avec
    `BS_CARTES=1` ; avant cela, le robot choisit et joue seul (démonstration, captures).
  - `BS_COFFRE=1` (ou `ender`) ajoute un coffre à ouvrir après `BS_DEBUT`, pour tester
    l'écran des objets.

### 3d : pause, fin, bilan, mode sans fin (fait)

- `bs-rendu/src/ecrans/` :
  - `style.rs` : style commun (couleurs d'or, cadre orné avec ses losanges, traits d'or
    estompés, boutons qui s'éclairent au survol, teinte violette de « Continuer sans
    fin »). L'écran de cartes l'utilise aussi.
  - `mod.rs` : quel écran est ouvert.
    - Échap ouvre ou ferme la pause. La perte du focus de la fenêtre l'ouvre aussi quand
      le joueur joue. Pas de pause pendant les cartes ni après la fin.
    - La pause propose Reprendre, Recommencer et Statistiques (Échap revient à la pause).
    - L'écran de fin s'ouvre 1,6 s après la mort ou la victoire. Rejouer (ou Entrée)
      relance la partie. Après une victoire, « Continuer sans fin » efface l'issue,
      relance les directeurs de boss et affiche l'alerte du mode sans fin.
    - Le HUD est masqué sous l'écran de fin ; sous un écran, la molette fait défiler le
      bilan au lieu de zoomer.
  - `bilan.rs` : résumé (pastilles de la pause, tuiles de la fin) et bilan : chiffres clés
    sur deux colonnes, équipement porté, ennemis vaincus par type (en texte : les
    portraits ne sont pas portés), tableau des dégâts par arme avec sa jauge.
  - `vues.rs` : bandeau de pause, panneau des statistiques, écran de fin sur fond rouge
    (défaite) ou doré (victoire).
- `simulation.rs` : pas de pas pendant la pause. En démonstration, la partie se relance
  seule 3 s après la fin ; le joueur, lui, choisit sur l'écran de fin.
- Essais : `BS_PAUSE=1` (ou `stats`) ouvre la pause (ou ses statistiques) ; `BS_FIN=mort`
  (ou `victoire`) termine la partie après `BS_DEBUT` (avec `BS_CARTES=1` pour l'écran).
- Ne sont pas portés : le menu principal, les options, la récompense de maîtrise et les
  profils.

## Étape 4 : sauvegardes et profils

### 4a : stockage, profils, maîtrise et records (fait)

- `bs-sauvegarde` reprend le format de la version TypeScript, pour qu'un joueur garde tout :
  - `stockage.rs` : comme le stockage local du navigateur, une clé et sa valeur JSON, un
    fichier `<clé>.json` par clé. Le dossier est celui du système (`%APPDATA%`, `~/Library/
    Application Support` ou `~/.local/share`, puis `block-survivors`), ou `BS_DOSSIER`.
    L'écriture passe par un fichier temporaire renommé.
  - `profils.rs` : registre des profils (8 au plus), clés « block-survivors.p.<id>.<nom> »,
    création, changement, suppression. Au premier lancement, une sauvegarde d'avant les
    profils devient le profil 1. Import d'un `.bssave` (même d'avant les profils) en
    nouveau profil actif, export du profil actif.
  - `progression/` : progression version 3 (sauvegarde.ts) au même JSON (entiers écrits
    sans « .0 »). Lecture tolérante, comme en TS : une valeur invalide vaut 0, une version
    plus ancienne perd ses talents. On y trouve la maîtrise par classe, les records (et
    records sans fin), les déblocages et la fin de partie (`terminer_partie`). `talents.rs`
    gère les rangs, paliers, prérequis, points libres, options, talent final et arme de
    départ.
  - Tests : un vrai profil exporté par la version TS 0.50.0 est importé puis relu, une
    fin de partie fait passer un niveau et débloque la Mine, un talent racine s'achète
    jusqu'au maximum, et tout se relit après écriture.
- Jeu (`bs-rendu/src/sauvegarde.rs`) : le profil actif est lu au lancement. Quand l'écran
  de fin s'ouvre sur une partie jouée (pas la démonstration), la partie est enregistrée.
  Après « Continuer sans fin », la suite compte à part, à sa propre fin (XP et boss de la
  suite, record sans fin).
- Écran de fin : fiole, « Classe · Maîtrise N », barre verte du niveau, XP gagnée (en
  jaune avec les points de talent gagnés), déblocages et nouveau record sans fin.
- Essais : `BS_IMPORTER=<fichier.bssave>` importe un profil au lancement ;
  `BS_EXPORTER=<fichier.bssave>` exporte le profil actif. Vérifié avec le profil TS de
  test : import, partie gagnée enregistrée (parties 2 → 3, XP de maîtrise du Guerrier),
  export.
- Reste pour l'étape 4 : les effets des talents en jeu (arbres achetés appliqués au
  départ de la partie), la Collection et les armures, les skins importées, l'apparence et
  les amis. Les écrans des profils et des talents viendront avec les menus (étape 5).

### 4b : arbre de talents actif appliqué en partie (fait)

- `Progression::bonus(personnage)` (`bs-sauvegarde/src/progression/bonus.rs`) fait comme
  `bonus` en TS. Il parcourt les talents actifs de l'arbre actif (talent final choisi,
  arme de départ active) et joue l'option choisie d'un talent à choix. Il cumule, rang
  après rang : modificateurs, bonus d'armes précises, rangs d'armes offerts et relances.
  Il donne aussi le rang de chaque mécanique propre. Le type `BonusTalents` est dans
  `bs-donnees`.
- `Partie::appliquer_talents` (`bs-regles/src/talents_arbre.rs`) se lance avant le premier
  pas. Il ajoute les modificateurs aux bonus du personnage, puis recalcule les PV max et
  les relances. Il donne à chaque arme ses bonus de talents (`BonusArmeCumule`, sur
  `ArmeEquipee` : ajouts avant les modificateurs, parts après, PV et vitesse des alliés),
  comme `calculerStats` en TS. Il offre aussi les rangs d'armes, après les armes de départ
  du personnage. `Talents::rang(nom)` donne le rang d'une mécanique propre.
- Jeu : chaque partie, et chaque relance, reçoit les talents du profil actif. Le journal
  les affiche au lancement.
- Essais :
  - un profil Guerrier avec Enchaînement, Lame affûtée 3 et Maître d'armes 2 commence
    avec l'Épée au rang 3 ;
  - tests : `bonus` d'un arbre acheté, `appliquer_talents` (PV, relances, rang et dégâts
    de l'arme).
- Les mécaniques propres (87 dans `mecaniques_pretes`) ont leur rang, mais leurs effets
  restent neutres. Elles seront portées classe par classe aux étapes 4c à 4f : Guerrier,
  Archère, Mage, Invocateur.

### 4c : mécaniques des talents du Guerrier (faites)

- Socle commun (`bs-regles/src/classes/mod.rs`), comme dans `talents.ts` : temps immobile
  et de marche, distance parcourue, soins plafonnés par la régénération, coups « en
  réaction » (ils ne relancent pas les talents), dégâts de zone, cercles de particules.
- Guerrier (`bs-regles/src/classes/guerrier/`, port de `guerrier.ts`). Les rangs sont lus
  une fois, après `appliquer_talents`. Les 23 mécaniques sont portées :
  - Gladiateur (`mod.rs`) : Enchaînement, Frénésie, Champion de l'arène, Fureur,
    Bourreau, Élan critique, Saignée, Lame lourde, Soif de combat, Fureur du gladiateur,
    Exécution ;
  - Gardien (`gardien.rs`) : Posture défensive, Épines, Ronces d'acier, Rempart,
    Inébranlable, Peau de fer, Tenir la ligne, Contre-attaque du bastion, Dernier
    rempart ;
  - Onde de choc (`combat/sol.rs`) : Répliques (2e fissure 0,5 s après) et Séisme
    (6 directions toutes les 3 ondes) ; Contre-coup (`combat/melee.rs`) : le Heurt de
    bouclier renvoie les projectiles dès le rang 1.
- Crochets ajoutés au combat : avant et après chaque déclenchement d'arme, délai
  modifié par les talents (Enchaînement, Frénésie, Élan).
- Test : `guerrier_defense` (Inébranlable, Tenir la ligne, Peau de fer, Dernier rempart).

### 4d : mécaniques des talents de l'Archère (faites)

- `bs-regles/src/classes/archere.rs` (état, mouvement, déclenchements) et
  `archere/coups.rs` (coups portés, ennemis tués, pièges), port de `archer.ts`. Les
  23 mécaniques sont portées :
  - Élite : Premier sang, Marque du chasseur, Point faible, Concentration, Puissance
    (flèche géante après 12 tirs), Œil de l'aigle (flèche spectrale vers l'ennemi le plus
    éloigné, sans rebond), Exécutrice ;
  - Rôdeuse : Pas du chasseur, Venin, Venin concentré, Venin virulent, Contagion, Nuage
    toxique, Course chargée, Second vent, Vent arrière, Piège à loups ;
  - lues par les systèmes d'armes : Visée stable et Perforation nette (projectiles.rs),
    Tir groupé et Vent arrière (volée de l'arsenal), Tir rebondissant (traque.rs), Ronces
    collantes et Ronces venimeuses (pièges, pose.rs).
- Le drapeau `archere` des talents disparaît : le tirage du Venin se fait dans les
  mécaniques de l'Archère, au même moment qu'avant (même sans le talent).
- Test : `archere_talents` (cumuls de poison, Pas du chasseur, Concentration, une minute
  de jeu avec 14 talents).

### 4e : mécaniques des talents du Mage (faites)

- `bs-regles/src/classes/mage/mod.rs` (état, déclenchements, Embrasement, Tempête
  d'orage, Écho) et `mage/coups.rs` (coups portés, ennemis tués, Phénix), port de
  `mage.ts`. Les 20 mécaniques sont portées :
  - Pyromancien : Cœur de blaze, Cendres protectrices, Propagation, Brasier, Fournaise,
    Embrasement, Phénix ;
  - Arcaniste : Résonance, Surcharge, Accélération, Tempête d'orage, Écho arcanique (le
    sort est relancé par `declencher`, dégâts × 0,6 par `talent_facteur_declenchement`),
    Zéro absolu (`Classes::nova_gel`) ;
  - lues par les systèmes d'armes : Pluie de feu, Lave en fusion et Tempête de feu
    (feu.rs : la tornade touchée par un météore devient une tornade de magma), Haute
    tension et Conduction (arcs de foudre, foudre.rs), Orbes givrées et Flux arcanique
    (traque.rs), Zéro absolu (nova gelante, sol.rs).
- Les arcs de foudre tirent 33 fois l'aléa (3 par point), comme `creerArc` en TS.
- Le temps du dernier sort est gardé en `f32`, comme le `Float32Array` du jeu TS.
- Correction : la couleur des éclairs affichés venait de la mauvaise arme (index de
  l'arsenal au lieu du catalogue).
- Test : `mage_talents` (20 talents, une minute avec tous les sorts, Phénix).

### 4f : mécaniques des talents de l'Invocateur (faites)

- `bs-regles/src/classes/invocateur/mod.rs` (état, Hurlement, Lune de sang, Ruée, Curée,
  Garde rapprochée, Évocation supérieure), `allies.rs` (crochets dans le système des
  invocations) et `ravageur.rs` (ravageur et crocs), port de `invocateur.ts`. Les 21
  mécaniques sont portées :
  - Maître-chien : Chef de meute, Morsure marquante, Réactivité, Meute soudée, Traque,
    Curée, Hurlement, Encerclement, Braises de blaze, Coordination, Alpha, Ruée de la
    meute, Lune de sang, Gardien alpha ;
  - Évocateur : Lien spirituel, Garde rapprochée, Allay chanteur, Essaim, Évocation
    supérieure, Crocs de l'évocateur, Bosquet.
- Système des invocations (`combat/invocations/`) : nouveaux champs des alliés (première
  morsure, alpha, revenu, échelle, temporaire), quota + Lune, cible prioritaire, dégâts,
  cadence et vitesse des alliés, morsure, mort rattrapée (Gardien alpha) ; alliés
  temporaires hors quota, allié proche du joueur et haie du Bosquet dans `talents.rs`.
- Affichage (`bs-rendu/src/creatures.rs`) : le ravageur marche en chargeant, les crocs
  sortent de terre, claquent et s'enfoncent ; l'alpha du Gardien est 1,25 fois plus grand.
  Les créatures des talents ont leur rendu à la suite des coffres.
- Test : `invocateur_talents` (21 talents, un peu plus d'une minute : alpha, crocs,
  ravageur, Lune de sang au-delà du quota, dégâts crédités aux loups).

### 4g : Collection (faite)

- `bs-sauvegarde/src/collection.rs` (port de `progression/collection.ts`) : pièces
  possédées (« monde:pièce »), pièce portée à chaque emplacement (une pièce portée doit être
  possédée), objets d'équipement découverts. Même JSON que le jeu TS (`collection`, version
  1). Une pièce gagnée est portée si son emplacement est libre. Bonus : chaque pièce portée,
  plus l'ensemble si les 7 pièces portées viennent du même monde. Avancée d'un monde
  (pièces et objets découverts) pour l'écran de la Collection de l'étape 5.
- Départ d'une partie (`bs-rendu/src/sauvegarde.rs`, `Depart`) : bonus de l'armure portée
  cumulés aux talents, pièces déjà possédées de ce monde jamais tirées sur un boss
  (`Partie::posseder`).
- Butin des boss (`bs-regles/src/butin.rs`) : événement `ButinBoss` (une ou deux pièces),
  son et gerbe « totem » sur le joueur, message « Butin du boss : X et Y ! » pendant 4 s.
  La pièce entre aussitôt dans la Collection (enregistrée) et compte dès la partie
  suivante. Objet d'équipement pris (nouvel objet ou remplacement) : événement
  `Decouverte`, inscrit dans la Collection.
- Armure dessinée sur le héros (`bs-rendu/src/modeles/armure.rs`, port de
  `modeles/armure.ts`) : un modèle par pièce visible, posé comme le joueur (mêmes
  rotations, partie par partie, par le nom). Textures : planches du jeu
  (`entity/equipment/humanoid(_leggings)/<matériau>`), cuir teint (multiplié par la teinte
  puis calque `_overlay`), bouclier et cape dessinés aux couleurs du monde. L'armure est
  celle portée au lancement : le choix des pièces viendra avec les menus (étape 5).
- Tests : `collection.rs` (pièce gagnée portée, relue, ensemble complet, pièce non
  possédée refusée), `butin.rs` (200 parties : jamais une pièce possédée ni deux fois la
  même), planches du bouclier et de la cape, cuir teint.

### 4h : Profil du joueur (fait)

- `bs-sauvegarde/src/profil_joueur.rs` (port de `profil.ts`) : pseudo (16 caractères,
  espaces nettoyés, pseudo au hasard), skin (celle de la classe, une des 9 skins par
  défaut du jeu en bras larges ou fins, ou la skin importée). Même JSON que le jeu TS (clé
  `profil`). L'apparence ne change que le modèle (`humanoide` ou `humanoide_fin`) et la
  skin : la classe garde ses statistiques, ses armes et son objet en main.
- Skins importées (`bs-rendu/src/skins.rs`, port de `skinsImportees.ts`) : patron carré
  64×64 ou HD (128 à 1024), taille voisine remise à la taille HD la plus proche, ancien
  format 64×32 converti (membres gauches = droits en miroir). Une skin HD qui n'est
  qu'une skin agrandie est gardée à sa vraie taille ; une skin trop lourde est réduite de
  moitié jusqu'à tenir. Gardée en PNG dans une data URL, comme le jeu TS, et donnée aux
  textures sous `profil/skin` au lancement (`Textures::inserer`).
- Pour les essais : `BS_SKIN=<fichier.png>` importe et choisit une skin,
  `BS_PSEUDO=<pseudo>` change le pseudo (tous deux enregistrés ; la page Profil, 5d, fait
  la même chose à la souris).
- Étiquette du pseudo (`bs-rendu/src/hud/etiquette.rs`) : texte blanc en police pixel sur
  fond noir translucide, 0,35 bloc au-dessus de la tête, taille qui suit le zoom par
  pixels de police entiers, cachée à la mort.
- Tests : profil relu, apparence, ancienne skin « perso » et skin importée absente
  ramenées à la skin de la classe ; formats refusés, skin HD agrandie gardée à sa vraie
  taille, ancien format complété.

### 4i : Amis (liste gardée)

- `bs-sauvegarde/src/amis.rs` (partie gardée de `reseau/amis.ts`) : code ami du profil
  (« ABC-DEF-GHJ », 9 signes d'un alphabet sans 0, O, 1, I, L, tiré une fois), amis
  ajoutés (50 au plus) avec leur dernier pseudo connu. Même JSON que le jeu TS (clé
  `amis`, version 1) : un profil importé garde ses amis et son code. Ajout refusé avec le
  même message que le jeu TS (code invalide, propre code, déjà ami, liste pleine).
- Présence en ligne, invitations et demandes : avec la coop par relais (étape 6).
- Tests : codes remis en forme et tirés au hasard, liste relue, JSON du jeu TS (code
  invalide écarté, pseudo absent).

## Étape 5 : menus

### 5a : Menu principal, lancement d'une partie, retour au menu (fait)

- Phases `Menu` → `Chargement` → `Partie`. Sans `BS_ARENE`, le jeu s'ouvre sur le menu ;
  avec, la partie se lance tout de suite (robot de démonstration, captures).
- `bs-rendu/src/lancement.rs` : arènes proposées (celles du jeu, puis les maps de
  `cartes/`), dernier choix gardé dans le profil (clé `choix`, même JSON que le jeu TS),
  préparation de la partie choisie (graine tirée de l'horloge), et `nettoyer` au retour au
  menu : toutes les entités racines et les ressources de la partie sont retirées, comme
  le rechargement de page du jeu TS.
- `bs-rendu/src/menu/` : classes à gauche (verrous, description, cœurs, vitesse, arme,
  bonus de classe), pseudo, maîtrise et talents au centre, arènes à droite (verrous,
  records, mobs dans l'ordre d'arrivée, calendrier des boss, record sans fin). « Jouer »
  (ou Entrée) lance, « Quitter » ferme le jeu. Caméra 2D propre au menu.
- Pause et écran de fin : bouton « Menu principal » (partie abandonnée ; une partie finie
  est déjà comptée). Le HUD d'une nouvelle partie est entièrement réécrit, les textures
  d'armure redemandées à chaque partie (la Collection peut changer entre deux).
- Essais : `BS_JOUER=n` lance la partie choisie aux n premières entrées au menu,
  `BS_PAUSE=menu` revient au menu dès le début de la partie. Vérifié : menu, partie,
  retour, seconde partie (une seule étiquette, HUD complet).


### 5b : Talents (fait)

- `bs-rendu/src/menu/talents/` : onglets des classes (verrous) et maîtrise, onglets des
  deux arbres (icône, points placés, « en partie »), « Utiliser en partie »,
  « Réinitialiser (+n points) ».
- Schéma (`schema.rs`) : liens de redstone allumés quand le talent du dessus est complet,
  paliers (« x / y rangs » puis « ouvert »), cases par type (bonus, capacité, à choix,
  final), rang acheté, « + » pour une arme de départ, « zZ » pour un talent inactif, pied
  à la couleur de la branche.
- Fiche (`fiche.rs`) : nom, type, rang, branche, effet ou options, prérequis, coût, refus
  d'achat expliqué, boutons Acheter, option, talent final actif, arme de départ active.
- Toute la logique reste dans `bs-sauvegarde` (`progression/talents.rs`) ; la page est
  refaite à chaque clic (hors de la boucle de jeu). Textes à gauche : `TextePixel::a_gauche`.
- Essais : `BS_MENU=talents` ouvre la page à l'entrée. Vérifié aux vrais clics (xdotool
  sous Xvfb) : choix d'un talent, achat, points et paliers mis à jour, sauvegarde écrite.

### 5c : Collection (faite)

- `bs-rendu/src/menu/collection.rs` : un onglet par monde (avancée en %), barre de
  progression, armure du monde (pièces gagnées avec leur bonus, ou grisées avec le boss
  qui les lâche), bonus d'ensemble (vert quand complet), bagues et amulettes du monde
  (« ??? » et rareté tant qu'elles ne sont pas trouvées), puis celles de tous les mondes.
  Ouverte sur le monde de l'arène choisie ; bouton « Collection » de l'accueil.
- Icônes (`bs-rendu/src/icones_armures.rs`) : bouclier, cape et talisman dessinés aux
  couleurs du monde ; casque, plastron, jambières, bottes : icônes du jeu, cuir multiplié
  par la teinte du monde sous son calque. Les dessins (`Dessins`) sont faits une fois, à la
  première entrée au menu ou au chargement, et servent à toutes les parties.
- Le choix des pièces portées viendra avec la page Profil (comme dans le jeu TS).
- Essais : `BS_MENU=collection`. Vérifié aux clics (onglets), avec une Collection remplie.

### 5d : Profil (fait)

- `bs-rendu/src/menu/profil/` : skins à gauche (celle de la classe, les 10 du jeu, la
  skin importée), avec le visage découpé dans la texture ; bras fins ou larges ; pseudo
  au centre (clic, saisie au clavier, Entrée valide, Échap annule, nettoyé comme dans le
  jeu TS) ; pièces d'armure portées à droite (« — » ou une des 4 pièces du monde, éteinte
  tant qu'elle n'est pas gagnée). Glisser un PNG sur la fenêtre importe la skin (même
  traitement que `BS_SKIN`). Tout est enregistré aussitôt.
- Aperçu 3D (`menu/apercu.rs`) : le menu a une caméra 3D sous sa caméra 2D ; sur la page
  Profil, le héros (skin, objet tenu, armure portée) tourne lentement au centre. Il est
  construit par `Rendus::heros` (le même code que les modèles en partie) une fois ses
  textures chargées, et refait quand la skin ou l'armure change.
- Essais : `BS_MENU=profil`. Vérifié aux clics (skin, casque, cape) et au clavier (pseudo).

### 5e : Profils (fait)

- `bs-rendu/src/menu/profils.rs` : les profils de l'installation (pseudo, maîtrise des
  classes jouées, lus sans charger le profil), « Jouer avec », « Supprimer » (second clic
  pour confirmer, jamais le dernier), nouveau profil (pseudo tapé ou tiré au hasard, au
  plus 8 profils), « Exporter ce profil » (`.bssave` dans Téléchargements, ou dans le
  dossier des sauvegardes), import en glissant un `.bssave` sur la fenêtre.
- Changer de profil (ou supprimer l'actif, créer, importer) relit toutes ses données
  (`Sauvegarde::recharger`), sa skin importée et son dernier choix, puis revient à
  l'accueil : pas besoin de relancer le jeu comme dans la version TS.
- Essais : `BS_MENU=profils`. Vérifié aux clics et au clavier (création, export,
  suppression confirmée de l'actif).

### 5f : Options (faites)

- `bs-rendu/src/options/` : les options du joueur (`block-survivors.options`, même format
  que la version TS ; les options pas encore reprises, compteur de dégâts, vue F5, effets
  des alliés, sont réécrites telles quelles). Chaque changement est appliqué et enregistré
  aussitôt : volumes des sons, compteur de FPS (coin haut droit), secousses de l'écran,
  ombres (non, oui, oui fines : carte d'ombre 4096), chiffres des dégâts.
- Panneau par-dessus le menu principal (bouton « Options ») et la pause : curseurs de
  volume (clic ou glissé, pas de 5 %), bascules, « Terminé » ou Échap pour fermer.
- Reste : le vent n'est pas coupé avec les ombres (shader), et les options compteur de
  dégâts, vue F5 et effets des alliés n'ont pas encore d'équivalent en jeu.
- Vérifié aux clics, au menu et en pause (fichier d'options écrit, FPS affichées).

### 5g : Nouveautés (faites)

- `bs-rendu/src/menu/nouveautes.rs` : le journal des versions (CHANGELOG.md, lu à la
  compilation), les 4 plus récentes, défilant à la molette. Le bouton du menu porte
  « Nouveau ! » tant que la page n'a pas été ouverte pour la version installée
  (`block-survivors.version-vue`, comme la version TS).
- Essais : `BS_MENU=nouveautes`. Vérifié aux clics (ouverture, défilement, pastille effacée).

### 5h : Écran de chargement (fait)

- `bs-rendu/src/chargement.rs` : pendant la phase `Chargement`, la photo de l'arène en
  plein écran (les 4 photos fournies, embarquées en `.webp` et décodées une fois), son
  nom, une astuce au hasard (`textes.astuces`), la barre ornée qui ne recule jamais
  (textures chargées ; « Construction de la carte… » pendant l'import d'une map).
- Pas encore repris : la photo prise à chaque fin de chargement pour remplacer celle
  fournie (version TS).
- Essais : `BS_CHARGEMENT=1` garde l'écran de chargement. Vérifié sur la Zone volcanique,
  le Jardin des champignons (map importée) et au lancement depuis le menu.

### 5i : Héros au centre du menu (fait)

- L'aperçu 3D de la page Profil sert aussi à l'accueil : le héros choisi (skin, armure
  portée) tourne au centre, au-dessus du pseudo, de la maîtrise et des boutons des pages
  (deux rangées). Le fond de l'accueil est celui de la caméra 3D.

### 5j : Compteur de dégâts en jeu (fait)

- Panneau compact en haut à droite (`hud/compteur.rs`) : une ligne par arme, de la plus
  forte à la plus faible (icône, barre relative à la meilleure, nom, dégâts abrégés,
  dégâts par seconde depuis l'obtention de l'arme, part du total) et le total en pied.
  Rafraîchi deux fois par seconde ; huit lignes créées une fois, seulement réécrites.
- Tab l'affiche ou le cache ; c'est l'option « compteurDegats » (enregistrée, aussi dans
  le panneau des options : « Compteur (Tab) »).
- Pas repris : le glisser du panneau par son titre (position retenue dans le jeu TS).
