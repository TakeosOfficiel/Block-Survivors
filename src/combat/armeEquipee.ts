/** Arme possédée par le joueur : rang, statistiques calculées, minuterie. */
import { ALLIES } from '../donnees/allies';
import { ARMES, IDS_ARMES, type IdArme } from '../donnees/armes';
import { PLAFONDS, PROFITE_PROJECTILES } from '../donnees/plafonds';
import type { DefinitionArme, Modificateurs, StatsArme } from '../donnees/types';

export type StatsCalculees = { -readonly [K in keyof StatsArme]: number };

export interface ArmeEquipee {
  readonly id: IdArme;
  /** Index dans IDS_ARMES (immunités des ennemis, angles d'orbite). */
  readonly index: number;
  readonly definition: DefinitionArme;
  rang: number;
  readonly stats: StatsCalculees;
  /** s avant le prochain déclenchement. */
  minuterie: number;
}

export function creerArmeEquipee(id: IdArme, modificateurs: Modificateurs): ArmeEquipee {
  const definition = ARMES[id];
  const arme: ArmeEquipee = {
    id,
    index: IDS_ARMES.indexOf(id),
    definition,
    rang: 1,
    stats: { ...definition.stats },
    minuterie: 0,
  };
  calculerStats(definition, 1, modificateurs, arme.stats);
  return arme;
}

export function rangMax(definition: DefinitionArme): number {
  return 1 + definition.niveaux.length;
}

/**
 * Statistiques d'une arme au rang donné : base + améliorations des rangs
 * atteints, puis modificateurs globaux des passifs, dans leurs plafonds.
 * Écrit dans `sortie`.
 */
export function calculerStats(
  definition: DefinitionArme,
  rang: number,
  modificateurs: Modificateurs,
  sortie: StatsCalculees,
): StatsCalculees {
  Object.assign(sortie, definition.stats);
  for (let r = 0; r < rang - 1; r++) {
    const modif = definition.niveaux[r].modif;
    for (const cle of Object.keys(modif) as (keyof StatsArme)[]) {
      sortie[cle] += modif[cle] ?? 0;
    }
  }
  const m = definition.mecanique;
  sortie.degats *= modificateurs.degats;
  sortie.cadence *= Math.max(PLAFONDS.delaiMin, modificateurs.cadence);
  sortie.zone *= modificateurs.zone;
  sortie.portee *= modificateurs.portee;
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
