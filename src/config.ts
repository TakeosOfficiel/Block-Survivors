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
export type IdSourceTexture = (typeof SOURCES_TEXTURES)[number]['id'];
