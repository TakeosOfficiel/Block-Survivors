/**
 * Protocole du multijoueur.
 *
 * Canal `fiable` : messages rares en JSON (salon, cartes, fin de partie).
 * Canal `rapide` : messages binaires fréquents (entrées des joueurs,
 * instantanés de l'hôte), écrits et lus dans des tampons préalloués.
 * Positions des entités en int16 au 1/64 de bloc, angles en int16.
 */
import type { IdPersonnage } from '../donnees/personnages';
import type { IssuePartie, StatistiquesPartie } from '../etat';
import type { StyleChiffre } from '../effets/chiffresDegats';
import type { Amelioration } from '../progression/ameliorations';
import type { BonusTalents } from '../progression/sauvegarde';
import type { ChoixSkin } from '../profil';

/** Ce que l'hôte sait d'un joueur du salon (index 0 : l'hôte). */
export interface InfoJoueur {
  readonly index: number;
  readonly pseudo: string;
  readonly personnage: IdPersonnage;
  readonly skin: ChoixSkin;
  readonly brasFins: boolean;
  /** Bonus permanents (sauvegarde multi du joueur) : chacun simule les armes des autres pour leurs effets. */
  readonly bonus: BonusTalents;
}

/** Bilan commun envoyé à la fin de la partie (les tableaux typés en listes). */
export interface StatistiquesReseau {
  readonly tuesParType: number[];
  readonly degatsParArme: number[];
  readonly coupsParArme: number[];
  readonly critiquesParArme: number[];
  readonly plusGrosCoupParArme: number[];
  readonly tuesParArme: number[];
  readonly acquisitionArme: number[];
  readonly degatsSubis: number;
  readonly pvSoignes: number;
  readonly xpTotale: number;
  readonly emeraudesRamassees: number;
  readonly coffresOuverts: number;
  readonly bossVaincus: number;
}

export type MessageFiable =
  /** Joueur → hôte, dès la connexion : qui il est et ses bonus permanents (sauvegarde multi). */
  | { type: 'bonjour'; version: string; pseudo: string; personnage: IdPersonnage; skin: ChoixSkin; brasFins: boolean; bonus: BonusTalents }
  | { type: 'accueil'; index: number }
  | { type: 'refus'; raison: string }
  /** Hôte → tous : composition du salon. */
  | { type: 'salon'; arene: string; joueurs: InfoJoueur[] }
  /** Hôte → tous : chacun charge l'arène, puis répond `pret`. */
  | { type: 'lancer'; arene: string; joueurs: InfoJoueur[] }
  | { type: 'pret' }
  /** Hôte → tous : tout le monde est prêt, la partie commence. */
  | { type: 'depart' }
  /** Joueur → hôte : carte choisie (montée de niveau ou coffre). */
  | { type: 'carte'; amelioration: Amelioration }
  /** Hôte → tous : la carte choisie par le joueur `index` (chacun garde l'arsenal des autres à jour). */
  | { type: 'carteJouee'; index: number; amelioration: Amelioration }
  /** Hôte → tous : un coffre ouvert, des cartes pour tout le monde. */
  | { type: 'coffre'; nombre: number; titre: string }
  /** Hôte → un joueur : son bilan (compteur de dégâts), régulièrement et à la fin. */
  | { type: 'stats'; stats: StatistiquesReseau }
  | { type: 'fin'; issue: Exclude<IssuePartie, null>; stats: StatistiquesReseau }
  /** Joueur → hôte : « Continuer sans fin » après la victoire ; hôte → tous : la partie reprend. */
  | { type: 'continuer' }
  | { type: 'reprise' };

/** Premier octet des messages `rapide`. */
export const TYPE_ENTREE = 1;
export const TYPE_INSTANTANE = 2;

/** Bits des touches de déplacement dans une entrée. */
export const BITS_ACTIONS = { haut: 1, bas: 2, gauche: 4, droite: 8 } as const;

/** Drapeaux d'un joueur dans un instantané. */
export const JOUEUR_MORT = 1;
export const JOUEUR_PROTEGE = 2;
/** Choisit ses cartes : protégé, ignoré des ennemis, ses armes au repos. */
export const JOUEUR_EN_CHOIX = 4;

/** Drapeaux d'un ennemi dans un instantané. */
export const ENNEMI_MORT = 1;
export const ENNEMI_BLESSE = 2;
export const ENNEMI_BRULURE = 4;
export const ENNEMI_POISON = 8;
export const ENNEMI_SAIGNEMENT = 16;
export const ENNEMI_RALENTI = 32;

/** Styles des chiffres de dégâts, par index. */
export const STYLES_CHIFFRES: readonly StyleChiffre[] = ['normal', 'critique', 'feu', 'poison'];

/** Positions : 1/64 de bloc (±512 blocs en int16). */
const ECHELLE_POSITION = 64;
const ECHELLE_ANGLE = 10000;

/** Écriture séquentielle dans un tampon préalloué. */
export class Ecrivain {
  readonly octets: Uint8Array<ArrayBuffer>;
  private readonly vue: DataView;
  position = 0;

  constructor(taille: number) {
    this.octets = new Uint8Array(taille);
    this.vue = new DataView(this.octets.buffer);
  }

  /** Place restante (octets). */
  get reste(): number {
    return this.octets.length - this.position;
  }

  /** Les octets écrits (vue sur le tampon, sans copie). */
  contenu(): Uint8Array<ArrayBuffer> {
    return this.octets.subarray(0, this.position);
  }

  u8(v: number): void {
    this.vue.setUint8(this.position, v);
    this.position += 1;
  }

  u16(v: number): void {
    this.vue.setUint16(this.position, v);
    this.position += 2;
  }

  u32(v: number): void {
    this.vue.setUint32(this.position, v);
    this.position += 4;
  }

  f32(v: number): void {
    this.vue.setFloat32(this.position, v);
    this.position += 4;
  }

  /** Position au sol (blocs), quantifiée. */
  coordonnee(v: number): void {
    this.vue.setInt16(this.position, Math.max(-32768, Math.min(32767, Math.round(v * ECHELLE_POSITION))));
    this.position += 2;
  }

  /** Angle (rad), ramené dans [−π, π]. */
  angle(v: number): void {
    this.vue.setInt16(this.position, Math.round(Math.atan2(Math.sin(v), Math.cos(v)) * ECHELLE_ANGLE));
    this.position += 2;
  }

  /** Nombre de 0 à 1 sur un octet. */
  part(v: number): void {
    this.u8(Math.max(0, Math.min(255, Math.round(v * 255))));
  }

  /** Réserve un u16 à écrire plus tard (compte d'une liste) ; renvoie sa position. */
  reserver16(): number {
    const p = this.position;
    this.position += 2;
    return p;
  }

  ecrire16(position: number, v: number): void {
    this.vue.setUint16(position, v);
  }
}

/** Lecture séquentielle d'un message reçu. */
export class Lecteur {
  private vue = new DataView(new ArrayBuffer(0));
  position = 0;

  lire(donnees: ArrayBuffer): void {
    this.vue = new DataView(donnees);
    this.position = 0;
  }

  get fin(): boolean {
    return this.position >= this.vue.byteLength;
  }

  u8(): number {
    return this.vue.getUint8(this.position++);
  }

  u16(): number {
    const v = this.vue.getUint16(this.position);
    this.position += 2;
    return v;
  }

  u32(): number {
    const v = this.vue.getUint32(this.position);
    this.position += 4;
    return v;
  }

  f32(): number {
    const v = this.vue.getFloat32(this.position);
    this.position += 4;
    return v;
  }

  coordonnee(): number {
    const v = this.vue.getInt16(this.position);
    this.position += 2;
    return v / ECHELLE_POSITION;
  }

  angle(): number {
    const v = this.vue.getInt16(this.position);
    this.position += 2;
    return v / ECHELLE_ANGLE;
  }

  part(): number {
    return this.u8() / 255;
  }
}

/** Bilan d'un joueur, en listes (JSON). */
export function statistiquesReseau(s: StatistiquesPartie): StatistiquesReseau {
  return {
    tuesParType: Array.from(s.tuesParType),
    degatsParArme: Array.from(s.degatsParArme),
    coupsParArme: Array.from(s.coupsParArme),
    critiquesParArme: Array.from(s.critiquesParArme),
    plusGrosCoupParArme: Array.from(s.plusGrosCoupParArme),
    tuesParArme: Array.from(s.tuesParArme),
    acquisitionArme: Array.from(s.acquisitionArme),
    degatsSubis: s.degatsSubis,
    pvSoignes: s.pvSoignes,
    xpTotale: s.xpTotale,
    emeraudesRamassees: s.emeraudesRamassees,
    coffresOuverts: s.coffresOuverts,
    bossVaincus: s.bossVaincus,
  };
}

/** Recopie un bilan reçu dans celui de la partie. */
export function appliquerStatistiques(s: StatistiquesPartie, r: StatistiquesReseau): void {
  s.tuesParType.set(r.tuesParType);
  s.degatsParArme.set(r.degatsParArme);
  s.coupsParArme.set(r.coupsParArme);
  s.critiquesParArme.set(r.critiquesParArme);
  s.plusGrosCoupParArme.set(r.plusGrosCoupParArme);
  s.tuesParArme.set(r.tuesParArme);
  s.acquisitionArme.set(r.acquisitionArme);
  s.degatsSubis = r.degatsSubis;
  s.pvSoignes = r.pvSoignes;
  s.xpTotale = r.xpTotale;
  s.emeraudesRamassees = r.emeraudesRamassees;
  s.coffresOuverts = r.coffresOuverts;
  s.bossVaincus = r.bossVaincus;
}
