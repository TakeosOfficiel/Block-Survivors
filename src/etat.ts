/**
 * État d'une partie : source de vérité de la simulation.
 * Aucun objet three.js ni élément DOM ici.
 */
import { JOUEUR_BASE } from './donnees/joueur';
import type { DefinitionArene } from './donnees/types';

export interface EtatJoueur {
  x: number;              // position au sol (blocs)
  z: number;
  vx: number;             // vitesse (blocs/s)
  vz: number;
  angle: number;          // orientation (rad), 0 = face vers +Z
  pv: number;
  pvMax: number;
  invulnerabilite: number;  // s restantes sans pouvoir être blessé
  mort: boolean;
  tempsMort: number;      // s écoulées depuis la mort
}

export interface EtatPartie {
  readonly arene: DefinitionArene;
  tempsJeu: number;       // secondes de jeu écoulées (suivent la vitesse du temps)
  ennemisTues: number;
  readonly joueur: EtatJoueur;
}

export function creerEtatPartie(arene: DefinitionArene): EtatPartie {
  return {
    arene,
    tempsJeu: 0,
    ennemisTues: 0,
    joueur: {
      x: 0, z: 0, vx: 0, vz: 0, angle: 0,
      pv: JOUEUR_BASE.pv,
      pvMax: JOUEUR_BASE.pv,
      invulnerabilite: 0,
      mort: false,
      tempsMort: 0,
    },
  };
}
