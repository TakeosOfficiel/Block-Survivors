/**
 * Configuration technique : moteur, caméra, contrôles, textures, debug.
 * Le contenu du jeu (blocs, modèles, arènes, personnages…) vit dans src/donnees/.
 */

/** Version de Minecraft dont proviennent les textures (branche vérifiée du dépôt d'assets). */
export const VERSION_MINECRAFT = '1.21.4';

/**
 * Sources de textures, essayées dans l'ordre. `chemin` est relatif à
 * assets/minecraft/textures/ et sans extension (ex. « block/dirt »).
 * Si aucune ne répond, le jeu utilise une couleur unie de secours.
 */
export const SOURCES_TEXTURES = [
  {
    id: 'github',
    nom: 'GitHub',
    url: (chemin: string) =>
      `https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/${VERSION_MINECRAFT}/assets/minecraft/textures/${chemin}.png`,
  },
  {
    id: 'mcasset',
    nom: 'mcasset.cloud',
    url: (chemin: string) =>
      `https://assets.mcasset.cloud/${VERSION_MINECRAFT}/assets/minecraft/textures/${chemin}.png`,
  },
  {
    id: 'local',
    nom: 'local',
    url: (chemin: string) => `./assets/mc/${chemin}.png`,
  },
] as const;

/**
 * Autres fichiers vanilla (table de la police), relatifs à assets/minecraft/
 * (ex. « font/include/default.json »). En local, ils rejoignent les textures.
 */
export const SOURCES_FICHIERS = [
  (chemin: string) =>
    `https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/${VERSION_MINECRAFT}/assets/minecraft/${chemin}`,
  (chemin: string) => `https://assets.mcasset.cloud/${VERSION_MINECRAFT}/assets/minecraft/${chemin}`,
  (chemin: string) => `./assets/mc/${chemin}`,
] as const;

/**
 * Sources des sons (absents du .jar) : serveur de ressources de Mojang par
 * empreinte, puis copie locale par nom (./assets/mc/sounds/random/orb.ogg).
 */
export const SOURCES_SONS = [
  (_chemin: string, hachage: string) => `https://resources.download.minecraft.net/${hachage.slice(0, 2)}/${hachage}`,
  (chemin: string) => `./assets/mc/sounds/${chemin}.ogg`,
] as const;

export const CONFIG = {
  temps: {
    dtMax: 0.1,          // deltaTime réel maximal (s) : évite les sauts après un onglet en arrière-plan
    pasSimMax: 1 / 60,   // durée max d'un sous-pas de simulation (s)
    sousPasMax: 40,      // sous-pas max par image (protège le mode ×16 sur les machines lentes)
  },

  camera: {
    inclinaisonDeg: 55,      // plongée sous l'horizontale
    fov: 45,
    hauteurVisee: 1,         // hauteur du point visé au-dessus du sol (blocs)
    distance: 24,            // distance caméra → point visé au démarrage (blocs)
    distanceMin: 10,
    distanceMax: 40,
    sensibiliteZoom: 0.0012, // facteur exponentiel par pixel de molette
    suivi: 10,               // raideur du suivi du joueur (1/s)
    lissageZoom: 12,         // raideur du zoom (1/s)
    // Secousse (explosions, coups, boss) : décalage ∝ intensité², intensité décroissante
    secousse: { amplitude: 0.55, decroissance: 2.2, frequence: 31 },
  },

  rendu: {
    pixelRatioMax: 2,
    // Ombrage façon Minecraft : dessus pleine lumière, faces latérales plus sombres
    ambiante: 0.35,
    soleil: 0.75,
    directionSoleil: [0.3, 1, 0.5],
    contrasteDamier: 0.9,    // en couleurs de secours : un bloc sur deux assombri pour lire la grille
  },

  // Ombre ronde sous les entités, comme en jeu
  ombre: {
    texture: 'misc/shadow',
    opacite: 0.45,
    hauteur: 0.01,           // décollement du sol (évite le z-fighting)
    capacite: 1024,
  },

  // Surcouches de couleur des modèles (rouge de blessure, blanc du creeper qui gonfle)
  surcouche: {
    blessure: { couleur: [1, 0, 0], force: 0.35, duree: 0.3 },
    flash: { couleur: [1, 1, 1], force: 0.7 },
  },

  ennemis: {
    capacite: 600,           // ennemis simultanés au maximum (pool)
    celluleGrille: 2,        // taille d'une cellule de la grille spatiale (blocs)
    separation: 6,           // force de répulsion entre ennemis (1/s)
    amortissementRecul: 6,   // décroissance du recul (1/s)
    vitesseRotation: 10,     // orientation vers la direction de marche (1/s)
    dureeMort: 0.6,          // chute sur le côté avant disparition (s)
    essaisApparition: 16,    // tirages pour trouver un point hors champ
    // Gonflement du creeper (valeurs du jeu)
    gonflementLargeur: 0.4,
    gonflementHauteur: 0.1,
  },

  orbes: {
    capacite: 1200,
    texture: 'entity/experience_orb',
    taille: 0.4,             // côté de l'orbe la plus petite (blocs)
    hauteur: 0.35,
    attraction: 30,          // accélération vers le joueur (blocs/s²)
    vitesseMax: 14,
    distanceCollecte: 0.6,
  },

  projectiles: {
    capacite: 256,
    intervalleTrainee: 0.05,   // s entre deux particules de traînée
  },

  armes: {
    distanceRetour: 0.7,       // le trident disparaît à cette distance du joueur (blocs)
    dureeBalayage: 0.25,       // durée de l'effet de balayage (s)
    foudre: { hauteur: 18, segments: 10, largeur: 0.18, ecart: 0.7, duree: 0.25, capacite: 24 },
    golems: { capacite: 32, distanceSuivi: 3 },   // alliés invoqués (golems, loups, vex)
  },

  // Chiffres des dégâts au-dessus des ennemis (option)
  chiffresDegats: {
    capacite: 160,          // chiffres affichés à la fois (le plus ancien laisse sa place)
    chiffresMax: 6,
    derive: 0.8,            // dérive latérale max (blocs/s)
    ecart: 0.9,             // dispersion du point d'apparition (blocs)
    dureePop: 0.15,         // s du grossissement initial
    fondu: 0.35,            // dernière part de la vie en fondu
    normal: { couleur: 0xffffff, taille: 0.38, montee: 1.1, pop: 0.3, duree: 0.8 },
    critique: { couleur: 0xffc22a, taille: 0.62, montee: 1.4, pop: 0.8, duree: 1.1 },
    // Dégâts des états : plus petits, aux couleurs de la brûlure et du poison
    feu: { couleur: 0xff8a2a, taille: 0.3, montee: 0.8, pop: 0, duree: 0.7 },
    poison: { couleur: 0x8fd45a, taille: 0.3, montee: 0.8, pop: 0, duree: 0.7 },
  },

  particules: {
    capacite: 1024,          // sprites vanilla simultanés (gerbes : donnees/particules.ts)
  },

  // Shaders (option) : ombres du soleil, feuillage et plantes qui ondulent au vent
  shaders: {
    tailleOmbres: { legers: 2048, complets: 4096 },  // résolution de la carte d'ombres
    biaisOmbres: -0.0005,
    biaisNormalOmbres: 0.03,
    flouOmbres: { legers: 2, complets: 4 },          // douceur du bord des ombres
    margeOmbres: 6,              // blocs couverts autour de l'arène par la carte d'ombres
    // Vent du feuillage et des plantes (blocs marqués « ondule »)
    vent: { direction: [0.93, 0.37], vitesse: 1.8, plantes: 0.1, feuillage: 0.035 },
  },

  textures: {
    delaiMaxMs: 8000,        // abandon d'une source qui ne répond pas
  },

  chargement: {
    textureFond: 'block/dirt',
  },

  // Contrôles par touche PHYSIQUE (event.code) : la même touche fonctionne
  // en AZERTY (Z Q S D) et en QWERTY (W A S D).
  controles: {
    haut:   ['KeyW', 'ArrowUp'],
    bas:    ['KeyS', 'ArrowDown'],
    gauche: ['KeyA', 'ArrowLeft'],
    droite: ['KeyD', 'ArrowRight'],
  },

  hud: {
    periodeFps: 0.5,         // fenêtre de mesure des FPS (s)
    dureeMessage: 3,         // durée d'affichage d'un message central (s)
    iconeTues: 'item/bone',  // compteur d'ennemis vaincus
    dernierCompteARebours: 60,  // le chrono passe en or dans la dernière minute (s)
    delaiFin: 1.6,           // s entre la fin de partie et l'écran de bilan
    // Voile rouge : après un coup, et pulsant quand la santé est basse
    vignette: { blessure: 0.85, santeBasse: 0.5, seuilSanteBasse: 0.3, pulsation: 5 },
    // Cœurs qui tremblent quand il reste peu de PV (comme en jeu, à 4 PV ou moins)
    tremblement: { seuilPv: 4, periode: 0.05, amplitude: 3 },
    barreXp: {
      fond: 'gui/sprites/hud/experience_bar_background',
      progression: 'gui/sprites/hud/experience_bar_progress',
    },
    coeurs: {
      plein: 'gui/sprites/hud/heart/full',
      conteneur: 'gui/sprites/hud/heart/container',
    },
  },

  joueur: {
    dureeMort: 0.6,          // chute sur le côté (s)
    // Totem d'immortalité : résurrection à la mort
    // Pseudo affiché au-dessus du joueur
    etiquette: { hauteurTexte: 0.42, marge: 0.35 },
    totem: {
      pv: 0.5,               // part des PV maximum rendue
      protection: 3,         // s d'invincibilité après la résurrection
      rayonRecul: 6,         // les ennemis proches sont repoussés…
      recul: 14,             // …avec cette force
      icone: 'item/totem_of_undying',
    },
  },

  debug: {
    actifAuDemarrage: false,
    touchesOverlay: ['F3', 'Backquote'],   // Backquote = touche ² en AZERTY
    periodeRafraichissement: 0.25,         // rafraîchissement de l'overlay (s)
    vitessesTemps: [0.25, 0.5, 1, 2, 4, 8, 16],
    distanceApparition: 7,                 // apparition manuelle autour du joueur (blocs)
    frappe: { rayon: 4, degats: 8, recul: 8 },
    // États infligés aux ennemis proches (rayon en blocs, durées en s)
    etats: {
      rayon: 6,
      brulure: { cumuls: 3, degatsParSeconde: 3, duree: 3 },
      poison: { cumuls: 5, degatsParSeconde: 2, duree: 4 },
      ralenti: { part: 0.4, duree: 3 },
      enracine: { duree: 2 },
      etourdi: { duree: 2 },
      marque: { bonus: 0.25, duree: 4 },
    },
  },
} as const;

export type ActionDeplacement = keyof typeof CONFIG.controles;
export type IdSourceTexture = (typeof SOURCES_TEXTURES)[number]['id'];
