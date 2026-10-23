/**
 * Collection (gardée d'une partie à l'autre) : pièces d'armure obtenues
 * (donnees/armures.ts), pièce portée par emplacement, et objets de coffre déjà
 * trouvés. Clé de stockage à part (`block-survivors.collection`) : la
 * sauvegarde de progression n'est pas touchée, et l'export/import chiffré la
 * prend avec le reste. Chaque changement est enregistré aussitôt.
 */
import { ARMURES, MONDES, PIECES_ARMURE } from '../donnees/armures';
import { IDS_OBJETS_EQUIPEMENT, OBJETS_EQUIPEMENT, type IdObjetEquipement } from '../donnees/equipement';
import type { IdMonde, Modificateurs, PieceArmure } from '../donnees/types';
import { ecrireStockage, lireStockage } from '../options';

const CLE = 'block-survivors.collection';
const VERSION = 1;

interface Donnees {
  version: number;
  /** Pièces obtenues, « monde:pièce ». */
  pieces: string[];
  /** Monde de la pièce portée à chaque emplacement. */
  portees: Partial<Record<PieceArmure, IdMonde>>;
  /** Objets de coffre déjà trouvés. */
  decouverts: string[];
}

/** Où en est un monde de la Collection. */
export interface AvanceeMonde {
  readonly pieces: number;
  readonly objets: number;
  readonly totalObjets: number;
  /** 0 à 1. */
  readonly part: number;
}

export class Collection {
  private readonly donnees: Donnees;

  private constructor(donnees: Donnees) {
    this.donnees = donnees;
  }

  /** Lecture tolérante : ce qui est illisible est ignoré, rien n'efface le reste. */
  static charger(): Collection {
    const lu = lireStockage<Partial<Donnees>>(CLE) ?? {};
    const textes = (v: unknown) => (Array.isArray(v) ? v.filter((x): x is string => typeof x === 'string') : []);
    const portees: Partial<Record<PieceArmure, IdMonde>> = {};
    if (lu.portees && typeof lu.portees === 'object') {
      for (const piece of PIECES_ARMURE) {
        const monde = (lu.portees as Record<string, unknown>)[piece];
        if (typeof monde === 'string' && (MONDES as readonly string[]).includes(monde)) portees[piece] = monde as IdMonde;
      }
    }
    const collection = new Collection({ version: VERSION, pieces: textes(lu.pieces), portees, decouverts: textes(lu.decouverts) });
    // Une pièce portée doit être possédée
    for (const piece of PIECES_ARMURE) {
      const monde = collection.donnees.portees[piece];
      if (monde && !collection.possede(monde, piece)) delete collection.donnees.portees[piece];
    }
    return collection;
  }

  possede(monde: IdMonde, piece: PieceArmure): boolean {
    return this.donnees.pieces.includes(`${monde}:${piece}`);
  }

  /** Monde de la pièce portée à cet emplacement (null : rien). */
  portee(piece: PieceArmure): IdMonde | null {
    return this.donnees.portees[piece] ?? null;
  }

  /** Monde de chaque pièce portée (copie). */
  portees(): Partial<Record<PieceArmure, IdMonde>> {
    return { ...this.donnees.portees };
  }

  /** Porte la pièce de ce monde (null : retire). Sans effet si elle n'est pas possédée. */
  porter(piece: PieceArmure, monde: IdMonde | null): void {
    if (monde && !this.possede(monde, piece)) return;
    if (monde) this.donnees.portees[piece] = monde;
    else delete this.donnees.portees[piece];
    this.enregistrer();
  }

  /** Une pièce manquante de ce monde, au hasard, obtenue et portée si l'emplacement est libre ; null si tout est déjà là. */
  obtenirPiece(monde: IdMonde): PieceArmure | null {
    const manquantes = PIECES_ARMURE.filter((p) => !this.possede(monde, p));
    if (manquantes.length === 0) return null;
    const piece = manquantes[Math.floor(Math.random() * manquantes.length)];
    this.donnees.pieces.push(`${monde}:${piece}`);
    if (!this.donnees.portees[piece]) this.donnees.portees[piece] = monde;
    this.enregistrer();
    return piece;
  }

  /** Debug : donne une pièce précise. */
  donnerPiece(monde: IdMonde, piece: PieceArmure): void {
    if (this.possede(monde, piece)) return;
    this.donnees.pieces.push(`${monde}:${piece}`);
    this.enregistrer();
  }

  decouvert(id: IdObjetEquipement): boolean {
    return this.donnees.decouverts.includes(id);
  }

  /** Premier objet de ce type trouvé : inscrit dans la Collection. */
  decouvrir(id: IdObjetEquipement): void {
    if (this.decouvert(id)) return;
    this.donnees.decouverts.push(id);
    this.enregistrer();
  }

  /** Bonus des pièces portées, plus celui de l'ensemble si les 4 sont du même monde. */
  bonus(): Partial<Modificateurs>[] {
    const liste: Partial<Modificateurs>[] = [];
    for (const piece of PIECES_ARMURE) {
      const monde = this.portee(piece);
      if (monde) liste.push(ARMURES[monde].pieces[piece].modif);
    }
    const ensemble = this.ensembleComplet();
    if (ensemble) liste.push(ARMURES[ensemble].ensemble.modif);
    return liste;
  }

  /** Monde dont les 4 pièces sont portées, ou null. */
  ensembleComplet(): IdMonde | null {
    const premier = this.portee(PIECES_ARMURE[0]);
    return premier && PIECES_ARMURE.every((p) => this.portee(p) === premier) ? premier : null;
  }

  /** Pièces et objets trouvés dans ce monde (objets de tous les mondes non comptés). */
  avancee(monde: IdMonde): AvanceeMonde {
    const pieces = PIECES_ARMURE.filter((p) => this.possede(monde, p)).length;
    const ids = IDS_OBJETS_EQUIPEMENT.filter((id) => OBJETS_EQUIPEMENT[id].monde === monde);
    const objets = ids.filter((id) => this.decouvert(id)).length;
    return { pieces, objets, totalObjets: ids.length, part: (pieces + objets) / (PIECES_ARMURE.length + ids.length) };
  }

  private enregistrer(): void {
    ecrireStockage(CLE, this.donnees);
  }
}
