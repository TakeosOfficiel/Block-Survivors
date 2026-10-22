/**
 * Calendriers des boss et des mini-boss, communs aux arènes. Les uns et les
 * autres sont des entrées de la table ENNEMIS (champs `boss` et `miniBoss`) ;
 * chaque arène choisit les siens (arenes.ts).
 */
import type { DefinitionCalendrierBoss } from './types';

export const CALENDRIER_BOSS: DefinitionCalendrierBoss = {
  premierPassage: 300,   // 5:00
  intervalle: 300,       // puis toutes les 5 minutes
  avertissement: 5,
  pvSelonTemps: false,
};

/** Un mini-boss toutes les 75 s dès 1:30 ; ses PV suivent la montée en difficulté. */
export const CALENDRIER_MINIBOSS: DefinitionCalendrierBoss = {
  premierPassage: 90,
  intervalle: 75,
  avertissement: 3,
  pvSelonTemps: true,
};
