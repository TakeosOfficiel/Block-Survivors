/** Contrats communs aux entités. */
import type { IdProjectile } from '../donnees/projectiles';

/** Tout ce qui contribue au compteur d'entités actives (joueur, pools d'ennemis…). */
export interface SourceEntites {
  readonly actifs: number;
}

/** Ce qui peut être blessé par les ennemis. */
export interface CibleJoueur {
  /** Applique des dégâts (ignorés pendant l'invulnérabilité) ; renvoie vrai si le coup porte. */
  subirDegats(degats: number): boolean;
}

/** Ce qui permet aux ennemis de tirer. */
export interface Tireur {
  tirer(type: IdProjectile, x: number, z: number, dirX: number, dirZ: number, degats: number): void;
}
