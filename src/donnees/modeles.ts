/**
 * Modèles cubiques : aucun modèle 3D importé, tout est construit en boîtes.
 *
 * Repère : le modèle regarde vers +Z, son côté droit est en −X, ses pieds en y = 0.
 * 1 pixel de texture = 1/16 de bloc. Les valeurs reprennent celles des modèles
 * vanilla, converties dans ce repère.
 *
 * `uv` est l'origine du patron de la boîte ; ses faces s'en déduisent selon le
 * layout vanilla (l = largeur, p = profondeur) :
 *   dessus (u+p, v)   dessous (u+p+l, v)
 *   droite (u, v+p)   avant (u+p, v+p)   gauche (u+p+l, v+p)   arrière (u+2p+l, v+p)
 * Pour un skin 64x64 cela redonne exactement le layout standard
 * (tête : top(8,0) bottom(16,0) right(0,8) front(8,8) left(16,8) back(24,8), etc.).
 */
import type { CubeModele, DefinitionModele, MainModele, MembreMarche, MembreRepos } from './types';

const DEMI_PI = Math.PI / 2;

// --------------------------------------------------------------------------
// Briques communes aux humanoïdes
// --------------------------------------------------------------------------
const TETE: CubeModele = { taille: [8, 8, 8], pivot: [0, 24, 0], centre: [0, 4, 0], uv: [0, 0] };
const CORPS: CubeModele = { taille: [8, 12, 4], pivot: [0, 24, 0], centre: [0, -6, 0], uv: [16, 16] };

/**
 * Main droite : l'objet est tenu au bout du bras droit, pointe vers l'avant.
 * La diagonale du sprite (manche en bas à gauche, bout en haut à droite)
 * suit le prolongement de la main ; son côté haut-gauche (la courbe de l'arc)
 * est tourné vers l'avant du bras, la corde vers le corps, comme en jeu.
 * `x` = milieu du bras (−1 pour les bras de 4 pixels, 0 pour ceux de 2 ou 3).
 */
function mainDroite(x: number): MainModele {
  return { partie: 'brasDroit', position: [x, -10, 6], rotation: [-Math.PI / 4, 0, Math.PI], taille: 12 };
}

// Valeurs de Minecraft : 0,6662 rad par pas d'animation, bras ×1, jambes ×1,4
const MARCHE_BRAS: readonly MembreMarche[] = [
  { partie: 'brasDroit', axe: 'x', amplitude: 1.0, dephasage: Math.PI },
  { partie: 'brasGauche', axe: 'x', amplitude: 1.0, dephasage: 0 },
];
const MARCHE_JAMBES: readonly MembreMarche[] = [
  { partie: 'jambeDroite', axe: 'x', amplitude: 1.4, dephasage: 0 },
  { partie: 'jambeGauche', axe: 'x', amplitude: 1.4, dephasage: Math.PI },
];
/**
 * Seconde couche des skins de joueur (chapeau, veste, manches, jambières) :
 * les mêmes boîtes, un peu gonflées, lues dans l'autre moitié du patron.
 * Les pixels transparents laissent voir la première couche.
 */
const SECONDE_COUCHE: Readonly<Record<string, { nom: string; uv: readonly [number, number]; gonflement: number }>> = {
  tete: { nom: 'chapeau', uv: [32, 0], gonflement: 0.5 },
  corps: { nom: 'veste', uv: [16, 32], gonflement: 0.25 },
  brasDroit: { nom: 'mancheDroite', uv: [40, 32], gonflement: 0.25 },
  brasGauche: { nom: 'mancheGauche', uv: [48, 48], gonflement: 0.25 },
  jambeDroite: { nom: 'jambiereDroite', uv: [0, 32], gonflement: 0.25 },
  jambeGauche: { nom: 'jambiereGauche', uv: [0, 48], gonflement: 0.25 },
};

/** Ajoute aux parties d'un joueur leur seconde couche. */
function avecSecondeCouche(parties: Readonly<Record<string, CubeModele>>): Record<string, CubeModele> {
  const sortie: Record<string, CubeModele> = { ...parties };
  for (const [nom, cube] of Object.entries(parties)) {
    const c = SECONDE_COUCHE[nom];
    if (c) sortie[c.nom] = { ...cube, uv: c.uv, gonflement: c.gonflement };
  }
  return sortie;
}

/** Les membres de la seconde couche suivent ceux de la première (marche, attaque…). */
function avecSecondeCoucheMembres<T extends { partie: string }>(membres: readonly T[]): T[] {
  return membres.flatMap((m) => {
    const c = SECONDE_COUCHE[m.partie];
    return c ? [m, { ...m, partie: c.nom }] : [m];
  });
}

const REPOS_BRAS = {
  amplitude: 0.05,
  frequenceZ: 1.8,
  frequenceX: 1.34,
  membres: [
    { partie: 'brasDroit', signe: 1 },
    { partie: 'brasGauche', signe: -1 },
  ] satisfies MembreRepos[],
};
const ALLURE = { frequence: 2.665, vitesseReference: 5.8, lissage: 8 };

// --------------------------------------------------------------------------
// Araignée : 8 pattes générées (arrière → avant)
// --------------------------------------------------------------------------
const PATTES_ARAIGNEE = [
  { nom: 'Arriere',        z: -2, lacet: -Math.PI / 4, roulis: Math.PI / 4, phase: 0 },
  { nom: 'MilieuArriere',  z: -1, lacet: -0.3927,      roulis: 0.5812,      phase: Math.PI },
  { nom: 'MilieuAvant',    z: 0,  lacet: 0.3927,       roulis: 0.5812,      phase: DEMI_PI },
  { nom: 'Avant',          z: 1,  lacet: Math.PI / 4,  roulis: Math.PI / 4, phase: 3 * DEMI_PI },
] as const;

function partiesAraignee(): Record<string, CubeModele> {
  const parties: Record<string, CubeModele> = {
    tete:  { taille: [8, 8, 8],   pivot: [0, 9, 3],  centre: [0, 0, 4], uv: [32, 4] },
    cou:   { taille: [6, 6, 6],   pivot: [0, 9, 0],  centre: [0, 0, 0], uv: [0, 0] },
    corps: { taille: [10, 8, 12], pivot: [0, 9, -9], centre: [0, 0, 0], uv: [0, 12] },
  };
  for (const p of PATTES_ARAIGNEE) {
    parties[`patteDroite${p.nom}`] = {
      taille: [16, 2, 2], pivot: [-4, 9, p.z], centre: [-7, 0, 0], uv: [18, 0], rotation: [0, p.lacet, p.roulis],
    };
    parties[`patteGauche${p.nom}`] = {
      taille: [16, 2, 2], pivot: [4, 9, p.z], centre: [7, 0, 0], uv: [18, 0], rotation: [0, -p.lacet, -p.roulis],
    };
  }
  return parties;
}

function marcheAraignee(): MembreMarche[] {
  return PATTES_ARAIGNEE.flatMap((p) => [
    // Balayage d'avant en arrière
    { partie: `patteDroite${p.nom}`, axe: 'y', amplitude: 0.4, dephasage: p.phase, frequence: 2 },
    { partie: `patteGauche${p.nom}`, axe: 'y', amplitude: -0.4, dephasage: p.phase, frequence: 2 },
    // Levée de la patte
    { partie: `patteDroite${p.nom}`, axe: 'z', amplitude: -0.4, dephasage: p.phase, absolu: true },
    { partie: `patteGauche${p.nom}`, axe: 'z', amplitude: 0.4, dephasage: p.phase, absolu: true },
  ] satisfies MembreMarche[]);
}

// --------------------------------------------------------------------------
// Blaze : tête et 12 bâtons sur 3 anneaux qui tournent (modèle vanilla)
// --------------------------------------------------------------------------
// Chaque bâton pivote autour de l'axe du blaze : son articulation est au
// centre, la boîte décalée du rayon ; la rotation continue le fait tourner.
const ANNEAUX_BLAZE = [
  { hauteur: 22, rayon: 8, vitesse: -3.1 },
  { hauteur: 17, rayon: 6, vitesse: 1.9 },
  { hauteur: 10, rayon: 4, vitesse: -2.4 },
] as const;

function partiesBlaze(): Record<string, CubeModele> {
  const parties: Record<string, CubeModele> = {
    tete: { taille: [8, 8, 8], pivot: [0, 24, 0], centre: [0, 0, 0], uv: [0, 0] },
  };
  ANNEAUX_BLAZE.forEach((anneau, a) => {
    for (let k = 0; k < 4; k++) {
      parties[`baton${a}${k}`] = {
        taille: [2, 8, 2], pivot: [0, anneau.hauteur, 0], centre: [anneau.rayon, 0, 0], uv: [0, 16],
        rotation: [0, (k * Math.PI) / 2 + a * 0.4, 0],
      };
    }
  });
  return parties;
}

function tournoiementBlaze(): { partie: string; vitesse: number }[] {
  return ANNEAUX_BLAZE.flatMap((anneau, a) =>
    [0, 1, 2, 3].map((k) => ({ partie: `baton${a}${k}`, vitesse: anneau.vitesse })));
}

// --------------------------------------------------------------------------
// Cube de magma : 8 tranches empilées et un noyau (modèle vanilla)
// --------------------------------------------------------------------------
function partiesCubeMagma(): Record<string, CubeModele> {
  const parties: Record<string, CubeModele> = {
    noyau: { taille: [4, 4, 4], pivot: [0, 0, 0], centre: [0, 4, 0], uv: [24, 40] },
  };
  for (let i = 0; i < 8; i++) {
    // Planche 64×64 : tranches 0 à 3 dans la moitié gauche, 4 à 7 dans la droite (9 pixels de haut chacune)
    const uv: readonly [number, number] = i < 4 ? [0, 9 * i] : [32, 9 * (i - 4)];
    parties[`tranche${i}`] = { taille: [8, 1, 8], pivot: [0, 0, 0], centre: [0, 7.5 - i, 0], uv };
  }
  return parties;
}

// --------------------------------------------------------------------------
// Poisson d'argent : 7 segments qui ondulent et 3 écailles (modèle vanilla)
// --------------------------------------------------------------------------
const SEGMENTS_POISSON = [
  { taille: [3, 2, 2], uv: [0, 0], z: 3.5 },
  { taille: [4, 3, 2], uv: [0, 4], z: 1.5 },
  { taille: [6, 4, 3], uv: [0, 9], z: -1 },
  { taille: [3, 3, 3], uv: [0, 16], z: -4 },
  { taille: [2, 2, 3], uv: [0, 22], z: -7 },
  { taille: [2, 1, 2], uv: [11, 0], z: -9.5 },
  { taille: [1, 1, 2], uv: [13, 4], z: -11.5 },
] as const;
const ECAILLES_POISSON = [
  { taille: [10, 8, 3], uv: [20, 0], segment: 2 },
  { taille: [6, 4, 3], uv: [20, 11], segment: 4 },
  { taille: [6, 5, 2], uv: [20, 18], segment: 1 },
] as const;

function partiesPoisson(): Record<string, CubeModele> {
  const parties: Record<string, CubeModele> = {};
  SEGMENTS_POISSON.forEach((seg, i) => {
    parties[`segment${i}`] = { taille: seg.taille, pivot: [0, 0, seg.z], centre: [0, seg.taille[1] / 2, 0], uv: seg.uv };
  });
  ECAILLES_POISSON.forEach((ecaille, i) => {
    const z = SEGMENTS_POISSON[ecaille.segment].z;
    parties[`ecaille${i}`] = { taille: ecaille.taille, pivot: [0, 0, z], centre: [0, ecaille.taille[1] / 2, 0], uv: ecaille.uv };
  });
  return parties;
}

/** Ondulation latérale : de plus en plus ample vers la queue (valeurs du jeu). */
function ondulationPoisson(): MembreMarche[] {
  const segment = (i: number) => ({ axe: 'y', amplitude: Math.PI * 0.05 * (1 + Math.abs(i - 2)), dephasage: i * 0.15 * Math.PI, frequence: 2.5 }) as const;
  return [
    ...SEGMENTS_POISSON.map((_, i) => ({ partie: `segment${i}`, ...segment(i) })),
    ...ECAILLES_POISSON.map((e, i) => ({ partie: `ecaille${i}`, ...segment(e.segment) })),
  ];
}

// --------------------------------------------------------------------------
// Table des modèles
// --------------------------------------------------------------------------
const definitions = {
  // Joueur : 6 boîtes (tête 8x8x8, corps 8x12x4, bras et jambes 4x12x4), skin 64x64
  humanoide: {
    main: mainDroite(-1),
    tailleTexture: [64, 64],
    parties: avecSecondeCouche({
      tete: TETE,
      corps: CORPS,
      brasDroit:   { taille: [4, 12, 4], pivot: [-5, 22, 0],   centre: [-1, -4, 0], uv: [40, 16] },
      brasGauche:  { taille: [4, 12, 4], pivot: [5, 22, 0],    centre: [1, -4, 0],  uv: [32, 48] },
      jambeDroite: { taille: [4, 12, 4], pivot: [-1.9, 12, 0], centre: [0, -6, 0],  uv: [0, 16] },
      jambeGauche: { taille: [4, 12, 4], pivot: [1.9, 12, 0],  centre: [0, -6, 0],  uv: [16, 48] },
    }),
    marche: { ...ALLURE, membres: avecSecondeCoucheMembres([...MARCHE_BRAS, ...MARCHE_JAMBES]) },
    repos: { ...REPOS_BRAS, membres: avecSecondeCoucheMembres(REPOS_BRAS.membres) },
    // Coup d'épée : le bras droit se lève vers l'avant
    attaque: { duree: 0.3, membres: avecSecondeCoucheMembres([{ partie: 'brasDroit', amplitude: 1.8 }]) },
  },

  // Joueur à bras fins (skins « slim », ex. Alex) : bras de 3 px
  humanoide_fin: {
    main: mainDroite(-0.5),
    tailleTexture: [64, 64],
    parties: avecSecondeCouche({
      tete: TETE,
      corps: CORPS,
      brasDroit:   { taille: [3, 12, 4], pivot: [-5, 21.5, 0], centre: [-0.5, -4, 0], uv: [40, 16] },
      brasGauche:  { taille: [3, 12, 4], pivot: [5, 21.5, 0],  centre: [0.5, -4, 0],  uv: [32, 48] },
      jambeDroite: { taille: [4, 12, 4], pivot: [-1.9, 12, 0], centre: [0, -6, 0],    uv: [0, 16] },
      jambeGauche: { taille: [4, 12, 4], pivot: [1.9, 12, 0],  centre: [0, -6, 0],    uv: [16, 48] },
    }),
    marche: { ...ALLURE, membres: avecSecondeCoucheMembres([...MARCHE_BRAS, ...MARCHE_JAMBES]) },
    repos: { ...REPOS_BRAS, membres: avecSecondeCoucheMembres(REPOS_BRAS.membres) },
    attaque: { duree: 0.3, membres: avecSecondeCoucheMembres([{ partie: 'brasDroit', amplitude: 1.8 }]) },
  },

  // Zombie : image 64x64 mais seule la moitié haute est remplie (ancien layout) :
  // bras et jambe gauches = côté droit en miroir. Bras tendus vers l'avant.
  zombie: {
    main: mainDroite(-1),
    tailleTexture: [64, 64],
    parties: {
      tete: TETE,
      corps: CORPS,
      brasDroit:   { taille: [4, 12, 4], pivot: [-5, 22, 0],   centre: [-1, -4, 0], uv: [40, 16], rotation: [-DEMI_PI, 0, 0] },
      brasGauche:  { taille: [4, 12, 4], pivot: [5, 22, 0],    centre: [1, -4, 0],  uv: [40, 16], rotation: [-DEMI_PI, 0, 0], miroir: true },
      jambeDroite: { taille: [4, 12, 4], pivot: [-1.9, 12, 0], centre: [0, -6, 0],  uv: [0, 16] },
      jambeGauche: { taille: [4, 12, 4], pivot: [1.9, 12, 0],  centre: [0, -6, 0],  uv: [0, 16], miroir: true },
    },
    marche: { ...ALLURE, membres: MARCHE_JAMBES },
    repos: REPOS_BRAS,
  },

  // Squelette : skin 64x32, membres de 2 px ; côté gauche = côté droit en miroir
  squelette: {
    main: mainDroite(0),
    tailleTexture: [64, 32],
    parties: {
      tete: TETE,
      corps: CORPS,
      brasDroit:   { taille: [2, 12, 2], pivot: [-5, 22, 0], centre: [0, -4, 0], uv: [40, 16], rotation: [-DEMI_PI, 0.1, 0] },
      brasGauche:  { taille: [2, 12, 2], pivot: [5, 22, 0],  centre: [0, -4, 0], uv: [40, 16], rotation: [-DEMI_PI, -0.5, 0], miroir: true },
      jambeDroite: { taille: [2, 12, 2], pivot: [-2, 12, 0], centre: [0, -6, 0], uv: [0, 16] },
      jambeGauche: { taille: [2, 12, 2], pivot: [2, 12, 0],  centre: [0, -6, 0], uv: [0, 16], miroir: true },
    },
    marche: { ...ALLURE, membres: MARCHE_JAMBES },
  },

  // Creeper : skin 64x32, tête, corps et 4 pattes
  creeper: {
    tailleTexture: [64, 32],
    parties: {
      tete:              { taille: [8, 8, 8],  pivot: [0, 18, 0],  centre: [0, 4, 0],  uv: [0, 0] },
      corps:             { taille: [8, 12, 4], pivot: [0, 18, 0],  centre: [0, -6, 0], uv: [16, 16] },
      patteArriereDroite:{ taille: [4, 6, 4],  pivot: [-2, 6, -4], centre: [0, -3, 0], uv: [0, 16] },
      patteArriereGauche:{ taille: [4, 6, 4],  pivot: [2, 6, -4],  centre: [0, -3, 0], uv: [0, 16] },
      patteAvantDroite:  { taille: [4, 6, 4],  pivot: [-2, 6, 4],  centre: [0, -3, 0], uv: [0, 16] },
      patteAvantGauche:  { taille: [4, 6, 4],  pivot: [2, 6, 4],   centre: [0, -3, 0], uv: [0, 16] },
    },
    marche: {
      ...ALLURE,
      membres: [
        { partie: 'patteArriereDroite', axe: 'x', amplitude: 1.4, dephasage: 0 },
        { partie: 'patteArriereGauche', axe: 'x', amplitude: 1.4, dephasage: Math.PI },
        { partie: 'patteAvantDroite', axe: 'x', amplitude: 1.4, dephasage: Math.PI },
        { partie: 'patteAvantGauche', axe: 'x', amplitude: 1.4, dephasage: 0 },
      ],
    },
  },

  // Golem de fer : skin 128x128 (modèle vanilla converti)
  golem_fer: {
    tailleTexture: [128, 128],
    parties: {
      tete:        { taille: [8, 10, 8],  pivot: [0, 31, 2],  centre: [0, 7, 1.5],      uv: [0, 0] },
      nez:         { taille: [2, 4, 2],   pivot: [0, 31, 2],  centre: [0, 3, 6.5],      uv: [24, 0] },
      torse:       { taille: [18, 12, 11], pivot: [0, 31, 0], centre: [0, -4, 0.5],     uv: [0, 40] },
      bassin:      { taille: [9, 5, 6],   pivot: [0, 31, 0],  centre: [0, -12.5, 0],    uv: [0, 70], gonflement: 0.5 },
      brasDroit:   { taille: [4, 30, 6],  pivot: [0, 31, 0],  centre: [-11, -12.5, 0],  uv: [60, 21] },
      brasGauche:  { taille: [4, 30, 6],  pivot: [0, 31, 0],  centre: [11, -12.5, 0],   uv: [60, 58] },
      jambeDroite: { taille: [6, 16, 5],  pivot: [-4, 13, 0], centre: [-0.5, -5, 0.5],  uv: [37, 0] },
      jambeGauche: { taille: [6, 16, 5],  pivot: [5, 13, 0],  centre: [-0.5, -5, 0.5],  uv: [60, 0], miroir: true },
    },
    marche: {
      frequence: 1.8,
      vitesseReference: 3.5,
      lissage: 8,
      membres: [
        { partie: 'jambeDroite', axe: 'x', amplitude: 0.8, dephasage: 0 },
        { partie: 'jambeGauche', axe: 'x', amplitude: 0.8, dephasage: Math.PI },
        { partie: 'brasDroit', axe: 'x', amplitude: 0.6, dephasage: Math.PI },
        { partie: 'brasGauche', axe: 'x', amplitude: 0.6, dephasage: 0 },
      ],
    },
    // Coup : les deux bras s'abattent
    attaque: {
      duree: 0.5,
      membres: [
        { partie: 'brasDroit', amplitude: 2 },
        { partie: 'brasGauche', amplitude: 2 },
      ],
    },
  },

  // Araignée : skin 64x32, tête, cou, abdomen et 8 pattes
  araignee: {
    tailleTexture: [64, 32],
    parties: partiesAraignee(),
    marche: { ...ALLURE, membres: marcheAraignee() },
  },

  // Piglin zombifié : tête large avec groin et oreilles, corps de joueur, bras tendus
  piglin: {
    main: mainDroite(-1),
    tailleTexture: [64, 64],
    parties: {
      tete:          { taille: [10, 8, 8], pivot: [0, 24, 0], centre: [0, 4, 0], uv: [0, 0] },
      groin:         { taille: [4, 4, 1], pivot: [0, 24, 0], centre: [0, 2, 4.5], uv: [31, 1] },
      defenseDroite: { taille: [1, 2, 1], pivot: [0, 24, 0], centre: [-2.5, 1, 4.5], uv: [2, 0] },
      defenseGauche: { taille: [1, 2, 1], pivot: [0, 24, 0], centre: [2.5, 1, 4.5], uv: [2, 4] },
      oreilleDroite: { taille: [1, 5, 4], pivot: [-4.5, 30, 0], centre: [-0.5, -2.5, 0], uv: [39, 6], rotation: [0, 0, -Math.PI / 6] },
      oreilleGauche: { taille: [1, 5, 4], pivot: [4.5, 30, 0], centre: [0.5, -2.5, 0], uv: [51, 6], rotation: [0, 0, Math.PI / 6] },
      corps: CORPS,
      brasDroit:   { taille: [4, 12, 4], pivot: [-5, 22, 0],   centre: [-1, -4, 0], uv: [40, 16], rotation: [-DEMI_PI, 0, 0] },
      brasGauche:  { taille: [4, 12, 4], pivot: [5, 22, 0],    centre: [1, -4, 0],  uv: [32, 48], rotation: [-DEMI_PI, 0, 0] },
      jambeDroite: { taille: [4, 12, 4], pivot: [-1.9, 12, 0], centre: [0, -6, 0],  uv: [0, 16] },
      jambeGauche: { taille: [4, 12, 4], pivot: [1.9, 12, 0],  centre: [0, -6, 0],  uv: [16, 48] },
    },
    marche: { ...ALLURE, membres: MARCHE_JAMBES },
    repos: REPOS_BRAS,
  },

  // Blaze : skin 64x32, flotte au-dessus du sol, ses bâtons tournent
  blaze: {
    tailleTexture: [64, 32],
    parties: partiesBlaze(),
    tournoiement: tournoiementBlaze(),
    flottement: { hauteur: 0.35, amplitude: 0.12, frequence: 2.2 },
  },

  // Cube de magma : skin 64x32, avance par bonds
  cube_magma: {
    tailleTexture: [64, 64],
    parties: partiesCubeMagma(),
    // Le visage est dessiné sur l'avant des 8 tranches, le noyau se voit entre elles
    portrait: ['noyau', ...Array.from({ length: 8 }, (_, i) => `tranche${i}`)],
    flottement: { hauteur: 0, amplitude: 0.45, frequence: 4.5, bonds: true },
  },

  // Poisson d'argent : skin 64x32, corps segmenté qui ondule
  poisson_argent: {
    tailleTexture: [64, 32],
    parties: partiesPoisson(),
    // Vu de face : la tête devant les écailles
    portrait: ['ecaille0', 'ecaille2', 'segment1', 'segment0'],
    marche: { frequence: 2.665, vitesseReference: 4, lissage: 8, membres: ondulationPoisson() },
  },

  // Loup apprivoisé : skin 64x32 (modèle vanilla converti), tête avec oreilles et museau,
  // corps et crinière couchés, queue relevée
  loup: {
    tailleTexture: [64, 32],
    parties: {
      tete:            { taille: [6, 6, 4], pivot: [-1, 10.5, 7], centre: [1, 0, 0],     uv: [0, 0] },
      oreilleDroite:   { taille: [2, 2, 1], pivot: [-1, 10.5, 7], centre: [-1, 4, -0.5], uv: [16, 14] },
      oreilleGauche:   { taille: [2, 2, 1], pivot: [-1, 10.5, 7], centre: [3, 4, -0.5],  uv: [16, 14] },
      museau:          { taille: [3, 3, 4], pivot: [-1, 10.5, 7], centre: [1, -1.5, 3],  uv: [0, 10] },
      corps:           { taille: [6, 9, 6], pivot: [0, 10, -2],   centre: [0, -2.5, 0],  uv: [18, 14], rotation: [DEMI_PI, 0, 0] },
      criniere:        { taille: [8, 6, 7], pivot: [-1, 10, 3],   centre: [1, 0, -0.5],  uv: [21, 0],  rotation: [DEMI_PI, 0, 0] },
      jambeArriereDroite: { taille: [2, 8, 2], pivot: [-2.5, 8, -7], centre: [1, -4, 0], uv: [0, 18] },
      jambeArriereGauche: { taille: [2, 8, 2], pivot: [0.5, 8, -7],  centre: [1, -4, 0], uv: [0, 18] },
      jambeAvantDroite:   { taille: [2, 8, 2], pivot: [-2.5, 8, 4],  centre: [1, -4, 0], uv: [0, 18] },
      jambeAvantGauche:   { taille: [2, 8, 2], pivot: [0.5, 8, 4],   centre: [1, -4, 0], uv: [0, 18] },
      queue:           { taille: [2, 8, 2], pivot: [-1, 12, -8],  centre: [1, -4, 0],    uv: [9, 18], rotation: [1.1, 0, 0] },
    },
    marche: {
      ...ALLURE,
      membres: [
        { partie: 'jambeArriereDroite', axe: 'x', amplitude: 1.4, dephasage: 0 },
        { partie: 'jambeArriereGauche', axe: 'x', amplitude: 1.4, dephasage: Math.PI },
        { partie: 'jambeAvantDroite', axe: 'x', amplitude: 1.4, dephasage: Math.PI },
        { partie: 'jambeAvantGauche', axe: 'x', amplitude: 1.4, dephasage: 0 },
        { partie: 'queue', axe: 'z', amplitude: 0.5, dephasage: 0 },
      ],
    },
    // Morsure : la tête plonge vers la cible
    attaque: {
      duree: 0.25,
      membres: [
        { partie: 'tete', amplitude: 0.7 },
        { partie: 'oreilleDroite', amplitude: 0.7 },
        { partie: 'oreilleGauche', amplitude: 0.7 },
        { partie: 'museau', amplitude: 0.7 },
      ],
    },
  },

  // Vex : petit esprit volant de l'évocateur, skin 32x32 (modèle vanilla converti),
  // ailes plates ouvertes vers l'arrière qui battent en vol
  vex: {
    tailleTexture: [32, 32],
    parties: {
      tete:       { taille: [5, 5, 5], pivot: [0, 6.5, 0],      centre: [0, 2.5, 0],   uv: [0, 0] },
      corps:      { taille: [3, 4, 2], pivot: [0, 6.5, 0],      centre: [0, -2, 0],    uv: [0, 10] },
      robe:       { taille: [3, 5, 2], pivot: [0, 6.5, 0],      centre: [0, -3.5, 0],  uv: [0, 16], gonflement: -0.2 },
      brasDroit:  { taille: [2, 4, 2], pivot: [-1.75, 6.25, 0], centre: [-0.25, -1.5, 0], uv: [23, 0], gonflement: -0.1 },
      brasGauche: { taille: [2, 4, 2], pivot: [1.75, 6.25, 0],  centre: [0.25, -1.5, 0],  uv: [23, 6], gonflement: -0.1 },
      aileDroite: { taille: [0, 5, 8], pivot: [-0.5, 5.5, -1],  centre: [0, -2.5, -4],  uv: [16, 14], rotation: [0, 0.6, 0] },
      aileGauche: { taille: [0, 5, 8], pivot: [0.5, 5.5, -1],   centre: [0, -2.5, -4],  uv: [16, 14], rotation: [0, -0.6, 0], miroir: true },
    },
    marche: {
      frequence: 3, vitesseReference: 4, lissage: 6,
      membres: [
        { partie: 'aileDroite', axe: 'y', amplitude: 0.5, dephasage: 0 },
        { partie: 'aileGauche', axe: 'y', amplitude: 0.5, dephasage: Math.PI },
      ],
    },
    attaque: { duree: 0.25, membres: [{ partie: 'brasDroit', amplitude: 1.6 }] },
  },

  // Trident lancé : skin 32x32, géométrie vanilla (bedrock-samples) ; pointes vers +Y
  trident: {
    tailleTexture: [32, 32],
    parties: {
      manche:        { taille: [1, 31, 1], pivot: [0, 0, 0], centre: [0, 12.5, 0], uv: [0, 0], gonflement: 0.01 },
      garde:         { taille: [3, 2, 1],  pivot: [0, 0, 0], centre: [0, 23, 0],   uv: [4, 0] },
      pointeDroite:  { taille: [1, 4, 1],  pivot: [0, 0, 0], centre: [-2, 25, 0],  uv: [4, 3] },
      pointeGauche:  { taille: [1, 4, 1],  pivot: [0, 0, 0], centre: [2, 25, 0],   uv: [4, 3], miroir: true },
    },
  },

  // Allay (Esprit protecteur) : skin 32x32, géométrie vanilla (bedrock-samples) convertie
  allay: {
    tailleTexture: [32, 32],
    parties: {
      tete:       { taille: [5, 5, 5], pivot: [0, 5, 0],       centre: [0, 2.51, 0],   uv: [0, 0] },
      corps:      { taille: [3, 4, 2], pivot: [0, 5, 0],       centre: [0, -2, 0],     uv: [0, 10] },
      robe:       { taille: [3, 5, 2], pivot: [0, 5, 0],       centre: [0, -2.5, 0],   uv: [0, 16], gonflement: -0.2 },
      brasDroit:  { taille: [1, 4, 2], pivot: [-1.75, 4.5, 0], centre: [-0.25, -1.5, 0], uv: [23, 0] },
      brasGauche: { taille: [1, 4, 2], pivot: [1.75, 4.5, 0],  centre: [0.25, -1.5, 0],  uv: [23, 6] },
      aileDroite: { taille: [0, 5, 8], pivot: [-0.5, 4, -1],   centre: [0, -2.5, -4],  uv: [16, 14], rotation: [0, 0.6, 0] },
      aileGauche: { taille: [0, 5, 8], pivot: [0.5, 4, -1],    centre: [0, -2.5, -4],  uv: [16, 14], rotation: [0, -0.6, 0], miroir: true },
    },
    marche: {
      frequence: 3, vitesseReference: 4, lissage: 6,
      membres: [
        { partie: 'aileDroite', axe: 'y', amplitude: 0.5, dephasage: 0 },
        { partie: 'aileGauche', axe: 'y', amplitude: 0.5, dephasage: Math.PI },
      ],
    },
  },

  // Ravageur (Évocation supérieure) : skin 128x128, géométrie vanilla (bedrock-samples) convertie
  ravageur: {
    tailleTexture: [128, 128],
    parties: {
      corps:      { taille: [14, 16, 20], pivot: [0, 19, -2],  centre: [0, -1, -6],     uv: [0, 55],  rotation: [DEMI_PI, 0, 0] },
      ventre:     { taille: [12, 13, 18], pivot: [0, 19, -2],  centre: [0, -15.5, -5], uv: [0, 91],  rotation: [DEMI_PI, 0, 0] },
      cou:        { taille: [10, 10, 18], pivot: [0, 20, 20],  centre: [0, 6, -19],    uv: [68, 73] },
      tete:       { taille: [16, 20, 16], pivot: [0, 28, 10],  centre: [0, -4, 6],     uv: [0, 0] },
      nez:        { taille: [4, 8, 4],    pivot: [0, 28, 10],  centre: [0, -12, 16],   uv: [0, 0] },
      machoire:   { taille: [16, 3, 16],  pivot: [0, 15, 10],  centre: [0, -0.5, 6],   uv: [0, 36] },
      corneDroite: { taille: [2, 14, 4],  pivot: [-5, 27, 19], centre: [-4, 7, -1],    uv: [74, 55], rotation: [-Math.PI / 3, 0, 0] },
      corneGauche: { taille: [2, 14, 4],  pivot: [-5, 27, 19], centre: [14, 7, -1],    uv: [74, 55], rotation: [-Math.PI / 3, 0, 0] },
      jambeArriereDroite: { taille: [8, 37, 8], pivot: [-12, 30, -22], centre: [4, -11.5, 1], uv: [96, 0] },
      jambeArriereGauche: { taille: [8, 37, 8], pivot: [4, 30, -22],   centre: [4, -11.5, 1], uv: [96, 0] },
      jambeAvantDroite:   { taille: [8, 37, 8], pivot: [-4, 26, 4],    centre: [-4, -7.5, 0], uv: [64, 0] },
      jambeAvantGauche:   { taille: [8, 37, 8], pivot: [-4, 26, 4],    centre: [12, -7.5, 0], uv: [64, 0] },
    },
    portrait: ['tete', 'nez'],
    marche: {
      frequence: 1.6, vitesseReference: 7, lissage: 8,
      membres: [
        { partie: 'jambeArriereDroite', axe: 'x', amplitude: 0.5, dephasage: 0 },
        { partie: 'jambeArriereGauche', axe: 'x', amplitude: 0.5, dephasage: Math.PI },
        { partie: 'jambeAvantDroite', axe: 'x', amplitude: 0.5, dephasage: Math.PI },
        { partie: 'jambeAvantGauche', axe: 'x', amplitude: 0.5, dephasage: 0 },
      ],
    },
  },

  // Crocs d'évocateur (Crocs de l'évocateur) : skin 64x32, socle enterré et deux mâchoires qui claquent
  crocs: {
    tailleTexture: [64, 32],
    parties: {
      socle:     { taille: [10, 12, 10], pivot: [0, 0, 0], centre: [0, -4, 0], uv: [0, 0] },
      machoireA: { taille: [4, 14, 8],   pivot: [0, 2, 0], centre: [2, 7, 0],  uv: [40, 0] },
      machoireB: { taille: [4, 14, 8],   pivot: [0, 2, 0], centre: [-2, 7, 0], uv: [40, 0], miroir: true },
    },
  },
} satisfies Record<string, DefinitionModele>;

export type IdModele = keyof typeof definitions;

export const MODELES: Readonly<Record<IdModele, DefinitionModele>> = definitions;
