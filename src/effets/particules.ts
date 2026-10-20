/**
 * Particules cubiques (mort des ennemis, explosions) : pool + un InstancedMesh.
 * Chaque particule est un petit cube qui retombe, rebondit et rétrécit.
 */
import * as THREE from 'three';
import { CONFIG } from '../config';
import { Pool } from '../utils/pool';

interface Particule {
  x: number; y: number; z: number;
  vx: number; vy: number; vz: number;
  vie: number;
  duree: number;
  taille: number;
  couleur: THREE.Color;
}

/** Paramètres d'une gerbe de particules. */
export interface Gerbe {
  readonly nombre: number;
  readonly vitesse: number;  // blocs/s
  readonly taille: number;   // blocs
  readonly duree: number;    // s
}

const matrice = new THREE.Matrix4();

export class Particules {
  private readonly pool = new Pool<Particule>(CONFIG.particules.capacite, () => ({
    x: 0, y: 0, z: 0, vx: 0, vy: 0, vz: 0, vie: 0, duree: 1, taille: 0.1, couleur: new THREE.Color(),
  }));
  private readonly maillage: THREE.InstancedMesh;

  constructor(scene: THREE.Scene) {
    this.maillage = new THREE.InstancedMesh(
      new THREE.BoxGeometry(1, 1, 1),
      new THREE.MeshLambertMaterial({ color: 0xffffff }),
      CONFIG.particules.capacite,
    );
    this.maillage.name = 'particules';
    this.maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.maillage.setColorAt(0, new THREE.Color(0xffffff));  // crée l'attribut de couleur
    this.maillage.frustumCulled = false;
    this.maillage.count = 0;
    scene.add(this.maillage);
  }

  get actives(): number {
    return this.pool.nombre;
  }

  /** Gerbe de particules autour d'un point ; `couleurs` est tiré au hasard par particule. */
  emettre(x: number, y: number, z: number, gerbe: Gerbe, couleurs: readonly number[]): void {
    for (let i = 0; i < gerbe.nombre; i++) {
      const p = this.pool.acquerir();
      if (!p) return;
      const angle = Math.random() * Math.PI * 2;
      const vitesse = gerbe.vitesse * (0.4 + Math.random() * 0.6);
      p.x = x + (Math.random() - 0.5) * 0.4;
      p.y = y + (Math.random() - 0.5) * 0.4;
      p.z = z + (Math.random() - 0.5) * 0.4;
      p.vx = Math.cos(angle) * vitesse;
      p.vz = Math.sin(angle) * vitesse;
      p.vy = gerbe.vitesse * (0.3 + Math.random() * 0.7);
      p.duree = gerbe.duree * (0.6 + Math.random() * 0.4);
      p.vie = p.duree;
      p.taille = gerbe.taille * (0.6 + Math.random() * 0.4);
      p.couleur.setHex(couleurs[Math.floor(Math.random() * couleurs.length)]);
    }
  }

  maj(dt: number): void {
    const gravite = CONFIG.particules.gravite;
    for (let i = this.pool.nombre - 1; i >= 0; i--) {
      const p = this.pool.elements[i];
      p.vie -= dt;
      if (p.vie <= 0) {
        this.pool.liberer(i);
        continue;
      }
      p.vy -= gravite * dt;
      p.x += p.vx * dt;
      p.y += p.vy * dt;
      p.z += p.vz * dt;
      const sol = p.taille / 2;
      if (p.y < sol) {  // rebond amorti
        p.y = sol;
        p.vy *= -0.3;
        p.vx *= 0.6;
        p.vz *= 0.6;
      }
    }
  }

  majVisuel(): void {
    const n = this.pool.nombre;
    for (let i = 0; i < n; i++) {
      const p = this.pool.elements[i];
      const s = p.taille * Math.min(1, (p.vie / p.duree) * 1.5);
      matrice.makeScale(s, s, s).setPosition(p.x, p.y, p.z);
      this.maillage.setMatrixAt(i, matrice);
      this.maillage.setColorAt(i, p.couleur);
    }
    this.maillage.count = n;
    this.maillage.instanceMatrix.needsUpdate = true;
    if (this.maillage.instanceColor) this.maillage.instanceColor.needsUpdate = true;
  }
}
