/**
 * Coup reçu (joueur et invocations) : l'armure retire ses points, au plus la
 * moitié du coup ; un coup fait toujours au moins 1 dégât.
 */
import { PLAFONDS } from '../donnees/plafonds';

export function degatsApresDefense(degats: number, armure: number): number {
  return Math.max(1, degats - Math.min(armure, degats * PLAFONDS.armure));
}
