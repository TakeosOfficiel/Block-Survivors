/**
 * Mécanique « tourbillon » (Tourbillon de lames) : le joueur tourne sur
 * lui-même et frappe tous les ennemis autour de lui toutes les `intervalle`
 * secondes, pendant la durée de l'arme. Trois balayages vanilla tournent
 * autour de lui. Avec un recul négatif, il aspire les ennemis.
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { ARMES, IDS_ARMES } from '../donnees/armes';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';

/** Balayages affichés autour du joueur. */
const LAMES = 3;

export class Tourbillons {
  private readonly contexte: ContexteCombat;
  private readonly textures: BibliothequeTextures;
  private readonly tampon = new Int32Array(128);
  private arme: ArmeEquipee | null = null;
  private temps = 0;
  private prochainCoup = 0;
  /** Frappes restantes : une au départ, puis une par intervalle pendant la durée. */
  private coupsRestants = 0;
  private readonly effets: THREE.Mesh<THREE.PlaneGeometry, THREE.MeshBasicMaterial>[];
  private readonly materiau: THREE.MeshBasicMaterial;

  constructor(contexte: ContexteCombat, textures: BibliothequeTextures, scene: THREE.Scene) {
    this.contexte = contexte;
    this.textures = textures;
    const premiere = IDS_ARMES.map((id) => ARMES[id].mecanique).find((m) => m.type === 'tourbillon');
    const map = premiere?.type === 'tourbillon' ? textures.obtenir(premiere.effet[0]) : null;
    this.materiau = new THREE.MeshBasicMaterial({ map, transparent: true, depthWrite: false, side: THREE.DoubleSide });
    const geometrie = new THREE.PlaneGeometry(1, 1);
    geometrie.rotateX(-Math.PI / 2);
    this.effets = Array.from({ length: LAMES }, () => {
      const effet = new THREE.Mesh(geometrie, this.materiau);
      effet.name = 'tourbillon';
      effet.visible = false;
      scene.add(effet);
      return effet;
    });
  }

  /** Lance la rotation ; faux s'il n'y a aucun ennemi assez près. */
  frapper(arme: ArmeEquipee): boolean {
    if (arme.definition.mecanique.type !== 'tourbillon') return false;
    const j = this.contexte.etat.joueur;
    if (this.contexte.ennemis.chercher(j.x, j.z, arme.stats.zone * CONFIG.armes.tourbillon.declenchement, this.tampon) === 0) return false;
    this.arme = arme;
    this.temps = 0;
    this.prochainCoup = 0;
    this.coupsRestants = 1 + Math.floor(arme.stats.duree / arme.definition.mecanique.intervalle + 1e-6);
    return true;
  }

  maj(dt: number): void {
    const arme = this.arme;
    const m = arme?.definition.mecanique;
    if (!arme || m?.type !== 'tourbillon') return;
    const { etat, ennemis, retours } = this.contexte;
    const j = etat.joueur;
    this.temps += dt;
    this.prochainCoup -= dt;
    while (this.prochainCoup <= 0 && this.coupsRestants > 0) {
      this.prochainCoup += m.intervalle;
      this.coupsRestants--;
      const n = ennemis.chercher(j.x, j.z, arme.stats.zone, this.tampon);
      for (let k = 0; k < n; k++) ennemis.toucher(ennemis.obtenir(this.tampon[k]), arme, j.x, j.z);
      retours.son('balayage', 1.1);
      this.contexte.declencherGesteJoueur(this.temps * CONFIG.armes.tourbillon.vitesseRotation);
    }
    if (this.coupsRestants === 0 && this.prochainCoup <= 0) this.arme = null;
  }

  majVisuel(): void {
    const arme = this.arme;
    const m = arme?.definition.mecanique;
    if (!arme || m?.type !== 'tourbillon') {
      for (const effet of this.effets) effet.visible = false;
      return;
    }
    const j = this.contexte.etat.joueur;
    const s = arme.stats;
    // Image du balayage selon l'avancée de la frappe en cours
    const avance = 1 - Math.max(0, this.prochainCoup) / m.intervalle;
    const image = this.textures.obtenir(m.effet[Math.min(m.effet.length - 1, Math.floor(avance * m.effet.length))]);
    if (this.materiau.map !== image) {
      this.materiau.map = image;
      this.materiau.needsUpdate = true;
    }
    const angle = this.temps * CONFIG.armes.tourbillon.vitesseRotation;
    for (let k = 0; k < LAMES; k++) {
      const effet = this.effets[k];
      const a = angle + (k * Math.PI * 2) / LAMES;
      effet.visible = image !== null;
      effet.position.set(j.x + Math.sin(a) * s.zone * 0.55, 0.6, j.z + Math.cos(a) * s.zone * 0.55);
      effet.rotation.y = a + Math.PI / 2;
      effet.scale.setScalar(s.zone * 1.2);
    }
  }
}
