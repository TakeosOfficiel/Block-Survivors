/** Arènes. Ajouter une arène = ajouter une entrée. */
import type { DefinitionArene } from './types';

export const ARENES: readonly DefinitionArene[] = [
  {
    id: 'prairie',
    nom: 'Prairie du village abandonné',
    taille: 60,
    hauteurMur: 2,
    sol: [{ bloc: 'grass_block', poids: 1 }],
    murs: [
      { bloc: 'cobblestone', poids: 3 },
      { bloc: 'mossy_cobblestone', poids: 1 },
    ],
    exterieur: { bloc: 'grass_block', assombrissement: 0.65, taille: 400 },
    ciel: 0x78a7ff,
    brouillard: { debut: 45, fin: 120 },
  },
];
