/**
 * États des ennemis : brûlure, poison et saignement (dégâts par seconde, en cumuls),
 * ralentissement, enracinement, étourdissement et marque. Chaque état a sa
 * teinte sur le modèle et ses particules vanilla. Valeurs provisoires.
 */
import type { DefinitionEtat } from './types';

export const ETATS = {
  brulure: { cumulsMax: 3, tic: 0.5, couleur: [1, 0.45, 0.08], force: 0.32, gerbe: 'brulure', periode: 0.3 },
  poison: { cumulsMax: 5, tic: 0.5, couleur: [0.45, 0.72, 0.2], force: 0.3, gerbe: 'poison', periode: 0.45 },
  // Saignée (talent du Gladiateur) : poussière rouge
  saignement: { cumulsMax: 3, tic: 0.5, couleur: [0.75, 0.05, 0.04], force: 0.3, gerbe: 'saignement', periode: 0.4 },
  // Bleu de l'effet Lenteur du jeu
  ralenti: { couleur: [0.55, 0.69, 0.88], force: 0.35, gerbe: 'lenteur', periode: 0.45 },
  etourdi: { gerbe: 'etourdi', periode: 0.35 },
  marque: { gerbe: 'marque', periode: 0.5 },
} as const satisfies Record<string, DefinitionEtat>;

/** Un boss reste une menace : ralenti moitié moins (25 % au plus), jamais enraciné ni étourdi. */
export const RESISTANCE_BOSS = {
  ralenti: 0.5,
  ralentiMax: 0.25,
  /** Un enracinement le ralentit seulement, de cette part. */
  enracinement: 0.25,
} as const;
