/** Bonus passifs possédés par le joueur et leur effet sur les modificateurs. */
import { PASSIFS, type IdPassif } from '../donnees/passifs';
import type { Modificateurs } from '../donnees/types';
import { creerModificateurs, cumulerModificateurs } from './modificateurs';

export interface PassifPossede {
  readonly id: IdPassif;
  rang: number;
}

export class Passifs {
  readonly possedes: PassifPossede[] = [];
  /** Incrémenté à chaque changement (pour rafraîchir l'interface). */
  version = 0;
  /** Bonus permanents du personnage, toujours comptés. */
  private readonly bonus: Partial<Modificateurs>;
  /** Bonus de l'équipement porté (bagues, amulette), remplacés à chaque changement d'équipement. */
  private equipement: readonly Partial<Modificateurs>[] = [];

  constructor(bonus: Partial<Modificateurs>) {
    this.bonus = bonus;
  }

  /** Remplace les bonus de l'équipement (pris en compte au prochain `calculer`). */
  definirEquipement(bonus: readonly Partial<Modificateurs>[]): void {
    this.equipement = bonus;
  }

  /** Rang possédé (0 = absent). */
  rang(id: IdPassif): number {
    return this.possedes.find((p) => p.id === id)?.rang ?? 0;
  }

  /** Ajoute le passif au rang 1, ou le monte d'un rang ; faux s'il est déjà au maximum (un consommable s'empile sans limite ici). */
  ameliorer(id: IdPassif): boolean {
    const existant = this.possedes.find((p) => p.id === id);
    if (!existant) {
      this.possedes.push({ id, rang: 1 });
    } else {
      if (!PASSIFS[id].consommable && existant.rang >= PASSIFS[id].niveaux.length) return false;
      existant.rang++;
    }
    this.version++;
    return true;
  }

  /**
   * Recalcule `sortie` depuis zéro (bonus du personnage + équipement + passifs) ;
   * `ajout` permet de simuler un passif monté d'un rang.
   */
  calculer(sortie: Modificateurs, ajout?: IdPassif): Modificateurs {
    Object.assign(sortie, creerModificateurs());
    cumulerModificateurs(sortie, this.bonus);
    for (const bonus of this.equipement) cumulerModificateurs(sortie, bonus);
    const ids = new Set<IdPassif>(this.possedes.map((p) => p.id));
    if (ajout) ids.add(ajout);
    for (const id of ids) {
      const rang = this.rang(id) + (id === ajout ? 1 : 0);
      const niveaux = PASSIFS[id].niveaux;
      for (let r = 0; r < rang; r++) cumulerModificateurs(sortie, niveaux[Math.min(r, niveaux.length - 1)].modif);
    }
    return sortie;
  }
}
