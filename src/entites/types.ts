/** Contrats communs aux entités. */

/** Tout ce qui contribue au compteur d'entités actives (joueur, pools d'ennemis…). */
export interface SourceEntites {
  readonly actifs: number;
}
