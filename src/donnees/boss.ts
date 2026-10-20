/**
 * Calendrier des boss, commun aux arènes. Les boss eux-mêmes sont des entrées
 * de la table ENNEMIS (champ `boss`) ; chaque arène choisit les siens (arenes.ts).
 */
import type { DefinitionCalendrierBoss } from './types';

export const CALENDRIER_BOSS: DefinitionCalendrierBoss = {
  premierPassage: 300,   // 5:00
  intervalle: 300,       // puis toutes les 5 minutes
  avertissement: 5,
};
