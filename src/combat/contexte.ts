/** Ce que les systèmes de combat du joueur partagent. */
import type { Particules } from '../effets/particules';
import type { Ennemis } from '../entites/ennemis';
import type { EtatPartie } from '../etat';

export interface ContexteCombat {
  readonly etat: EtatPartie;
  readonly ennemis: Ennemis;
  readonly particules: Particules;
  /** Joue le geste d'attaque du joueur (coup d'épée). */
  declencherGesteJoueur(): void;
}
