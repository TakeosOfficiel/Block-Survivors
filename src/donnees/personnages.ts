/**
 * Personnages jouables. Ajouter un personnage = ajouter une entrée.
 * Les cartes exclusives sont des bonus passifs marqués `exclusif` (passifs.ts).
 */
import type { DefinitionPersonnage } from './types';

const HUMAIN_SECOURS = {
  tete: 0xa8805e, corps: 0x2fa8a8, brasDroit: 0xa8805e, brasGauche: 0xa8805e, jambeDroite: 0x3b3b9c, jambeGauche: 0x3b3b9c,
};

const definitions = {
  guerrier: {
    nom: 'Guerrier',
    description: "Robuste et cuirassé, il taille la horde à l'épée.",
    modele: 'humanoide',
    skin: 'entity/player/wide/steve',
    couleursSecours: HUMAIN_SECOURS,
    rayonOmbre: 0.5,
    pv: 24,
    vitesse: 4.8,
    armesDepart: ['epee'],
    bonus: { armure: 1 },
    cartesExclusives: ['cuirasse', 'totem'],
  },
  archer: {
    nom: 'Archère',
    description: 'Rapide et précise, elle tire de loin.',
    modele: 'humanoide_fin',
    skin: 'entity/player/slim/alex',
    couleursSecours: { ...HUMAIN_SECOURS, corps: 0x5f8f3a, jambeDroite: 0x5a4030, jambeGauche: 0x5a4030 },
    rayonOmbre: 0.5,
    pv: 18,
    vitesse: 5.6,
    armesDepart: ['arc'],
    bonus: { portee: 0.15 },
    cartesExclusives: ['fleches_spectrales', 'patte_de_lapin'],
  },
  mage: {
    nom: 'Mage',
    description: 'Fragile, mais ses sorts couvrent de larges zones.',
    modele: 'humanoide',
    skin: 'entity/player/wide/zuri',
    couleursSecours: { ...HUMAIN_SECOURS, corps: 0x7a3fa0 },
    rayonOmbre: 0.5,
    pv: 16,
    vitesse: 5,
    armesDepart: ['boules_feu'],
    bonus: { zone: 0.15, degats: 0.1 },
    cartesExclusives: ['grimoire', 'cristal'],
  },
} satisfies Record<string, DefinitionPersonnage>;

export type IdPersonnage = keyof typeof definitions;

export const PERSONNAGES: Readonly<Record<IdPersonnage, DefinitionPersonnage>> = definitions;

export const IDS_PERSONNAGES = Object.keys(definitions) as IdPersonnage[];

export const PERSONNAGE_PAR_DEFAUT: IdPersonnage = 'guerrier';
