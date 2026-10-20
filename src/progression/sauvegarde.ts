/**
 * Sauvegarde de la progression entre les parties (stockage local) :
 * émeraudes, talents achetés dans les deux arbres de chaque classe, réglages
 * de chaque arbre (option des talents à choix, talent final actif, arme de
 * départ active), arbre actif, records qui débloquent personnages et arènes.
 *
 * Version 2 : arbres en graphe. Les achats de la version 1 sont remboursés.
 */
import type { IdArme } from '../donnees/armes';
import type { IdPersonnage } from '../donnees/personnages';
import { ARBRES, coutsTalent, DEBLOCAGES, PALIERS, RECOMPENSES } from '../donnees/talents';
import type { BonusArmeTalent, DefinitionDeblocage, EffetTalent, Modificateurs, NoeudTalent } from '../donnees/types';
import { ecrireStockage, lireStockage } from '../options';
import { cumulerModificateurs } from './modificateurs';

const CLE = 'block-survivors.progression';
const VERSION = 2;

interface Donnees {
  version: number;
  emeraudes: number;
  /** Rang acheté, par « personnage.arbre.talent ». */
  rangs: Record<string, number>;
  /** Option choisie (0 ou 1) des talents à choix, par « personnage.arbre.talent ». */
  options: Record<string, number>;
  /** Talent final actif, par « personnage.arbre ». */
  finals: Record<string, string>;
  /** Talent d'arme de départ actif, par « personnage.arbre » ('' : aucune arme en plus). */
  armesDepart: Record<string, string>;
  /** Arbre actif (0 ou 1), par personnage. */
  arbres: Record<string, number>;
  /** Meilleur temps survécu (s), par arène. */
  records: Record<string, number>;
  bossVaincus: number;
  parties: number;
  /** Émeraudes rendues au passage aux nouveaux arbres, pas encore annoncées. */
  rembourse: number;
}

/** Ce que l'arbre actif donne au départ d'une partie. */
export interface BonusTalents {
  readonly modif: Partial<Modificateurs>;
  /** Rangs d'armes offerts au départ (une entrée par rang). */
  readonly armes: readonly IdArme[];
  readonly relances: number;
  /** Bonus d'armes précises (une entrée par rang). */
  readonly bonusArmes: readonly BonusArmeTalent[];
  /** Mécaniques propres actives et leur rang. */
  readonly mecaniques: Readonly<Record<string, number>>;
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
export type RefusAchat = 'max' | 'verrou' | 'prix' | null;

/** Coûts des talents de la version 1, pour les rembourser (« personnage.arbre.talent »). */
const ANCIENS_COUTS: Readonly<Record<string, readonly number[]>> = {
  'guerrier.gladiateur.lame': [5, 10, 15], 'guerrier.gladiateur.precision': [5, 10, 15], 'guerrier.gladiateur.maitre': [15, 25],
  'guerrier.gladiateur.fougue': [15, 25, 35], 'guerrier.gladiateur.fatal': [40, 70], 'guerrier.gladiateur.tourbillon': [40, 70],
  'guerrier.gardien.robustesse': [5, 10, 15], 'guerrier.gardien.souffle': [5, 10, 15], 'guerrier.gardien.bouclier': [15, 25],
  'guerrier.gardien.endurance': [15, 25, 35], 'guerrier.gardien.immortel': [80], 'guerrier.gardien.forteresse': [40, 70],
  'archer.elite.pointes': [5, 10, 15], 'archer.elite.oeil': [5, 10, 15], 'archer.elite.visee': [15, 25, 35],
  'archer.elite.arc': [15, 25], 'archer.elite.tete': [40, 70], 'archer.elite.salve': [80],
  'archer.rodeuse.pas': [5, 10, 15], 'archer.rodeuse.flair': [5, 10, 15], 'archer.rodeuse.ruse': [15, 25],
  'archer.rodeuse.cadence': [15, 25, 35], 'archer.rodeuse.baies': [40, 70], 'archer.rodeuse.sous_bois': [40, 70],
  'mage.pyromancien.braise': [5, 10, 15], 'mage.pyromancien.souffle': [5, 10, 15], 'mage.pyromancien.boules': [15, 25],
  'mage.pyromancien.combustion': [15, 25, 35], 'mage.pyromancien.brasier': [40, 70], 'mage.pyromancien.meteores': [80],
  'mage.arcaniste.concentration': [5, 10, 15], 'mage.arcaniste.vigueur': [5, 10, 15], 'mage.arcaniste.meditation': [15, 25, 35],
  'mage.arcaniste.savoir': [15, 25], 'mage.arcaniste.tempete': [80], 'mage.arcaniste.egide': [40, 70],
  'invocateur.maitre_chien.crocs': [5, 10, 15], 'invocateur.maitre_chien.piste': [5, 10, 15], 'invocateur.maitre_chien.dressage': [15, 25],
  'invocateur.maitre_chien.instinct': [15, 25, 35], 'invocateur.maitre_chien.alpha': [80], 'invocateur.maitre_chien.jugulaire': [40, 70],
  'invocateur.evocateur.robustesse': [5, 10, 15], 'invocateur.evocateur.incantation': [5, 10, 15], 'invocateur.evocateur.vex': [15, 25],
  'invocateur.evocateur.protection': [15, 25], 'invocateur.evocateur.golem': [80], 'invocateur.evocateur.longevite': [40, 70],
};

export class Sauvegarde {
  private readonly donnees: Donnees;

  private constructor(donnees: Donnees) {
    this.donnees = donnees;
  }

  static charger(): Sauvegarde {
    const lu = lireStockage<Partial<Donnees> & { talents?: unknown }>(CLE) ?? {};
    const nombre = (v: unknown) => (typeof v === 'number' && Number.isFinite(v) ? Math.max(0, v) : 0);
    const table = (v: unknown): Record<string, number> => {
      const sortie: Record<string, number> = {};
      if (v && typeof v === 'object') for (const [cle, val] of Object.entries(v)) sortie[cle] = nombre(val);
      return sortie;
    };
    const textes = (v: unknown): Record<string, string> => {
      const sortie: Record<string, string> = {};
      if (v && typeof v === 'object') for (const [cle, val] of Object.entries(v)) if (typeof val === 'string') sortie[cle] = val;
      return sortie;
    };
    const actuelle = lu.version === VERSION;
    const donnees: Donnees = {
      version: VERSION,
      emeraudes: Math.floor(nombre(lu.emeraudes)),
      rangs: actuelle ? table(lu.rangs) : {},
      options: actuelle ? table(lu.options) : {},
      finals: actuelle ? textes(lu.finals) : {},
      armesDepart: actuelle ? textes(lu.armesDepart) : {},
      arbres: table(lu.arbres),
      records: table(lu.records),
      bossVaincus: nombre(lu.bossVaincus),
      parties: nombre(lu.parties),
      rembourse: nombre(lu.rembourse),
    };
    // Version 1 : chaque rang acheté est rendu à son prix d'achat
    if (!actuelle) {
      let rendu = 0;
      for (const [cle, rang] of Object.entries(table(lu.talents))) {
        const couts = ANCIENS_COUTS[cle];
        if (couts) for (let r = 0; r < Math.min(rang, couts.length); r++) rendu += couts[r];
      }
      donnees.emeraudes += rendu;
      donnees.rembourse += rendu;
    }
    const sauvegarde = new Sauvegarde(donnees);
    if (!actuelle && lireStockage(CLE) !== null) sauvegarde.enregistrer();
    return sauvegarde;
  }

  get emeraudes(): number {
    return this.donnees.emeraudes;
  }

  /** Émeraudes rendues au passage aux nouveaux arbres (une seule fois, puis 0). */
  annoncerRemboursement(): number {
    const rendu = this.donnees.rembourse;
    if (rendu > 0) {
      this.donnees.rembourse = 0;
      this.enregistrer();
    }
    return rendu;
  }

  // --- Arbres --------------------------------------------------------------

  arbreActif(personnage: IdPersonnage): number {
    return this.donnees.arbres[personnage] === 1 ? 1 : 0;
  }

  choisirArbre(personnage: IdPersonnage, arbre: number): void {
    this.donnees.arbres[personnage] = arbre;
    this.enregistrer();
  }

  rang(personnage: IdPersonnage, arbre: number, noeud: NoeudTalent): number {
    return this.donnees.rangs[cleNoeud(personnage, arbre, noeud)] ?? 0;
  }

  complet(personnage: IdPersonnage, arbre: number, noeud: NoeudTalent): boolean {
    return this.rang(personnage, arbre, noeud) >= noeud.rangMax;
  }

  /** Rangs achetés dans l'arbre (au-dessus de la ligne `ligne` si elle est donnée). */
  points(personnage: IdPersonnage, arbre: number, ligne = Infinity): number {
    let total = 0;
    for (const noeud of ARBRES[personnage][arbre].noeuds) if (noeud.ligne < ligne) total += this.rang(personnage, arbre, noeud);
    return total;
  }

  /** Le palier de cette ligne est-il atteint (assez de rangs achetés au-dessus) ? */
  palierAtteint(personnage: IdPersonnage, arbre: number, ligne: number): boolean {
    return PALIERS.every((p) => ligne < p.ligne || this.points(personnage, arbre, p.ligne) >= p.rangs);
  }

  /** Le talent peut-il être acheté (palier atteint, et racine, libre ou relié à un talent complet au-dessus) ? */
  ouvert(personnage: IdPersonnage, arbre: number, noeud: NoeudTalent): boolean {
    if (!this.palierAtteint(personnage, arbre, noeud.ligne)) return false;
    if (noeud.libre || !noeud.prerequis || noeud.prerequis.length === 0) return true;
    const noeuds = ARBRES[personnage][arbre].noeuds;
    return noeud.prerequis.some((id) => {
      const parent = noeuds.find((n) => n.id === id);
      return parent !== undefined && this.complet(personnage, arbre, parent);
    });
  }

  refusAchat(personnage: IdPersonnage, arbre: number, noeud: NoeudTalent): RefusAchat {
    const rang = this.rang(personnage, arbre, noeud);
    if (rang >= noeud.rangMax) return 'max';
    if (!this.ouvert(personnage, arbre, noeud)) return 'verrou';
    if (this.donnees.emeraudes < coutsTalent(noeud)[rang]) return 'prix';
    return null;
  }

  acheter(personnage: IdPersonnage, arbre: number, noeud: NoeudTalent): boolean {
    if (this.refusAchat(personnage, arbre, noeud) !== null) return false;
    const rang = this.rang(personnage, arbre, noeud);
    this.donnees.emeraudes -= coutsTalent(noeud)[rang];
    this.donnees.rangs[cleNoeud(personnage, arbre, noeud)] = rang + 1;
    this.enregistrer();
    return true;
  }

  /** Émeraudes rendues si l'arbre était remis à zéro. */
  valeur(personnage: IdPersonnage, arbre: number): number {
    let total = 0;
    for (const noeud of ARBRES[personnage][arbre].noeuds) {
      const couts = coutsTalent(noeud);
      const rang = this.rang(personnage, arbre, noeud);
      for (let r = 0; r < rang; r++) total += couts[r];
    }
    return total;
  }

  /** Rend toutes les émeraudes dépensées dans l'arbre et efface ses réglages. */
  reinitialiser(personnage: IdPersonnage, arbre: number): void {
    const d = this.donnees;
    d.emeraudes += this.valeur(personnage, arbre);
    for (const noeud of ARBRES[personnage][arbre].noeuds) {
      delete d.rangs[cleNoeud(personnage, arbre, noeud)];
      delete d.options[cleNoeud(personnage, arbre, noeud)];
    }
    delete d.finals[cleArbre(personnage, arbre)];
    delete d.armesDepart[cleArbre(personnage, arbre)];
    this.enregistrer();
  }

  /** Option choisie (0 ou 1) d'un talent à choix. */
  option(personnage: IdPersonnage, arbre: number, noeud: NoeudTalent): number {
    return this.donnees.options[cleNoeud(personnage, arbre, noeud)] === 1 ? 1 : 0;
  }

  choisirOption(personnage: IdPersonnage, arbre: number, noeud: NoeudTalent, option: number): void {
    this.donnees.options[cleNoeud(personnage, arbre, noeud)] = option === 1 ? 1 : 0;
    this.enregistrer();
  }

  /** Talent final actif de l'arbre (le choisi s'il est acheté, sinon le premier acheté), ou null. */
  finalActif(personnage: IdPersonnage, arbre: number): NoeudTalent | null {
    const noeuds = ARBRES[personnage][arbre].noeuds;
    const id = this.donnees.finals[cleArbre(personnage, arbre)];
    const achete = (n: NoeudTalent) => n.type === 'final' && this.rang(personnage, arbre, n) > 0;
    return noeuds.find((n) => n.id === id && achete(n)) ?? noeuds.find(achete) ?? null;
  }

  choisirFinal(personnage: IdPersonnage, arbre: number, noeud: NoeudTalent): void {
    if (noeud.type !== 'final' || this.rang(personnage, arbre, noeud) === 0) return;
    this.donnees.finals[cleArbre(personnage, arbre)] = noeud.id;
    this.enregistrer();
  }

  /** Talent d'arme de départ actif (le choisi s'il est acheté, sinon le premier acheté), ou null si aucun. */
  armeDepartActive(personnage: IdPersonnage, arbre: number): NoeudTalent | null {
    const noeuds = ARBRES[personnage][arbre].noeuds;
    const id = this.donnees.armesDepart[cleArbre(personnage, arbre)];
    if (id === '') return null;
    const achete = (n: NoeudTalent) => n.armeDepart !== undefined && this.rang(personnage, arbre, n) > 0;
    return noeuds.find((n) => n.id === id && achete(n)) ?? noeuds.find(achete) ?? null;
  }

  /** Active ce talent d'arme de départ, ou aucune arme en plus (null). */
  choisirArmeDepart(personnage: IdPersonnage, arbre: number, noeud: NoeudTalent | null): void {
    this.donnees.armesDepart[cleArbre(personnage, arbre)] = noeud?.id ?? '';
    this.enregistrer();
  }

  /** Le talent acheté joue-t-il dans son arbre (final choisi, arme de départ active) ? */
  actif(personnage: IdPersonnage, arbre: number, noeud: NoeudTalent): boolean {
    if (this.rang(personnage, arbre, noeud) === 0) return false;
    if (noeud.type === 'final') return this.finalActif(personnage, arbre) === noeud;
    if (noeud.armeDepart !== undefined) return this.armeDepartActive(personnage, arbre) === noeud;
    return true;
  }

  /** Effet en jeu du talent (celui de l'option choisie pour un talent à choix). */
  effet(personnage: IdPersonnage, arbre: number, noeud: NoeudTalent): EffetTalent {
    return noeud.options ? noeud.options[this.option(personnage, arbre, noeud)] : noeud;
  }

  /** Effets de l'arbre actif du personnage. */
  bonus(personnage: IdPersonnage): BonusTalents {
    const arbre = this.arbreActif(personnage);
    const modif: Partial<Modificateurs> = {};
    const armes: IdArme[] = [];
    const bonusArmes: BonusArmeTalent[] = [];
    const mecaniques: Record<string, number> = {};
    let relances = 0;
    for (const noeud of ARBRES[personnage][arbre].noeuds) {
      if (!this.actif(personnage, arbre, noeud)) continue;
      const rang = this.rang(personnage, arbre, noeud);
      const effet = this.effet(personnage, arbre, noeud);
      for (let r = 0; r < rang; r++) {
        if (effet.modif) cumulerModificateurs(modif, effet.modif);
        if (effet.armes) bonusArmes.push(...effet.armes);
        if (effet.rangsArme) armes.push(effet.rangsArme);
        if (effet.armeDepart) armes.push(effet.armeDepart);
        relances += effet.relances ?? 0;
      }
      if (effet.mecanique) mecaniques[effet.mecanique] = rang;
    }
    return { modif, armes, relances, bonusArmes, mecaniques };
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

function cleArbre(personnage: IdPersonnage, arbre: number): string {
  return `${personnage}.${ARBRES[personnage][arbre].id}`;
}

function cleNoeud(personnage: IdPersonnage, arbre: number, noeud: NoeudTalent): string {
  return `${cleArbre(personnage, arbre)}.${noeud.id}`;
}
