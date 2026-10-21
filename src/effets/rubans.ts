/**
 * Rubans texturés (bandes de quads) réécrits à chaque image dans un seul
 * maillage : commencer(), puis pour chaque ruban `segments + 1` appels à
 * point() et un fermer(), enfin terminer(). Le long du ruban la texture se
 * répète (une tuile tous les `pasU` segments) ; en travers, elle va du bas
 * de l'image (v = 0) jusqu'à `vHaut` (1 : tout en haut), à un demi-pixel des
 * bords pour ne pas déborder sur l'image voisine d'une texture animée. Non
 * éclairé, découpé (feu, magma) et teinté par sommet (point()).
 */
import * as THREE from 'three';

/** Demi-pixel d'une image de 16 pixels de haut. */
const MARGE_V = 1 / 32;

export class Rubans {
  private readonly maillage: THREE.Mesh;
  private readonly positions: Float32Array;
  private readonly couleurs: Float32Array;
  private readonly attribut: THREE.BufferAttribute;
  private readonly attributCouleurs: THREE.BufferAttribute;
  private readonly segments: number;
  private readonly capacite: number;
  private n = 0;

  constructor(
    scene: THREE.Scene, map: THREE.Texture | null, couleurSecours: number, capacite: number, segments: number, pasU: number, nom: string,
    vHaut = 1,
  ) {
    this.segments = segments;
    this.capacite = capacite;
    const sommets = capacite * (segments + 1) * 2;
    this.positions = new Float32Array(sommets * 3);
    this.couleurs = new Float32Array(sommets * 3).fill(1);
    const uvs = new Float32Array(sommets * 2);
    const indices = new Uint32Array(capacite * segments * 6);
    for (let r = 0; r < capacite; r++) {
      const base = r * (segments + 1) * 2;
      for (let i = 0; i <= segments; i++) {
        const s = base + i * 2;
        uvs[s * 2] = i * pasU;          // bas
        uvs[s * 2 + 1] = MARGE_V;
        uvs[s * 2 + 2] = i * pasU;      // haut
        uvs[s * 2 + 3] = vHaut - MARGE_V;
        if (i === segments) continue;
        const k = (r * segments + i) * 6;
        indices[k] = s;
        indices[k + 1] = s + 2;
        indices[k + 2] = s + 1;
        indices[k + 3] = s + 1;
        indices[k + 4] = s + 2;
        indices[k + 5] = s + 3;
      }
    }
    const geometrie = new THREE.BufferGeometry();
    this.attribut = new THREE.BufferAttribute(this.positions, 3);
    this.attribut.setUsage(THREE.DynamicDrawUsage);
    geometrie.setAttribute('position', this.attribut);
    this.attributCouleurs = new THREE.BufferAttribute(this.couleurs, 3);
    this.attributCouleurs.setUsage(THREE.DynamicDrawUsage);
    geometrie.setAttribute('color', this.attributCouleurs);
    geometrie.setAttribute('uv', new THREE.BufferAttribute(uvs, 2));
    geometrie.setIndex(new THREE.BufferAttribute(indices, 1));
    geometrie.setDrawRange(0, 0);
    const materiau = map
      ? new THREE.MeshBasicMaterial({ map, vertexColors: true, alphaTest: 0.5, side: THREE.DoubleSide })
      : new THREE.MeshBasicMaterial({ color: couleurSecours, vertexColors: true, side: THREE.DoubleSide });
    this.maillage = new THREE.Mesh(geometrie, materiau);
    this.maillage.name = nom;
    this.maillage.frustumCulled = false;
    scene.add(this.maillage);
  }

  /** Vrai s'il reste de la place pour un ruban. */
  get place(): boolean {
    return this.n < this.capacite;
  }

  commencer(): void {
    this.n = 0;
  }

  /** Point `i` (0 à segments) du ruban en cours : bord bas puis bord haut, et leur teinte (r, v, b). */
  point(
    i: number, xBas: number, yBas: number, zBas: number, xHaut: number, yHaut: number, zHaut: number,
    r = 1, v = 1, b = 1,
  ): void {
    const p = ((this.n * (this.segments + 1) + i) * 2) * 3;
    const positions = this.positions;
    positions[p] = xBas;
    positions[p + 1] = yBas;
    positions[p + 2] = zBas;
    positions[p + 3] = xHaut;
    positions[p + 4] = yHaut;
    positions[p + 5] = zHaut;
    const couleurs = this.couleurs;
    couleurs[p] = couleurs[p + 3] = r;
    couleurs[p + 1] = couleurs[p + 4] = v;
    couleurs[p + 2] = couleurs[p + 5] = b;
  }

  fermer(): void {
    this.n++;
  }

  terminer(): void {
    this.maillage.geometry.setDrawRange(0, this.n * this.segments * 6);
    this.attribut.needsUpdate = true;
    this.attributCouleurs.needsUpdate = true;
  }
}
