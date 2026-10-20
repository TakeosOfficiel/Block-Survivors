/**
 * Ombres rondes au sol sous toutes les entités, comme en jeu (texture vanilla
 * misc/shadow). Un seul InstancedMesh. C'est un disque à opacité partielle :
 * il est translucide, pas découpé par alphaTest.
 */
import * as THREE from 'three';
import { CONFIG } from '../config';

const matrice = new THREE.Matrix4();

export class Ombres {
  private readonly maillage: THREE.InstancedMesh;
  private compte = 0;

  constructor(texture: THREE.Texture | null, scene: THREE.Scene) {
    const geometrie = texture ? new THREE.PlaneGeometry(1, 1) : new THREE.CircleGeometry(0.5, 16);
    geometrie.rotateX(-Math.PI / 2);
    const materiau = new THREE.MeshBasicMaterial({
      map: texture,
      color: texture ? 0xffffff : 0x000000,
      transparent: true,
      opacity: CONFIG.ombre.opacite,
      depthWrite: false,
    });
    this.maillage = new THREE.InstancedMesh(geometrie, materiau, CONFIG.ombre.capacite);
    this.maillage.name = 'ombres';
    this.maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.maillage.frustumCulled = false;
    this.maillage.count = 0;
    scene.add(this.maillage);
  }

  commencer(): void {
    this.compte = 0;
  }

  ajouter(x: number, z: number, rayon: number): void {
    if (this.compte >= CONFIG.ombre.capacite) return;
    matrice.makeScale(rayon * 2, 1, rayon * 2).setPosition(x, CONFIG.ombre.hauteur, z);
    this.maillage.setMatrixAt(this.compte++, matrice);
  }

  terminer(): void {
    this.maillage.count = this.compte;
    this.maillage.instanceMatrix.needsUpdate = true;
  }
}
