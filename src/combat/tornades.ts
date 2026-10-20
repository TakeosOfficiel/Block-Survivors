/**
 * Mécanique « tornade » (Tornade de feu) : une colonne de cubes de magma qui
 * tourbillonne et avance vers l'ennemi le plus proche ; toutes les
 * `intervalle` secondes, elle blesse les ennemis dans son rayon et les aspire
 * vers son centre (recul négatif). Elle crache des braises et de la fumée.
 */
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { ARMES, IDS_ARMES } from '../donnees/armes';
import { GERBES } from '../donnees/particules';
import { CubesTextures } from '../effets/cubesTextures';
import { longueur } from '../utils/maths';
import { Pool } from '../utils/pool';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';

interface Tornade {
  arme: ArmeEquipee | null;
  x: number;
  z: number;
  /** Direction de dérive quand aucun ennemi n'est proche. */
  dirX: number;
  dirZ: number;
  temps: number;
  prochainCoup: number;
  prochainesBraises: number;
}

export class Tornades {
  private readonly contexte: ContexteCombat;
  private readonly tampon = new Int32Array(128);
  private readonly tornades: Pool<Tornade>;
  private readonly cubes: CubesTextures;

  constructor(contexte: ContexteCombat, textures: BibliothequeTextures, scene: THREE.Scene) {
    const c = CONFIG.armes.tornade;
    this.contexte = contexte;
    this.tornades = new Pool<Tornade>(c.capacite, () => ({ arme: null, x: 0, z: 0, dirX: 0, dirZ: 1, temps: 0, prochainCoup: 0, prochainesBraises: 0 }));
    const m = IDS_ARMES.map((id) => ARMES[id].mecanique).find((mec) => mec.type === 'tornade');
    const texture = m?.type === 'tornade' ? m.texture : 'block/magma';
    textures.animer(texture, c.imagesParSeconde);
    this.cubes = new CubesTextures(scene, textures, texture, c.capacite * c.cubes, true, 'tornades');
  }

  /** Lance les tornades vers l'ennemi le plus proche ; faux si personne n'est à portée. */
  frapper(arme: ArmeEquipee): boolean {
    if (arme.definition.mecanique.type !== 'tornade') return false;
    const j = this.contexte.etat.joueur;
    const cible = this.contexte.ennemis.plusProche(j.x, j.z, CONFIG.armes.tornade.portee);
    if (!cible) return false;
    const angle = Math.atan2(cible.x - j.x, cible.z - j.z);
    const s = arme.stats;
    for (let k = 0; k < s.nombre; k++) {
      const t = this.tornades.acquerir();
      if (!t) break;
      const a = angle + (k - (s.nombre - 1) / 2) * 0.8;
      t.arme = arme;
      t.dirX = Math.sin(a);
      t.dirZ = Math.cos(a);
      t.x = j.x + t.dirX;
      t.z = j.z + t.dirZ;
      t.temps = 0;
      t.prochainCoup = 0;
      t.prochainesBraises = 0;
    }
    return true;
  }

  maj(dt: number): void {
    const c = CONFIG.armes.tornade;
    const { ennemis, particules, carte } = this.contexte;
    for (let i = this.tornades.nombre - 1; i >= 0; i--) {
      const t = this.tornades.elements[i];
      const arme = t.arme;
      const m = arme?.definition.mecanique;
      t.temps += dt;
      if (!arme || m?.type !== 'tornade' || t.temps >= arme.stats.duree) {
        t.arme = null;
        this.tornades.liberer(i);
        continue;
      }
      // Avance vers l'ennemi le plus proche, sinon continue sur sa lancée
      const cible = ennemis.plusProche(t.x, t.z, c.recherche);
      if (cible) {
        const dx = cible.x - t.x;
        const dz = cible.z - t.z;
        const d = longueur(dx, dz);
        if (d > 0.3) {
          t.dirX = dx / d;
          t.dirZ = dz / d;
        }
      }
      const nx = t.x + t.dirX * m.vitesse * dt;
      const nz = t.z + t.dirZ * m.vitesse * dt;
      if (!carte.bloqueMarche(nx, nz)) {
        t.x = nx;
        t.z = nz;
      }
      t.prochainCoup -= dt;
      if (t.prochainCoup <= 0) {
        t.prochainCoup += m.intervalle;
        const n = ennemis.chercher(t.x, t.z, arme.stats.zone, this.tampon);
        for (let k = 0; k < n; k++) ennemis.toucher(ennemis.obtenir(this.tampon[k]), arme, t.x, t.z);
      }
      t.prochainesBraises -= dt;
      if (t.prochainesBraises <= 0) {
        t.prochainesBraises = c.periodeBraises;
        particules.emettre(t.x, 0.6, t.z, GERBES.tornade);
      }
    }
  }

  majVisuel(): void {
    const c = CONFIG.armes.tornade;
    this.cubes.commencer();
    for (let i = 0; i < this.tornades.nombre; i++) {
      const t = this.tornades.elements[i];
      if (!t.arme) continue;
      const largeur = t.arme.stats.zone / 1.5;
      // Apparition et disparition en douceur
      const reste = t.arme.stats.duree - t.temps;
      const echelle = Math.min(1, t.temps / 0.25, reste / 0.35);
      // Colonne en spirale qui s'évase vers le haut
      for (let k = 0; k < c.cubes; k++) {
        const h = k / (c.cubes - 1);
        const rayon = (0.15 + h * 0.75) * largeur * echelle;
        const angle = t.temps * c.rotation + k * 0.9;
        const taille = (0.28 + h * 0.22) * echelle;
        this.cubes.ajouter(
          t.x + Math.sin(angle) * rayon,
          0.2 + h * c.hauteur * echelle,
          t.z + Math.cos(angle) * rayon,
          taille,
          angle,
          t.temps * 3 + k,
        );
      }
    }
    this.cubes.terminer();
  }
}
