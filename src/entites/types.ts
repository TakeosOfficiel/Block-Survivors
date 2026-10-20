/** Contrats communs aux entités. */
import type { Allie } from '../combat/invocations';
import type { IdProjectile } from '../donnees/projectiles';

/** Alliés invoqués que les ennemis peuvent prendre pour cible (voir Invocations). */
export interface CiblesAllies {
  provocateur(x: number, z: number, distanceJoueur: number): Allie | null;
  vivant(a: Allie, id: number): boolean;
  blesser(a: Allie, degats: number): void;
  blesserZone(x: number, z: number, rayon: number, degats: number): void;
}

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
