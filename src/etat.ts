/**
 * État d'une partie : source de vérité de la simulation.
 * Aucun objet three.js ni élément DOM ici.
 */
import { JOUEUR_BASE } from './donnees/joueur';
import { PROGRESSION } from './donnees/progression';
import type { DefinitionArene, Modificateurs } from './donnees/types';
import { creerModificateurs } from './progression/modificateurs';

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
  niveau: number;
  xp: number;             // XP accumulée dans le niveau en cours
  xpNiveau: number;       // XP nécessaire pour passer au niveau suivant
}

export interface EtatPartie {
  readonly arene: DefinitionArene;
  tempsJeu: number;       // secondes de jeu écoulées (suivent la vitesse du temps)
  ennemisTues: number;
  /** Montées de niveau dont les cartes n'ont pas encore été choisies. */
  niveauxEnAttente: number;
  /** Bonus cumulés des passifs. */
  readonly modificateurs: Modificateurs;
  readonly joueur: EtatJoueur;
}

export function creerEtatPartie(arene: DefinitionArene): EtatPartie {
  return {
    arene,
    tempsJeu: 0,
    ennemisTues: 0,
    niveauxEnAttente: 0,
    modificateurs: creerModificateurs(),
    joueur: {
      x: 0, z: 0, vx: 0, vz: 0, angle: 0,
      pv: JOUEUR_BASE.pv,
      pvMax: JOUEUR_BASE.pv,
      invulnerabilite: 0,
      mort: false,
      tempsMort: 0,
      niveau: 1,
      xp: 0,
      xpNiveau: PROGRESSION.xpPremierNiveau,
    },
  };
}
