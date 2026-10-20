/**
 * Petites barres de vie au-dessus des alliés blessés : un fond sombre et un
 * remplissage du vert au rouge, toujours face à la caméra et lisibles par-dessus
 * le décor. Un InstancedMesh de rectangles (deux par barre), rempli à chaque image.
 */
import * as THREE from 'three';
import { MeshBasicNodeMaterial } from 'three/webgpu';

const LARGEUR = 0.8;
const HAUTEUR = 0.09;
const BORD = 0.02;

// Temporaires réutilisés
const matrice = new THREE.Matrix4();
const position = new THREE.Vector3();
const echelle = new THREE.Vector3();
const droite = new THREE.Vector3();
const couleur = new THREE.Color();
const FOND = new THREE.Color(0x1a1a1a);
const PLEIN = new THREE.Color(0x55ff55);
const BAS = new THREE.Color(0xff5555);

export class BarresVie {
  private readonly maillage: THREE.InstancedMesh;
  private readonly camera: THREE.Camera;
  private compte = 0;

  constructor(scene: THREE.Scene, camera: THREE.Camera, capacite: number) {
    this.camera = camera;
    const materiau = new MeshBasicNodeMaterial({ depthTest: false, depthWrite: false, transparent: true });
    this.maillage = new THREE.InstancedMesh(new THREE.PlaneGeometry(1, 1), materiau, capacite * 2);
    this.maillage.name = 'barres-vie';
    this.maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.maillage.setColorAt(0, FOND);  // crée l'attribut de couleur
    this.maillage.frustumCulled = false;
    this.maillage.renderOrder = 8;
    this.maillage.count = 0;
    scene.add(this.maillage);
  }

  commencer(): void {
    this.compte = 0;
    droite.set(1, 0, 0).applyQuaternion(this.camera.quaternion);
  }

  /** Barre centrée en (x, y, z) ; `part` de 0 à 1. */
  ajouter(x: number, y: number, z: number, part: number): void {
    if (this.compte + 2 > this.maillage.instanceMatrix.count) return;
    const q = this.camera.quaternion;
    // Fond
    position.set(x, y, z);
    matrice.compose(position, q, echelle.set(LARGEUR + BORD * 2, HAUTEUR + BORD * 2, 1));
    this.maillage.setMatrixAt(this.compte, matrice);
    this.maillage.setColorAt(this.compte++, FOND);
    // Remplissage, aligné à gauche
    const largeur = LARGEUR * Math.max(0, Math.min(1, part));
    position.set(x, y, z).addScaledVector(droite, (largeur - LARGEUR) / 2);
    matrice.compose(position, q, echelle.set(Math.max(1e-3, largeur), HAUTEUR, 1));
    this.maillage.setMatrixAt(this.compte, matrice);
    this.maillage.setColorAt(this.compte++, couleur.copy(BAS).lerp(PLEIN, part));
  }

  terminer(): void {
    this.maillage.count = this.compte;
    this.maillage.instanceMatrix.needsUpdate = true;
    if (this.maillage.instanceColor) this.maillage.instanceColor.needsUpdate = true;
  }
}
