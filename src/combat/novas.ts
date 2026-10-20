/**
 * Mécanique « nova » (Nova de glace) : un anneau d'éclats de glace s'étend
 * autour du joueur ; chaque ennemi touché par l'anneau est blessé et ralenti
 * (une fois par nova). Dès le rang `solAuRang`, la nova laisse de la neige
 * poudreuse qui ralentit les ennemis qui la traversent.
 * Talent Zéro absolu : une nova sur cinq gèle (enracine) et fait double dégâts.
 */
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { ARMES, IDS_ARMES } from '../donnees/armes';
import { GERBES } from '../donnees/particules';
import type { CheminTexture } from '../donnees/types';
import { CubesTextures } from '../effets/cubesTextures';
import { longueur } from '../utils/maths';
import { Pool } from '../utils/pool';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';

interface Nova {
  arme: ArmeEquipee | null;
  x: number;
  z: number;
  temps: number;
  /** Laisse de la neige poudreuse (rang suffisant au lancer). */
  sol: boolean;
  prochainSol: number;
  /** Rotation des dalles de neige, tirée au lancer. */
  graine: number;
  /** Part des dégâts (Écho, Zéro absolu) et gel des ennemis touchés (Zéro absolu). */
  facteur: number;
  gel: boolean;
}

export class Novas {
  private readonly contexte: ContexteCombat;
  private readonly tampon = new Int32Array(256);
  private readonly novas: Pool<Nova>;
  private readonly eclats: CubesTextures;
  private readonly neige: CubesTextures;
  private readonly texture: CheminTexture;

  constructor(contexte: ContexteCombat, textures: BibliothequeTextures, scene: THREE.Scene) {
    const c = CONFIG.armes.nova;
    this.contexte = contexte;
    this.novas = new Pool<Nova>(c.capacite, () => ({ arme: null, x: 0, z: 0, temps: 0, sol: false, prochainSol: 0, graine: 0, facteur: 1, gel: false }));
    const m = IDS_ARMES.map((id) => ARMES[id].mecanique).find((mec) => mec.type === 'nova');
    this.texture = m?.type === 'nova' ? m.texture : 'block/packed_ice';
    this.eclats = new CubesTextures(scene, textures, this.texture, c.capacite * c.eclats, false, 'nova');
    this.neige = new CubesTextures(scene, textures, m?.type === 'nova' ? m.sol : 'block/powder_snow', c.capacite * c.dalles, false, 'neige');
  }

  /** Lance la nova autour du joueur ; faux si aucun ennemi n'est dans son rayon. */
  frapper(arme: ArmeEquipee): boolean {
    const m = arme.definition.mecanique;
    if (m.type !== 'nova') return false;
    const j = this.contexte.etat.joueur;
    if (this.contexte.ennemis.chercher(j.x, j.z, arme.stats.zone, this.tampon) === 0) return false;
    const n = this.novas.acquerir();
    if (!n) return false;
    n.arme = arme;
    n.x = j.x;
    n.z = j.z;
    n.temps = 0;
    n.sol = arme.rang >= m.solAuRang;
    n.prochainSol = 0;
    n.graine = Math.random() * 10;
    const talents = this.contexte.talents;
    n.gel = talents.novaGel;
    n.facteur = talents.facteurDeclenchement * (n.gel ? CONFIG.armes.nova.zeroAbsolu.degats : 1);
    return true;
  }

  maj(dt: number): void {
    const c = CONFIG.armes.nova;
    const { ennemis, particules, etat } = this.contexte;
    for (let i = this.novas.nombre - 1; i >= 0; i--) {
      const n = this.novas.elements[i];
      const arme = n.arme;
      const m = arme?.definition.mecanique;
      n.temps += dt;
      if (!arme || m?.type !== 'nova') {
        this.novas.liberer(i);
        continue;
      }
      const s = arme.stats;
      if (n.temps <= m.expansion + dt) {
        // L'anneau touche les ennemis qu'il dépasse
        const rayon = s.zone * Math.min(1, n.temps / m.expansion);
        const k = ennemis.chercher(n.x, n.z, rayon, this.tampon);
        for (let q = 0; q < k; q++) {
          const e = ennemis.obtenir(this.tampon[q]);
          if (e.immunites[arme.index] > etat.tempsJeu) continue;
          e.immunites[arme.index] = etat.tempsJeu + c.immunite;
          ennemis.toucher(e, arme, n.x, n.z, s.recul, n.facteur);
          if (n.gel) ennemis.enraciner(e, c.zeroAbsolu.gel);
          particules.emettre(e.x, 0.8, e.z, GERBES.gel, this.texture);
        }
      } else if (n.sol) {
        // Neige poudreuse : ralentit ceux qui la traversent
        n.prochainSol -= dt;
        if (n.prochainSol <= 0) {
          n.prochainSol = c.periodeSol;
          const k = ennemis.chercher(n.x, n.z, s.zone, this.tampon);
          for (let q = 0; q < k; q++) ennemis.ralentir(ennemis.obtenir(this.tampon[q]), s.ralenti, c.periodeSol * 2);
        }
      }
      if (n.temps >= m.expansion + c.eclatsRestent + (n.sol ? m.dureeSol : 0)) {
        n.arme = null;
        this.novas.liberer(i);
      }
    }
  }

  majVisuel(): void {
    const c = CONFIG.armes.nova;
    this.eclats.commencer();
    this.neige.commencer();
    for (let i = 0; i < this.novas.nombre; i++) {
      const n = this.novas.elements[i];
      const m = n.arme?.definition.mecanique;
      if (!n.arme || m?.type !== 'nova') continue;
      const zone = n.arme.stats.zone;
      const avance = Math.min(1, n.temps / m.expansion);
      // Éclats : anneau qui s'étend, puis rétrécit et disparaît
      const apres = n.temps - m.expansion;
      const taille = apres > 0 ? c.tailleEclat * Math.max(0, 1 - apres / c.eclatsRestent) : c.tailleEclat;
      const rayon = zone * (1 - (1 - avance) * (1 - avance));
      for (let k = 0; k < c.eclats; k++) {
        const a = (k / c.eclats) * Math.PI * 2;
        const ondule = 0.15 * Math.sin(k * 2.3 + n.graine);
        this.eclats.ajouter(n.x + Math.sin(a) * (rayon + ondule), 0.35 + 0.2 * Math.sin(k * 1.7), n.z + Math.cos(a) * (rayon + ondule), taille, a + n.temps * 4, k * 0.6);
      }
      // Neige poudreuse : dalles éparses dans le disque, qui fondent à la fin
      if (n.sol && apres > 0) {
        const fonte = Math.min(1, (m.dureeSol + c.eclatsRestent - apres) / 0.4);
        for (let k = 0; k < c.dalles; k++) {
          const a = k * 2.399 + n.graine;
          const r = zone * Math.sqrt((k + 0.5) / c.dalles);
          const x = n.x + Math.sin(a) * r;
          const z = n.z + Math.cos(a) * r;
          if (longueur(x - n.x, z - n.z) > zone) continue;
          this.neige.ajouterDalle(x, z, c.tailleDalle * Math.max(0, fonte), 0.06, a);
        }
      }
    }
    this.eclats.terminer();
    this.neige.terminer();
  }
}
