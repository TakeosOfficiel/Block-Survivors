/**
 * Sprites en deux plans croisés, comme les plantes du jeu (buissons, flèches
 * plantées, flammes) : un seul InstancedMesh réécrit à chaque image,
 * commencer(), ajouter() pour chaque sprite, terminer(). Texture découpée
 * (alphaTest), toujours lumineuse.
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import type { CheminTexture } from '../donnees/types';

const matrice = new THREE.Matrix4();
const position = new THREE.Vector3();
const orientation = new THREE.Quaternion();
const euler = new THREE.Euler(0, 0, 0, 'YXZ');
const echelle = new THREE.Vector3();

export class PlansCroises {
  private readonly maillage: THREE.InstancedMesh;
  private n = 0;

  constructor(scene: THREE.Scene, textures: BibliothequeTextures, chemin: CheminTexture, capacite: number, nom: string) {
    const map = textures.obtenir(chemin);
    this.maillage = new THREE.InstancedMesh(
      new THREE.PlaneGeometry(1, 1),
      new THREE.MeshBasicMaterial(map ? { map, alphaTest: 0.5, side: THREE.DoubleSide } : { color: 0x888888, side: THREE.DoubleSide }),
      capacite * 2,
    );
    this.maillage.name = nom;
    this.maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.maillage.frustumCulled = false;
    this.maillage.count = 0;
    scene.add(this.maillage);
  }

  commencer(): void {
    this.n = 0;
  }

  /**
   * Sprite centré en (x, y, z), de côté `taille`, tourné de `rotationY` autour
   * de la verticale et de `rotationPlan` dans son propre plan (une icône en
   * diagonale pointe vers le bas avec −3π/4).
   */
  ajouter(x: number, y: number, z: number, taille: number, rotationY: number, rotationPlan = 0): void {
    if (this.n + 2 > this.maillage.instanceMatrix.count || taille <= 0) return;
    position.set(x, y, z);
    echelle.set(taille, taille, taille);
    for (let plan = 0; plan < 2; plan++) {
      orientation.setFromEuler(euler.set(0, rotationY + plan * Math.PI * 0.5, rotationPlan));
      matrice.compose(position, orientation, echelle);
      this.maillage.setMatrixAt(this.n++, matrice);
    }
  }

  terminer(): void {
    this.maillage.count = this.n;
    this.maillage.instanceMatrix.needsUpdate = true;
  }
}
