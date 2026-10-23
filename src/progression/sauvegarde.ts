/**
 * Sauvegarde de la progression entre les parties (stockage local) : XP de
 * maîtrise de chaque classe, talents achetés dans les deux arbres de chaque
 * classe, réglages de chaque arbre (option des talents à choix, talent final
 * actif, arme de départ active), arbre actif, records qui débloquent
 * personnages et arènes.
 *
 * Chaque niveau de maîtrise donne un point de talent ; un rang coûte un point.
 * Les deux arbres d'une classe ont chacun tous ses points (comme une double
 * spécialisation) : changer d'arbre ne coûte rien.
 *
 * Version 3 : maîtrise par classe. Les talents achetés avec les émeraudes des
 * versions précédentes sont remis à zéro (records et arbre actif gardés).
 */
import type { IdArme } from '../donnees/armes';
import type { IdPersonnage } from '../donnees/personnages';
import { ARBRES, DEBLOCAGES, niveauMaitrise, PALIERS, xpPourNiveau } from '../donnees/talents';
import type { BonusArmeTalent, DefinitionDeblocage, EffetTalent, Modificateurs, NoeudTalent } from '../donnees/types';
import { ecrireStockage, lireStockage } from '../options';
import { cumulerModificateurs } from './modificateurs';

const CLE = 'block-survivors.progression';
/** Sauvegarde à part du multijoueur : une partie à plusieurs ne touche jamais au solo. */
export const CLE_MULTIJOUEUR = 'block-survivors.progression.multi';
const VERSION = 3;

interface Donnees {
  version: number;
  /** XP de maîtrise cumulée, par personnage. */
  maitrise: Record<string, number>;
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
  /** Meilleur temps total (s) d'une partie continuée en mode sans fin, par arène. */
  recordsSansFin: Record<string, number>;
  bossVaincus: number;
  parties: number;
  /** Talents remis à zéro au passage à la maîtrise, pas encore annoncé. */
  reinitialise: boolean;
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
  readonly personnage: IdPersonnage;
  readonly arene: string;
  readonly temps: number;
  /** XP ramassée pendant la partie : elle va à la maîtrise de la classe. */
  readonly xp: number;
  readonly boss: number;
  /**
   * Suite d'une partie gagnée (mode sans fin) : la victoire a déjà compté la partie, le record et
   * l'XP d'avant ; `xp` et `boss` ne comptent que la suite, `temps` est le temps total.
   */
  readonly sansFin: boolean;
}

/** Où en est la maîtrise d'une classe. */
export interface EtatMaitrise {
  readonly niveau: number;
  readonly xp: number;
  /** XP du début du niveau actuel et du suivant. */
  readonly debutNiveau: number;
  readonly finNiveau: number;
}

export interface Recompense {
  readonly personnage: IdPersonnage;
  readonly xp: number;
  readonly avant: EtatMaitrise;
  readonly apres: EtatMaitrise;
  readonly deblocages: readonly DefinitionDeblocage[];
  /** Mode sans fin : le temps bat le record sans fin de l'arène. */
  readonly recordSansFin: boolean;
}

/** Pourquoi un rang ne peut pas être acheté. */
export type RefusAchat = 'max' | 'verrou' | 'points' | null;

export class Sauvegarde {
  private readonly donnees: Donnees;
  private readonly cle: string;

  private constructor(donnees: Donnees, cle: string) {
    this.donnees = donnees;
    this.cle = cle;
  }

  /** `cle` : CLE_MULTIJOUEUR pour la progression du multijoueur. */
  static charger(cle = CLE): Sauvegarde {
    const lu = lireStockage<Partial<Donnees> & { talents?: unknown }>(cle) ?? {};
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
      maitrise: actuelle ? table(lu.maitrise) : {},
      rangs: actuelle ? table(lu.rangs) : {},
      options: actuelle ? table(lu.options) : {},
      finals: actuelle ? textes(lu.finals) : {},
      armesDepart: actuelle ? textes(lu.armesDepart) : {},
      arbres: table(lu.arbres),
      records: table(lu.records),
      recordsSansFin: table(lu.recordsSansFin),
      bossVaincus: nombre(lu.bossVaincus),
      parties: nombre(lu.parties),
      // Une ancienne sauvegarde existait : ses talents sont remis à zéro, on le dira une fois
      reinitialise: actuelle ? lu.reinitialise === true : lireStockage(cle) !== null,
    };
    const sauvegarde = new Sauvegarde(donnees, cle);
    if (!actuelle && lireStockage(cle) !== null) sauvegarde.enregistrer();
    return sauvegarde;
  }

  /** Vrai une seule fois si les talents viennent d'être remis à zéro (passage à la maîtrise). */
  annoncerReinitialisation(): boolean {
    if (!this.donnees.reinitialise) return false;
    this.donnees.reinitialise = false;
    this.enregistrer();
    return true;
  }

  // --- Maîtrise ------------------------------------------------------------

  maitrise(personnage: IdPersonnage): EtatMaitrise {
    return etatMaitrise(this.donnees.maitrise[personnage] ?? 0);
  }

  /** Points de talent encore libres dans cet arbre (chaque arbre a tous les points de la classe). */
  pointsLibres(personnage: IdPersonnage, arbre: number): number {
    return Math.max(0, this.maitrise(personnage).niveau - this.points(personnage, arbre));
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
    if (this.pointsLibres(personnage, arbre) < 1) return 'points';
    return null;
  }

  acheter(personnage: IdPersonnage, arbre: number, noeud: NoeudTalent): boolean {
    if (this.refusAchat(personnage, arbre, noeud) !== null) return false;
    this.donnees.rangs[cleNoeud(personnage, arbre, noeud)] = this.rang(personnage, arbre, noeud) + 1;
    this.enregistrer();
    return true;
  }

  /** Rend tous les points dépensés dans l'arbre et efface ses réglages. */
  reinitialiser(personnage: IdPersonnage, arbre: number): void {
    const d = this.donnees;
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

  /** Meilleur temps total en mode sans fin dans l'arène (s), 0 si jamais tenté. */
  recordSansFin(arene: string): number {
    return this.donnees.recordsSansFin[arene] ?? 0;
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

  /** Ajoute l'XP de la partie à la maîtrise de la classe et enregistre ses records. */
  terminerPartie(bilan: BilanPartie): Recompense {
    const verrouillesAvant = DEBLOCAGES.filter((d) => !this.rempli(d));
    const d = this.donnees;
    const avant = this.maitrise(bilan.personnage);
    const xp = Math.max(0, Math.floor(bilan.xp));
    d.maitrise[bilan.personnage] = avant.xp + xp;
    d.bossVaincus += bilan.boss;
    let recordSansFin = false;
    if (bilan.sansFin) {
      recordSansFin = bilan.temps > (d.recordsSansFin[bilan.arene] ?? 0);
      if (recordSansFin) d.recordsSansFin[bilan.arene] = bilan.temps;
    } else {
      d.records[bilan.arene] = Math.max(d.records[bilan.arene] ?? 0, bilan.temps);
      d.parties++;
    }
    this.enregistrer();
    return {
      personnage: bilan.personnage,
      xp,
      avant,
      apres: this.maitrise(bilan.personnage),
      deblocages: verrouillesAvant.filter((deblocage) => this.rempli(deblocage)),
      recordSansFin,
    };
  }

  /** Debug : un niveau de maîtrise offert à la classe. */
  offrirNiveau(personnage: IdPersonnage): EtatMaitrise {
    const m = this.maitrise(personnage);
    this.donnees.maitrise[personnage] = m.finNiveau;
    this.enregistrer();
    return this.maitrise(personnage);
  }

  // --- Multijoueur --------------------------------------------------------

  /** Vrai si la classe a déjà sa progression dans cette sauvegarde. */
  connait(personnage: IdPersonnage): boolean {
    return personnage in this.donnees.maitrise;
  }

  /** Repart de zéro pour la classe (maîtrise, talents, réglages des arbres) : un personnage neuf. */
  nouveauPersonnage(personnage: IdPersonnage): void {
    const d = this.donnees;
    const prefixe = `${personnage}.`;
    for (const table of [d.rangs, d.options, d.finals, d.armesDepart] as Record<string, unknown>[]) {
      for (const cle of Object.keys(table)) if (cle.startsWith(prefixe)) delete table[cle];
    }
    d.maitrise[personnage] = 0;
    delete d.arbres[personnage];
    this.enregistrer();
  }

  private enregistrer(): void {
    ecrireStockage(this.cle, this.donnees);
  }
}

function etatMaitrise(xp: number): EtatMaitrise {
  const niveau = niveauMaitrise(xp);
  return { niveau, xp, debutNiveau: xpPourNiveau(niveau), finNiveau: xpPourNiveau(niveau + 1) };
}

function cleArbre(personnage: IdPersonnage, arbre: number): string {
  return `${personnage}.${ARBRES[personnage][arbre].id}`;
}

function cleNoeud(personnage: IdPersonnage, arbre: number, noeud: NoeudTalent): string {
  return `${cleArbre(personnage, arbre)}.${noeud.id}`;
}
