/** Projectiles (ennemis et armes du joueur). Ajouter un projectile = ajouter une entrée. */
import type { DefinitionProjectile } from './types';

const ICONE: readonly [number, number, number, number] = [0, 0, 16, 16];

const definitions = {
  fleche: {
    nom: 'Flèche',
    forme: 'croisee',
    texture: 'entity/projectiles/arrow',
    tailleTexture: [32, 32],
    uv: [0, 0, 32, 5],      // vue de profil, en haut de la texture
    longueur: 1,
    largeur: 5 / 32,
    vitesse: 14,
    duree: 3,
    rayon: 0.2,
    hauteur: 1.3,
    couleurSecours: 0x9b7a4a,
  },
  trident: {
    nom: 'Trident',
    forme: 'plat',
    texture: 'item/trident',
    tailleTexture: [16, 16],
    uv: ICONE,
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
    forme: 'plat',
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
    trainee: [0xffd23a, 0xff8a1e, 0x5a5a5a],
    couleurSecours: 0xff7a1a,
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
    trainee: [0xffd23a, 0xff8a1e],
    couleurSecours: 0xff7a1a,
  },
} satisfies Record<string, DefinitionProjectile>;

export type IdProjectile = keyof typeof definitions;

export const PROJECTILES: Readonly<Record<IdProjectile, DefinitionProjectile>> = definitions;

export const IDS_PROJECTILES = Object.keys(definitions) as IdProjectile[];
