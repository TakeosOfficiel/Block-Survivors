/**
 * Particules vanilla : sprites de textures/particle, toujours face à la caméra,
 * réglés d'après le comportement des particules du jeu (durées en ticks / 20).
 * Les gerbes disent combien en émettre et comment ; les systèmes n'utilisent
 * que les gerbes.
 */
import type { CheminTexture, DefinitionGerbe, DefinitionParticule } from './types';

/** `prefixe_debut` … `prefixe_fin` (les séries du jeu se lisent parfois à rebours). */
function serie(prefixe: string, debut: number, fin: number): CheminTexture[] {
  const pas = debut <= fin ? 1 : -1;
  return Array.from({ length: Math.abs(fin - debut) + 1 }, (_, i) => `${prefixe}${debut + i * pas}` as CheminTexture);
}

const particules = {
  // Grosse boule de l'explosion (creeper, boules de feu) : immobile, lumineuse, grisée au hasard
  explosion: {
    images: serie('particle/explosion_', 0, 15), animation: 'age',
    taille: [1.4, 2.6], duree: [0.3, 0.5], gravite: 0, frottement: 0, gris: [0.4, 1],
  },
  // « poof » : nuage blanc de la mort d'un mob, qui monte un peu et se dissipe
  poof: {
    images: serie('particle/generic_', 7, 0), animation: 'age',
    taille: [0.25, 0.55], duree: [0.45, 1.1], gravite: -0.8, frottement: 0.15, gris: [0.7, 1],
  },
  // Fumée sombre (traînées, impacts)
  fumee: {
    images: serie('particle/generic_', 7, 0), animation: 'age',
    taille: [0.3, 0.6], duree: [0.5, 1], gravite: -1, frottement: 0.2, gris: [0.15, 0.35],
  },
  flamme: {
    images: ['particle/flame'], animation: 'fixe',
    taille: [0.2, 0.3], duree: [0.25, 0.5], gravite: 0, frottement: 0.4, gris: [1, 1], retrecit: true,
  },
  lave: {
    images: ['particle/lava'], animation: 'fixe',
    taille: [0.15, 0.3], duree: [0.6, 1.2], gravite: 12, frottement: 0.9, gris: [1, 1], retrecit: true,
  },
  critique: {
    images: ['particle/critical_hit'], animation: 'fixe',
    taille: [0.18, 0.3], duree: [0.3, 0.6], gravite: 6, frottement: 0.05, gris: [0.6, 0.9], retrecit: true,
  },
  etincelle: {
    images: ['particle/glow'], animation: 'fixe',
    taille: [0.15, 0.3], duree: [0.25, 0.6], gravite: 0, frottement: 0.1, gris: [1, 1], retrecit: true,
  },
  // Paillettes du totem d'immortalité : jaunes ou vertes, retombent lentement
  totem: {
    images: serie('particle/glitter_', 7, 0), animation: 'age',
    taille: [0.2, 0.35], duree: [1.6, 2.6], gravite: 2.5, frottement: 0.25,
    teintes: [0xe6d22e, 0xd9c73a, 0xf0e050, 0xccb820, 0x3c9a1e, 0x4fb028],
  },
  // happy_villager : étoiles vertes (montée de niveau)
  joie: {
    images: ['particle/glint'], animation: 'fixe',
    taille: [0.2, 0.3], duree: [0.8, 1.3], gravite: -0.6, frottement: 0.4, gris: [1, 1],
  },
  // Tourbillons des effets de potion (entity_effect), teintés à la couleur de l'effet
  effetPoison: {
    images: serie('particle/effect_', 7, 0), animation: 'age',
    taille: [0.15, 0.25], duree: [0.6, 1], gravite: -1.2, frottement: 0.3, teintes: [0x87a363, 0x6f8f4a],
  },
  effetLenteur: {
    images: serie('particle/effect_', 7, 0), animation: 'age',
    taille: [0.15, 0.25], duree: [0.6, 1], gravite: -1.2, frottement: 0.3, teintes: [0x8bafe0, 0x6d8fc2],
  },
  // Étoiles magiques au-dessus d'un ennemi étourdi
  etourdi: {
    images: ['particle/enchanted_hit'], animation: 'fixe',
    taille: [0.15, 0.25], duree: [0.3, 0.5], gravite: 0, frottement: 0.5, gris: [1, 1], retrecit: true,
  },
  // Étincelles de feu d'artifice (firework), de toutes les couleurs
  artifice: {
    images: serie('particle/spark_', 7, 0), animation: 'age',
    taille: [0.15, 0.3], duree: [0.5, 1], gravite: 3, frottement: 0.5,
    teintes: [0xff4040, 0x40ff60, 0x4080ff, 0xffe040, 0xff60ff, 0x60ffff],
  },
  // Lueur dorée de la flèche spectrale
  spectrale: {
    images: ['particle/glow'], animation: 'fixe',
    taille: [0.12, 0.2], duree: [0.25, 0.4], gravite: 0, frottement: 0.6, teintes: [0xffe680], retrecit: true,
  },
  // Lueur violette des orbes arcaniques
  arcane: {
    images: ['particle/glow'], animation: 'fixe',
    taille: [0.15, 0.25], duree: [0.3, 0.5], gravite: -0.3, frottement: 0.6, teintes: [0xb36cff, 0xd9a6ff], retrecit: true,
  },
  // Lueur dorée au-dessus d'un ennemi marqué
  marque: {
    images: ['particle/glow'], animation: 'fixe',
    taille: [0.2, 0.3], duree: [0.5, 0.7], gravite: -0.4, frottement: 0.5, teintes: [0xffd84a], retrecit: true,
  },
  // Éclats d'un objet consommé : un quart de son icône pris au hasard
  morceau: {
    images: [], animation: 'morceau',
    taille: [0.12, 0.2], duree: [0.25, 0.7], gravite: 16, frottement: 0.05, gris: [1, 1],
  },
} satisfies Record<string, DefinitionParticule>;

export type IdParticule = keyof typeof particules;

export const PARTICULES: Readonly<Record<IdParticule, DefinitionParticule>> = particules;

export const IDS_PARTICULES = Object.keys(particules) as IdParticule[];

const gerbes = {
  // Mort d'un ennemi
  mort: [{ particule: 'poof', nombre: 12, vitesse: 1.2, dispersion: 0.5, montee: 0.6 }],
  // Creeper : un amas de boules d'explosion et de la fumée
  explosion: [
    { particule: 'explosion', nombre: 14, vitesse: 0, dispersion: 2.6, montee: 0 },
    { particule: 'fumee', nombre: 16, vitesse: 2.5, dispersion: 1.5, montee: 1 },
  ],
  // Boule de feu de l'arme (plus petite)
  explosionArme: [
    { particule: 'explosion', nombre: 5, vitesse: 0, dispersion: 1.5, montee: 0 },
    { particule: 'flamme', nombre: 8, vitesse: 2.5, dispersion: 0.6, montee: 1 },
  ],
  traineeFeu: [
    { particule: 'flamme', nombre: 1, vitesse: 0.3, dispersion: 0.15, montee: 0.2 },
    { particule: 'fumee', nombre: 1, vitesse: 0.3, dispersion: 0.1, montee: 0.5 },
  ],
  impactFoudre: [
    { particule: 'etincelle', nombre: 14, vitesse: 4, dispersion: 0.4, montee: 2 },
    { particule: 'fumee', nombre: 6, vitesse: 1, dispersion: 0.6, montee: 1 },
  ],
  // Arrivée et départ d'un golem
  invocation: [{ particule: 'poof', nombre: 20, vitesse: 1.6, dispersion: 0.9, montee: 0.8 }],
  // Coup critique sur un ennemi
  critique: [{ particule: 'critique', nombre: 5, vitesse: 2.5, dispersion: 0.3, montee: 2 }],
  // Coup d'épée sans texture de balayage
  coup: [{ particule: 'critique', nombre: 8, vitesse: 3, dispersion: 0.6, montee: 1.5 }],
  niveau: [{ particule: 'joie', nombre: 16, vitesse: 1, dispersion: 0.8, montee: 1.5 }],
  totem: [{ particule: 'totem', nombre: 70, vitesse: 5, dispersion: 0.4, montee: 5 }],
  ramassage: [{ particule: 'morceau', nombre: 10, vitesse: 2.2, dispersion: 0.2, montee: 3 }],
  // Éclats de blocs (Onde de choc, Lame colossale) : morceaux de la texture du sol
  eboulis: [
    { particule: 'morceau', nombre: 5, vitesse: 2.5, dispersion: 0.4, montee: 5 },
    { particule: 'fumee', nombre: 2, vitesse: 0.8, dispersion: 0.4, montee: 0.8 },
  ],
  impactLame: [
    { particule: 'explosion', nombre: 3, vitesse: 0, dispersion: 1.2, montee: 0 },
    { particule: 'critique', nombre: 14, vitesse: 5, dispersion: 0.8, montee: 3 },
  ],
  heurt: [
    { particule: 'critique', nombre: 10, vitesse: 4, dispersion: 0.5, montee: 1.5 },
    { particule: 'poof', nombre: 5, vitesse: 2, dispersion: 0.4, montee: 0.4 },
  ],
  // Mage : braises de la tornade, éclats de la nova, impact du météore, orbes
  tornade: [
    { particule: 'flamme', nombre: 2, vitesse: 1.5, dispersion: 0.5, montee: 2.5 },
    { particule: 'fumee', nombre: 1, vitesse: 0.6, dispersion: 0.4, montee: 1.5 },
  ],
  gel: [
    { particule: 'morceau', nombre: 3, vitesse: 3, dispersion: 0.2, montee: 2.5 },
    { particule: 'effetLenteur', nombre: 1, vitesse: 1, dispersion: 0.3, montee: 0.6 },
  ],
  impactMeteore: [
    { particule: 'explosion', nombre: 6, vitesse: 0, dispersion: 1.6, montee: 0 },
    { particule: 'lave', nombre: 10, vitesse: 4, dispersion: 0.6, montee: 6 },
    { particule: 'fumee', nombre: 8, vitesse: 2, dispersion: 1, montee: 1.5 },
  ],
  arcane: [{ particule: 'arcane', nombre: 1, vitesse: 0.2, dispersion: 0.05, montee: 0.2 }],
  impactArcane: [{ particule: 'arcane', nombre: 8, vitesse: 2.5, dispersion: 0.2, montee: 1.5 }],
  // Archère : traînées et explosion d'artifice, flèche spectrale, impacts de la pluie, buissons
  traineeArtifice: [{ particule: 'artifice', nombre: 1, vitesse: 0.4, dispersion: 0.05, montee: 0.2 }],
  artifice: [
    { particule: 'artifice', nombre: 26, vitesse: 6, dispersion: 0.3, montee: 4 },
    { particule: 'explosion', nombre: 2, vitesse: 0, dispersion: 0.6, montee: 0 },
  ],
  petitArtifice: [{ particule: 'artifice', nombre: 10, vitesse: 4, dispersion: 0.2, montee: 3 }],
  spectrale: [{ particule: 'spectrale', nombre: 1, vitesse: 0.1, dispersion: 0.05, montee: 0 }],
  impactFleche: [{ particule: 'critique', nombre: 3, vitesse: 2, dispersion: 0.2, montee: 2 }],
  ronces: [{ particule: 'morceau', nombre: 6, vitesse: 2, dispersion: 0.5, montee: 3 }],
  // Bouclier donné par l'allay : paillettes dorées du totem
  bouclier: [{ particule: 'totem', nombre: 14, vitesse: 1.5, dispersion: 0.4, montee: 1.5 }],
  // États des ennemis (émis régulièrement tant qu'ils durent)
  brulure: [{ particule: 'flamme', nombre: 3, vitesse: 0.4, dispersion: 0.4, montee: 1 }],
  poison: [{ particule: 'effetPoison', nombre: 1, vitesse: 0.3, dispersion: 0.35, montee: 0.5 }],
  lenteur: [{ particule: 'effetLenteur', nombre: 1, vitesse: 0.3, dispersion: 0.35, montee: 0.5 }],
  etourdi: [{ particule: 'etourdi', nombre: 3, vitesse: 1.5, dispersion: 0.2, montee: 0.4 }],
  marque: [{ particule: 'marque', nombre: 1, vitesse: 0, dispersion: 0.15, montee: 0.3 }],
} satisfies Record<string, DefinitionGerbe>;

export type IdGerbe = keyof typeof gerbes;

export const GERBES: Readonly<Record<IdGerbe, DefinitionGerbe>> = gerbes;
