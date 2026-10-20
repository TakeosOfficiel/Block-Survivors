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
    golems: { capacite: 8, distanceSuivi: 3 },
  },

  particules: {
    capacite: 1024,
    gravite: 14,
    mort: { nombre: 10, vitesse: 2.5, taille: 0.18, duree: 0.7 },
    trainee: { nombre: 1, vitesse: 0.6, taille: 0.16, duree: 0.4 },
    explosionArme: { nombre: 28, vitesse: 6, taille: 0.35, duree: 0.8 },
    impactFoudre: { nombre: 14, vitesse: 5, taille: 0.2, duree: 0.5 },
    invocation: { nombre: 20, vitesse: 3, taille: 0.3, duree: 0.8 },
    explosion: { nombre: 70, vitesse: 8, taille: 0.45, duree: 1, couleurs: [0xffffff, 0xb0b0b0, 0xff9a2e] },
    niveau: { nombre: 36, vitesse: 4.5, taille: 0.22, duree: 1, couleurs: [0x80ff20, 0xffff55, 0xffffff] },
  },

  // Styles d'herbe (option) posés sur les blocs d'herbe libres de l'arène
  herbe: {
    vent: {
      amplitude: 0.22,             // pliage par le vent au sommet (blocs)
      vitesseRafales: 1.4,         // vitesse des rafales qui traversent le champ
      direction: [0.93, 0.37],     // unitaire, en (x, z)
      rayonCouche: 1.2,            // les plantes se couchent autour du joueur dans ce rayon (blocs)
      couche: 0.55,                // amplitude du pliage près du joueur (blocs)
    },
    assombrissementSol: 0.4,       // dessus des blocs d'herbe sous un tapis de brins
    // Brins effilés : « réaliste » (tapis dense, pied sombre) et « stylisée » (cartoon)
    brins: {
      realiste: {
        densite: 90, hauteur: [0.28, 0.7], largeur: 0.045,
        couleurPied: 0x0f1f08, couleurMilieu: 0x4a7d22, couleurSeche: 0x9a9a3c, couleurPointe: 0xb8d86a,
        ombrePied: 0.15, refletRafales: 0.35,
      },
      stylisee: {
        densite: 24, hauteur: [0.22, 0.42], largeur: 0.12,
        couleurPied: 0x2f7a2a, couleurMilieu: 0x62c23a, couleurSeche: 0xb7d84e, couleurPointe: 0xe2f59a,
        ombrePied: 0.6, refletRafales: 0.25,
      },
    },
    // Touffes Minecraft (sprite short_grass en croix) qui ondulent, comme les packs de shaders
    touffes: { texture: 'block/short_grass', teinte: 0x91bd59, parBloc: 1.1, taille: [0.55, 0.95] },
    // Herbe en pixels 3D : colonnes d'un pixel qui sortent de la texture du dessus
    pixels: { texture: 'block/grass_block_top', teinte: 0x91bd59, parBloc: 34, hauteur: [2, 7] },
  },

  // Shaders (option) : ombres du soleil, lueurs, occlusion ambiante
  shaders: {
    tailleOmbres: { legers: 2048, complets: 4096 },  // résolution de la carte d'ombres
    biaisOmbres: -0.0005,
    biaisNormalOmbres: 0.03,
    flouOmbres: { legers: 2, complets: 4 },          // douceur du bord des ombres
    margeOmbres: 6,              // blocs couverts autour de l'arène par la carte d'ombres
    intensiteLumineux: 1.25,     // lave, glowstone, magma : assez vifs pour rayonner
    lueur: { force: 0.8, rayon: 0.5, seuil: 0.85 },
    occlusion: { resolution: 0.5, rayon: 0.6 },
  },

  // Lave et eau calculées par la carte graphique quand les shaders sont actifs
  liquides: {
    lave: {
      couleurCoeur: 0xffc040, couleurFusion: 0xd8400c, couleurCroute: 0x1e0604,
      intensite: 1.7,            // le cœur des veines dépasse le blanc pour rayonner
      taillePlaques: 1.2,        // plaques de croûte par bloc (plus grand = plus petites)
      largeurFissures: 0.08,
      couvertureCroute: 0.12,    // −0,3 = presque pas de croûte, 0,3 = presque tout couvert
    },
    eau: {
      couleurProfonde: 0x062a3e, couleurClaire: 0x23798f,
      relief: 0.55,              // force des vagues sur les reflets
      refletCiel: 0.12,          // part du ciel reflétée même vue de haut
      brillance: 90, eclatSoleil: 2.2,
      opacite: 0.86,
    },
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
      moitie: 'gui/sprites/hud/heart/half',
      conteneur: 'gui/sprites/hud/heart/container',
    },
  },

  joueur: {
    dureeMort: 0.6,          // chute sur le côté (s)
    // Totem d'immortalité : résurrection à la mort
    totem: {
      pv: 0.5,               // part des PV maximum rendue
      protection: 3,         // s d'invincibilité après la résurrection
      rayonRecul: 6,         // les ennemis proches sont repoussés…
      recul: 14,             // …avec cette force
      icone: 'item/totem_of_undying',
      particules: { nombre: 60, vitesse: 6, taille: 0.25, duree: 1.4, couleurs: [0xf4e04d, 0x7ed957, 0x3fae3f, 0xfff7a8] },
    },
  },

  debug: {
    actifAuDemarrage: false,
    touchesOverlay: ['F3', 'Backquote'],   // Backquote = touche ² en AZERTY
    periodeRafraichissement: 0.25,         // rafraîchissement de l'overlay (s)
    vitessesTemps: [0.25, 0.5, 1, 2, 4, 8, 16],
    distanceApparition: 7,                 // apparition manuelle autour du joueur (blocs)
    frappe: { rayon: 4, degats: 8, recul: 8 },
  },
} as const;

export type ActionDeplacement = keyof typeof CONFIG.controles;
export type PresetBrins = (typeof CONFIG.herbe.brins)[keyof typeof CONFIG.herbe.brins];
export type IdSourceTexture = (typeof SOURCES_TEXTURES)[number]['id'];
