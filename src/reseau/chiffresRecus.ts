/**
 * Chiffres de dégâts dans les instantanés : l'hôte retient ceux qu'il affiche
 * d'un envoi à l'autre et envoie à chaque joueur ceux qui l'entourent ; le
 * joueur les affiche tels quels (les vrais dégâts, décidés par l'hôte).
 */
import { REGLES_MULTIJOUEUR } from '../donnees/multijoueur';
import type { ChiffresDegats, StyleChiffre } from '../effets/chiffresDegats';
import { STYLES_CHIFFRES, type Ecrivain, type Lecteur } from './protocole';

/** Chiffres retenus d'un envoi à l'autre. */
const CAPACITE = 256;

/** Hôte : les chiffres affichés depuis le dernier envoi. */
export class ChiffresEnvoyes {
  /** x, y, z, valeur, style. */
  private readonly valeurs = new Float32Array(CAPACITE * 5);
  private nombre = 0;

  ajouter(x: number, y: number, z: number, valeur: number, style: StyleChiffre): void {
    if (this.nombre >= CAPACITE) return;
    const o = this.nombre++ * 5;
    this.valeurs[o] = x;
    this.valeurs[o + 1] = y;
    this.valeurs[o + 2] = z;
    this.valeurs[o + 3] = valeur;
    this.valeurs[o + 4] = STYLES_CHIFFRES.indexOf(style);
  }

  /** Ceux à moins de √r2 de (cx, cz), au plus `chiffresMax`. */
  ecrire(w: Ecrivain, cx: number, cz: number, r2: number): void {
    const compte = w.reserver16();
    let n = 0;
    for (let i = 0; i < this.nombre && n < REGLES_MULTIJOUEUR.reseau.chiffresMax && w.reste > 64; i++) {
      const o = i * 5;
      if ((this.valeurs[o] - cx) ** 2 + (this.valeurs[o + 2] - cz) ** 2 > r2) continue;
      w.coordonnee(this.valeurs[o]);
      w.u8(Math.min(255, Math.round(this.valeurs[o + 1] * 16)));
      w.coordonnee(this.valeurs[o + 2]);
      w.f32(this.valeurs[o + 3]);
      w.u8(this.valeurs[o + 4]);
      n++;
    }
    w.ecrire16(compte, n);
  }

  /** Après l'envoi à tous les joueurs. */
  vider(): void {
    this.nombre = 0;
  }
}

/** Joueur : affiche les chiffres de l'instantané. */
export function lireChiffres(l: Lecteur, chiffres: ChiffresDegats): void {
  const n = l.u16();
  for (let i = 0; i < n; i++) {
    const x = l.coordonnee();
    const y = l.u8() / 16;
    const z = l.coordonnee();
    const valeur = l.f32();
    const style = STYLES_CHIFFRES[l.u8()] ?? 'normal';
    chiffres.ajouter(x, y, z, valeur, style);
  }
}
