/**
 * Retours sensoriels demandés par la simulation : sons, secousses de caméra,
 * chiffres des dégâts.
 * Les systèmes ne connaissent que ce contrat ; le jeu le relie au lecteur de
 * sons et à la caméra (en respectant les options du joueur).
 */
import type { IdSon } from '../donnees/sons';
import type { StyleChiffre } from './chiffresDegats';

export interface Retours {
  /** `hauteur` multiplie la hauteur du son (1 = normale). */
  son(id: IdSon, hauteur?: number): void;
  /** Ajoute de l'intensité à la secousse de caméra (0 à 1). */
  secousse(force: number): void;
  /** Dégâts infligés par le joueur, affichés au-dessus de la cible (`y` : hauteur en blocs). */
  degats(x: number, y: number, z: number, valeur: number, style: StyleChiffre): void;
}
