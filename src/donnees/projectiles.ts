/** Projectiles. Ajouter un projectile = ajouter une entrée. */
import type { DefinitionProjectile } from './types';

const definitions = {
  fleche: {
    nom: 'Flèche',
    texture: 'entity/projectiles/arrow',
    tailleTexture: [32, 32],
    uv: [0, 0, 32, 5],      // vue de profil, en haut de la texture
    longueur: 1,
    largeur: 5 / 32,
    vitesse: 14,
    duree: 3,
    rayon: 0.15,
    hauteur: 1.3,
    couleurSecours: 0x9b7a4a,
  },
} satisfies Record<string, DefinitionProjectile>;

export type IdProjectile = keyof typeof definitions;

export const PROJECTILES: Readonly<Record<IdProjectile, DefinitionProjectile>> = definitions;

export const IDS_PROJECTILES = Object.keys(definitions) as IdProjectile[];
