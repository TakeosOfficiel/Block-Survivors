/** Objets ramassables (soins). Ajouter un objet = ajouter une entrée. */
import type { DefinitionObjet } from './types';

const definitions = {
  pomme: {
    nom: 'Pomme',
    icone: 'item/apple',
    soin: 4,
    chance: 0.02,
    taille: 0.7,
    couleurParticules: [0xff4040, 0xffffff],
  },
  pomme_doree: {
    nom: 'Pomme dorée',
    icone: 'item/golden_apple',
    soin: 12,
    chance: 0.004,
    taille: 0.8,
    couleurParticules: [0xffd23a, 0xff4040, 0xffffff],
  },
} satisfies Record<string, DefinitionObjet>;

export type IdObjet = keyof typeof definitions;

export const OBJETS: Readonly<Record<IdObjet, DefinitionObjet>> = definitions;

export const IDS_OBJETS = Object.keys(definitions) as IdObjet[];
