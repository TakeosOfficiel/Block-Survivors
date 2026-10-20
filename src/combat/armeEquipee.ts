/** Arme possédée par le joueur : rang, statistiques calculées, minuterie. */
import { ARMES, IDS_ARMES, type IdArme } from '../donnees/armes';
import type { DefinitionArme, StatsArme } from '../donnees/types';

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

export function creerArmeEquipee(id: IdArme): ArmeEquipee {
  const definition = ARMES[id];
  const arme: ArmeEquipee = {
    id,
    index: IDS_ARMES.indexOf(id),
    definition,
    rang: 1,
    stats: { ...definition.stats },
    minuterie: 0,
  };
  return arme;
}

export function rangMax(definition: DefinitionArme): number {
  return 1 + definition.niveaux.length;
}

/** Recalcule les statistiques : base + améliorations des rangs atteints. */
export function recalculerStats(arme: ArmeEquipee): void {
  Object.assign(arme.stats, arme.definition.stats);
  for (let r = 0; r < arme.rang - 1; r++) {
    const modif = arme.definition.niveaux[r].modif;
    for (const cle of Object.keys(modif) as (keyof StatsArme)[]) {
      arme.stats[cle] += modif[cle] ?? 0;
    }
  }
}
