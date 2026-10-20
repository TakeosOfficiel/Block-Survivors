/** Projectiles (ennemis et armes du joueur). Ajouter un projectile = ajouter une entrée. */
import type { DefinitionProjectile } from './types';

const ICONE: readonly [number, number, number, number] = [0, 0, 16, 16];

const definitions = {
  fleche: {
    nom: 'Flèche',
    forme: 'croisee',
    texture: 'entity/projectiles/arrow',
    tailleTexture: [32, 32],
    // Vue de profil : 16 × 5 pixels en haut de la texture (la seconde moitié de
    // la bande est la même flèche en miroir, que le jeu n'affiche pas)
    uv: [0, 0, 16, 5],
    longueur: 0.9,
    largeur: 0.9 * 5 / 16,
    vitesse: 14,
    duree: 3,
    rayon: 0.2,
    hauteur: 1.3,
    couleurSecours: 0x9b7a4a,
  },
  // Trident lancé : le modèle de l'entité du jeu (manche, garde et trois pointes)
  trident: {
    nom: 'Trident',
    forme: 'modele',
    modele: 'trident',
    texture: 'entity/trident/trident',
    tailleTexture: [32, 32],
    uv: [0, 0, 32, 32],
    longueur: 1.4,
    largeur: 0,
    vitesse: 13,
    duree: 3,
    rayon: 0.5,
    hauteur: 1.1,
    couleurSecours: 0x3fa8a0,
  },
  hache: {
    nom: 'Hache',
    forme: 'relief',
    texture: 'item/iron_axe',
    tailleTexture: [16, 16],
    uv: ICONE,
    longueur: 1,
    largeur: 0,
    vitesse: 0,
    duree: 0,
    rayon: 0.55,
    hauteur: 1,
    rotationPropre: 14,
    couleurSecours: 0xc8c8c8,
  },
  boule_feu: {
    nom: 'Boule de feu',
    forme: 'face_camera',
    texture: 'item/fire_charge',
    tailleTexture: [16, 16],
    uv: ICONE,
    longueur: 0.9,
    largeur: 0,
    vitesse: 10,
    duree: 3,
    rayon: 0.45,
    hauteur: 1,
    trainee: 'traineeFeu',
    couleurSecours: 0xff7a1a,
  },
  // Flèche spectrale (Flèche ricocheuse) : même profil que la flèche, lumineuse
  fleche_spectrale: {
    nom: 'Flèche spectrale',
    forme: 'croisee',
    texture: 'entity/projectiles/arrow_spectral',
    tailleTexture: [32, 32],
    uv: [0, 0, 16, 5],
    longueur: 0.9,
    largeur: 0.9 * 5 / 16,
    vitesse: 22,
    duree: 3,
    rayon: 0.25,
    hauteur: 1.3,
    trainee: 'spectrale',
    couleurSecours: 0xf0d060,
  },
  // Fusée de feu d'artifice (Fusée explosive)
  // Puissance (talent de l'Élite) : flèche spectrale géante qui traverse tout
  fleche_geante: {
    nom: 'Flèche spectrale géante',
    forme: 'croisee',
    texture: 'entity/projectiles/arrow_spectral',
    tailleTexture: [32, 32],
    uv: [0, 0, 16, 5],
    longueur: 2.6,
    largeur: 2.6 * 5 / 16,
    vitesse: 30,
    duree: 2,
    rayon: 0.9,
    hauteur: 1.3,
    trainee: 'spectrale',
    couleurSecours: 0xf0d060,
  },
  fusee: {
    nom: "Fusée d'artifice",
    forme: 'relief',
    pointeEnHaut: true,
    texture: 'item/firework_rocket',
    tailleTexture: [16, 16],
    uv: ICONE,
    longueur: 0.9,
    largeur: 0,
    vitesse: 14,
    duree: 3,
    rayon: 0.4,
    hauteur: 1.1,
    trainee: 'traineeArtifice',
    couleurSecours: 0xd04040,
  },
  // Petite boule de feu tirée par les blazes
  boule_feu_blaze: {
    nom: 'Petite boule de feu',
    forme: 'face_camera',
    texture: 'item/fire_charge',
    tailleTexture: [16, 16],
    uv: ICONE,
    longueur: 0.55,
    largeur: 0,
    vitesse: 9,
    duree: 2.5,
    rayon: 0.3,
    hauteur: 1.2,
    trainee: 'traineeFeu',
    couleurSecours: 0xff7a1a,
  },
} satisfies Record<string, DefinitionProjectile>;

export type IdProjectile = keyof typeof definitions;

export const PROJECTILES: Readonly<Record<IdProjectile, DefinitionProjectile>> = definitions;

export const IDS_PROJECTILES = Object.keys(definitions) as IdProjectile[];
