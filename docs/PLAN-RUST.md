# Plan : Block Survivors en Rust (Bevy), multi par relais, mises à jour

Objectif : reprendre la technique d'Azeroth Survivors (Rust + Bevy, WebGPU, WebAssembly
pour le navigateur) et leur modèle de coop (un joueur hôte, tout passe par un relais
WebSocket), tout en gardant ce que Block Survivors a déjà : contenu, progression,
sauvegardes, amis, mises à jour automatiques.

Taille du chantier : le jeu actuel fait environ **30 000 lignes de TypeScript (176
fichiers)**. C'est une réécriture complète, pas une conversion automatique. Le plan
avance par étapes jouables : à chaque étape, l'ancienne version reste celle que les
joueurs téléchargent, jusqu'à ce que la nouvelle fasse au moins aussi bien.

---

## 1. Ce qu'utilise Azeroth Survivors (relevé dans leur dépôt et leur jeu en ligne)

| Rôle | Leur choix | Pour Block Survivors |
|---|---|---|
| Langage | Rust 1.98, épinglé (`rust-toolchain.toml`) | Pareil |
| Moteur | **Bevy 0.18** sans son audio intégré (2D, 3D, UI, scènes, picking) | Bevy 0.18 (ou la version stable du moment) |
| Rendu | wgpu 27 (WebGPU dans le navigateur ; Vulkan, DirectX 12, Metal sur PC) | Pareil ; repli WebGL2 à vérifier (Bevy le permet, avec moins d'effets) |
| Son | **kira** (mixage, son spatial), patché | kira |
| Collisions | **avian3d** (contrôleur de personnage) | Inutile au départ : nos collisions sur grille de blocs suffisent, à réécrire telles quelles |
| Texte et UI | bevy_ui + cosmic-text ; menus dessinés par le jeu | bevy_ui ; menus refaits (voir étape 5) |
| Navigateur | wasm-bindgen 0.2.122 épinglé, wasm-opt (binaryen), quelques JS de démarrage | Pareil |
| Serveur | **axum 0.8 + tokio** (WebSocket), derrière **Cloudflare** | Pareil pour le relais de coop |
| Données | Lues dans le vrai client WoW au lancement | Textures et sons Minecraft : même approche qu'aujourd'hui (voir étape 2) |

Leur build web pèse **90 Mo** de WebAssembly : un premier chargement lourd, à surveiller
pour le nôtre (viser bien moins grâce à `wasm-opt -Oz` et en chargeant les textures à part).

---

## 2. Architecture visée

```
block-survivors/            (espace de travail Cargo)
  crates/
    bs-donnees/   données du jeu (armes, ennemis, arènes, talents…) : la copie de src/donnees
    bs-regles/    simulation pure, sans rendu : combat, ennemis, progression, déterministe
    bs-monde/     arènes (génération, maps importées .schem), collisions sur grille
    bs-rendu/     Bevy : modèles Minecraft, instancing, particules, shaders, caméra
    bs-ui/        menus, HUD, cartes, pages (Collection, Talents, Multijoueur…)
    bs-reseau/    protocole de coop (instantanés binaires), client du relais
    bs-relais/    serveur relais (axum + tokio), salons, liste publique
    bs-maj/       vérification et installation des mises à jour (PC)
    block-survivors/  le jeu (PC)
    bs-web/          le jeu (navigateur, wasm)
```

Principe clé, déjà vrai aujourd'hui : **la simulation (`bs-regles`) ne dépend pas du
rendu.** Elle reçoit les commandes des joueurs et un pas de temps, et produit l'état. Ça
permet le robot d'équilibrage, les tests rapides et la coop (l'hôte simule, les autres
affichent).

---

## 3. Étapes

### Étape 0 : socle (1 à 2 semaines)
- Espace de travail Cargo, toolchain épinglée, CI GitHub Actions : builds Windows, Linux
  et web, `clippy`, tests.
- Fenêtre Bevy qui s'ouvre sur PC et dans le navigateur (WebGPU), avec un cube texturé.
- Script de build web : `cargo build --target wasm32-unknown-unknown`, `wasm-bindgen`,
  `wasm-opt`.

### Étape 1 : données et règles, sans rendu (3 à 4 semaines)
- Porter `src/donnees` (environ 5 000 lignes) en Rust : structures `serde`, gardées
  « data-driven ».
- Porter la simulation : combat et armes (environ 5 600 lignes), ennemis et directeurs,
  progression (XP, cartes, coffres, équipement, talents, maîtrise, Collection).
- Porter le robot d'équilibrage : il doit donner **les mêmes chiffres** que la version
  TypeScript (même graine, mêmes parties). C'est le garde-fou de toute la migration.

### Étape 2 : monde et rendu (4 à 6 semaines)
- Arènes générées et maps importées (`.schem`), blocs, eau et lave animées.
- Modèles Minecraft (joueur, mobs, armures, objets tenus) avec animation, rendus en
  instancing pour des centaines d'ennemis.
- Textures et sons : même source qu'aujourd'hui, chargés au lancement (rien de
  Mojang dans le dépôt).
- Particules, chiffres de dégâts, ombres, lumières, shaders (en WGSL), vent, herbe.
- Caméra du dessus et à la 3e personne (F5).

### Étape 3 : jouable en solo (2 à 3 semaines)
- HUD complet : cœurs, XP, chrono, barre de boss, équipement, compteur de dégâts, F3.
- Écrans de cartes, pause et fin (avec ornements), mode sans fin.
- Sons séparés par catégorie (kira).
- **Jalon : une partie solo complète, pareille à la version TypeScript.**

### Étape 4 : sauvegardes et profils (1 semaine)
- Même format de données et même chiffrement (AES-GCM) que la version actuelle :
  **un profil exporté depuis la version TypeScript s'importe dans la version Rust.**
- Au premier lancement de la version Rust, elle reprend toute seule les profils de la
  version d'avant (dans Electron : le stockage local de l'appli ; dans le navigateur :
  le même stockage du même site).
- Skins importées (HD comprises), Apparence, Collection.

### Étape 5 : menus (2 à 3 semaines)
- Menu « héros au centre », Apparence (aperçu 3D), Talents, Collection, Profils,
  Options, Nouveautés, avec le même style qu'aujourd'hui.
- Option : garder certains menus en HTML pour la version navigateur (plus rapide à
  faire, plus facile à styler), dessinés par le jeu sur PC. À décider à cette étape.

### Étape 6 : coop par relais (3 à 4 semaines), voir section 4

### Étape 7 : mises à jour et distribution (1 à 2 semaines), voir section 5

### Étape 8 : bascule (1 à 2 semaines)
- Bêta : les deux versions publiées côte à côte, la Rust en « essai ».
- Comparaison : robot d'équilibrage, performances (images par seconde à 4 joueurs en fin
  de partie), poids du téléchargement, temps de chargement web.
- La version Rust devient la version principale ; la version TypeScript reste
  téléchargeable un moment.

**Total estimé : 4 à 6 mois** de travail suivi, avant d'avoir le jeu actuel à l'identique.

---

## 4. Multijoueur : comme Azeroth Survivors

### Principe
- **Un joueur héberge** et fait tourner la simulation (`bs-regles`), comme aujourd'hui.
- **Tout passe par un relais WebSocket** (`bs-relais`, en Rust avec axum + tokio) : pas de
  connexion directe entre joueurs. Le relais ne simule rien, il fait suivre les messages
  entre l'hôte et les autres joueurs.
- Avantages : ça se connecte **à tous les coups** (plus de problème de box trop fermées)
  et ça marche pareil sur PC et dans le navigateur.
- Inconvénients : un serveur à payer et à surveiller, un petit détour de latence, et
  plus de multi si le relais tombe (leur relais répondait « 502 » pendant notre test).

### Ce qu'on reprend de leur coop
| Fonction | Détail |
|---|---|
| **Jusqu'à 5 joueurs** | Aujourd'hui 4 ; la difficulté s'adapte au nombre (déjà en place) |
| **Code de salon** | Comme aujourd'hui (ABC-DEF), donné par le relais |
| **Salons publics** | Liste des salons ouverts de la même version, filtres (arène, places libres), un clic pour entrer |
| **Reconnexion de l'hôte** | Si l'hôte coupe, les autres voient « L'hôte a perdu la connexion, reconnexion… » ; la partie reprend à son retour (le relais garde le salon ouvert un moment) |
| **Reconnexion des joueurs** | Même chose pour un joueur qui coupe : il retrouve sa place |
| **Refus si versions différentes** | Le relais range les salons par version ; un joueur d'une autre version voit « ce salon joue une autre version du jeu » |
| **L'hôte peut exclure** | « L'hôte vous a retiré du salon » |
| **Succès de coop** | Ex. « Finir une arène à plusieurs », « Vaincre un boss à 4 de la même classe » |
| **Résultats signés** | L'hôte signe le bilan de partie (clé ed25519) ; utile si un jour il y a des classements en ligne |

### Ce qu'on garde de notre version
- **Les amis** (code ami, présence, Inviter, Rejoindre, Proposer de jouer) : ils passent
  par le relais au lieu des relais Nostr, c'est plus simple et plus fiable.
- Le protocole binaire (instantanés), la prédiction du joueur, le spectateur, les cartes
  différées à terre, les flèches fluides.

### Le relais (`bs-relais`)
- Un seul petit programme Rust (axum + tokio), une machine à 5 €/mois suffit pour démarrer
  (des centaines de joueurs : le relais ne fait que transmettre).
- Derrière Cloudflare (comme eux) pour le HTTPS et la protection contre les abus.
- Limites : nombre de salons par adresse, taille et débit des messages, salons fermés après
  inactivité.
- Messages : `créer`, `rejoindre(code)`, `lister(version, filtres)`, `données(vers qui)`,
  `quitter`, `exclure`, plus les messages des amis (présence, invitations).
- Surveillance : page d'état, alerte si le relais tombe.

### Option à garder en tête
Hybride : essayer d'abord la connexion directe (WebRTC, gratuite et plus rapide), et
passer par le relais seulement si elle échoue. Plus de code, mais moins de trafic sur le
serveur. À décider selon le coût réel du relais.

---

## 5. Mises à jour

### Aujourd'hui
Electron + electron-updater : l'appli vérifie le dépôt public `Block-Survivors-version`
au lancement, télécharge la nouvelle version en arrière-plan et l'installe à la fermeture
(ou tout de suite en un clic). La CI publie automatiquement depuis `main`.

### En Rust (plus d'Electron)
- **PC (Windows, Linux)** : un **petit lanceur** en Rust (`bs-maj`), lancé à la place du
  jeu :
  1. il lit `version.json` sur le dépôt public (comme aujourd'hui) ;
  2. si une version plus récente existe, il télécharge le nouveau jeu avec une barre de
     progression ;
  3. il **vérifie sa signature** (clé ed25519 gardée dans les secrets de la CI : un fichier
     modifié en route est refusé) ;
  4. il remplace l'ancien jeu et le lance.
  Crates possibles : `self_update` ou un code maison (reqwest + ed25519-dalek + zip).
  Sans réseau, le lanceur démarre la version installée.
- **Mises à jour en cours de partie** : comme aujourd'hui, un message sur le menu
  (« Version x téléchargée, redémarrer maintenant ? »).
- **Navigateur** : une mise à jour est simplement une nouvelle publication du site ; les
  fichiers portent leur numéro de version (pas de vieux fichiers gardés en cache).
- **Multi** : versions identiques exigées ; le salon dit à un joueur en retard de mettre à
  jour, avec le bouton pour le faire.
- **CI** : construit Windows, Linux et web, signe, publie la release et `version.json`,
  uniquement depuis `main` (comme aujourd'hui). Le poids du jeu passe d'environ 150 Mo
  (Electron) à quelques dizaines de Mo.
- **Passage d'Electron au lanceur Rust** : une dernière mise à jour Electron installe le
  lanceur Rust et y transfère les profils ; ensuite, l'appli Electron n'est plus utilisée.

---

## 6. Risques et points à décider

- **Durée** : 4 à 6 mois sans nouveautés de contenu (ou presque), le temps de rattraper la
  version actuelle. Alternative : continuer les nouveautés en TypeScript pendant la
  migration, mais il faudra les porter deux fois.
- **WebGPU** : pas encore partout (certains navigateurs Linux, vieux PC). Bevy a un repli
  WebGL2 avec moins d'effets ; à tester tôt (étape 0).
- **Poids web** : Bevy en WebAssembly est lourd ; viser moins de 20 Mo compressés.
- **Coût du relais** : quelques euros par mois au début, à suivre avec le nombre de joueurs.
- **Droits Minecraft** : rien ne change tant que le jeu reste gratuit et non affilié ; à
  revoir si un jour il est vendu (Steam).

## 7. Première action proposée
Étape 0 + un **prototype de 2 semaines** : une arène, le joueur, 300 zombies, une arme,
dans le navigateur et sur PC. On mesure les performances, le poids et le temps de
chargement face à la version actuelle avant de s'engager sur tout le reste.
