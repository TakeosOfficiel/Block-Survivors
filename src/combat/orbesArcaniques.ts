/**
 * Mécanique « orbes » (Orbes arcaniques) : des cubes d'améthyste tournent
 * autour du joueur pendant la durée de l'arme (dès le rang `contactAuRang`,
 * ils blessent au contact), puis foncent chacun sur un ennemi, à tête
 * chercheuse : ils changent de cible si la leur meurt en route.
 */
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { ARMES, IDS_ARMES } from '../donnees/armes';
import { GERBES } from '../donnees/particules';
import { CubesTextures } from '../effets/cubesTextures';
import type { Ennemi } from '../entites/ennemi';
import { longueur } from '../utils/maths';
import { Pool } from '../utils/pool';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';

interface Orbe {
  arme: ArmeEquipee | null;
  x: number;
  z: number;
  /** Place sur l'orbite (rad) et temps écoulé. */
  angle: number;
  temps: number;
  /** En vol vers sa cible (sinon en orbite). */
  lance: boolean;
  cible: Ennemi | null;
  cibleId: number;
  prochaineLueur: number;
}

export class OrbesArcaniques {
  private readonly contexte: ContexteCombat;
  private readonly tampon = new Int32Array(64);
  private readonly orbes: Pool<Orbe>;
  private readonly cubes: CubesTextures;

  constructor(contexte: ContexteCombat, textures: BibliothequeTextures, scene: THREE.Scene) {
    const c = CONFIG.armes.orbes;
    this.contexte = contexte;
    this.orbes = new Pool<Orbe>(c.capacite, () => ({
      arme: null, x: 0, z: 0, angle: 0, temps: 0, lance: false, cible: null, cibleId: 0, prochaineLueur: 0,
    }));
    const m = IDS_ARMES.map((id) => ARMES[id].mecanique).find((mec) => mec.type === 'orbes');
    this.cubes = new CubesTextures(scene, textures, m?.type === 'orbes' ? m.texture : 'block/amethyst_block', c.capacite, true, 'orbes');
  }

  /** Fait naître les orbes autour du joueur ; faux si aucun ennemi n'est à portée. */
  frapper(arme: ArmeEquipee): boolean {
    if (arme.definition.mecanique.type !== 'orbes') return false;
    const j = this.contexte.etat.joueur;
    if (!this.contexte.ennemis.plusProche(j.x, j.z, arme.stats.portee)) return false;
    const n = arme.stats.nombre;
    const depart = Math.random() * Math.PI * 2;
    for (let k = 0; k < n; k++) {
      const o = this.orbes.acquerir();
      if (!o) break;
      o.arme = arme;
      o.angle = depart + (k / n) * Math.PI * 2;
      o.temps = 0;
      o.lance = false;
      o.cible = null;
      o.cibleId = 0;
      o.prochaineLueur = 0;
      o.x = j.x + Math.sin(o.angle) * arme.stats.zone;
      o.z = j.z + Math.cos(o.angle) * arme.stats.zone;
    }
    return true;
  }

  maj(dt: number): void {
    const c = CONFIG.armes.orbes;
    const { etat, ennemis, particules } = this.contexte;
    const j = etat.joueur;
    for (let i = this.orbes.nombre - 1; i >= 0; i--) {
      const o = this.orbes.elements[i];
      const arme = o.arme;
      const m = arme?.definition.mecanique;
      o.temps += dt;
      if (!arme || m?.type !== 'orbes' || o.temps > arme.stats.duree + c.volMax) {
        o.arme = null;
        this.orbes.liberer(i);
        continue;
      }
      const s = arme.stats;
      o.prochaineLueur -= dt;
      if (o.prochaineLueur <= 0) {
        o.prochaineLueur = c.periodeLueur;
        particules.emettre(o.x, c.hauteur, o.z, GERBES.arcane);
      }

      if (!o.lance) {
        // Orbite autour du joueur
        const a = o.angle + o.temps * c.vitesseOrbite;
        o.x = j.x + Math.sin(a) * s.zone;
        o.z = j.z + Math.cos(a) * s.zone;
        if (arme.rang >= m.contactAuRang) {
          const n = ennemis.chercher(o.x, o.z, c.rayonContact, this.tampon);
          for (let k = 0; k < n; k++) ennemis.blesserAvecDelai(ennemis.obtenir(this.tampon[k]), arme.index, c.delaiContact, s.degats * m.contact, o.x, o.z, 1);
        }
        if (o.temps >= s.duree) o.lance = true;
        continue;
      }

      // En vol : vise un ennemi, en change s'il meurt
      if (!o.cible || o.cible.mort >= 0 || o.cible.id !== o.cibleId) {
        o.cible = ennemis.auHasard(o.x, o.z, s.portee) ?? ennemis.plusProche(o.x, o.z, s.portee * 2);
        o.cibleId = o.cible?.id ?? 0;
        if (!o.cible) {
          o.arme = null;
          this.orbes.liberer(i);
          continue;
        }
      }
      const dx = o.cible.x - o.x;
      const dz = o.cible.z - o.z;
      const d = longueur(dx, dz);
      if (d <= o.cible.definition.rayon + c.rayonContact) {
        ennemis.toucher(o.cible, arme, o.x, o.z);
        particules.emettre(o.x, c.hauteur, o.z, GERBES.impactArcane);
        o.arme = null;
        this.orbes.liberer(i);
        continue;
      }
      const pas = Math.min(d, s.vitesse * dt);
      o.x += (dx / d) * pas;
      o.z += (dz / d) * pas;
    }
  }

  majVisuel(): void {
    const c = CONFIG.armes.orbes;
    this.cubes.commencer();
    for (let i = 0; i < this.orbes.nombre; i++) {
      const o = this.orbes.elements[i];
      if (!o.arme) continue;
      const naissance = Math.min(1, o.temps / 0.2);
      this.cubes.ajouter(o.x, c.hauteur + Math.sin(o.temps * 5 + o.angle) * 0.12, o.z, c.taille * naissance, o.temps * 5, o.temps * 3);
    }
    this.cubes.terminer();
  }
}
