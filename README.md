# Block Survivors

Survivor-like en vue de dessus dans un monde voxel aux textures et aux sons
Minecraft. Survivez 15 minutes à la horde. TypeScript + three.js (rendu
WebGPU, repli automatique sur WebGL2), construit avec Vite.

Le moteur utilisé (WebGPU ou WebGL2) s'affiche en bas du menu, à côté de la
version. Si le navigateur annonce WebGPU mais ne le gère pas assez bien, ou si
le GPU décroche en cours de partie, le jeu repasse tout seul en WebGL2.

## Jouer sans rien installer (Windows)

Chaque push construit l'application de bureau : télécharge
**`BlockSurvivors.exe`** dans la release « derniere-version » (onglet *Releases*
du dépôt) et double-clique dessus. C'est un exécutable portable : rien à
installer, rien à construire. La fenêtre s'ouvre agrandie, l'interface
s'adapte à sa taille (sans défilement) et `F11` bascule en plein écran.

## Lancer depuis le code

```bash
npm install
npm run dev        # serveur de développement : http://localhost:5173
npm run build      # vérifie les types puis produit dist/index.html (autonome)
npm run preview    # sert le build sur http://localhost:4173
npm run typecheck  # vérification TypeScript seule
npm run app        # construit puis ouvre le jeu dans une fenêtre Electron
npm run app:dev    # fenêtre Electron sur le serveur de développement (rechargement à chaud)
npm run exe        # produit release/BlockSurvivors.exe (Windows, portable)
```

Si les scripts npm échouent sous Windows avec « "node" n'est pas reconnu »,
c'est que `cmd.exe` ne trouve pas Node (PATH) ; en attendant, appelle les
outils directement : `node node_modules/vite/bin/vite.js build`.

`dist/index.html` est un fichier unique : il s'ouvre d'un double-clic et se
charge tel quel dans Electron (`win.loadFile('dist/index.html')`).

## Versions

Chaque version a son numéro (`0.x.0` nouveautés, `0.x.y` corrections), son
tag git `vX.Y.Z` et sa release GitHub avec l'exe ; le lien « derniere-version »
pointe toujours sur la plus récente. Le détail est dans `CHANGELOG.md`, affiché
dans le jeu (bouton « Nouveautés » de l'accueil, avec une pastille « Nouveau ! »
après une mise à jour). L'accueil prévient quand une version plus récente existe,
d'après `version.json` du dépôt public `Block-Survivors-version`, et renvoie vers
les releases de ce même dépôt, où l'exe est aussi publié (le code, lui, reste
privé). Le workflow publie l'exe et `version.json` si le secret `VERSION_TOKEN`
(droit « Contents : lecture et écriture » sur `Block-Survivors-version`) est configuré.

## Structure

```
electron/               application de bureau : fenêtre, pont vers la page (Quitter)
scripts/app-dev.mjs     Vite + Electron avec rechargement à chaud
scripts/icone.py        génère l'icône de l'application (electron/icone.png)
cartes/                 maps construites dans Minecraft (.schem, .nbt) et leurs réglages (.json)
.github/workflows/      construction automatique de l'exe Windows
index.html              balisage de l'interface (chargement, menu, HUD, écrans, debug)
src/
  main.ts               point d'entrée : interface, menu, puis génération et lancement
  jeu.ts                assemble les systèmes et fixe l'ordre de mise à jour
  lancement.ts          choix de la partie et navigation menu ↔ partie
  options.ts            options du joueur (volume, FPS, secousses), gardées dans le navigateur
  config.ts             réglages techniques + version et sources des textures, fichiers et sons
  etat.ts               état de la partie (source de vérité de la simulation) et statistiques
  donnees/              CONTENU data-driven : blocs, structures, arènes, modèles, personnages,
                        ennemis et boss, armes, passifs, objets, progression, talents, particules, sons
  assets/               chargement des textures, listes des textures requises, police pixel
  audio/                lecture des sons vanilla
  moteur/               rendu, boucle de jeu, caméra, géométries UV
  monde/                génération des arènes, affichage, carte de collision, navigation
  minecraft/            maps Minecraft : lecture NBT et schématiques, modèles de blocs vanilla,
                        atlas et maillage, zone jouable
  entites/              joueur, ennemis et comportements, projectiles ennemis, orbes d'XP,
                        objets de soin, butin, directeurs d'apparition (hordes et boss)
  combat/               armes du joueur : arsenal et une classe par mécanique
  progression/          niveaux, bonus passifs, modificateurs, cartes d'amélioration, sauvegarde (maîtrise, talents)
  modeles/              modèles cubiques : compilation, animation, rendu instancié
  effets/               ombres, particules vanilla (sprites face caméra), rendu des projectiles (instanciés), retours (sons, secousses)
  controles/            clavier et molette
  ui/                   menu, HUD, cartes, pause, options, écran de fin et bilan, panneau debug,
                        écran de chargement, habillage (sprites d'interface), portraits, styles
  utils/                fonctions pures, pool d'objets, grille spatiale
```

Ajouter du contenu = ajouter une entrée dans `src/donnees/`. Chaque système
expose `maj(dt)` (simulation, ne touche qu'à l'état) et `majVisuel(...)`
(recopie dans la scène et le DOM).

## Textures

Textures vanilla de Minecraft **26.3** (`VERSION_MINECRAFT` dans
`src/config.ts`), cherchées dans l'ordre :

1. `raw.githubusercontent.com/InventivetalentDev/minecraft-assets`
2. `assets.mcasset.cloud`
3. local : `public/assets/mc/<catégorie>/<nom>.png` (ex. `public/assets/mc/block/dirt.png`)
4. couleur unie de secours

Pour le local, extrais les PNG de `assets/minecraft/textures/` du `.jar` de ton
client. Les navigateurs refusent les PNG locaux en `file://` : utilise
`npm run dev` ou `npm run preview`.

## Police

La police pixel du jeu est reconstruite au lancement à partir des planches
`font/ascii.png`, `font/accented.png` et `font/nonlatin_european.png` et de la
table `assets/minecraft/font/include/default.json` (mêmes sources ; en local :
`public/assets/mc/font/include/default.json`). Sans elles, l'interface garde une
police de secours.

## Sons

Les sons ne sont pas dans le `.jar` : ils sont lus depuis le serveur de
ressources de Mojang (`resources.download.minecraft.net`, par empreinte de
l'index d'assets 1.21.4), sinon en local dans
`public/assets/mc/sounds/<chemin>.ogg` (ex. `public/assets/mc/sounds/random/orb.ogg`,
chemins dans `src/donnees/sons.ts`). Sans source, le jeu reste muet. Les
navigateurs n'autorisent le son qu'après une première interaction (clic ou
touche).

## Partie

Le menu principal choisit le personnage et l'arène (le dernier choix est
retenu). Tenir jusqu'à **15:00** donne la victoire ; un boss arrive à 5:00
puis à 10:00. L'écran de fin (mort ou victoire) affiche le bilan : temps,
niveau, expérience, ennemis vaincus par type, boss, dégâts infligés par arme,
dégâts subis, soins. « Rejouer » relance le même choix, « Menu principal »
revient au menu.

L'adresse peut aussi fixer le choix et sauter le menu :
`index.html?personnage=archer&arene=mine&jouer=1`.

Chaque coup affiche ses **dégâts** au-dessus de l'ennemi (critiques plus gros
et dorés ; désactivable dans les options). Les armes peuvent porter des **coups critiques** (5 % de chance au départ,
dégâts ×1,5, étincelles du jeu) ; le bonus *Silex taillé* et certains talents
les rendent plus fréquents ou plus forts.

## Progression entre les parties

Chaque classe a sa **maîtrise** : toute l'XP ramassée en partie la fait monter
(niveau 1 à 300 XP, chaque niveau suivant en demande 75 de plus, voir
`MAITRISE` dans donnees/talents.ts). Chaque niveau donne **un point de
talent**, affiché à la fin de la partie, au menu et dans l'écran des talents.

**Talents** (bouton du menu) : chaque personnage a **deux arbres** au choix,
un seul actif à la fois ; chacun dispose de tous les points de la classe. Un
rang coûte un point ; les paliers s'ouvrent après 8 puis 18 rangs achetés au-dessus.
« Réinitialiser » rend gratuitement tous les points de l'arbre.

## Émeraudes, coffres, mini-boss

En partie, les ennemis lâchent parfois un **tas d'émeraudes** (3,5 %, de 2 à 6
émeraudes, plus gros quand il en vaut plus), aspiré vers le joueur comme les
orbes d'XP ; elles ne servent que pendant la partie. Un **coffre** apparaît près du joueur tous
les tant d'ennemis tués (100, puis +150, +200…) et à la mort de chaque
mini-boss : marcher dessus l'ouvre si l'on a assez d'émeraudes (8, puis 13, 18…
à chaque coffre ouvert), et l'on choisit une carte d'amélioration. Les boss
lâchent un **coffre de l'Ender** gratuit (deux cartes). Le prix flotte
au-dessus des coffres.
Données : donnees/coffres.ts, donnees/objets.ts.

Les **mini-boss** (Zombie colosse, Vagabond, Araignée géante, Brute piglin,
Champion wither) arrivent sans prévenir toutes les 75 s dès 1:30
(`CALENDRIER_MINIBOSS`), avec une barre de vie au-dessus de la tête ; chaque
arène choisit les siens.

| Personnage | Arbres |
| --- | --- |
| Guerrier | Gladiateur (dégâts, critiques, épée renforcée) · Gardien (cœurs, armure, totem au départ) |
| Archère | Tireuse d'élite (portée, critiques, arc renforcé, +1 projectile) · Rôdeuse (vitesse, collecte, relances, régénération) |
| Mage | Pyromancien (zone, boules de feu renforcées, critiques) · Arcaniste (cadence, soins, relances, Éclairs dès le départ) |
| Invocateur | Maître-chien (loups renforcés, critiques, +1 allié) · Évocateur (vex renforcés, armure, golem dès le départ) |

**Déblocages** : les trois classes sont jouables d'emblée ; la Mine s'ouvre en
survivant 5:00 dans la Prairie, le Nether en survivant 5:00 dans la Mine. Tout
est sauvegardé dans le navigateur (`src/donnees/talents.ts` pour les réglages).

## Accueil et apparence

L'accueil a trois colonnes : la **classe** (cœurs, vitesse, arme de départ,
bonus exclusifs, arbres de talents), le **héros** en 3D (glisser pour le
tourner), son **pseudo** (bouton « Aléatoire »), « Apparence… » et
« Talents… », et l'**arène** (mobs, boss et leur heure d'arrivée, record,
conditions de déblocage). Le grand bouton « Jouer » (ou Entrée) lance la partie.

« Apparence… » : **skin** de la classe, le **perso** de l'éditeur (panneau
« Traits » : peau, yeux, coiffure, cheveux, barbe, haut et sa couleur,
pantalon, chaussures, bouton Aléatoire), une des 9 skins par défaut du jeu, ou
un PNG importé (64×64, format du jeu, HD acceptée), bras larges ou fins, avec
le héros en 3D. La seconde couche des skins (chapeau, veste…) est affichée. Le pseudo s'affiche au-dessus du personnage comme dans le jeu ;
la skin ne change que l'apparence.

## Personnages

Guerrier (épée), Archère (arc), Mage (boules de feu) ou Invocateur (meute de
loups ; vex volants et golem de fer à gagner en cartes ou en talents).

## Arènes

Chaque arène a ses propres mobs et ses boss (5:00 puis 10:00) :

| Arène | Mobs | Boss |
| --- | --- | --- |
| Prairie du village abandonné | zombie, araignée, creeper, squelette | Géant, Seigneur wither |
| Mine souterraine | zombie, araignée venimeuse, poisson d'argent, squelette, creeper | Reine araignée, Géant |
| Zone volcanique | piglin zombifié, cube de magma, blaze, squelette wither | Cube de magma géant, Seigneur wither |

Une arène se décrit entièrement dans `src/donnees/arenes.ts` : sol, taches de
sol, murs, structures (dessinées couche par couche dans `structures.ts`)
placées ou dispersées, ambiance, et population d'ennemis par zone.

### Maps construites dans Minecraft

Chaque schématique du dossier `cartes/` devient une arène, au nom du fichier :
`.schem` de WorldEdit / FAWE (`//copy` puis `//schem save nom`) ou `.nbt` de
bloc de structure. Tous les blocs vanilla s'affichent avec leur vraie forme :
le jeu lit les fichiers officiels `blockstates/` et `models/` (même source que
les textures) et les assemble en trois maillages (opaque, découpé,
translucide), sans les faces cachées.

La zone jouable est le sol plat où se trouve un panneau `depart` (sinon le
centre de la schématique), avec tout ce qui y est relié à pied au même
niveau ; le reste de la construction est le décor. Réglages facultatifs (nom,
icône, ennemis, boss, ambiance) dans un `.json` du même nom : voir
`cartes/LISEZMOI.md`. Code : `src/minecraft/`.

## Contrôles

ZQSD / WASD / flèches : se déplacer · molette : zoom · `1` `2` `3` ou clic :
choisir une carte · `R` : relancer les cartes (3 fois par partie) · `Tab` : compteur de dégâts (déplaçable par son titre) · `Échap` : pause · `Entrée` : jouer (menu) ou rejouer (fin).

La pause propose Reprendre, Options (volume général et volumes séparés
joueur / armes / créatures / interface, compteur de FPS, secousses de l'écran,
ombres et vent, chiffres des dégâts), Recommencer, Menu
principal, et le bilan en cours.

**Ombres et vent** (actifs par défaut) : ombres du soleil, feuillage et
plantes (herbes, fleurs) qui ondulent au vent ; *OUI (fines)* affine les
ombres, *NON* revient au rendu d'origine. Les textures restent celles du jeu.



## Debug

`F3` ou `²` : panneau compact avec FPS, appels de dessin, moteur, entités,
textures ; vitesse du temps (`-` / `+`), invincibilité (`I`), frappe de zone
(`K`), +1 niveau (`L`), prochain boss (`B`), saut à 14:50, +1 niveau de maîtrise ; puis des rangées
d'icônes (nom au survol) pour faire apparaître chaque ennemi, donner chaque
arme ou bonus (exclusifs compris, ex. le totem), poser chaque objet, et
changer de personnage ou d'arène (même verrouillés).

En `npm run dev`, la partie est accessible dans la console du navigateur via
`window.blockSurvivors`.

---

Les textures, sons, police et noms Minecraft appartiennent à Mojang ; projet de fan non commercial.
