/**
 * Ennemis. Ajouter un ennemi = ajouter une entrée (avec un comportement existant).
 * PV et dégâts en demi-cœurs, comme en jeu (20 PV = 10 cœurs).
 */
import type { DefinitionEnnemi } from './types';

const HUMANOIDE_SECOURS = { tete: 0x4f7a3a, corps: 0x2f8f8f, brasDroit: 0x4f7a3a, brasGauche: 0x4f7a3a, jambeDroite: 0x3b3b9c, jambeGauche: 0x3b3b9c };
const SQUELETTE_SECOURS = { tete: 0xc8c8c8, corps: 0xb0b0b0, brasDroit: 0xc8c8c8, brasGauche: 0xc8c8c8, jambeDroite: 0xc8c8c8, jambeGauche: 0xc8c8c8 };
const CREEPER_SECOURS = { tete: 0x5fb04a, corps: 0x4f9a3e, patteArriereDroite: 0x3f8a2e, patteArriereGauche: 0x3f8a2e, patteAvantDroite: 0x3f8a2e, patteAvantGauche: 0x3f8a2e };

const definitions = {
  zombie: {
    nom: 'Zombie',
    modele: 'zombie',
    skin: 'entity/zombie/zombie',
    couleursSecours: HUMANOIDE_SECOURS,
    rayonOmbre: 0.5,
    pv: 20,
    vitesse: 2.2,
    rayon: 0.35,
    comportement: { type: 'melee', degats: 3, portee: 0.4, cadence: 1 },
    xp: 1,
    couleurParticules: 0x4f7a3a,
  },
  squelette: {
    nom: 'Squelette',
    modele: 'squelette',
    skin: 'entity/skeleton/skeleton',
    couleursSecours: SQUELETTE_SECOURS,
    rayonOmbre: 0.5,
    pv: 16,
    vitesse: 2.4,
    rayon: 0.35,
    comportement: { type: 'archer', projectile: 'fleche', degats: 3, distanceIdeale: 9, portee: 13, cadence: 2.2 },
    xp: 2,
    couleurParticules: 0xd0d0d0,
  },
  araignee: {
    nom: 'Araignée',
    modele: 'araignee',
    skin: 'entity/spider/spider',
    couleursSecours: {},
    couleurSecoursParDefaut: 0x3a2e28,
    rayonOmbre: 0.9,
    pv: 12,
    vitesse: 4.6,
    rayon: 0.6,
    comportement: { type: 'melee', degats: 2, portee: 0.3, cadence: 0.8 },
    xp: 1,
    couleurParticules: 0x3a2e28,
  },
  creeper: {
    nom: 'Creeper',
    modele: 'creeper',
    skin: 'entity/creeper/creeper',
    couleursSecours: CREEPER_SECOURS,
    rayonOmbre: 0.5,
    pv: 16,
    vitesse: 2.6,
    rayon: 0.35,
    comportement: { type: 'explosif', distanceAmorcage: 2.5, distanceAnnulation: 5, duree: 1.5, rayon: 3.5, degats: 10 },
    xp: 3,
    couleurParticules: 0x5fb04a,
  },
} satisfies Record<string, DefinitionEnnemi>;

export type IdEnnemi = keyof typeof definitions;

export const ENNEMIS: Readonly<Record<IdEnnemi, DefinitionEnnemi>> = definitions;

export const IDS_ENNEMIS = Object.keys(definitions) as IdEnnemi[];
