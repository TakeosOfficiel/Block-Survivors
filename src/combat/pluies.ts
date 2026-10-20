/**
 * Mécanique « pluie » (Pluie de flèches) : pendant la durée de l'arme, des
 * flèches tombent du ciel en des points au hasard du plus gros groupe
 * d'ennemis ; chacune blesse autour de son point d'impact, reste plantée un
 * instant dans le sol, puis disparaît. Les flèches sont les entités du jeu
 * (plans croisés de la texture de la flèche), pointe en bas.
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { ARMES, IDS_ARMES } from '../donnees/armes';
import { GERBES } from '../donnees/particules';
import { PROJECTILES } from '../donnees/projectiles';
import { creerGeometrieCroisee } from '../moteur/geometrie';
import type { Point } from '../monde/carte';
import { Pool } from '../utils/pool';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';

interface Zone {
  arme: ArmeEquipee | null;
  x: number;
  z: number;
  temps: number;
  prochaineFleche: number;
}

interface FlecheTombante {
  arme: ArmeEquipee | null;
  x: number;
  z: number;
  age: number;
  rotation: number;
  touche: boolean;
}

const groupe: Point = { x: 0, z: 0 };
const matrice = new THREE.Matrix4();
const position = new THREE.Vector3();
const orientation = new THREE.Quaternion();
const euler = new THREE.Euler(0, 0, 0, 'YXZ');
const unite = new THREE.Vector3(1, 1, 1);

export class Pluies {
  private readonly contexte: ContexteCombat;
  private readonly tampon = new Int32Array(64);
  private readonly zones: Pool<Zone>;
  private readonly fleches: Pool<FlecheTombante>;
  private readonly maillage: THREE.InstancedMesh;

  constructor(contexte: ContexteCombat, textures: BibliothequeTextures, scene: THREE.Scene) {
    const c = CONFIG.armes.pluie;
    this.contexte = contexte;
    this.zones = new Pool<Zone>(c.zones, () => ({ arme: null, x: 0, z: 0, temps: 0, prochaineFleche: 0 }));
    this.fleches = new Pool<FlecheTombante>(c.fleches, () => ({ arme: null, x: 0, z: 0, age: 0, rotation: 0, touche: false }));
    const m = IDS_ARMES.map((id) => ARMES[id].mecanique).find((mec) => mec.type === 'pluie');
    const def = PROJECTILES[m?.type === 'pluie' ? m.projectile : 'fleche'];
    // Flèche du jeu, pointe vers le bas (+Z du modèle → −Y)
    const geometrie = creerGeometrieCroisee(def.longueur, def.largeur, def.uv, def.tailleTexture).rotateX(Math.PI / 2);
    const map = textures.obtenir(def.texture);
    const materiau = map
      ? new THREE.MeshLambertMaterial({ map, alphaTest: 0.5, side: THREE.DoubleSide })
      : new THREE.MeshLambertMaterial({ color: def.couleurSecours, side: THREE.DoubleSide });
    this.maillage = new THREE.InstancedMesh(geometrie, materiau, c.fleches);
    this.maillage.name = 'pluie de flèches';
    this.maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.maillage.frustumCulled = false;
    this.maillage.count = 0;
    scene.add(this.maillage);
  }

  /** Vise les plus gros groupes ; faux si personne n'est à portée. */
  frapper(arme: ArmeEquipee): boolean {
    if (arme.definition.mecanique.type !== 'pluie') return false;
    const { etat, ennemis } = this.contexte;
    const j = etat.joueur;
    let lancee = false;
    for (let k = 0; k < arme.stats.nombre; k++) {
      if (!ennemis.plusGrosGroupe(j.x, j.z, arme.stats.portee, arme.stats.zone, groupe)) break;
      const z = this.zones.acquerir();
      if (!z) break;
      z.arme = arme;
      z.x = groupe.x;
      z.z = groupe.z;
      z.temps = 0;
      z.prochaineFleche = 0;
      lancee = true;
    }
    return lancee;
  }

  maj(dt: number): void {
    const c = CONFIG.armes.pluie;
    const { ennemis, particules, retours } = this.contexte;
    for (let i = this.zones.nombre - 1; i >= 0; i--) {
      const z = this.zones.elements[i];
      const arme = z.arme;
      const m = arme?.definition.mecanique;
      z.temps += dt;
      if (!arme || m?.type !== 'pluie' || z.temps > arme.stats.duree) {
        z.arme = null;
        this.zones.liberer(i);
        continue;
      }
      z.prochaineFleche -= dt;
      while (z.prochaineFleche <= 0) {
        z.prochaineFleche += 1 / m.impactsParSeconde;
        const f = this.fleches.acquerir();
        if (!f) break;
        // Point au hasard dans le disque (uniforme)
        const a = Math.random() * Math.PI * 2;
        const r = arme.stats.zone * Math.sqrt(Math.random());
        f.arme = arme;
        f.x = z.x + Math.sin(a) * r;
        f.z = z.z + Math.cos(a) * r;
        f.age = 0;
        f.rotation = Math.random() * Math.PI;
        f.touche = false;
      }
    }

    for (let i = this.fleches.nombre - 1; i >= 0; i--) {
      const f = this.fleches.elements[i];
      f.age += dt;
      if (!f.touche && f.age >= c.chute && f.arme) {
        f.touche = true;
        const n = ennemis.chercher(f.x, f.z, c.rayonImpact, this.tampon);
        for (let k = 0; k < n; k++) ennemis.toucher(ennemis.obtenir(this.tampon[k]), f.arme, f.x, f.z);
        particules.emettre(f.x, 0.2, f.z, GERBES.impactFleche);
        if (n > 0) retours.son('impact_fleche');
      }
      if (f.age >= c.chute + c.plantee) {
        f.arme = null;
        this.fleches.liberer(i);
      }
    }
  }

  majVisuel(): void {
    const c = CONFIG.armes.pluie;
    const longueur = PROJECTILES.fleche.longueur;
    let n = 0;
    for (let i = 0; i < this.fleches.nombre; i++) {
      const f = this.fleches.elements[i];
      // Tombe presque à la verticale, pointe en bas, puis reste plantée dans le sol
      const p = Math.min(1, f.age / c.chute);
      const y = c.hauteur * (1 - p) + longueur * 0.3;
      position.set(f.x, y, f.z);
      orientation.setFromEuler(euler.set(0.12, f.rotation, 0));
      matrice.compose(position, orientation, unite);
      this.maillage.setMatrixAt(n++, matrice);
    }
    this.maillage.count = n;
    this.maillage.instanceMatrix.needsUpdate = true;
  }
}
