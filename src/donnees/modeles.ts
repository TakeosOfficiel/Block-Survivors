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
import type { CubeModele, DefinitionModele, MembreMarche, MembreRepos } from './types';

const DEMI_PI = Math.PI / 2;

// --------------------------------------------------------------------------
// Briques communes aux humanoïdes
// --------------------------------------------------------------------------
const TETE: CubeModele = { taille: [8, 8, 8], pivot: [0, 24, 0], centre: [0, 4, 0], uv: [0, 0] };
const CORPS: CubeModele = { taille: [8, 12, 4], pivot: [0, 24, 0], centre: [0, -6, 0], uv: [16, 16] };

// Valeurs de Minecraft : 0,6662 rad par pas d'animation, bras ×1, jambes ×1,4
const MARCHE_BRAS: readonly MembreMarche[] = [
  { partie: 'brasDroit', axe: 'x', amplitude: 1.0, dephasage: Math.PI },
  { partie: 'brasGauche', axe: 'x', amplitude: 1.0, dephasage: 0 },
];
const MARCHE_JAMBES: readonly MembreMarche[] = [
  { partie: 'jambeDroite', axe: 'x', amplitude: 1.4, dephasage: 0 },
  { partie: 'jambeGauche', axe: 'x', amplitude: 1.4, dephasage: Math.PI },
];
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
// Table des modèles
// --------------------------------------------------------------------------
const definitions = {
  // Joueur : 6 boîtes (tête 8x8x8, corps 8x12x4, bras et jambes 4x12x4), skin 64x64
  humanoide: {
    tailleTexture: [64, 64],
    parties: {
      tete: TETE,
      corps: CORPS,
      brasDroit:   { taille: [4, 12, 4], pivot: [-5, 22, 0],   centre: [-1, -4, 0], uv: [40, 16] },
      brasGauche:  { taille: [4, 12, 4], pivot: [5, 22, 0],    centre: [1, -4, 0],  uv: [32, 48] },
      jambeDroite: { taille: [4, 12, 4], pivot: [-1.9, 12, 0], centre: [0, -6, 0],  uv: [0, 16] },
      jambeGauche: { taille: [4, 12, 4], pivot: [1.9, 12, 0],  centre: [0, -6, 0],  uv: [16, 48] },
    },
    marche: { ...ALLURE, membres: [...MARCHE_BRAS, ...MARCHE_JAMBES] },
    repos: REPOS_BRAS,
    // Coup d'épée : le bras droit se lève vers l'avant
    attaque: { duree: 0.3, membres: [{ partie: 'brasDroit', amplitude: 1.8 }] },
  },

  // Zombie : image 64x64 mais seule la moitié haute est remplie (ancien layout) :
  // bras et jambe gauches = côté droit en miroir. Bras tendus vers l'avant.
  zombie: {
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
} satisfies Record<string, DefinitionModele>;

export type IdModele = keyof typeof definitions;

export const MODELES: Readonly<Record<IdModele, DefinitionModele>> = definitions;
