/**
 * Équipement de la partie (donnees/equipement.ts) : 2 bagues et 1 amulette au
 * plus. Les coffres en proposent (`tirer`) : objets de la réserve du monde et
 * de tous les mondes, rareté tirée selon le coffre, ou renforcement d'un objet
 * porté ; plus rien à proposer : un sac d'émeraudes. Emplacements pleins :
 * l'écran des cartes demande ensuite quel objet remplacer (`cartesRemplacement`).
 * Les bonus des objets portés passent par Passifs (`definirEquipement`), leurs
 * effets propres par combat/talents/objets.ts (`effets`).
 */
import type { Arsenal } from '../combat/arsenal';
import { IDS_OBJETS_EQUIPEMENT, OBJETS_EQUIPEMENT, REGLES_EQUIPEMENT, type IdObjetEquipement } from '../donnees/equipement';
import { OBJETS } from '../donnees/objets';
import type { CheminTexture, EffetObjet, EmplacementObjet, Modificateurs, RareteObjet } from '../donnees/types';
import type { EtatPartie } from '../etat';
import { lignesModificateurs, type Carte } from './ameliorations';
import { creerModificateurs } from './modificateurs';
import type { Passifs } from './passifs';

export type ChoixEquipement =
  | { readonly type: 'objet'; readonly id: IdObjetEquipement }
  | { readonly type: 'renfort'; readonly id: IdObjetEquipement }
  /** Prendre `id` à la place de l'objet porté à l'emplacement `index`. */
  | { readonly type: 'remplacer'; readonly id: IdObjetEquipement; readonly index: number }
  /** Ne rien remplacer : revenir aux propositions du coffre. */
  | { readonly type: 'retour' }
  | { readonly type: 'emeraudes'; readonly nombre: number };

export interface ObjetPorte {
  readonly id: IdObjetEquipement;
  niveau: number;
}

/** Effet propre d'un objet porté, avec le facteur de son niveau. */
export interface EffetPorte {
  readonly effet: EffetObjet;
  readonly facteur: number;
}

/** Icône dessinée au lancement (ui/iconesObjets.ts). */
export function cheminIconeObjet(id: IdObjetEquipement): CheminTexture {
  return `bs/objets/${id}`;
}

export function niveauMaxObjet(id: IdObjetEquipement): number {
  return OBJETS_EQUIPEMENT[id].rarete === 'legendaire' ? 1 : REGLES_EQUIPEMENT.niveaux.length;
}

const NOMS_EMPLACEMENTS: Readonly<Record<EmplacementObjet, string>> = { bague: 'Bague', amulette: 'Amulette' };

export class Equipement {
  readonly bagues: (ObjetPorte | null)[] = Array.from({ length: REGLES_EQUIPEMENT.bagues }, () => null);
  readonly amulettes: (ObjetPorte | null)[] = Array.from({ length: REGLES_EQUIPEMENT.amulettes }, () => null);
  /** Incrémenté à chaque changement (HUD, mécaniques). */
  version = 0;
  /** Effets propres des objets portés (reconstruit à chaque changement, jamais dans la boucle). */
  effets: readonly EffetPorte[] = [];
  private readonly etat: EtatPartie;
  private readonly passifs: Passifs;
  private readonly arsenal: Arsenal;
  private readonly surDecouverte: (id: IdObjetEquipement) => void;

  constructor(etat: EtatPartie, passifs: Passifs, arsenal: Arsenal, surDecouverte: (id: IdObjetEquipement) => void) {
    this.etat = etat;
    this.passifs = passifs;
    this.arsenal = arsenal;
    this.surDecouverte = surDecouverte;
  }

  emplacements(emplacement: EmplacementObjet): (ObjetPorte | null)[] {
    return emplacement === 'bague' ? this.bagues : this.amulettes;
  }

  /** Objets portés, bagues puis amulette. */
  portes(): ObjetPorte[] {
    return [...this.bagues, ...this.amulettes].filter((o): o is ObjetPorte => o !== null);
  }

  porte(id: IdObjetEquipement): ObjetPorte | null {
    return this.portes().find((o) => o.id === id) ?? null;
  }

  facteur(o: ObjetPorte): number {
    return REGLES_EQUIPEMENT.niveaux[o.niveau - 1];
  }

  /** Vrai si prendre cet objet oblige à en retirer un (emplacements pleins). */
  remplacementNecessaire(id: IdObjetEquipement): boolean {
    return !this.emplacements(OBJETS_EQUIPEMENT[id].emplacement).includes(null);
  }

  /** Propositions d'un coffre (`ender` : coffre de boss ; `ouverts` : coffres en bois déjà ouverts). */
  tirer(ender: boolean, ouverts: number): Carte[] {
    const r = REGLES_EQUIPEMENT;
    const monde = this.etat.arene.monde;
    const reserve = IDS_OBJETS_EQUIPEMENT.filter((id) => {
      const def = OBJETS_EQUIPEMENT[id];
      return (def.monde === null || def.monde === monde) && !this.porte(id) && (ender || def.rarete !== 'legendaire');
    });
    const renforcables = this.portes().filter((o) => o.niveau < niveauMaxObjet(o.id));
    const pris = new Set<IdObjetEquipement>();
    const cartes: Carte[] = [];
    // Coffre de boss : la légendaire du monde, tant qu'on ne la porte pas
    if (ender) {
      const legendaire = reserve.find((id) => OBJETS_EQUIPEMENT[id].rarete === 'legendaire');
      if (legendaire) {
        pris.add(legendaire);
        cartes.push(this.carteObjet(legendaire));
      }
    }
    while (cartes.length < r.propositions) {
      const renfort = renforcables.find((o) => !pris.has(o.id));
      if (renfort && Math.random() < r.chanceRenfort) {
        pris.add(renfort.id);
        cartes.push(this.carteRenfort(renfort));
        continue;
      }
      const rarete = this.tirerRarete(ender, ouverts);
      let candidats = reserve.filter((id) => !pris.has(id) && OBJETS_EQUIPEMENT[id].rarete === rarete);
      // Plus rien de cette rareté : une autre rareté permise par ce coffre (rare au moins pour l'Ender), sinon n'importe laquelle
      const permises = Object.keys(ender ? r.ender.poids : r.bois.poids);
      if (candidats.length === 0) candidats = reserve.filter((id) => !pris.has(id) && permises.includes(OBJETS_EQUIPEMENT[id].rarete));
      if (candidats.length === 0) candidats = reserve.filter((id) => !pris.has(id));
      if (candidats.length === 0) {
        if (!renfort) break;
        pris.add(renfort.id);
        cartes.push(this.carteRenfort(renfort));
        continue;
      }
      const id = candidats[Math.floor(Math.random() * candidats.length)];
      pris.add(id);
      cartes.push(this.carteObjet(id));
    }
    if (cartes.length === 0) {
      cartes.push({
        amelioration: { type: 'emeraudes', nombre: r.emeraudesConsolation },
        rarete: 'action',
        nom: 'Sac d\'émeraudes',
        icone: OBJETS.emeraude.icone,
        etiquette: 'Tout est au maximum',
        description: `Plus rien à trouver : ${r.emeraudesConsolation} émeraudes.`,
        lignes: [],
      });
    }
    // Ordre mélangé (la légendaire n'est pas toujours la première)
    for (let i = cartes.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [cartes[i], cartes[j]] = [cartes[j], cartes[i]];
    }
    return cartes;
  }

  /** Cartes de la 2e étape : quel objet porté retirer pour prendre `id`, ou revenir. */
  cartesRemplacement(id: IdObjetEquipement): Carte[] {
    const def = OBJETS_EQUIPEMENT[id];
    const emplacements = this.emplacements(def.emplacement);
    const cartes: Carte[] = [];
    emplacements.forEach((o, index) => {
      if (!o) return;
      const ancien = OBJETS_EQUIPEMENT[o.id];
      cartes.push({
        amelioration: { type: 'remplacer', id, index },
        rarete: ancien.rarete,
        nom: ancien.nom,
        icone: cheminIconeObjet(o.id),
        etiquette: `Remplacer par ${def.nom}`,
        description: ancien.description(this.facteur(o)),
        lignes: this.lignes((liste) => {
          const copie = liste.slice();
          copie[liste.indexOf(o)] = { id, niveau: 1 };
          return copie;
        }),
      });
    });
    cartes.push({
      amelioration: { type: 'retour' },
      rarete: 'action',
      nom: 'Garder mes objets',
      icone: cheminIconeObjet(id),
      etiquette: 'Retour',
      description: `Ne pas prendre ${def.nom} : revenir aux objets du coffre.`,
      lignes: [],
    });
    return cartes;
  }

  appliquer(choix: ChoixEquipement): void {
    switch (choix.type) {
      case 'objet': {
        const emplacements = this.emplacements(OBJETS_EQUIPEMENT[choix.id].emplacement);
        const libre = emplacements.indexOf(null);
        if (libre < 0) return;
        emplacements[libre] = { id: choix.id, niveau: 1 };
        this.surDecouverte(choix.id);
        break;
      }
      case 'remplacer': {
        const emplacements = this.emplacements(OBJETS_EQUIPEMENT[choix.id].emplacement);
        if (choix.index < 0 || choix.index >= emplacements.length) return;
        emplacements[choix.index] = { id: choix.id, niveau: 1 };
        this.surDecouverte(choix.id);
        break;
      }
      case 'renfort': {
        const o = this.porte(choix.id);
        if (!o || o.niveau >= niveauMaxObjet(o.id)) return;
        o.niveau++;
        break;
      }
      case 'emeraudes':
        this.etat.emeraudes += choix.nombre;
        return;
      case 'retour':
        return;
    }
    this.recalculer();
  }

  /** Bonus et effets des objets portés, puis PV : les PV gagnés arrivent pleins, ceux perdus bornent les PV actuels. */
  private recalculer(): void {
    const j = this.etat.joueur;
    const pvMaxAvant = this.etat.modificateurs.pvMax;
    this.passifs.definirEquipement(this.bonus(this.portes()));
    this.passifs.calculer(this.etat.modificateurs);
    const gain = this.etat.modificateurs.pvMax - pvMaxAvant;
    j.pvMax += gain;
    j.pv = gain > 0 ? j.pv + gain : Math.min(j.pv, j.pvMax);
    this.arsenal.recalculer();
    this.effets = this.portes().flatMap((o) => {
      const effet = OBJETS_EQUIPEMENT[o.id].effet;
      return effet ? [{ effet, facteur: this.facteur(o) }] : [];
    });
    this.version++;
  }

  /** Bonus de chaque objet, multipliés par le facteur de son niveau. */
  private bonus(liste: readonly ObjetPorte[]): Partial<Modificateurs>[] {
    return liste.flatMap((o) => {
      const modif = OBJETS_EQUIPEMENT[o.id].modif;
      if (!modif) return [];
      const f = this.facteur(o);
      const sortie: Partial<Modificateurs> = {};
      for (const cle of Object.keys(modif) as (keyof Modificateurs)[]) sortie[cle] = (modif[cle] ?? 0) * f;
      return [sortie];
    });
  }

  /** Lignes « avant → après » si l'équipement devenait `modifier(portés)`. */
  private lignes(modifier: (liste: readonly ObjetPorte[]) => ObjetPorte[]): string[] {
    const portes = this.portes();
    const avant = this.passifs.calculer(creerModificateurs());
    this.passifs.definirEquipement(this.bonus(modifier(portes)));
    const apres = this.passifs.calculer(creerModificateurs());
    this.passifs.definirEquipement(this.bonus(portes));
    return lignesModificateurs(avant, apres);
  }

  private carteObjet(id: IdObjetEquipement): Carte {
    const def = OBJETS_EQUIPEMENT[id];
    const plein = this.remplacementNecessaire(id);
    return {
      amelioration: { type: 'objet', id },
      rarete: def.rarete,
      nom: def.nom,
      icone: cheminIconeObjet(id),
      etiquette: plein ? `${NOMS_EMPLACEMENTS[def.emplacement]} · remplace un objet` : `Nouvelle ${NOMS_EMPLACEMENTS[def.emplacement].toLowerCase()}`,
      description: def.description(1),
      // Emplacements pleins : les bonus dépendent de l'objet remplacé (2e étape)
      lignes: plein ? [] : this.lignes((liste) => [...liste, { id, niveau: 1 }]),
    };
  }

  private carteRenfort(o: ObjetPorte): Carte {
    const def = OBJETS_EQUIPEMENT[o.id];
    return {
      amelioration: { type: 'renfort', id: o.id },
      rarete: def.rarete,
      nom: def.nom,
      icone: cheminIconeObjet(o.id),
      etiquette: `Renforcer · niveau ${o.niveau} → ${o.niveau + 1}`,
      description: def.description(REGLES_EQUIPEMENT.niveaux[o.niveau]),
      lignes: this.lignes((liste) => liste.map((p) => (p === o ? { id: p.id, niveau: p.niveau + 1 } : p))),
    };
  }

  private tirerRarete(ender: boolean, ouverts: number): RareteObjet {
    const r = REGLES_EQUIPEMENT;
    const poids: Partial<Record<RareteObjet, number>> = {};
    if (ender) {
      Object.assign(poids, r.ender.poids);
    } else {
      for (const cle of Object.keys(r.bois.poids) as RareteObjet[]) {
        const valeur = (r.bois.poids[cle] ?? 0) + (r.bois.parCoffre[cle] ?? 0) * ouverts;
        poids[cle] = Math.min(r.bois.maximum[cle] ?? Infinity, Math.max(r.bois.minimum[cle] ?? 0, valeur));
      }
    }
    const entrees = Object.entries(poids) as [RareteObjet, number][];
    let seuil = Math.random() * entrees.reduce((t, [, p]) => t + p, 0);
    for (const [rarete, p] of entrees) {
      seuil -= p;
      if (seuil < 0) return rarete;
    }
    return entrees[0][0];
  }
}
