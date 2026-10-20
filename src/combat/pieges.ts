/**
 * Mécanique « piège » (Piège de ronces) : des buissons de baies sucrées posés
 * sous le joueur, qui blessent et ralentissent les ennemis à l'intérieur
 * toutes les `intervalle` secondes, comme le buisson du jeu. Au-delà du
 * nombre de pièges de l'arme, le plus ancien est remplacé. Dès le rang
 * `enracineAuRang`, un ennemi qui entre est enraciné un instant.
 */
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { ARMES, IDS_ARMES } from '../donnees/armes';
import { GERBES } from '../donnees/particules';
import type { CheminTexture } from '../donnees/types';
import { PlansCroises } from '../effets/plansCroises';
import { Pool } from '../utils/pool';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';

interface Piege {
  arme: ArmeEquipee | null;
  x: number;
  z: number;
  temps: number;
  prochainTic: number;
  graine: number;
}

export class Pieges {
  private readonly contexte: ContexteCombat;
  private readonly tampon = new Int32Array(128);
  private readonly pieges: Pool<Piege>;
  private readonly buissons: PlansCroises;
  private readonly image: CheminTexture;

  constructor(contexte: ContexteCombat, textures: BibliothequeTextures, scene: THREE.Scene) {
    const c = CONFIG.armes.piege;
    this.contexte = contexte;
    this.pieges = new Pool<Piege>(c.capacite, () => ({ arme: null, x: 0, z: 0, temps: 0, prochainTic: 0, graine: 0 }));
    const m = IDS_ARMES.map((id) => ARMES[id].mecanique).find((mec) => mec.type === 'piege');
    this.image = m?.type === 'piege' ? m.image : 'block/sweet_berry_bush_stage3';
    this.buissons = new PlansCroises(scene, textures, this.image, c.capacite * c.buissons, 'pièges');
  }

  /** Pose un piège sous le joueur (remplace le plus ancien au-delà du nombre) ; faux sans ennemi en vue. */
  frapper(arme: ArmeEquipee): boolean {
    if (arme.definition.mecanique.type !== 'piege') return false;
    const { etat, ennemis, particules } = this.contexte;
    const j = etat.joueur;
    if (!ennemis.plusProche(j.x, j.z, CONFIG.armes.piege.recherche)) return false;
    // Remplace le plus ancien piège de l'arme s'ils sont déjà tous posés
    let poses = 0;
    let ancien = -1;
    for (let i = 0; i < this.pieges.nombre; i++) {
      const p = this.pieges.elements[i];
      if (p.arme !== arme) continue;
      poses++;
      if (ancien < 0 || p.temps > this.pieges.elements[ancien].temps) ancien = i;
    }
    if (poses >= arme.stats.nombre && ancien >= 0) this.retirer(ancien);
    const p = this.pieges.acquerir();
    if (!p) return false;
    p.arme = arme;
    p.x = j.x;
    p.z = j.z;
    p.temps = 0;
    p.prochainTic = 0;
    p.graine = Math.random() * 10;
    particules.emettre(p.x, 0.3, p.z, GERBES.ronces, this.image);
    return true;
  }

  maj(dt: number): void {
    const c = CONFIG.armes.piege;
    const { ennemis, etat } = this.contexte;
    for (let i = this.pieges.nombre - 1; i >= 0; i--) {
      const p = this.pieges.elements[i];
      const arme = p.arme;
      const m = arme?.definition.mecanique;
      p.temps += dt;
      if (!arme || m?.type !== 'piege' || p.temps >= arme.stats.duree) {
        this.retirer(i);
        continue;
      }
      p.prochainTic -= dt;
      if (p.prochainTic > 0) continue;
      p.prochainTic += m.intervalle;
      const s = arme.stats;
      const n = ennemis.chercher(p.x, p.z, s.zone, this.tampon);
      for (let k = 0; k < n; k++) {
        const e = ennemis.obtenir(this.tampon[k]);
        ennemis.blesser(e, s.degats, p.x, p.z, 0, arme.index);
        ennemis.ralentir(e, s.ralenti, m.intervalle * 1.5);
        // Enracine à l'entrée, une fois par passage
        if (arme.rang >= m.enracineAuRang && e.immunites[arme.index] <= etat.tempsJeu) {
          ennemis.enraciner(e, c.enracinement);
          e.immunites[arme.index] = etat.tempsJeu + c.delaiEnracinement;
        }
      }
    }
  }

  majVisuel(): void {
    const c = CONFIG.armes.piege;
    this.buissons.commencer();
    for (let i = 0; i < this.pieges.nombre; i++) {
      const p = this.pieges.elements[i];
      if (!p.arme) continue;
      const zone = p.arme.stats.zone;
      // Pousse à la pose, se fane à la fin
      const croissance = Math.min(1, p.temps / 0.3, (p.arme.stats.duree - p.temps) / 0.5);
      for (let k = 0; k < c.buissons; k++) {
        const a = k * 2.399 + p.graine;
        const r = zone * 0.75 * Math.sqrt((k + 0.5) / c.buissons);
        const taille = c.taille * croissance;
        this.buissons.ajouter(p.x + Math.sin(a) * r, taille / 2, p.z + Math.cos(a) * r, taille, a);
      }
    }
    this.buissons.terminer();
  }

  private retirer(i: number): void {
    this.pieges.elements[i].arme = null;
    this.pieges.liberer(i);
  }
}
