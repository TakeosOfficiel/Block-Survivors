/**
 * Mécanique « foudre » : des éclairs frappent des ennemis au hasard autour du
 * joueur (dégâts dans une petite zone). L'éclair est une ligne brisée de
 * segments lumineux, rendue par un seul InstancedMesh.
 */
import * as THREE from 'three';
import { CONFIG } from '../config';
import { ARMES, IDS_ARMES } from '../donnees/armes';
import { Pool } from '../utils/pool';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';

interface Eclair {
  points: Float32Array;   // (segments + 1) points x, y, z
  vie: number;
  couleur: THREE.Color;
}

const AXE_Y = new THREE.Vector3(0, 1, 0);
const debut = new THREE.Vector3();
const fin = new THREE.Vector3();
const milieu = new THREE.Vector3();
const direction = new THREE.Vector3();
const echelle = new THREE.Vector3();
const orientation = new THREE.Quaternion();
const matrice = new THREE.Matrix4();

export class Foudre {
  private readonly contexte: ContexteCombat;
  private readonly tampon = new Int32Array(64);
  private readonly pool: Pool<Eclair>;
  private readonly maillage: THREE.InstancedMesh;
  /** Couleurs des particules d'impact, par index d'arme. */
  private readonly couleursImpact: readonly (readonly number[])[];

  constructor(contexte: ContexteCombat, scene: THREE.Scene) {
    this.contexte = contexte;
    this.couleursImpact = IDS_ARMES.map((id) => {
      const m = ARMES[id].mecanique;
      return m.type === 'foudre' ? [m.couleur, 0xffffff] : [];
    });
    const f = CONFIG.armes.foudre;
    this.pool = new Pool<Eclair>(f.capacite, () => ({
      points: new Float32Array((f.segments + 1) * 3),
      vie: 0,
      couleur: new THREE.Color(),
    }));
    this.maillage = new THREE.InstancedMesh(
      new THREE.BoxGeometry(1, 1, 1),
      new THREE.MeshBasicMaterial({ color: 0xffffff, fog: false }),  // lumineux, non éclairé
      f.capacite * f.segments,
    );
    this.maillage.name = 'foudre';
    this.maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.maillage.setColorAt(0, new THREE.Color(0xffffff));  // crée l'attribut de couleur
    this.maillage.frustumCulled = false;
    this.maillage.count = 0;
    scene.add(this.maillage);
  }

  /** Lance les éclairs de l'arme ; faux si aucun ennemi n'est à portée. */
  frapper(arme: ArmeEquipee): boolean {
    const m = arme.definition.mecanique;
    if (m.type !== 'foudre') return false;
    const { etat, ennemis, particules } = this.contexte;
    const j = etat.joueur;
    const s = arme.stats;
    let frappe = false;

    for (let k = 0; k < s.nombre; k++) {
      const cible = ennemis.auHasard(j.x, j.z, s.portee);
      if (!cible) break;
      frappe = true;
      const x = cible.x;
      const z = cible.z;
      const n = ennemis.chercher(x, z, s.zone, this.tampon);
      for (let i = 0; i < n; i++) ennemis.blesser(ennemis.obtenir(this.tampon[i]), s.degats, x, z, s.recul);
      this.creerEclair(x, z, m.couleur);
      particules.emettre(x, 0.3, z, CONFIG.particules.impactFoudre, this.couleursImpact[arme.index]);
    }
    return frappe;
  }

  maj(dt: number): void {
    for (let i = this.pool.nombre - 1; i >= 0; i--) {
      const e = this.pool.elements[i];
      e.vie -= dt;
      if (e.vie <= 0) this.pool.liberer(i);
    }
  }

  majVisuel(): void {
    const f = CONFIG.armes.foudre;
    let n = 0;
    for (let i = 0; i < this.pool.nombre; i++) {
      const e = this.pool.elements[i];
      for (let s = 0; s < f.segments; s++) {
        debut.fromArray(e.points, s * 3);
        fin.fromArray(e.points, s * 3 + 3);
        direction.subVectors(fin, debut);
        const longueur = direction.length();
        direction.divideScalar(longueur);
        milieu.addVectors(debut, fin).multiplyScalar(0.5);
        orientation.setFromUnitVectors(AXE_Y, direction);
        echelle.set(f.largeur, longueur + f.largeur, f.largeur);
        matrice.compose(milieu, orientation, echelle);
        this.maillage.setMatrixAt(n, matrice);
        this.maillage.setColorAt(n, e.couleur);
        n++;
      }
    }
    this.maillage.count = n;
    this.maillage.instanceMatrix.needsUpdate = true;
    if (this.maillage.instanceColor) this.maillage.instanceColor.needsUpdate = true;
  }

  /** Ligne brisée du ciel jusqu'au sol en (x, z), plus déviée en hauteur. */
  private creerEclair(x: number, z: number, couleur: number): void {
    const e = this.pool.acquerir();
    if (!e) return;
    const f = CONFIG.armes.foudre;
    e.vie = f.duree;
    e.couleur.setHex(couleur);
    for (let s = 0; s <= f.segments; s++) {
      const t = s / f.segments;               // 0 = en haut, 1 = au sol
      const ecart = s === f.segments ? 0 : f.ecart * (1 - t * 0.7);
      e.points[s * 3] = x + (Math.random() - 0.5) * 2 * ecart;
      e.points[s * 3 + 1] = f.hauteur * (1 - t);
      e.points[s * 3 + 2] = z + (Math.random() - 0.5) * 2 * ecart;
    }
  }
}
