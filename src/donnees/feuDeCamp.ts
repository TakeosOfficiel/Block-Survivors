/**
 * Écran de chargement « La pause des mobs » : le soir, autour d'un feu de
 * camp, le héros, un zombie et un squelette attendent la partie ; un mouton
 * dort, un creeper n'ose pas approcher du feu. La vue tourne lentement autour
 * d'eux. Tout ce qui se règle (décor, places, rythmes, couleurs) est ici ;
 * src/ui/sceneChargement.ts construit et anime la scène.
 *
 * Angles en radians autour du feu (0 = vers +Z), distances en blocs depuis
 * le centre du feu.
 */
import type { CheminTexture, CubeModele, DefinitionModele } from './types';

const DEMI_PI = Math.PI / 2;

/** Textures de la scène (le feu de camp ajoute les siennes, lues dans son modèle de bloc). */
export const TEXTURES_FEU_DE_CAMP = {
  herbeDessus: 'block/grass_block_top',
  herbeCote: 'block/grass_block_side',
  terre: 'block/dirt',
  terreBattue: 'block/coarse_dirt',
  buche: 'block/oak_log',
  bucheDessus: 'block/oak_log_top',
  feuilles: 'block/oak_leaves',
  touffe: 'block/short_grass',
  laineTente: 'block/white_wool',
  zombie: 'entity/zombie/zombie',
  squelette: 'entity/skeleton/skeleton',
  creeper: 'entity/creeper/creeper',
  mouton: 'entity/sheep/sheep',
  laineMouton: 'entity/sheep/sheep_wool',
  baton: 'item/stick',
  cookie: 'item/cookie',
  os: 'item/bone',
  braise: 'particle/lava',
  lune: 'environment/celestial/moon/full_moon',
} as const satisfies Record<string, CheminTexture>;

/** Fleurs et champignons au pied des arbres. */
export const DECO_FEU_DE_CAMP: readonly CheminTexture[] = [
  'block/red_mushroom', 'block/brown_mushroom', 'block/cornflower', 'block/poppy', 'block/oxeye_daisy',
];
/** Grande fumée du feu de camp, du début à la fin de sa vie. */
export const FUMEE_FEU_DE_CAMP: readonly CheminTexture[] = [
  'particle/big_smoke_0', 'particle/big_smoke_3', 'particle/big_smoke_6', 'particle/big_smoke_9', 'particle/big_smoke_11',
];

/** Les étapes du chargement, racontées par les mobs (une par cinquième de la barre). */
export const PHRASES_FEU_DE_CAMP: readonly string[] = [
  'Le zombie fait griller son cookie…',
  'Le squelette compte ses os…',
  'Le creeper reste loin du feu…',
  'On câline le mouton…',
  'Tout le monde en place !',
];

export const FEU_DE_CAMP = {
  /** État du bloc vanilla dont on lit le modèle. */
  bloc: 'campfire[facing=south,lit=true]',
  tailleFeu: 1.25,
  /** Images par seconde des flammes et des braises du feu. */
  vitesseFlammes: 10,
  vitesseBraisesFeu: 1,
  ciel: { haut: '#0f1533', bas: '#4a3d62' },
  brouillard: { couleur: 0x241f3a, debut: 10, fin: 26 },
  lumieres: {
    ciel: 0x8a96d8, sol: 0x3a3024, ambiance: 1.5,
    lune: 0x9fb0ff, intensiteLune: 0.9,
    feu: 0xff9a3c, intensiteFeu: 7, porteeFeu: 16, attenuationFeu: 1.4, vacillement: 0.6,
    lanterne: 0xffb850, intensiteLanterne: 2.6, porteeLanterne: 5,
  },
  teintes: { herbe: 0x91bd59, feuilles: 0x4f8a32, laineTente: 0xefe3c8, fondTente: 0xb9ad92 },
  camera: { champ: 40, rayon: 6.4, hauteur: 3.7, visee: 1.35, vitesse: 0.07, depart: 0.3 },
  clairiere: { rayon: 19, rayonTerre: 3.2, arbres: 22, distanceArbres: [12, 17] as const, touffes: 170, rayonLibre: 4.5 },
  /** Les bûches où l'on s'assoit, et la place de chacun. */
  bancs: { rayon: 2.45, longueur: 1.6, epaisseur: 0.4, assis: 2.3, avanceDebout: 0.4 },
  places: { heros: 0, zombie: Math.PI * 0.62, squelette: -Math.PI * 0.62 },
  /** Le mouton couché contre le héros (position, cap). */
  mouton: { x: -1.21, z: 1.74, cap: 1.265 },
  creeper: { angle: Math.PI + 0.35, distance: 6.2, avance: 1.4, cycle: 7 },
  tente: { angle: 0.95, distance: 5.6, pente: 0.58, cote: 1.75, profondeur: 2.4 },
  pile: { angle: 2.1, distance: 5.2 },
  souche: { angle: -2.25, distance: 5 },
  fumee: { nombre: 10, duree: 5.5, montee: 4.2, couleur: 0xc4c4cc, opacite: 0.9 },
  braises: { nombre: 7, duree: 1.6, taille: 0.13 },
  lucioles: 18,
  etoiles: 300,
  /** Bâton du zombie : il prolonge son bras tendu vers le feu. */
  baton: { taille: 1.1, cookie: 0.42 },
  /** Cadence plafonnée de la scène, et ombres du feu recalculées une image sur `ombresToutesLes`. */
  imagesParSeconde: 30,
  ombresToutesLes: 3,
  /** La fin à 100 % (secondes) : tout le monde se lève, puis l'écran s'efface. */
  finale: { duree: 1.6 },
} as const;

/**
 * Seconde couche des skins de joueur, rattachée à l'articulation de sa partie
 * (elle bouge avec elle).
 */
export const COUCHES_JOUEUR: Readonly<Record<string, string>> = {
  chapeau: 'tete', veste: 'corps', mancheDroite: 'brasDroit', mancheGauche: 'brasGauche', jambiereDroite: 'jambeDroite', jambiereGauche: 'jambeGauche',
};

// Mouton vanilla (skin 64x32) dans le repère des modèles du jeu, et sa laine (couche gonflée)
const PATTES_MOUTON = { patteArriereDroite: [-3, -7], patteArriereGauche: [3, -7], patteAvantDroite: [-3, 5], patteAvantGauche: [3, 5] } as const;

export const MODELE_MOUTON: DefinitionModele = {
  tailleTexture: [64, 32],
  parties: {
    tete: { taille: [6, 6, 8], pivot: [0, 18, 8], centre: [0, 1, 2], uv: [0, 0] },
    corps: { taille: [8, 16, 6], pivot: [0, 19, -2], centre: [0, 2, 4], uv: [28, 8], rotation: [DEMI_PI, 0, 0] },
    ...Object.fromEntries(Object.entries(PATTES_MOUTON).map(([nom, [x, z]]): [string, CubeModele] =>
      [nom, { taille: [4, 12, 4], pivot: [x, 12, z], centre: [0, -6, 0], uv: [0, 16] }])),
  },
};

export const MODELE_LAINE_MOUTON: DefinitionModele = {
  tailleTexture: [64, 32],
  parties: {
    tete: { taille: [6, 6, 6], pivot: [0, 18, 8], centre: [0, 1, 1], uv: [0, 0], gonflement: 0.6 },
    corps: { taille: [8, 16, 6], pivot: [0, 19, -2], centre: [0, 2, 4], uv: [28, 8], rotation: [DEMI_PI, 0, 0], gonflement: 1.75 },
    ...Object.fromEntries(Object.entries(PATTES_MOUTON).map(([nom, [x, z]]): [string, CubeModele] =>
      [nom, { taille: [4, 6, 4], pivot: [x, 12, z], centre: [0, -3, 0], uv: [0, 16], gonflement: 0.5 }])),
  },
};
