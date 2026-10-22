/** Objets ramassables (soins, émeraudes). Ajouter un objet = ajouter une entrée. */
import type { DefinitionObjet } from './types';

const definitions = {
  pomme: {
    nom: 'Pomme',
    icone: 'item/apple',
    soin: 4,
    chance: 0.02,
    taille: 0.7,
    couleurSecours: 0xff4040,
  },
  pomme_doree: {
    nom: 'Pomme dorée',
    icone: 'item/golden_apple',
    soin: 12,
    chance: 0.004,
    taille: 0.8,
    couleurSecours: 0xffd23a,
  },
  // Monnaie de la partie : un tas de 2 à 6 émeraudes, qui ouvrent les coffres (voir coffres.ts).
  // Environ 450 émeraudes sur une partie complète (≈ 3 300 ennemis tués)
  emeraude: {
    nom: 'Émeraude',
    icone: 'item/emerald',
    soin: 0,
    emeraudes: [2, 6],
    chance: 0.035,
    taille: 0.6,
    couleurSecours: 0x17dd62,
  },
} satisfies Record<string, DefinitionObjet>;

export type IdObjet = keyof typeof definitions;

export const OBJETS: Readonly<Record<IdObjet, DefinitionObjet>> = definitions;

export const IDS_OBJETS = Object.keys(definitions) as IdObjet[];
