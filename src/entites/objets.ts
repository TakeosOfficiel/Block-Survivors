/**
 * Objets ramassables posés au sol (soins) : icône d'objet tournée vers la
 * caméra, qui flotte doucement ; ramassée au contact du joueur.
 * Pool + un InstancedMesh par type d'objet.
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { JOUEUR_BASE } from '../donnees/joueur';
import { IDS_OBJETS, OBJETS, type IdObjet } from '../donnees/objets';
import type { Particules } from '../effets/particules';
import type { Retours } from '../effets/retours';
import type { EtatPartie } from '../etat';
import { Pool } from '../utils/pool';

interface Objet {
  indexType: number;
  x: number;
  z: number;
  phase: number;
}

const CAPACITE = 64;
const GERBE_RAMASSAGE = { nombre: 12, vitesse: 3, taille: 0.18, duree: 0.6 };

const matrice = new THREE.Matrix4();

export class Objets {
  private readonly etat: EtatPartie;
  private readonly particules: Particules;
  private readonly retours: Retours;
  private readonly pool = new Pool<Objet>(CAPACITE, () => ({ indexType: 0, x: 0, z: 0, phase: 0 }));
  private readonly maillages: THREE.InstancedMesh[];
  private readonly comptes: Int32Array;

  constructor(etat: EtatPartie, particules: Particules, retours: Retours, scene: THREE.Scene, textures: BibliothequeTextures) {
    this.etat = etat;
    this.particules = particules;
    this.retours = retours;
    const inclinaison = (CONFIG.camera.inclinaisonDeg * Math.PI) / 180;

    this.maillages = IDS_OBJETS.map((id) => {
      const def = OBJETS[id];
      const map = textures.obtenir(def.icone);
      const geometrie = new THREE.PlaneGeometry(def.taille, def.taille);
      geometrie.rotateX(-inclinaison);  // face à la caméra (qui ne tourne pas)
      const materiau = map
        ? new THREE.MeshBasicMaterial({ map, alphaTest: 0.5 })
        : new THREE.MeshBasicMaterial({ color: def.couleurParticules[0] });
      const maillage = new THREE.InstancedMesh(geometrie, materiau, CAPACITE);
      maillage.name = `objet:${id}`;
      maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
      maillage.frustumCulled = false;
      maillage.count = 0;
      scene.add(maillage);
      return maillage;
    });
    this.comptes = new Int32Array(this.maillages.length);
  }

  get actifs(): number {
    return this.pool.nombre;
  }

  poser(id: IdObjet, x: number, z: number): void {
    const o = this.pool.acquerir();
    if (!o) return;
    o.indexType = IDS_OBJETS.indexOf(id);
    o.x = x;
    o.z = z;
    o.phase = Math.random() * 10;
  }

  maj(): void {
    const j = this.etat.joueur;
    if (j.mort) return;
    const portee = JOUEUR_BASE.rayonRamassage;
    for (let i = this.pool.nombre - 1; i >= 0; i--) {
      const o = this.pool.elements[i];
      if ((o.x - j.x) ** 2 + (o.z - j.z) ** 2 > portee * portee) continue;
      const def = OBJETS[IDS_OBJETS[o.indexType]];
      const avant = j.pv;
      j.pv = Math.min(j.pvMax, j.pv + def.soin);
      this.etat.stats.pvSoignes += j.pv - avant;
      this.retours.son('manger');
      this.particules.emettre(o.x, 0.8, o.z, GERBE_RAMASSAGE, def.couleurParticules);
      this.pool.liberer(i);
    }
  }

  majVisuel(temps: number): void {
    this.comptes.fill(0);
    for (let i = 0; i < this.pool.nombre; i++) {
      const o = this.pool.elements[i];
      matrice.makeTranslation(o.x, 0.55 + Math.sin(temps * 3 + o.phase) * 0.08, o.z);
      this.maillages[o.indexType].setMatrixAt(this.comptes[o.indexType]++, matrice);
    }
    for (let t = 0; t < this.maillages.length; t++) {
      this.maillages[t].count = this.comptes[t];
      this.maillages[t].instanceMatrix.needsUpdate = true;
    }
  }
}
