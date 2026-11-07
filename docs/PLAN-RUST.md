# Plan : Block Survivors en Rust (Bevy), multi par relais, mises à jour

Objectif : reprendre la technique d'Azeroth Survivors (Rust + Bevy, WebGPU, WebAssembly
pour le navigateur) et leur modèle de coop (un joueur hôte, tout passe par un relais
WebSocket), tout en gardant ce que Block Survivors a déjà : contenu, progression,
sauvegardes, amis, mises à jour automatiques.

Taille du chantier : le jeu actuel fait environ **30 000 lignes de TypeScript (176
fichiers)**. C'est une réécriture complète, pas une conversion automatique. Le plan
avance par étapes jouables : à chaque étape, l'ancienne version reste celle que les
joueurs téléchargent, jusqu'à ce que la nouvelle fasse au moins aussi bien.

> **Décision du 01/11/2026 : plus de version navigateur.** Le jeu Rust sort seulement sur
> PC (Windows, Linux). Le crate `bs-web`, les scripts de build web et la tâche web de la
> CI sont retirés ; tout ce qui parle de WebAssembly, WebGPU/WebGL2 ou de poids du wasm
> plus bas ne s'applique plus (gardé pour l'historique).

---

## 1. Ce qu'utilise Azeroth Survivors (relevé dans leur dépôt et leur jeu en ligne)

| Rôle | Leur choix | Pour Block Survivors |
|---|---|---|
| Langage | Rust 1.98, épinglé (`rust-toolchain.toml`) | Pareil |
| Moteur | **Bevy 0.18** sans son audio intégré (2D, 3D, UI, scènes, picking) | Bevy 0.18 (ou la version stable du moment) |
| Rendu | wgpu 27 (WebGPU dans le navigateur ; Vulkan, DirectX 12, Metal sur PC) ; **deux builds web** (WebGPU et WebGL2, les fonctionnalités s'excluent dans Bevy), leur build WebGL2 se limite aux écrans d'accueil | Pareil : deux builds wasm, un chargeur JS qui teste `navigator.gpu` ; le build WebGL2 doit faire tourner le jeu (navigateurs Linux, vieux PC) |
| Son | **kira** (mixage, son spatial), modifié par eux pour calculer les gains spatiaux par bloc et non par échantillon (performances) | kira, avec leur modification ; à tester en wasm (un seul fil d'exécution : risque de son haché) |
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
    bs-maj/       mises à jour (Velopack, manifeste signé, voies, retour arrière)
    block-survivors/  le jeu (PC)
    bs-web/          le jeu (navigateur, wasm)
```

Principe clé, déjà vrai aujourd'hui : **la simulation (`bs-regles`) ne dépend pas du
rendu.** Elle reçoit les commandes des joueurs et un pas de temps, et produit l'état. Ça
permet le robot d'équilibrage, les tests rapides et la coop (l'hôte simule, les autres
affichent).

---

## 2 bis. Structure pour la maintenance

But : qu'on puisse ajouter une arme, un ennemi ou une arène sans toucher au moteur, trouver
n'importe quoi en quelques secondes, et que deux personnes (ou deux sessions) travaillent
en même temps sans se marcher dessus.

### Arborescence détaillée

```
block-survivors/
  Cargo.toml                 espace de travail : versions des dépendances épinglées une seule fois
  rust-toolchain.toml        version de Rust épinglée
  CHANGELOG.md  README.md  ARCHITECTURE.md (carte du code, une page)
  donnees/                   CONTENU DU JEU, en fichiers RON (pas de code)
    armes/  ennemis/  arenes/  talents/  equipement/  armures/  textes/fr.ron
  crates/
    bs-commun/     types partagés : identifiants, maths, horloge, erreurs
    bs-donnees/    chargement + validation des fichiers de donnees/ (types serde)
    bs-regles/     simulation pure (aucun Bevy) : un module par système
      combat/ (une arme = un fichier)  ennemis/  progression/  equipement/  equipe/
    bs-monde/      arènes, maps .schem, grille de collisions, champs de flux
    bs-rendu/      plugins Bevy : modeles/  particules/  shaders/ (WGSL)  camera/
    bs-ui/         plugins Bevy : un écran = un module (menu, cartes, pause, fin, pages)
    bs-audio/      kira, catégories de sons
    bs-sauvegarde/ profils, format, chiffrement, migrations de version
    bs-protocole/  messages versionnés et format binaire (sans Bevy ni tokio)
    bs-reseau/     client du relais, prédiction, interpolation
    bs-relais/     serveur (axum + tokio), sans dépendance au moteur
    bs-maj/        mises à jour (Velopack, manifeste signé, voies, retour arrière)
    bs-robot/      robot d'équilibrage (ligne de commande, sans rendu)
    block-survivors/  binaire PC : assemble les plugins
    bs-web/           point d'entrée wasm
  outils/          scripts de build web, import de maps, génération d'icônes
  tests/           tests de bout en bout (parties complètes, coop à plusieurs)
  .github/workflows/  CI : vérifs, builds, publication
```

### Règles de dépendance (vérifiées par la CI)
- Le sens va toujours **du bas vers le haut** :
  `bs-commun → bs-donnees → bs-monde → bs-regles → bs-protocole → bs-reseau → bs-rendu / bs-ui / bs-audio → binaires`
  (`bs-monde` sous `bs-regles` : les ennemis suivent les champs de flux du monde).
- **`bs-regles` et `bs-monde` ne connaissent pas Bevy** : la simulation se teste et tourne
  pour le robot et les tests sans fenêtre ni carte graphique.
- **`bs-protocole`** (messages et format binaire, sans Bevy ni tokio) est le seul lien entre
  le jeu et le relais : `bs-relais` ne dépend que de lui, pas du client ni de la prédiction
  (`bs-reseau`). Le relais **ne simule rien**.
- `bs-sauvegarde` dépend de `bs-donnees` ; `bs-audio` de Bevy ; `bs-maj` est réservé au PC
  (`cfg(not(target_arch = "wasm32"))`) ; `bs-robot` dépend de `bs-regles` seulement.
- Aucun cycle entre crates ; un crate n'expose qu'une petite API publique (`pub` minimal).

### Simulation et Bevy : le pont
- La simulation garde **son propre état** (tableaux à plat, pools, comme aujourd'hui) et
  avance par pas fixes. Elle n'utilise pas l'ECS de Bevy.
- Un seul système Bevy recopie cet état vers les entités de rendu à chaque image, en
  interpolant entre les deux derniers pas. Le rendu ne modifie jamais la simulation.
- Même pont chez un joueur qui rejoint : il remplit l'état à partir des instantanés reçus.

### Déterminisme (pour les rejeux et les tests, pas pour la coop)
La coop n'en a pas besoin (l'hôte envoie des instantanés, comme aujourd'hui). Les rejeux, la
vérification des résultats et les tests si :
- horloge fixe (`bs-commun`) ; un générateur à graine par système (`rand_pcg`) ;
- aucun `HashMap` parcouru dans la simulation (`BTreeMap`, `IndexMap` ou tri par identifiant) ;
- pas de dépendance à l'ordre des requêtes ECS ;
- sin, cos, sqrt… via `libm`, pour les mêmes résultats en natif et en wasm ; pas de FMA
  implicite ;
- un test en CI : la même partie en natif et en wasm donne le même état final.

### Le contenu en données
- Armes, ennemis, arènes, talents, objets, textes : des **fichiers RON** dans `donnees/`,
  relus au lancement (et rechargés à chaud en développement, sans recompiler).
- **Validation au chargement** : une référence inconnue (un ennemi qui n'existe pas dans une
  arène, un talent qui pointe vers une arme absente) donne une erreur claire avec le fichier
  et la ligne. La CI charge toutes les données à chaque commit.
- Ajouter une arme = un fichier de données + (si sa mécanique est nouvelle) un fichier dans
  `bs-regles/combat/`. Rien d'autre à toucher.
- Les **formules** restent en code : par exemple l'équipement (environ 30 fonctions dans
  `donnees/equipement.ts` aujourd'hui) devient un module Rust, appelé par son identifiant
  depuis les données.
- Textes de l'interface dans `donnees/textes/fr.ron` : prêt pour d'autres langues.

### Code
- **Une responsabilité par fichier**, 400 lignes au plus en règle générale ; au-delà, on
  découpe (c'est ce qu'on a fait pour le code réseau).
- Bevy organisé en **plugins** (un par domaine) et en **ensembles de systèmes ordonnés**
  (entrées → simulation → rendu → interface), déclarés à un seul endroit.
- Identifiants, noms et commentaires en français, comme aujourd'hui ; `rustfmt` et `clippy`
  obligatoires (la CI refuse sinon).
- Pas d'allocation dans la boucle de jeu (pools réutilisés, comme aujourd'hui) ; un test le
  vérifie sur une partie simulée, avec un allocateur qui compte les allocations.
- Erreurs : `thiserror` dans les crates, `anyhow` seulement dans les binaires et outils.

### Versions et compatibilité
- **Protocole réseau versionné** : chaque message a un numéro de format ; le relais refuse
  les versions différentes avec un message clair.
- **Sauvegardes versionnées** avec des migrations écrites une fois pour toutes
  (`v1 → v2 → …`), testées sur des sauvegardes réelles gardées dans `tests/`.
- Une seule source pour le numéro de version (`Cargo.toml` de l'espace de travail), reprise
  par le jeu, le lanceur, le relais et `version.json`.

### Tests (du plus rapide au plus lent)
1. **Tests unitaires** dans chaque crate (`cargo test`), en quelques secondes.
2. **Tests de données** : tout `donnees/` se charge et se valide.
3. **Parties simulées** par le robot : graines fixes, résultats comparés à des valeurs de
   référence (puissance, durée de survie) ; un écart trop grand fait échouer la CI.
4. **Déterminisme** : même graine et mêmes entrées → même partie, octet pour octet, en natif
   et en wasm (base des rejeux, voir plus haut).
5. **Coop de bout en bout** : un relais local et 3 clients sans fenêtre jouent une partie.
6. **Captures** de chaque écran comparées à des références (les changements visuels sont
   repérés).

### CI (GitHub Actions)
- Sur chaque commit : format, clippy, tests 1 à 5, build web, poids du wasm affiché (alerte
  s'il grossit trop).
- Sur `main` : builds Windows, Linux et web signés, publication de la release et de
  `version.json` (comme aujourd'hui), déploiement du relais.
- Cache de compilation (sccache) pour garder la CI rapide.

### Documentation
- `ARCHITECTURE.md` : la carte (quel crate fait quoi, le sens des dépendances, où ajouter
  quoi).
- Un commentaire d'en-tête par module (ce qu'il fait, pourquoi), comme aujourd'hui.
- `CHANGELOG.md` en français, une section par version (comme aujourd'hui).
- Guides courts : « ajouter une arme », « ajouter une arène », « publier une version ».

### Travail à plusieurs
- Chaque domaine a son crate : deux sessions peuvent avancer en parallèle (ex. une sur
  `bs-reseau`, une sur `bs-ui`) avec peu de conflits.
- Toute nouveauté passe par une branche, puis la CI, avant d'arriver sur `main`.

---

## 3. Étapes

Ordre revu après relecture : les paris techniques risqués (rendu web, son, relais, mises à
jour, reprise des sauvegardes) sont vérifiés **au début**, dans le prototype, et les
sauvegardes et mises à jour passent **avant** la bêta.

### Étape −1 : préparer la version TypeScript (1 à 2 semaines)
- Écrire un **vrai robot d'équilibrage dans le dépôt** (aujourd'hui il n'existe que dans des
  scripts de test à part).
- Faire passer tout l'aléatoire de la simulation par un générateur à graine (`Math.random`
  est appelé dans 32 fichiers ; `utils/aleatoire.ts` a déjà mulberry32) et la simulation par un
  pas fixe (1/60 s).
- Le robot produit des **statistiques de référence** sur N graines (puissance, durée de
  survie, dégâts par arme). La version Rust devra s'en approcher **avec une tolérance**
  (on ne peut pas viser l'identique octet pour octet entre deux langages).
- La **dernière version Electron** sait exporter tous les profils au format `.bssave` existant
  dans un fichier connu (voir étape 4).

### Étape 0 : socle (1 à 2 semaines)
- Espace de travail Cargo, toolchain et Bevy épinglés, CI GitHub Actions : builds Windows,
  Linux et web (WebGPU **et** WebGL2), `clippy`, tests, poids du wasm affiché.
- Fenêtre Bevy qui s'ouvre sur PC et dans le navigateur, avec un bloc.
- Script de build web : `cargo build --target wasm32-unknown-unknown`, `wasm-bindgen`,
  `wasm-opt`, chargeur qui choisit WebGPU ou WebGL2.

### Étape 0 bis : prototype des cinq paris (3 à 4 semaines)
1. **Rendu** : une arène, le joueur, 300 zombies, une arme, à 60 images/s en WebGPU **et**
   en WebGL2 (navigateur) et sur PC.
2. **Son** : kira en wasm, sans son haché.
3. **Relais** : un aller-retour réel avec un instantané de fin de partie, débit et temps
   d'aller-retour mesurés depuis la France.
4. **Mises à jour** : Velopack de N à N+1 (delta compris) sur Windows **et** Linux, avec
   le manifeste signé.
5. **Sauvegardes** : import d'un vrai `.bssave` exporté par la version actuelle.
Décision à la fin : on continue, ou on ajuste le plan (moteur, coop, mises à jour).

### Étape 1 : données et règles, sans rendu (5 à 7 semaines)
- Porter `src/donnees` (environ 5 000 lignes) : contenu en RON, formules en Rust.
- Porter la simulation : combat et armes (environ 5 600 lignes), ennemis et directeurs,
  progression (XP, cartes, coffres, équipement, talents, maîtrise, Collection).
- Porter le robot : ses statistiques doivent tomber dans la tolérance de celles de la
  version TypeScript. C'est le garde-fou de la migration.

### Étape 2 : monde et rendu (8 à 12 semaines)
- Arènes générées et maps importées (`.schem`) : tous les blocs vanilla, teintes de biome,
  eau et lave animées.
- Modèles Minecraft (joueur, mobs, armures, objets tenus) avec animation, en instancing.
- Textures et sons : même source qu'aujourd'hui, chargés au lancement (rien de Mojang dans
  le dépôt).
- Particules, chiffres de dégâts, ombres, lumières, shaders WGSL, vent, herbe.
- Caméra du dessus et à la 3e personne (F5).

### Étape 3 : jouable en solo (3 à 4 semaines)
- HUD complet, écrans de cartes, pause et fin (ornements), mode sans fin, sons par catégorie.
- **Jalon : une partie solo complète, au niveau de la version TypeScript.**

### Étape 4 : sauvegardes et profils (2 semaines)
- **Format repris tel quel** : AES-256-GCM, clé = SHA-256 du secret, IV de 12 octets,
  en-tête `BSSAVE1:`, base64 (crates `aes-gcm`, `sha2`, `base64`) ; valeurs en JSON dans du
  JSON ; testé sur de vraies exportations.
- **Depuis Electron** : la version Rust ne lit pas le stockage de Chromium (illisible
  proprement) ; elle lit le fichier `.bssave` exporté par la dernière version Electron
  (étape −1). Import manuel d'un `.bssave` toujours possible.
- **Navigateur** : même domaine que la version actuelle, sinon le stockage local n'est pas
  accessible.
- Ce chiffrement n'est qu'un brouillage (la clé est dans le jeu) : il évite la retouche
  facile, pas une triche décidée.
- Skins importées (HD comprises), Apparence, Collection, amis.

### Étape 5 : menus (4 à 6 semaines)
- Menu « héros au centre », Apparence (aperçu 3D), Talents, Collection, Profils,
  Multijoueur, Options, Nouveautés, au même style qu'aujourd'hui (bevy_ui).

### Étape 6 : coop par relais (4 à 5 semaines), voir section 4

### Étape 7 : mises à jour et distribution (2 à 3 semaines), voir section 5

### Étape 8 : bêta puis bascule (2 à 4 semaines)
- Bêta : les deux versions publiées côte à côte, la Rust en « essai ». Les joueurs des deux
  versions **ne se voient pas en multi** (Nostr d'un côté, relais de l'autre) : à annoncer.
- Comparaison : statistiques du robot, images/s à 4 joueurs en fin de partie, poids, temps
  de chargement web.
- La version Rust devient la version principale ; la version Electron garde son export de
  profils actif pour ceux qui ne relancent pas le jeu pendant des mois.

**Total estimé : 8 à 12 mois** de travail suivi (marge comprise : apprentissage de Bevy,
rendu Minecraft complet en WGSL, menus). **Bevy est épinglé** ; on ne change de version
qu'entre deux étapes.

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
| **Reconnexion de l'hôte** | L'hôte met la partie en pause dès qu'il perd le relais ; les autres voient « L'hôte a perdu la connexion, reconnexion… ». Il revient avec un **jeton de session** donné par le relais (pas le code de salon), dans un délai fixe ; sinon le salon se ferme proprement. **Si l'hôte plante, la partie est perdue** (pas de changement d'hôte) |
| **Reconnexion des joueurs** | Un joueur qui coupe retrouve sa place avec son jeton ; il reçoit d'abord un **instantané complet**, puis les mises à jour |
| **Refus si versions différentes** | Le relais range les salons par version ; un joueur d'une autre version voit « ce salon joue une autre version du jeu » |
| **L'hôte peut exclure** | « L'hôte vous a retiré du salon » |
| **Succès de coop** | Ex. « Finir une arène à plusieurs », « Vaincre un boss à 4 de la même classe » |
| **Résultats vérifiés** | Pas de signature par l'hôte (il pourrait tricher) : si un jour il y a des classements, le relais **rejoue** la partie à partir des entrées enregistrées (demande le déterminisme) |

### Ce qu'on garde de notre version
- **Les amis** (code ami, présence, Inviter, Rejoindre, Proposer de jouer) : ils passent
  par le relais au lieu des relais Nostr. Le relais range la présence sous une **empreinte**
  du code ami (pas le code en clair), et chaque profil **signe** sa présence avec sa propre clé :
  personne ne peut se faire passer pour un autre.
- Le protocole binaire (instantanés), la prédiction du joueur, le spectateur, les cartes
  différées à terre, les flèches fluides.

### Le relais (`bs-relais`)
- Un seul petit programme Rust (axum + tokio). Le coût dépend du **trafic**, pas du
  processeur : le relais reçoit et réémet chaque instantané. On mesure d'abord le débit
  réel à 4-5 joueurs en fin de partie (prototype), on le double, et on choisit la machine
  (serveur en Europe, temps d'aller-retour mesuré).
- Derrière Cloudflare (comme eux) pour le HTTPS et la protection contre les abus.
  **Battement de cœur** toutes les 30 s : Cloudflare coupe les WebSocket inactifs (environ
  100 s) et lors de ses redémarrages, la reconnexion doit être transparente.
- Limites : nombre de salons par adresse, **tentatives de `rejoindre(code)` limitées par
  adresse** (contre l'essai de tous les codes), taille et débit des messages, salons fermés
  après inactivité.
- Messages : `créer`, `rejoindre(code)`, `lister(version, filtres)`, `données(vers qui)`,
  `quitter`, `exclure`, plus les messages des amis (présence, invitations).
- Surveillance : page d'état, alerte si le relais tombe.

### Option à décider après le prototype
Hybride : connexion directe d'abord (WebRTC, avec la crate `matchbox` en natif et en wasm,
comme la coop actuelle qui marche déjà en pair-à-pair), et le relais **seulement si elle
échoue**. Plus de code, mais beaucoup moins de trafic (donc de coût) sur le serveur. Si les
mesures du prototype montrent un relais cher, on part sur l'hybride.

---

## 5. Mises à jour

Choix : **Velopack** (installeur et mises à jour, écrit en Rust, utilisé en production par
de grosses applis), complété par **notre propre signature** des versions. Pas de lanceur
maison simplifié : on vise directement une chaîne propre, sécurisée et optimisée.

### Ce que fait le développeur
1. Change le numéro de version (un seul endroit : `Cargo.toml` de l'espace de travail) et
   écrit le CHANGELOG.
2. Pousse sur `main` (ou crée un tag pour la voie « bêta », voir plus bas).
3. Tout le reste est fait par la CI.

### Ce que fait la CI (GitHub Actions)
1. Vérifications complètes (format, clippy, tests, données, robot, coop de bout en bout) :
   **rien n'est publié si un seul test échoue**.
2. Compile le jeu (profil `ship` : LTO, optimisations maximales) pour Windows, Linux et le web.
3. **Signe le code Windows** (Authenticode) : vérifier si Azure Trusted Signing est ouvert
   à un particulier en France ; sinon un certificat de signature de code. Un certificat OV
   ne fait pas disparaître l'alerte SmartScreen tout de suite (la réputation se construit
   avec les téléchargements). Sur Linux, personne ne vérifie une AppImage signée : la vraie
   protection est le manifeste signé. Velopack (`vpk`) demande .NET dans la CI.
4. Construit avec Velopack :
   - l'installeur complet (première installation) ;
   - un **paquet « delta »** par version : seulement les octets qui ont changé (souvent
     quelques Mo au lieu du jeu entier).
5. Écrit le **manifeste de la version** (`version.json`) : numéro, voie (stable ou bêta),
   empreinte SHA-256 de chaque fichier, version minimale compatible en multi, notes de
   version, **numéro de série croissant** et **date d'expiration** (à la manière de TUF).
   Il est **signé en ed25519** dans un job séparé, dans un environnement GitHub protégé
   (validation manuelle, jamais lancé par une pull request).
6. Publie la release sur le dépôt public `Block-Survivors-version` (GitHub sert les fichiers en
   HTTPS, gratuitement, partout dans le monde), puis le manifeste en dernier : une version
   n'est annoncée qu'une fois tous ses fichiers en ligne.
7. Déploie la version web (fichiers nommés avec leur empreinte, jamais de vieux fichier en cache)
   et le relais de coop si son protocole a changé.

### Ce que fait le jeu chez les joueurs
1. Au lancement (et toutes les 30 min en jeu), il lit le manifeste en HTTPS.
2. Il **vérifie la signature ed25519 du manifeste** avec la clé publique intégrée au jeu :
   un manifeste falsifié (site piraté, réseau détourné) est refusé. Il refuse aussi un
   manifeste **plus ancien** que le dernier vu (numéro de série) ou **expiré** : personne ne
   peut le ramener de force à une vieille version, ni le bloquer sur une version périmée.
   Velopack ne signe pas son propre flux : le jeu utilise une source de mises à jour maison
   qui ne propose que les paquets listés dans le manifeste signé.
3. S'il y a plus récent sur sa voie, il télécharge **le delta** en arrière-plan (reprise si
   la connexion coupe, débit limité pendant une partie pour ne pas gêner le multi).
4. Il **vérifie l'empreinte SHA-256** de chaque fichier téléchargé contre le manifeste signé.
5. Il applique la mise à jour **à côté** de la version installée, puis bascule d'un coup
   (atomique) : jamais de jeu à moitié mis à jour, même si le PC s'éteint en plein milieu.
6. Au menu : « Version x.y.z prête, redémarrer maintenant ? » ; sinon elle s'installe au
   prochain lancement. Pendant une partie : rien ne s'interrompt.
7. **Retour arrière automatique** : si la nouvelle version plante au démarrage deux fois de
   suite, le jeu revient à la précédente et le signale (journal envoyé à la prochaine
   connexion, si le joueur l'accepte). Les deltas ne vont que vers l'avant : le jeu **garde le
   paquet complet de la version précédente** pour pouvoir y revenir.
8. Sans Internet : le jeu démarre la version installée, sans attendre.

### Voies de publication
- **Stable** : ce que reçoivent tous les joueurs (push sur `main`).
- **Bêta** : pour ceux qui l'activent dans les Options (tag `vX.Y.Z-beta.N`) ; permet de tester
  une grosse nouveauté avec quelques potes avant tout le monde.
- **Retrait d'une version** : un manifeste plus récent marque une version comme retirée et
  désigne **explicitement** la version vers laquelle revenir (jamais un retour implicite).
- **Déploiement progressif** (optionnel) : une nouvelle version proposée d'abord à 10 % des
  joueurs (tirage stable par installation : empreinte d'un identifiant local), puis à tous.

### Multijoueur
- Le manifeste donne la **version minimale compatible** pour la coop : une correction qui ne
  touche pas au protocole ne force pas tout le monde à mettre à jour.
- Si les versions ne sont pas compatibles, le salon le dit et propose « Mettre à jour
  maintenant ».
- Le relais refuse un protocole trop ancien avec un message clair.

### Sécurité
- Tout en HTTPS ; **deux signatures indépendantes** (signature de code Windows et
  manifeste ed25519) ; empreinte vérifiée pour chaque fichier.
- **Deux clés publiques intégrées au jeu** : la clé de la CI (secrets GitHub) et une **clé de
  secours gardée hors ligne** (jamais sur GitHub). Seule la clé de secours peut révoquer ou
  remplacer la clé de la CI : un vol de la clé de CI ne suffit pas à prendre la main.
- Les sauvegardes ne sont jamais touchées par une mise à jour ; avant une migration de format,
  une copie de sécurité est faite automatiquement.

### Version navigateur
- Une nouvelle publication du site suffit ; le jeu ouvert affiche « Nouvelle version
  disponible, recharger ? » au lieu de changer en pleine partie.

### Passage depuis Electron
- Une dernière mise à jour Electron exporte tous les profils (`.bssave`), télécharge
  l'installeur Velopack de la version Rust, **vérifie sa signature**, installe, puis
  désinstalle l'appli Electron **sans effacer ses données** (copie de sécurité gardée) et sans
  laisser de raccourcis en double. Les joueurs n'ont rien à faire à la main.
- Linux : l'AppImage demande libfuse2, absent par défaut sur Ubuntu 22.04 et plus récents ;
  prévoir le message et le lancement par extraction en secours.

### Tests de la chaîne de mise à jour (dans la CI)
- Installer la version N, publier N+1 sur un dépôt de test, vérifier que N se met à jour seule.
- Manifeste falsifié, fichier corrompu, téléchargement coupé, coupure pendant l'application :
  chaque cas doit être refusé ou repris proprement.
- Retour arrière après un plantage simulé au démarrage.

## 6. Risques et points à décider

- **Durée** : 8 à 12 mois sans nouveautés de contenu (ou presque), le temps de rattraper la
  version actuelle. Alternative : continuer les nouveautés en TypeScript pendant la
  migration, mais il faudra les porter deux fois.
- **WebGPU** : pas encore partout (certains navigateurs Linux, vieux PC). Deux builds web
  (WebGPU et WebGL2) ; le WebGL2 testé avec 300 ennemis dès le prototype.
- **Poids web** : Bevy 3D en WebAssembly est lourd (Azeroth : 90 Mo) ; mesuré dès l'étape 0,
  textures chargées à part ; 20 Mo compressés est un objectif ambitieux.
- **Coût du relais** : quelques euros par mois au début, à suivre avec le nombre de joueurs.
- **Droits Minecraft** : rien ne change tant que le jeu reste gratuit et non affilié ; à
  revoir si un jour il est vendu (Steam).

## 7. Première action
Étape −1 (robot et aléatoire à graine côté TypeScript) en parallèle de l'étape 0, puis le
**prototype des cinq paris** (3 à 4 semaines) avant de s'engager sur la suite.

## 8. Relecture
Plan relu par une seconde IA le 31/10/2026. Corrections intégrées : robot à créer et
concordance statistique, déterminisme réservé aux rejeux, deux builds web, crate
`bs-protocole`, pont simulation/Bevy, reconnexion par jeton, anti-abus du relais, présence
signée, sécurité des mises à jour (série, expiration, clé de secours, source Velopack
maison, paquet complet gardé pour le retour arrière), reprise des profils par `.bssave`,
calendrier de 8 à 12 mois.
