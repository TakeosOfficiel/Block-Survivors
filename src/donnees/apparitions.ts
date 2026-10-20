/**
 * Apparition des ennemis et montée en difficulté.
 * Chaque valeur « Progression » passe linéairement de `debut` à `fin` sur `dureeMontee`.
 */
import type { DefinitionApparitions } from './types';

export const APPARITIONS: DefinitionApparitions = {
  dureeMontee: 900,                          // 15 minutes
  intervalle: { debut: 2, fin: 0.35 },
  tailleGroupe: { debut: 1, fin: 5 },
  maxSimultanes: { debut: 12, fin: 400 },
  multiplicateurPv: { debut: 1, fin: 4 },
  distanceMin: 8,
  dispersionGroupe: 1.5,
  ennemis: [
    { ennemi: 'zombie', depuis: 0, poids: 10 },
    { ennemi: 'araignee', depuis: 30, poids: 4 },
    { ennemi: 'squelette', depuis: 60, poids: 4 },
    { ennemi: 'creeper', depuis: 90, poids: 2 },
  ],
};
