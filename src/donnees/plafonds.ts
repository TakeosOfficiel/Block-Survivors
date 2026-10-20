/**
 * Plafonds des cumuls et statistiques globales dont profite chaque mécanique
 * d'arme. Valeurs provisoires, à ajuster après les essais.
 */
import type { MecaniqueArme } from './types';

export const PLAFONDS = {
  /** Délai des armes : au mieux −50 % (multiplicateur minimal). */
  delaiMin: 0.5,
  /** Projectiles permanents en plus (Carquois, Flèches spectrales…), par arme. */
  projectilesBonus: 2,
  /** Chance de coup critique. */
  critique: 0.6,
  /** Régénération : part des PV max rendue par seconde, toutes sources confondues. */
  regeneration: 0.02,
  /** Réserve de bouclier du joueur : part de ses PV max. */
  bouclier: 0.25,
  /** Part d'un coup reçu que l'armure peut retirer. */
  armure: 0.5,
  /** Dégâts en plus d'une invocation par allié au-delà de son quota. */
  surplusAllie: 0.1,
} as const;

/**
 * Mécaniques qui profitent des « +1 projectile » permanents. Les invocations
 * ont leur propre bonus (« +1 allié ») ; le balayage frappe une seule fois.
 */
export const PROFITE_PROJECTILES: Readonly<Record<MecaniqueArme['type'], boolean>> = {
  balayage: false,
  tir: true,
  retour: true,
  orbite: false,
  explosif: true,
  foudre: true,
  invocation: false,
  fissure: false,
  chute: false,
  tourbillon: false,
  heurt: false,
  tornade: false,
  nova: false,
  meteore: false,
  orbes: true,
  ricochet: true,
  pluie: false,
  piege: false,
};
