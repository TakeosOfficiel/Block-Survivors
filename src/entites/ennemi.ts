/** Données d'un ennemi actif (objet du pool, réinitialisé à chaque apparition). */
import { IDS_ARMES } from '../donnees/armes';
import { ENNEMIS, IDS_ENNEMIS } from '../donnees/ennemis';
import type { DefinitionEnnemi } from '../donnees/types';
import { creerEtatAnimation, type EtatAnimation } from '../modeles/animation';

export interface Ennemi {
  id: number;                   // identifiant unique de l'apparition (les index du pool changent)
  indexType: number;            // index dans IDS_ENNEMIS
  definition: DefinitionEnnemi;
  x: number;
  z: number;
  /** Recul (blocs/s), amorti avec le temps. */
  vx: number;
  vz: number;
  /** Direction voulue par le comportement (longueur 0 ou 1). */
  dirX: number;
  dirZ: number;
  angle: number;
  vitesseSol: number;           // pour l'animation
  pv: number;
  pvMax: number;
  minuterie: number;            // s avant la prochaine attaque
  blessure: number;             // s de surcouche rouge restantes
  meche: number;                // s de mèche écoulées (creeper)
  mort: number;                 // −1 = vivant, sinon s depuis la mort
  aExploser: boolean;
  /** Par arme (index dans IDS_ARMES) : temps de jeu avant lequel elle ne peut plus le toucher. */
  immunites: Float32Array;
  animation: EtatAnimation;
}

export function creerEnnemi(): Ennemi {
  return {
    id: 0,
    indexType: 0,
    definition: ENNEMIS[IDS_ENNEMIS[0]],
    x: 0, z: 0, vx: 0, vz: 0, dirX: 0, dirZ: 0,
    angle: 0, vitesseSol: 0,
    pv: 1, pvMax: 1,
    minuterie: 0, blessure: 0, meche: 0, mort: -1,
    aExploser: false,
    immunites: new Float32Array(IDS_ARMES.length),
    animation: creerEtatAnimation(),
  };
}
