/**
 * Grille spatiale uniforme sur le plan XZ (listes chaînées dans des tableaux
 * typés) pour trouver les voisins sans tout comparer à tout. Aucune allocation.
 */
export class GrilleSpatiale {
  private readonly origine: number;
  private readonly cellule: number;
  private readonly colonnes: number;
  private readonly tetes: Int32Array;
  private readonly suivants: Int32Array;

  /** Couvre le carré [−demiCote, demiCote]² ; les points hors du carré sont rangés au bord. */
  constructor(demiCote: number, cellule: number, capacite: number) {
    this.origine = -demiCote;
    this.cellule = cellule;
    this.colonnes = Math.ceil((demiCote * 2) / cellule) + 1;
    this.tetes = new Int32Array(this.colonnes * this.colonnes);
    this.suivants = new Int32Array(capacite);
    this.vider();
  }

  vider(): void {
    this.tetes.fill(-1);
  }

  inserer(index: number, x: number, z: number): void {
    const c = this.indexCellule(this.colonne(x), this.colonne(z));
    this.suivants[index] = this.tetes[c];
    this.tetes[c] = index;
  }

  /**
   * Écrit dans `sortie` les index rangés dans les cellules touchées par le
   * cercle (candidats à filtrer par distance) ; renvoie leur nombre.
   */
  requeter(x: number, z: number, rayon: number, sortie: Int32Array): number {
    const c0 = this.colonne(x - rayon);
    const c1 = this.colonne(x + rayon);
    const l0 = this.colonne(z - rayon);
    const l1 = this.colonne(z + rayon);
    let n = 0;
    for (let l = l0; l <= l1; l++) {
      for (let c = c0; c <= c1; c++) {
        for (let i = this.tetes[this.indexCellule(c, l)]; i !== -1; i = this.suivants[i]) {
          if (n >= sortie.length) return n;
          sortie[n++] = i;
        }
      }
    }
    return n;
  }

  private colonne(valeur: number): number {
    const c = Math.floor((valeur - this.origine) / this.cellule);
    return c < 0 ? 0 : c >= this.colonnes ? this.colonnes - 1 : c;
  }

  private indexCellule(colonne: number, ligne: number): number {
    return ligne * this.colonnes + colonne;
  }
}
