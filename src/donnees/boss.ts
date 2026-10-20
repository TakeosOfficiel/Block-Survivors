/**
 * Calendrier des boss. Les boss eux-mêmes sont des entrées de la table ENNEMIS
 * (champ `boss`) : ils réutilisent modèles et comportements existants.
 */
import type { DefinitionCalendrierBoss } from './types';

export const CALENDRIER_BOSS: DefinitionCalendrierBoss = {
  premierPassage: 300,   // 5:00
  intervalle: 300,       // puis toutes les 5 minutes
  avertissement: 5,
  ordre: ['geant', 'seigneur_wither'],
};
