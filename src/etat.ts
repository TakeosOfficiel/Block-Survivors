/**
 * État d'une partie : source de vérité de la simulation.
 * Aucun objet three.js ni élément DOM ici.
 */
import type { DefinitionArene } from './donnees/types';

export interface EtatJoueur {
  x: number;      // position au sol (blocs)
  z: number;
  vx: number;     // vitesse (blocs/s)
  vz: number;
  angle: number;  // orientation (rad), 0 = face vers +Z
}

export interface EtatPartie {
  readonly arene: DefinitionArene;
  tempsJeu: number;  // secondes de jeu écoulées (suivent la vitesse du temps)
  readonly joueur: EtatJoueur;
}

export function creerEtatPartie(arene: DefinitionArene): EtatPartie {
  return {
    arene,
    tempsJeu: 0,
    joueur: { x: 0, z: 0, vx: 0, vz: 0, angle: 0 },
  };
}
