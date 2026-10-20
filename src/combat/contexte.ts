/** Ce que les systèmes de combat du joueur partagent. */
import type { Particules } from '../effets/particules';
import type { Retours } from '../effets/retours';
import type { Ennemis } from '../entites/ennemis';
import type { EtatPartie } from '../etat';
import type { CarteMonde } from '../monde/carte';

export interface ContexteCombat {
  readonly etat: EtatPartie;
  readonly carte: CarteMonde;
  readonly ennemis: Ennemis;
  readonly particules: Particules;
  readonly retours: Retours;
  /** Joue le geste d'attaque du joueur (coup d'épée). */
  declencherGesteJoueur(): void;
}
