/**
 * Coup reçu (joueur et invocations) : l'armure retire ses points, au plus la
 * moitié du coup ; un coup fait toujours au moins 1 dégât. Le bouclier du
 * joueur (cœurs dorés) absorbe ensuite ce qui reste avant les PV.
 */
import { PLAFONDS } from '../donnees/plafonds';
import type { EtatJoueur } from '../etat';

/** Ajoute des PV de bouclier, dans la limite de la réserve (part des PV max). */
export function ajouterBouclier(j: EtatJoueur, pv: number): void {
  j.bouclier = Math.min(PLAFONDS.bouclier * j.pvMax, j.bouclier + pv);
}

export function degatsApresDefense(degats: number, armure: number): number {
  return Math.max(1, degats - Math.min(armure, degats * PLAFONDS.armure));
}
