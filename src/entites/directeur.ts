/**
 * Directeur d'apparition : fait venir des groupes d'ennemis hors du champ de
 * la caméra, sur des cases libres d'où le joueur est atteignable, de plus en
 * plus nombreux et résistants. Le type d'ennemi dépend de la zone de l'arène
 * où le groupe apparaît.
 */
import { CONFIG } from '../config';
import { APPARITIONS } from '../donnees/apparitions';
import type { IdEnnemi } from '../donnees/ennemis';
import type { ChoixEnnemi, Progression } from '../donnees/types';
import type { EtatPartie } from '../etat';
import type { Terrain } from '../monde/navigation';
import { longueur } from '../utils/maths';
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
  private readonly terrain: Terrain;
  private readonly ennemis: Ennemis;
  private readonly vue: DetecteurVisibilite;
  private minuterie = 0;
  // Résultat de trouverPoint (évite d'allouer un objet)
  private pointX = 0;
  private pointZ = 0;

  constructor(etat: EtatPartie, terrain: Terrain, ennemis: Ennemis, vue: DetecteurVisibilite) {
    this.etat = etat;
    this.terrain = terrain;
    this.ennemis = ennemis;
    this.vue = vue;
  }

  maj(dt: number): void {
    if (this.etat.issue !== null) return;
    if (this.etat.joueur.mort) return;
    const a = APPARITIONS;
    const t = Math.min(this.etat.tempsJeu / a.dureeMontee, 1);

    this.minuterie -= dt;
    if (this.minuterie > 0) return;
    this.minuterie += interpoler(a.intervalle, t);

    const places = Math.floor(interpoler(a.maxSimultanes, t)) - this.ennemis.actifs;
    if (places <= 0 || !this.trouverPointHorsChamp()) return;
    const type = this.choisirType(this.populationEn(this.pointX, this.pointZ));
    if (!type) return;

    const taille = Math.min(Math.round(interpoler(a.tailleGroupe, t)), places);
    const pv = interpoler(a.multiplicateurPv, t);
    const carte = this.terrain.carte;
    for (let i = 0; i < taille; i++) {
      const angle = Math.random() * Math.PI * 2;
      const r = Math.random() * a.dispersionGroupe;
      let x = this.pointX + Math.cos(angle) * r;
      let z = this.pointZ + Math.sin(angle) * r;
      if (carte.bloqueMarche(x, z)) {
        x = this.pointX;
        z = this.pointZ;
      }
      this.ennemis.apparaitre(type, x, z, pv);
    }
  }

  /** Population de la première zone de l'arène qui contient le point, sinon celle de l'arène. */
  private populationEn(x: number, z: number): readonly ChoixEnnemi[] {
    for (const zone of this.etat.arene.zones) {
      const [x0, z0, x1, z1] = zone.rect;
      if (x >= x0 && x < x1 && z >= z0 && z < z1) return zone.ennemis;
    }
    return this.etat.arene.ennemis;
  }

  /** Type tiré au hasard parmi ceux déjà débloqués, selon leur poids. */
  private choisirType(population: readonly ChoixEnnemi[]): IdEnnemi | null {
    const temps = this.etat.tempsJeu;
    let total = 0;
    for (const e of population) if (e.depuis <= temps) total += e.poids;
    let seuil = Math.random() * total;
    for (const e of population) {
      if (e.depuis > temps) continue;
      seuil -= e.poids;
      if (seuil < 0) return e.ennemi;
    }
    return null;
  }

  /**
   * Case libre, loin du joueur, d'où il est atteignable, et hors champ. Si la
   * caméra voit toute la zone accessible, garde le point valable le plus éloigné.
   */
  private trouverPointHorsChamp(): boolean {
    const j = this.etat.joueur;
    const { carte, flux } = this.terrain;
    const limite = this.etat.arene.taille / 2 - 1;
    let meilleure = -1;
    for (let essai = 0; essai < CONFIG.ennemis.essaisApparition; essai++) {
      const x = (Math.random() * 2 - 1) * limite;
      const z = (Math.random() * 2 - 1) * limite;
      const distance = longueur(x - j.x, z - j.z);
      if (distance < APPARITIONS.distanceMin || carte.bloqueMarche(x, z) || !flux.atteignable(x, z)) continue;
      if (!this.vue.estVisible(x, z, 2)) {
        this.pointX = x;
        this.pointZ = z;
        return true;
      }
      if (distance > meilleure) {
        meilleure = distance;
        this.pointX = x;
        this.pointZ = z;
      }
    }
    return meilleure >= 0;
  }
}
