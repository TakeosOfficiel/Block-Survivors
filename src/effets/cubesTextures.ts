/**
 * Cubes texturés d'un même bloc (magma, glace, améthyste…) dessinés en un
 * seul InstancedMesh, réécrit à chaque image : commencer(), ajouter() pour
 * chaque cube, terminer(). Lumineux : non éclairé (magma, feu).
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import type { CheminTexture } from '../donnees/types';

const matrice = new THREE.Matrix4();
const position = new THREE.Vector3();
const orientation = new THREE.Quaternion();
const euler = new THREE.Euler();
const echelle = new THREE.Vector3();

export class CubesTextures {
  private readonly maillage: THREE.InstancedMesh;
  private n = 0;

  constructor(scene: THREE.Scene, textures: BibliothequeTextures, chemin: CheminTexture, capacite: number, lumineux: boolean, nom: string) {
    const map = textures.obtenir(chemin);
    const reglages = map ? { map } : { color: 0x888888 };
    const materiau = lumineux ? new THREE.MeshBasicMaterial(reglages) : new THREE.MeshLambertMaterial(reglages);
    this.maillage = new THREE.InstancedMesh(new THREE.BoxGeometry(1, 1, 1), materiau, capacite);
    this.maillage.name = nom;
    this.maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.maillage.frustumCulled = false;
    this.maillage.count = 0;
    scene.add(this.maillage);
  }

  commencer(): void {
    this.n = 0;
  }

  /** Cube de côté `taille` centré en (x, y, z), tourné de `rotationY` puis incliné de `inclinaison` (rad). */
  ajouter(x: number, y: number, z: number, taille: number, rotationY = 0, inclinaison = 0): void {
    if (this.n >= this.maillage.instanceMatrix.count || taille <= 0) return;
    position.set(x, y, z);
    orientation.setFromEuler(euler.set(inclinaison, rotationY, inclinaison * 0.7));
    matrice.compose(position, orientation, echelle.set(taille, taille, taille));
    this.maillage.setMatrixAt(this.n++, matrice);
  }

  /** Pavé plat (sol) de côté `taille` et d'épaisseur `epaisseur`. */
  ajouterDalle(x: number, z: number, taille: number, epaisseur: number, rotationY: number): void {
    if (this.n >= this.maillage.instanceMatrix.count || taille <= 0) return;
    position.set(x, epaisseur / 2, z);
    orientation.setFromEuler(euler.set(0, rotationY, 0));
    matrice.compose(position, orientation, echelle.set(taille, epaisseur, taille));
    this.maillage.setMatrixAt(this.n++, matrice);
  }

  terminer(): void {
    this.maillage.count = this.n;
    this.maillage.instanceMatrix.needsUpdate = true;
  }
}
