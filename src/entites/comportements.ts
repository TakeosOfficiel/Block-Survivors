/**
 * Comportements des ennemis, choisis par le champ `comportement` de leur
 * définition. Chaque comportement fixe la direction voulue de l'ennemi et
 * déclenche ses attaques ; le déplacement lui-même est générique (ennemis.ts).
 */
import { JOUEUR_BASE } from '../donnees/joueur';
import type { ComportementArcher, ComportementExplosif, ComportementMelee } from '../donnees/types';
import type { Ennemi } from './ennemi';
import type { CibleJoueur, Tireur } from './types';

export interface ContexteComportement {
  readonly cible: CibleJoueur;
  readonly tireur: Tireur;
}

/** Direction et distance vers le joueur, et chemin à suivre pour l'atteindre. */
export interface Vers {
  /** Direction unitaire en ligne droite vers le joueur (pour viser). */
  dx: number;
  dz: number;
  distance: number;
  /** Aucun bloc n'arrête un tir vers lui. */
  ligneDeTir: boolean;
  /** Direction unitaire à suivre pour aller vers lui en contournant le décor. */
  cheminX: number;
  cheminZ: number;
}

/** Écrit e.dirX / e.dirZ (direction voulue, 0 = immobile). Renvoie vrai si l'ennemi explose. */
export function appliquerComportement(e: Ennemi, contexte: ContexteComportement, dt: number, vers: Vers): boolean {
  const c = e.definition.comportement;
  switch (c.type) {
    case 'melee':
      melee(e, c, contexte, vers);
      return false;
    case 'archer':
      archer(e, c, contexte, vers);
      return false;
    case 'explosif':
      return explosif(e, c, dt, vers);
  }
}

/** Va au contact et frappe à intervalle régulier. */
function melee(e: Ennemi, c: ComportementMelee, contexte: ContexteComportement, vers: Vers): void {
  e.dirX = vers.cheminX;
  e.dirZ = vers.cheminZ;
  const distance = vers.distance;
  const contact = e.definition.rayon + JOUEUR_BASE.rayon + c.portee;
  if (distance <= contact && e.minuterie <= 0) {
    contexte.cible.subirDegats(c.degats);
    e.minuterie = c.cadence;
  }
}

/**
 * Se place à sa distance idéale (avance ou recule) et tire quand le joueur est
 * à portée ; s'il est caché derrière un bloc, le contourne au lieu de tirer.
 */
function archer(e: Ennemi, c: ComportementArcher, contexte: ContexteComportement, vers: Vers): void {
  const { dx, dz, distance, ligneDeTir } = vers;
  if (!ligneDeTir || distance > c.distanceIdeale + 1) {
    e.dirX = vers.cheminX;
    e.dirZ = vers.cheminZ;
  } else if (distance < c.distanceIdeale - 1.5) {
    e.dirX = -dx;
    e.dirZ = -dz;
  } else {
    e.dirX = 0;
    e.dirZ = 0;
  }
  if (ligneDeTir && distance <= c.portee && e.minuterie <= 0) {
    // Salve en éventail autour de la direction du joueur
    const nombre = c.nombre ?? 1;
    const base = Math.atan2(dx, dz);
    for (let k = 0; k < nombre; k++) {
      const a = base + (k - (nombre - 1) / 2) * (c.dispersion ?? 0);
      contexte.tireur.tirer(c.projectile, e.x + dx * 0.5, e.z + dz * 0.5, Math.sin(a), Math.cos(a), c.degats);
    }
    e.minuterie = c.cadence;
  }
}

/**
 * S'approche, puis s'arrête et gonfle. La mèche continue tant que le joueur
 * reste dans la distance d'annulation, et redescend s'il s'éloigne.
 */
function explosif(e: Ennemi, c: ComportementExplosif, dt: number, vers: Vers): boolean {
  const distance = vers.distance;
  const amorce = (e.meche > 0 || distance < c.distanceAmorcage) && distance < c.distanceAnnulation;
  if (amorce) {
    e.meche += dt;
    e.dirX = 0;
    e.dirZ = 0;
  } else {
    e.meche = Math.max(0, e.meche - dt);
    e.dirX = vers.cheminX;
    e.dirZ = vers.cheminZ;
  }
  return e.meche >= c.duree;
}
