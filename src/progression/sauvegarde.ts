/**
 * Sauvegarde de la progression entre les parties (stockage local) :
 * émeraudes, rangs des talents, arbre actif de chaque personnage, records
 * qui débloquent personnages et arènes.
 */
import type { IdArme } from '../donnees/armes';
import type { IdPersonnage } from '../donnees/personnages';
import { ARBRES, DEBLOCAGES, RECOMPENSES, SEUILS_PALIERS } from '../donnees/talents';
import type { DefinitionDeblocage, DefinitionTalent, Modificateurs } from '../donnees/types';
import { ecrireStockage, lireStockage } from '../options';
import { cumulerModificateurs } from './modificateurs';

const CLE = 'block-survivors.progression';

interface Donnees {
  emeraudes: number;
  /** Rang acheté, par « personnage.arbre.talent ». */
  talents: Record<string, number>;
  /** Arbre actif (0 ou 1), par personnage. */
  arbres: Record<string, number>;
  /** Meilleur temps survécu (s), par arène. */
  records: Record<string, number>;
  bossVaincus: number;
  parties: number;
}

/** Ce que les talents actifs donnent au départ d'une partie. */
export interface BonusTalents {
  readonly modif: Partial<Modificateurs>;
  /** Rangs d'armes offerts (une entrée par rang). */
  readonly armes: readonly IdArme[];
  readonly relances: number;
}

/** Bilan d'une partie terminée, pour les récompenses. */
export interface BilanPartie {
  readonly arene: string;
  readonly temps: number;
  readonly tues: number;
  readonly boss: number;
  readonly victoire: boolean;
}

export interface Recompense {
  readonly emeraudes: number;
  readonly deblocages: readonly DefinitionDeblocage[];
}

/** Pourquoi un rang ne peut pas être acheté. */
export type RefusAchat = 'max' | 'palier' | 'prix' | null;

export class Sauvegarde {
  private readonly donnees: Donnees;

  private constructor(donnees: Donnees) {
    this.donnees = donnees;
  }

  static charger(): Sauvegarde {
    const lu = lireStockage<Partial<Donnees>>(CLE) ?? {};
    const nombre = (v: unknown) => (typeof v === 'number' && Number.isFinite(v) ? Math.max(0, v) : 0);
    const table = (v: unknown): Record<string, number> => {
      const sortie: Record<string, number> = {};
      if (v && typeof v === 'object') for (const [cle, val] of Object.entries(v)) sortie[cle] = nombre(val);
      return sortie;
    };
    return new Sauvegarde({
      emeraudes: Math.floor(nombre(lu.emeraudes)),
      talents: table(lu.talents),
      arbres: table(lu.arbres),
      records: table(lu.records),
      bossVaincus: nombre(lu.bossVaincus),
      parties: nombre(lu.parties),
    });
  }

  get emeraudes(): number {
    return this.donnees.emeraudes;
  }

  // --- Talents -------------------------------------------------------------

  arbreActif(personnage: IdPersonnage): number {
    return this.donnees.arbres[personnage] === 1 ? 1 : 0;
  }

  choisirArbre(personnage: IdPersonnage, arbre: number): void {
    this.donnees.arbres[personnage] = arbre;
    this.enregistrer();
  }

  rang(personnage: IdPersonnage, arbre: number, talent: DefinitionTalent): number {
    return this.donnees.talents[cleTalent(personnage, arbre, talent)] ?? 0;
  }

  /** Rangs achetés dans l'arbre (ouvrent les paliers). */
  points(personnage: IdPersonnage, arbre: number): number {
    let total = 0;
    for (const talent of ARBRES[personnage][arbre].talents) total += this.rang(personnage, arbre, talent);
    return total;
  }

  palierOuvert(personnage: IdPersonnage, arbre: number, palier: number): boolean {
    return this.points(personnage, arbre) >= SEUILS_PALIERS[palier];
  }

  refusAchat(personnage: IdPersonnage, arbre: number, talent: DefinitionTalent): RefusAchat {
    const rang = this.rang(personnage, arbre, talent);
    if (rang >= talent.couts.length) return 'max';
    if (!this.palierOuvert(personnage, arbre, talent.palier)) return 'palier';
    if (this.donnees.emeraudes < talent.couts[rang]) return 'prix';
    return null;
  }

  acheter(personnage: IdPersonnage, arbre: number, talent: DefinitionTalent): boolean {
    if (this.refusAchat(personnage, arbre, talent) !== null) return false;
    const rang = this.rang(personnage, arbre, talent);
    this.donnees.emeraudes -= talent.couts[rang];
    this.donnees.talents[cleTalent(personnage, arbre, talent)] = rang + 1;
    this.enregistrer();
    return true;
  }

  /** Émeraudes rendues si l'arbre était remis à zéro. */
  valeur(personnage: IdPersonnage, arbre: number): number {
    let total = 0;
    for (const talent of ARBRES[personnage][arbre].talents) {
      const rang = this.rang(personnage, arbre, talent);
      for (let r = 0; r < rang; r++) total += talent.couts[r];
    }
    return total;
  }

  /** Rend toutes les émeraudes dépensées dans l'arbre. */
  reinitialiser(personnage: IdPersonnage, arbre: number): void {
    this.donnees.emeraudes += this.valeur(personnage, arbre);
    for (const talent of ARBRES[personnage][arbre].talents) delete this.donnees.talents[cleTalent(personnage, arbre, talent)];
    this.enregistrer();
  }

  /** Effets de l'arbre actif du personnage. */
  bonus(personnage: IdPersonnage): BonusTalents {
    const arbre = this.arbreActif(personnage);
    const modif: Partial<Modificateurs> = {};
    const armes: IdArme[] = [];
    let relances = 0;
    for (const talent of ARBRES[personnage][arbre].talents) {
      const rang = this.rang(personnage, arbre, talent);
      for (let r = 0; r < rang; r++) {
        if (talent.modif) cumulerModificateurs(modif, talent.modif);
        if (talent.arme) armes.push(talent.arme);
        relances += talent.relances ?? 0;
      }
    }
    return { modif, armes, relances };
  }

  /** Meilleur temps survécu dans l'arène (s), 0 si jamais jouée. */
  record(arene: string): number {
    return this.donnees.records[arene] ?? 0;
  }

  // --- Déblocages ----------------------------------------------------------

  /** Condition à remplir, ou null si c'est déjà disponible. */
  verrou(type: DefinitionDeblocage['type'], id: string): DefinitionDeblocage | null {
    const deblocage = DEBLOCAGES.find((d) => d.type === type && d.id === id);
    return deblocage && !this.rempli(deblocage) ? deblocage : null;
  }

  private rempli(deblocage: DefinitionDeblocage): boolean {
    const c = deblocage.condition;
    switch (c.type) {
      case 'survie': {
        const records = this.donnees.records;
        const meilleur = c.arene !== undefined ? records[c.arene] ?? 0 : Math.max(0, ...Object.values(records));
        return meilleur >= c.duree;
      }
      case 'boss':
        return this.donnees.bossVaincus >= c.nombre;
    }
  }

  // --- Fin de partie -------------------------------------------------------

  /** Récompense une partie terminée et enregistre ses records. */
  terminerPartie(bilan: BilanPartie): Recompense {
    const r = RECOMPENSES;
    const emeraudes = Math.floor(bilan.temps / r.parSecondes) + Math.floor(bilan.tues / r.parTues)
      + bilan.boss * r.parBoss + (bilan.victoire ? r.victoire : 0);
    const verrouillesAvant = DEBLOCAGES.filter((d) => !this.rempli(d));
    const d = this.donnees;
    d.emeraudes += emeraudes;
    d.records[bilan.arene] = Math.max(d.records[bilan.arene] ?? 0, bilan.temps);
    d.bossVaincus += bilan.boss;
    d.parties++;
    this.enregistrer();
    return { emeraudes, deblocages: verrouillesAvant.filter((deblocage) => this.rempli(deblocage)) };
  }

  /** Debug : émeraudes offertes. */
  offrir(emeraudes: number): void {
    this.donnees.emeraudes += emeraudes;
    this.enregistrer();
  }

  private enregistrer(): void {
    ecrireStockage(CLE, this.donnees);
  }
}

function cleTalent(personnage: IdPersonnage, arbre: number, talent: DefinitionTalent): string {
  return `${personnage}.${ARBRES[personnage][arbre].id}.${talent.id}`;
}
