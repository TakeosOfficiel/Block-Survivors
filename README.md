# Block Survivors

Survivor-like en vue de dessus dans un monde voxel aux textures Minecraft.
TypeScript + three.js, construit avec Vite.

## Lancer

```bash
npm install
npm run dev        # serveur de développement : http://localhost:5173
npm run build      # vérifie les types puis produit dist/index.html (autonome)
npm run preview    # sert le build sur http://localhost:4173
npm run typecheck  # vérification TypeScript seule
```

`dist/index.html` est un fichier unique : il s'ouvre d'un double-clic et se
charge tel quel dans Electron (`win.loadFile('dist/index.html')`).

## Structure

```
index.html              balisage de l'interface (chargement, HUD, debug)
src/
  main.ts               point d'entrée : chargement des textures puis lancement
  jeu.ts                assemble les systèmes et fixe l'ordre de mise à jour
  config.ts             réglages techniques + version et sources des textures
  etat.ts               état de la partie (source de vérité de la simulation)
  donnees/              CONTENU data-driven : blocs, modèles, joueur, arènes, boss
  assets/               chargement des textures et liste des textures requises
  moteur/               rendu, boucle de jeu, caméra, géométries UV
  monde/                construction de l'arène en blocs
  entites/              joueur, ennemis et comportements, projectiles ennemis, directeur d'apparition
  combat/               armes du joueur : arsenal et une classe par mécanique
  modeles/              modèles cubiques : compilation, animation, rendu instancié
  effets/               ombres, particules, rendu des projectiles (instanciés)
  controles/            clavier et molette
  ui/                   HUD, panneau debug, écran de chargement, styles
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

## Debug

`F3` ou `²` : FPS, entités, appels de dessin, textures, vitesse du temps
(`-` / `+`), invincibilité (`I`), apparition de chaque ennemi, ajout ou rang
supérieur de chaque arme, frappe de zone (`K`), apparition d'un boss (`B`).

En `npm run dev`, la partie est accessible dans la console du navigateur via
`window.blockSurvivors`.

---

Les textures et noms Minecraft appartiennent à Mojang ; projet de fan non commercial.
