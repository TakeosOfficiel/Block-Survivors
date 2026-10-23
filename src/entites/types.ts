/** Contrats communs aux entités. */
import type { EtatJoueur } from '../etat';
import type { ChampDeFlux } from '../monde/navigation';
import type { ContexteComportement } from './comportements';
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
  /** Applique des dégâts (ignorés pendant l'invulnérabilité) ; renvoie vrai si le coup porte. `boss` : coup d'un boss. */
  subirDegats(degats: number, boss?: boolean): boolean;
}

/** Talents branchés sur le joueur (combat/talents) : vitesse, coups reçus, mort évitée. */
export interface CrochetsJoueur {
  vitesse(): number;
  degatsRecus(degats: number, boss: boolean): number;
  armure(): number;
  /** `evites` : dégâts retirés par l'armure et absorbés par le bouclier. */
  apresCoupRecu(subis: number, evites: number): void;
  eviterMort(): boolean;
}

/** Ce qui permet aux ennemis de tirer. */
export interface Tireur {
  tirer(type: IdProjectile, x: number, z: number, dirX: number, dirZ: number, degats: number, boss?: boolean): void;
}

/**
 * Multijoueur (src/reseau, hôte) : les joueurs de l'équipe. Les ennemis, leurs
 * projectiles, les orbes et les objets s'en servent à la place du seul joueur ;
 * absente en solo.
 */
export interface EquipeJoueurs {
  /** Index du joueur vivant le plus proche de (x, z), −1 si tous sont morts. */
  plusProche(x: number, z: number): number;
  joueur(k: number): EtatJoueur;
  /** Contexte des ennemis qui visent ce joueur (ses dégâts, les tirs). */
  contexte(k: number): ContexteComportement;
  /** Champ de flux vers ce joueur. */
  flux(k: number): ChampDeFlux;
  readonly nombre: number;
}
