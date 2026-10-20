/**
 * Directeur d'apparition : fait venir des groupes d'ennemis hors du champ de
 * la caméra, de plus en plus nombreux et résistants (table APPARITIONS).
 */
import { CONFIG } from '../config';
import { APPARITIONS } from '../donnees/apparitions';
import type { IdEnnemi } from '../donnees/ennemis';
import type { Progression } from '../donnees/types';
import type { EtatPartie } from '../etat';
import type { Ennemis } from './ennemis';

/** Ce qui sait dire si un point est visible à l'écran. */
export interface DetecteurVisibilite {
  estVisible(x: number, z: number, rayon: number): boolean;
}

function interpoler(p: Progression, t: number): number {
  return p.debut + (p.fin - p.debut) * t;
}

export class DirecteurApparitions {
  private readonly etat: EtatPartie;
  private readonly ennemis: Ennemis;
  private readonly vue: DetecteurVisibilite;
  private minuterie = 0;
  // Résultat de trouverPoint (évite d'allouer un objet)
  private pointX = 0;
  private pointZ = 0;

  constructor(etat: EtatPartie, ennemis: Ennemis, vue: DetecteurVisibilite) {
    this.etat = etat;
    this.ennemis = ennemis;
    this.vue = vue;
  }

  maj(dt: number): void {
    if (this.etat.joueur.mort) return;
    const a = APPARITIONS;
    const t = Math.min(this.etat.tempsJeu / a.dureeMontee, 1);

    this.minuterie -= dt;
    if (this.minuterie > 0) return;
    this.minuterie += interpoler(a.intervalle, t);

    const places = Math.floor(interpoler(a.maxSimultanes, t)) - this.ennemis.actifs;
    if (places <= 0) return;
    const type = this.choisirType();
    if (!type) return;

    this.trouverPointHorsChamp();
    const taille = Math.min(Math.round(interpoler(a.tailleGroupe, t)), places);
    const pv = interpoler(a.multiplicateurPv, t);
    for (let i = 0; i < taille; i++) {
      const angle = Math.random() * Math.PI * 2;
      const r = Math.random() * a.dispersionGroupe;
      this.ennemis.apparaitre(type, this.pointX + Math.cos(angle) * r, this.pointZ + Math.sin(angle) * r, pv);
    }
  }

  /** Type tiré au hasard parmi ceux déjà débloqués, selon leur poids. */
  private choisirType(): IdEnnemi | null {
    const temps = this.etat.tempsJeu;
    let total = 0;
    for (const e of APPARITIONS.ennemis) if (e.depuis <= temps) total += e.poids;
    let seuil = Math.random() * total;
    for (const e of APPARITIONS.ennemis) {
      if (e.depuis > temps) continue;
      seuil -= e.poids;
      if (seuil < 0) return e.ennemi;
    }
    return null;
  }

  /**
   * Point de l'arène loin du joueur et hors champ. Si la caméra voit toute la
   * zone accessible, on garde le point le plus éloigné du joueur.
   */
  private trouverPointHorsChamp(): void {
    const j = this.etat.joueur;
    const limite = this.etat.arene.taille / 2 - 1;
    let meilleure = -1;
    for (let essai = 0; essai < CONFIG.ennemis.essaisApparition; essai++) {
      const x = (Math.random() * 2 - 1) * limite;
      const z = (Math.random() * 2 - 1) * limite;
      const distance = Math.hypot(x - j.x, z - j.z);
      if (distance < APPARITIONS.distanceMin) continue;
      if (!this.vue.estVisible(x, z, 2)) {
        this.pointX = x;
        this.pointZ = z;
        return;
      }
      if (distance > meilleure) {
        meilleure = distance;
        this.pointX = x;
        this.pointZ = z;
      }
    }
    if (meilleure < 0) {  // tous les essais trop proches : coin opposé au joueur
      this.pointX = j.x > 0 ? -limite : limite;
      this.pointZ = j.z > 0 ? -limite : limite;
    }
  }
}
