/**
 * Cartes d'amélioration : tirage des propositions à chaque niveau (nouvelle
 * arme, rang d'arme, nouveau passif, rang de passif, ou soin si tout est au
 * maximum), textes affichés et application du choix.
 */
import { calculerStats, rangMax } from '../combat/armeEquipee';
import type { Arsenal } from '../combat/arsenal';
import { ARMES, IDS_ARMES, type IdArme } from '../donnees/armes';
import { IDS_PASSIFS, PASSIFS, type IdPassif } from '../donnees/passifs';
import { PROGRESSION } from '../donnees/progression';
import type { CheminTexture, MecaniqueArme, Modificateurs, StatsArme } from '../donnees/types';
import type { EtatPartie } from '../etat';
import { creerModificateurs, cumulerModificateurs } from './modificateurs';
import type { Passifs } from './passifs';

export type Amelioration =
  | { readonly type: 'arme'; readonly id: IdArme }
  | { readonly type: 'passif'; readonly id: IdPassif }
  | { readonly type: 'soin' };

/** Famille de la carte, qui donne la couleur de son nom (comme la rareté des objets du jeu). */
export type RareteCarte = 'nouvelle_arme' | 'arme' | 'passif' | 'exclusif' | 'soin';

/** Tout ce qu'une carte affiche. */
export interface Carte {
  readonly amelioration: Amelioration;
  readonly rarete: RareteCarte;
  readonly nom: string;
  readonly icone: CheminTexture;
  /** « Nouveau » ou « Rang 2 → 3 ». */
  readonly etiquette: string;
  readonly description: string;
  /** Statistiques modifiées, ex. « Dégâts : 10 → 15 ». */
  readonly lignes: readonly string[];
}

// --------------------------------------------------------------------------
// Libellés
// --------------------------------------------------------------------------
const LIBELLES_STATS: Record<keyof StatsArme, { nom: string; unite: string }> = {
  degats: { nom: 'Dégâts', unite: '' },
  cadence: { nom: 'Délai', unite: ' s' },
  nombre: { nom: 'Nombre', unite: '' },
  portee: { nom: 'Portée', unite: ' blocs' },
  vitesse: { nom: 'Vitesse', unite: '' },
  zone: { nom: 'Zone', unite: ' blocs' },
  duree: { nom: 'Durée', unite: ' s' },
  perforation: { nom: 'Perforation', unite: '' },
  recul: { nom: 'Recul', unite: '' },
};

/** Statistiques utiles à chaque mécanique (les autres ne sont pas affichées). */
const STATS_UTILES: Record<MecaniqueArme['type'], readonly (keyof StatsArme)[]> = {
  balayage: ['degats', 'cadence', 'zone', 'recul'],
  tir: ['degats', 'cadence', 'nombre', 'portee', 'perforation'],
  retour: ['degats', 'cadence', 'nombre', 'portee'],
  orbite: ['degats', 'nombre', 'zone', 'vitesse'],
  explosif: ['degats', 'cadence', 'nombre', 'zone'],
  foudre: ['degats', 'cadence', 'nombre', 'zone'],
  invocation: ['degats', 'cadence', 'nombre', 'duree'],
};

/** Modificateurs multiplicatifs (affichés en %) ; les autres sont des ajouts. */
const LIBELLES_MODIFICATEURS: Record<keyof Modificateurs, { nom: string; pourcentage: boolean; unite: string }> = {
  degats: { nom: 'Dégâts des armes', pourcentage: true, unite: '' },
  cadence: { nom: 'Délai des armes', pourcentage: true, unite: '' },
  zone: { nom: 'Zone des armes', pourcentage: true, unite: '' },
  portee: { nom: 'Portée des armes', pourcentage: true, unite: '' },
  projectiles: { nom: 'Projectiles en plus', pourcentage: false, unite: '' },
  vitesse: { nom: 'Vitesse', pourcentage: true, unite: '' },
  pvMax: { nom: 'Santé maximale', pourcentage: false, unite: ' PV' },
  armure: { nom: 'Dégâts reçus', pourcentage: false, unite: '' },
  regeneration: { nom: 'Régénération', pourcentage: false, unite: ' PV/s' },
  vies: { nom: 'Vies en plus', pourcentage: false, unite: '' },
  collecte: { nom: 'Rayon de collecte', pourcentage: true, unite: '' },
};

const FORMAT = new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 2 });

// --------------------------------------------------------------------------
// Tirage et application
// --------------------------------------------------------------------------
export class Ameliorations {
  private readonly etat: EtatPartie;
  private readonly arsenal: Arsenal;
  private readonly passifs: Passifs;

  constructor(etat: EtatPartie, arsenal: Arsenal, passifs: Passifs) {
    this.etat = etat;
    this.arsenal = arsenal;
    this.passifs = passifs;
  }

  /** Propose jusqu'à `nombre` cartes différentes, tirées au hasard parmi les améliorations possibles. */
  tirer(nombre = PROGRESSION.cartesParNiveau): Carte[] {
    const possibles = this.ameliorationsPossibles();
    const cartes: Carte[] = [];
    while (cartes.length < nombre && possibles.length > 0) {
      const [choix] = possibles.splice(Math.floor(Math.random() * possibles.length), 1);
      cartes.push(this.decrire(choix));
    }
    if (cartes.length === 0) cartes.push(this.decrire({ type: 'soin' }));
    return cartes;
  }

  appliquer(amelioration: Amelioration): void {
    const j = this.etat.joueur;
    switch (amelioration.type) {
      case 'arme':
        this.arsenal.ameliorer(amelioration.id);
        break;
      case 'passif': {
        const pvMaxAvant = this.etat.modificateurs.pvMax;
        this.passifs.ameliorer(amelioration.id);
        this.passifs.calculer(this.etat.modificateurs);
        // Les cœurs gagnés arrivent pleins
        const gainPv = this.etat.modificateurs.pvMax - pvMaxAvant;
        j.pvMax += gainPv;
        j.pv += gainPv;
        this.arsenal.recalculer();
        break;
      }
      case 'soin': {
        const avant = j.pv;
        j.pv = Math.min(j.pvMax, j.pv + PROGRESSION.soin.pv);
        this.etat.stats.pvSoignes += j.pv - avant;
        break;
      }
    }
  }

  private ameliorationsPossibles(): Amelioration[] {
    const liste: Amelioration[] = [];
    const placeArme = this.arsenal.armes.length < PROGRESSION.emplacementsArmes;
    const placePassif = this.passifs.possedes.length < PROGRESSION.emplacementsPassifs;
    for (const id of IDS_ARMES) {
      const rang = this.arsenal.rang(id);
      if (rang === 0 ? placeArme : rang < rangMax(ARMES[id])) liste.push({ type: 'arme', id });
    }
    const exclusifs: readonly IdPassif[] = this.etat.personnage.cartesExclusives;
    for (const id of IDS_PASSIFS) {
      if (PASSIFS[id].exclusif && !exclusifs.includes(id)) continue;
      const rang = this.passifs.rang(id);
      if (rang === 0 ? placePassif : rang < PASSIFS[id].niveaux.length) liste.push({ type: 'passif', id });
    }
    return liste;
  }

  private decrire(amelioration: Amelioration): Carte {
    switch (amelioration.type) {
      case 'arme': return this.decrireArme(amelioration.id);
      case 'passif': return this.decrirePassif(amelioration.id);
      case 'soin': {
        const s = PROGRESSION.soin;
        return { amelioration, rarete: 'soin', nom: s.nom, icone: s.icone, etiquette: 'Soin', description: s.description, lignes: [`Santé : +${s.pv} PV`] };
      }
    }
  }

  private decrireArme(id: IdArme): Carte {
    const def = ARMES[id];
    const rang = this.arsenal.rang(id);
    const mods = this.etat.modificateurs;
    const apres = calculerStats(def, rang + 1, mods, { ...def.stats });
    const utiles = STATS_UTILES[def.mecanique.type];
    const amelioration: Amelioration = { type: 'arme', id };

    if (rang === 0) {
      return {
        amelioration,
        rarete: 'nouvelle_arme',
        nom: def.nom,
        icone: def.icone,
        etiquette: 'Nouvelle arme',
        description: def.description,
        lignes: utiles.map((cle) => `${LIBELLES_STATS[cle].nom} : ${formaterStat(cle, apres[cle])}`),
      };
    }
    const avant = calculerStats(def, rang, mods, { ...def.stats });
    return {
      amelioration,
      rarete: 'arme',
      nom: def.nom,
      icone: def.icone,
      etiquette: `Rang ${rang} → ${rang + 1}`,
      description: def.niveaux[rang - 1].description,
      lignes: (Object.keys(LIBELLES_STATS) as (keyof StatsArme)[])
        .filter((cle) => Math.abs(apres[cle] - avant[cle]) > 1e-6)
        .map((cle) => `${LIBELLES_STATS[cle].nom} : ${formaterStat(cle, avant[cle])} → ${formaterStat(cle, apres[cle])}`),
    };
  }

  private decrirePassif(id: IdPassif): Carte {
    const def = PASSIFS[id];
    const rang = this.passifs.rang(id);
    const avant = this.passifs.calculer(creerModificateurs());
    const apres = this.passifs.calculer(creerModificateurs(), id);
    return {
      amelioration: { type: 'passif', id },
      rarete: def.exclusif ? 'exclusif' : 'passif',
      nom: def.nom,
      icone: def.icone,
      etiquette: rang === 0 ? (def.exclusif ? 'Nouveau bonus exclusif' : 'Nouveau bonus') : `Rang ${rang} → ${rang + 1}`,
      description: rang === 0 ? `${def.description} ${def.niveaux[0].description}.` : def.niveaux[rang].description,
      lignes: (Object.keys(LIBELLES_MODIFICATEURS) as (keyof Modificateurs)[])
        .filter((cle) => Math.abs(apres[cle] - avant[cle]) > 1e-6)
        .map((cle) => `${LIBELLES_MODIFICATEURS[cle].nom} : ${formaterModificateur(cle, avant[cle])} → ${formaterModificateur(cle, apres[cle])}`),
    };
  }
}

/** Lignes décrivant des bonus permanents (personnages), ex. « Dégâts reçus : −1 par coup ». */
export function decrireBonus(bonus: Partial<Modificateurs>): string[] {
  const valeurs = creerModificateurs();
  cumulerModificateurs(valeurs, bonus);
  return (Object.keys(bonus) as (keyof Modificateurs)[])
    .filter((cle) => (bonus[cle] ?? 0) !== 0)
    .map((cle) => `${LIBELLES_MODIFICATEURS[cle].nom} : ${formaterModificateur(cle, valeurs[cle])}`);
}

function formaterStat(cle: keyof StatsArme, valeur: number): string {
  return `${FORMAT.format(valeur)}${LIBELLES_STATS[cle].unite}`;
}

function formaterModificateur(cle: keyof Modificateurs, valeur: number): string {
  const libelle = LIBELLES_MODIFICATEURS[cle];
  if (cle === 'armure') return valeur === 0 ? 'normaux' : `−${FORMAT.format(valeur)} par coup`;
  if (libelle.pourcentage) {
    const pourcent = Math.round((valeur - 1) * 100);
    return `${pourcent >= 0 ? '+' : '−'}${Math.abs(pourcent)} %`;
  }
  return `+${FORMAT.format(valeur)}${libelle.unite}`;
}

