/** Arme possédée par le joueur : rang, statistiques calculées, minuterie. */
import { ARMES, IDS_ARMES, type IdArme } from '../donnees/armes';
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
 * atteints, puis modificateurs globaux des passifs. Écrit dans `sortie`.
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
  sortie.degats *= modificateurs.degats;
  sortie.cadence *= modificateurs.cadence;
  sortie.zone *= modificateurs.zone;
  sortie.portee *= modificateurs.portee;
  // Le balayage frappe une seule fois : il n'a pas de « nombre »
  if (definition.mecanique.type !== 'balayage') sortie.nombre += modificateurs.projectiles;
  return sortie;
}
