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

## Structure

```
electron/               application de bureau : fenêtre, pont vers la page (Quitter)
scripts/app-dev.mjs     Vite + Electron avec rechargement à chaud
scripts/icone.py        génère l'icône de l'application (electron/icone.png)
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
                        ennemis et boss, armes, passifs, objets, progression, sons
  assets/               chargement des textures, listes des textures requises, police pixel
  audio/                lecture des sons vanilla
  moteur/               rendu, boucle de jeu, caméra, géométries UV
  monde/                génération des arènes, affichage, carte de collision, navigation
  entites/              joueur, ennemis et comportements, projectiles ennemis, orbes d'XP,
                        objets de soin, butin, directeurs d'apparition (hordes et boss)
  combat/               armes du joueur : arsenal et une classe par mécanique
  progression/          niveaux, bonus passifs, modificateurs, cartes d'amélioration
  modeles/              modèles cubiques : compilation, animation, rendu instancié
  effets/               ombres, particules, rendu des projectiles (instanciés), retours (sons, secousses)
  controles/            clavier et molette
  ui/                   menu, HUD, cartes, pause, options, écran de fin et bilan, panneau debug,
                        écran de chargement, habillage (sprites d'interface), portraits, styles
  utils/                fonctions pures, pool d'objets, grille spatiale
```

Ajouter du contenu = ajouter une entrée dans `src/donnees/`. Chaque système
expose `maj(dt)` (simulation, ne touche qu'à l'état) et `majVisuel(...)`
(recopie dans la scène et le DOM).

## Textures

Textures vanilla de Minecraft **1.21.4** (`VERSION_MINECRAFT` dans
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

## Personnages

Guerrier (épée), Archère (arc) ou Mage (boules de feu).

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

## Contrôles

ZQSD / WASD / flèches : se déplacer · molette : zoom · `1` `2` `3` ou clic :
choisir une carte · `R` : relancer les cartes (3 fois par partie) · `Échap` : pause · `Entrée` : jouer (menu) ou rejouer (fin).

La pause propose Reprendre, Options (volume des sons, compteur de FPS,
secousses de l'écran, shaders, style de l'herbe), Recommencer, Menu
principal, et le bilan en cours.

**Shaders** : *Désactivés* (rendu Minecraft d'origine), *Légers* (ombres du
soleil, lave et glowstone qui rayonnent, hautes lumières adoucies) ou
*Complets* (ombres plus fines et occlusion ambiante en plus).

**Herbe** (essais, au choix) : *Minecraft* (d'origine), *Touffes ondulantes*
(sprites d'herbe vanilla qui ondulent au vent), *Pixels 3D* (colonnes d'un
pixel tirées de la texture), *Stylisée* (brins cartoon) ou *Réaliste* (tapis
dense). Le vent et le passage du joueur sont calculés par la carte graphique.

## Debug

`F3` ou `²` : panneau compact avec FPS, appels de dessin, moteur, entités,
textures ; vitesse du temps (`-` / `+`), invincibilité (`I`), frappe de zone
(`K`), +1 niveau (`L`), prochain boss (`B`), saut à 14:50 ; puis des rangées
d'icônes (nom au survol) pour faire apparaître chaque ennemi, donner chaque
arme ou bonus (exclusifs compris, ex. le totem), poser chaque objet, et
changer de personnage ou d'arène.

En `npm run dev`, la partie est accessible dans la console du navigateur via
`window.blockSurvivors`.

---

Les textures, sons, police et noms Minecraft appartiennent à Mojang ; projet de fan non commercial.
