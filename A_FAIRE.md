# Reste à faire — maps Minecraft et suites

État au 20/10/2026, version 0.16.0 (branche `claude/trusting-meitner-tgaseg`).
Les cases cochées sont faites ; le reste est classé par priorité. Mention « (Rust) » :
fait dans la version Rust (branche `rust`, état au 04/11/2026).

## Déjà fait (0.16.0)

- [x] Import des `.schem` (WorldEdit / FAWE, formats Sponge 1 à 3) et des `.nbt` (bloc de structure)
- [x] Tous les blocs vanilla avec leur vraie forme, lus dans les fichiers officiels (`blockstates/`, `models/`)
- [x] Atlas de textures, faces cachées, liquides, teintes de biome (plaines)
- [x] Zone jouable trouvée seule (panneau `depart` ou centre), décor autour
- [x] Arènes rectangulaires ; réglages par `.json` (nom, icône, ennemis, boss, ambiance)
- [x] Flèches arrêtées par tous les murs, dans les deux sens ; archers qui ne tirent plus à l'aveugle
- [x] Textures Minecraft 26.3 (284 textures vérifiées)
- [x] Première map : Jardin des champignons (`cartes/map_mushroom.schem`)

## À faire tout de suite

- [ ] **Tester sur un vrai PC** (pas pu le faire sans carte graphique) :
  - [ ] une partie complète sur le Jardin des champignons
  - [ ] la caméra au pied des champignons géants et des collines (est-ce qu'ils cachent le joueur ?)
  - [ ] les performances (FPS) avec tout le décor, ombres activées et désactivées
  - [ ] l'exe Windows : la map se charge bien
- [ ] **Fusionner dans `main`** (pull request), pour que l'autre session reparte de la 0.16.0
- [x] Workflow de `claude/zen-turing-g3zghh` (publication de l'exe sur le dépôt public `Block-Survivors-version`) fusionné dans la branche
- [ ] **Coordonner les numéros de version** avec l'autre session : la prochaine est la 0.17.0

## Tirs et murs

- [ ] (Rust : fait ; un mur plein protège des coups portés au sol — mêlée, novas, explosions, arcs de foudre, zones des talents — mais pas de ce qui tombe du ciel : météores, pluies, impact de la foudre, lame colossale) Vérifier que les armes ajoutées par la refonte des classes respectent les murs : seuls les projectiles du joueur et des ennemis les testent aujourd'hui. À contrôler : `ricochets`, `tornades`, `tourbillons`, `orbesArcaniques`, `orbites`, `pluies`, `meteores`, `novas`, `foudre`, `balayages`, tirs des invocations (esprit de blaze)
- [ ] (Rust : oui, sauf ce qui tombe du ciel) Explosions et zones d'effet : décider si un mur protège (aujourd'hui non)

## Maps : contenu et réglages

- [ ] **Panneaux de configuration** posés dans la map : `[zone] Village`, `[ennemis] zombie squelette` (Rust : fait, voir cartes/LISEZMOI.md), `[boss]` (Rust : fait, lieu d'arrivée et liste des boss, plus `[mini-boss]`)
- [ ] Zones d'ennemis par défaut pour une map importée (aujourd'hui : une seule population pour toute l'arène)
- [x] (Rust) Rapport d'import visible en jeu : liste des blocs inconnus (aujourd'hui seulement dans la console)
- [x] (Rust) Format `.litematic` (Litematica)
- [ ] Monde entier (dossier de sauvegarde, régions `.mca`) avec choix de la zone (Rust : fait en glissant le dossier du monde, zone de 96 × 96 autour du panneau `depart` ; pas de choix de la taille)
- [x] (Rust) Bedrock (`.mcstructure`) : blocs courants traduits en blocs Java
- [x] (Rust) Importer une map depuis le jeu (glisser le fichier sur la fenêtre), sans passer par GitHub

## Maps : affichage

- [x] Coffres (simples, piégés, de l'Ender, de cuivre ; doubles, réunis aussi pour Bedrock) et lits : vrais modèles (jeu Rust)
- [x] Panneaux (modèles vanilla) et têtes (squelette, wither, zombie, creeper, joueur) : vrais modèles (jeu Rust)
- [x] Bannières (Rust : tissu en laine de leur couleur, sans motifs)
- [ ] Têtes de dragon et de piglin, motifs des bannières (aujourd'hui rien)
- [x] (Rust) Textures animées dans l'atlas (eau, lave, magma, portail) : aujourd'hui figées sur la première image
- [x] (Rust) Eau qui coule (hauteurs 1 à 7) au lieu d'une eau pleine (surface plate par bloc, sans pente)
- [ ] Teinte selon le biome de la map (aujourd'hui toujours « plaines ») (Rust : fait, biome le plus présent du .schem ou champ `biome` du .json ; pas encore biome par colonne)
- [x] (Rust) Plantes et feuillage qui ondulent au vent (shaders), comme dans les arènes du jeu
- [x] (Rust) Blocs lumineux (torches, lanternes, glowstone, lave) qui éclairent vraiment : indispensable pour les caves
- [x] (Rust) Ombres du soleil sur tout le décor (la carte d'ombres ne couvre que l'arène et 6 blocs autour)
- [x] (Rust) Textures gardées en cache pour jouer hors ligne (aujourd'hui téléchargées à chaque lancement)

## Relief et caves (gros chantier)

- [ ] Plusieurs niveaux de sol : monter d'un bloc tout seul, escaliers et dalles comme rampes, sauter d'une falaise
- [ ] Chemin des mobs en 3D (graphe de sols au lieu de la grille plate)
- [ ] Hauteur pour les entités, projectiles et zones d'effet (aujourd'hui tout est à plat, hauteurs écrites en dur)
- [ ] Coupe automatique des plafonds au-dessus du joueur (caves, maisons) et murs transparents entre la caméra et le joueur
- [ ] Apparition des mobs à une distance de marche (pas à vol d'oiseau) (Rust : fait, un point qui oblige à un long détour à pied n'est pris qu'en dernier recours)
- [ ] Largeur de passage par case : boss trop gros pour un tunnel de 2 blocs (Rust : fait, les gros ennemis prennent un passage où tient tout leur corps ; s'il n'y en a aucun, ils se faufilent quand même)
- [ ] Échelles et lianes

## Équilibrage

- [ ] Durée d'une partie : passer de 15 à 20 minutes (montée de difficulté `dureeMontee`, boss, fin de partie)
- [x] (Rust) Nombre de mobs selon la surface jouable de la map (petite map = moins de mobs à la fois)

## Divers

- [ ] Commentaire de `src/donnees/sons.ts` : les empreintes des sons viennent de l'index 1.21.4 (elles marchent toujours, à mettre à jour si un son change)
