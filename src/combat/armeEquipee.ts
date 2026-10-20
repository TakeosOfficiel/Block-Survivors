/** Arme possédée par le joueur : rang, statistiques calculées, minuterie. */
import { ALLIES } from '../donnees/allies';
import { ARMES, IDS_ARMES, type IdArme } from '../donnees/armes';
import { PLAFONDS, PROFITE_PROJECTILES } from '../donnees/plafonds';
import type { BonusArmeTalent, DefinitionArme, EveilArme, Modificateurs, StatsArme } from '../donnees/types';

export type StatsCalculees = { -readonly [K in keyof StatsArme]: number };

/** Bonus de talents d'une arme précise, cumulés rang après rang. */
export interface BonusArmeCumule {
  readonly ajout: Partial<Record<keyof StatsArme, number>>;
  readonly part: Partial<Record<keyof StatsArme, number>>;
  /** Alliés invoqués : PV et vitesse en plus (parts). */
  pv: number;
  vitesse: number;
}

/** Regroupe les bonus d'armes des talents, par arme. */
export function cumulerBonusArmes(liste: readonly BonusArmeTalent[]): Partial<Record<IdArme, BonusArmeCumule>> {
  const sortie: Partial<Record<IdArme, BonusArmeCumule>> = {};
  for (const b of liste) {
    const cumul = (sortie[b.arme] ??= { ajout: {}, part: {}, pv: 0, vitesse: 0 });
    for (const cle of Object.keys(b.ajout ?? {}) as (keyof StatsArme)[]) cumul.ajout[cle] = (cumul.ajout[cle] ?? 0) + (b.ajout?.[cle] ?? 0);
    for (const cle of Object.keys(b.part ?? {}) as (keyof StatsArme)[]) cumul.part[cle] = (cumul.part[cle] ?? 0) + (b.part?.[cle] ?? 0);
    cumul.pv += b.pv ?? 0;
    cumul.vitesse += b.vitesse ?? 0;
  }
  return sortie;
}

export interface ArmeEquipee {
  readonly id: IdArme;
  /** Index dans IDS_ARMES (immunités des ennemis, angles d'orbite). */
  readonly index: number;
  readonly definition: DefinitionArme;
  rang: number;
  readonly stats: StatsCalculees;
  /** Bonus des talents propres à cette arme, ou null. */
  readonly talent: BonusArmeCumule | null;
  /** Éveil choisi (index dans `definition.eveils`), −1 avant l'éveil. */
  eveil: number;
  /** s avant le prochain déclenchement. */
  minuterie: number;
}

export function creerArmeEquipee(id: IdArme, modificateurs: Modificateurs, talent: BonusArmeCumule | null = null): ArmeEquipee {
  const definition = ARMES[id];
  const arme: ArmeEquipee = {
    id,
    index: IDS_ARMES.indexOf(id),
    definition,
    rang: 1,
    stats: { ...definition.stats },
    talent,
    eveil: -1,
    minuterie: 0,
  };
  calculerStats(definition, 1, modificateurs, arme.stats, talent);
  return arme;
}

export function rangMax(definition: DefinitionArme): number {
  return 1 + definition.niveaux.length;
}

/** Éveil choisi de l'arme, ou null. */
export function eveilDe(arme: ArmeEquipee): EveilArme | null {
  return arme.eveil >= 0 ? arme.definition.eveils?.[arme.eveil] ?? null : null;
}

/**
 * Statistiques d'une arme au rang donné : base + améliorations des rangs
 * atteints + ajouts des talents de l'arme, puis modificateurs globaux et de
 * sa famille, puis parts en plus des talents de l'arme, dans leurs plafonds.
 * Écrit dans `sortie`.
 */
export function calculerStats(
  definition: DefinitionArme,
  rang: number,
  modificateurs: Modificateurs,
  sortie: StatsCalculees,
  talent: BonusArmeCumule | null = null,
  eveil: EveilArme | null = null,
): StatsCalculees {
  Object.assign(sortie, definition.stats);
  for (let r = 0; r < rang - 1; r++) {
    const modif = definition.niveaux[r].modif;
    for (const cle of Object.keys(modif) as (keyof StatsArme)[]) {
      sortie[cle] += modif[cle] ?? 0;
    }
  }
  if (talent) for (const cle in talent.ajout) sortie[cle as keyof StatsArme] += talent.ajout[cle as keyof StatsArme] ?? 0;
  if (eveil?.ajout) for (const cle in eveil.ajout) sortie[cle as keyof StatsArme] += eveil.ajout[cle as keyof StatsArme] ?? 0;
  const m = definition.mecanique;
  const mod = modificateurs;
  let degats = mod.degats;
  let cadence = mod.cadence;
  let zone = mod.zone;
  switch (definition.famille) {
    case 'melee':
      degats += mod.degatsMelee;
      zone += mod.zoneMelee;
      break;
    case 'projectile':
      degats += mod.degatsProjectiles;
      sortie.vitesse *= 1 + mod.vitesseProjectiles;
      break;
    case 'feu':
      degats += mod.degatsFeu;
      cadence += mod.cadenceFeu;
      break;
    case 'allie':
      degats += mod.degatsAllies;
      sortie.duree += mod.dureeAllies;
      break;
  }
  sortie.degats *= degats;
  sortie.cadence *= Math.max(PLAFONDS.delaiMin, cadence);
  sortie.zone *= zone;
  sortie.portee *= modificateurs.portee;
  sortie.recul *= modificateurs.recul;
  if (talent) for (const cle in talent.part) sortie[cle as keyof StatsArme] *= 1 + (talent.part[cle as keyof StatsArme] ?? 0);
  if (eveil?.part) for (const cle in eveil.part) sortie[cle as keyof StatsArme] *= 1 + (eveil.part[cle as keyof StatsArme] ?? 0);
  if (PROFITE_PROJECTILES[m.type]) sortie.nombre += Math.min(PLAFONDS.projectilesBonus, modificateurs.projectiles);
  if (m.type === 'invocation') {
    // Les alliés en plus qui dépasseraient le quota deviennent des dégâts
    const allie = ALLIES[m.allie];
    if (allie.renforts) sortie.nombre += modificateurs.allies;
    const surplus = Math.max(0, sortie.nombre - allie.quota);
    sortie.nombre -= surplus;
    sortie.degats *= 1 + surplus * PLAFONDS.surplusAllie;
  }
  return sortie;
}
