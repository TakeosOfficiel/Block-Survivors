/** Réglages communs à tous les personnages jouables. */
import type { DefinitionJoueur } from './types';

export const JOUEUR_BASE: DefinitionJoueur = {
  invulnerabilite: 0.5,
  rayon: 0.4,
  reactivite: 25,
  vitesseRotation: 14,
  collecte: 2.5,
  rayonRamassage: 0.9,
};
