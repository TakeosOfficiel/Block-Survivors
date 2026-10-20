/**
 * Mécanique « météore » : un bloc de magma tombe en biais sur le plus gros
 * groupe d'ennemis (son ombre annonce l'impact) en laissant une traînée de
 * feu ; à l'impact, dégâts de zone, lave et fumée, puis le sol brûle (feu
 * vanilla animé) et blesse ceux qui s'y trouvent toutes les 0,5 s.
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { ARMES, IDS_ARMES } from '../donnees/armes';
import { GERBES } from '../donnees/particules';
import type { CheminTexture } from '../donnees/types';
import { CubesTextures } from '../effets/cubesTextures';
import type { Ombres } from '../effets/ombres';
import type { Point } from '../monde/carte';
import { Pool } from '../utils/pool';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';

interface Meteore {
  arme: ArmeEquipee | null;
  x: number;
  z: number;
  /** s depuis le lancer (négatif : attend son tour). */
  t: number;
  prochaineTrainee: number;
}

interface Feu {
  arme: ArmeEquipee | null;
  x: number;
  z: number;
  temps: number;
  prochainTic: number;
  graine: number;
}

const groupe: Point = { x: 0, z: 0 };
const matrice = new THREE.Matrix4();
const position = new THREE.Vector3();
const orientation = new THREE.Quaternion();
const echelle = new THREE.Vector3();
const AXE_Y = new THREE.Vector3(0, 1, 0);

export class Meteores {
  private readonly contexte: ContexteCombat;
  private readonly tampon = new Int32Array(256);
  private readonly meteores: Pool<Meteore>;
  private readonly feux: Pool<Feu>;
  private readonly blocs: CubesTextures;
  /** Flammes : deux plans croisés par flamme, comme le bloc de feu du jeu. */
  private readonly flammes: THREE.InstancedMesh;
  private readonly texture: CheminTexture;

  constructor(contexte: ContexteCombat, textures: BibliothequeTextures, scene: THREE.Scene) {
    const c = CONFIG.armes.meteore;
    this.contexte = contexte;
    this.meteores = new Pool<Meteore>(c.capacite, () => ({ arme: null, x: 0, z: 0, t: 0, prochaineTrainee: 0 }));
    this.feux = new Pool<Feu>(c.capacite, () => ({ arme: null, x: 0, z: 0, temps: 0, prochainTic: 0, graine: 0 }));
    const m = IDS_ARMES.map((id) => ARMES[id].mecanique).find((mec) => mec.type === 'meteore');
    this.texture = m?.type === 'meteore' ? m.texture : 'block/magma';
    const feu = m?.type === 'meteore' ? m.feu : 'block/fire_0';
    textures.animer(this.texture, CONFIG.armes.tornade.imagesParSeconde);
    textures.animer(feu, c.imagesFeu);
    this.blocs = new CubesTextures(scene, textures, this.texture, c.capacite, true, 'météores');
    const map = textures.obtenir(feu);
    this.flammes = new THREE.InstancedMesh(
      new THREE.PlaneGeometry(1, 1).translate(0, 0.5, 0),
      new THREE.MeshBasicMaterial(map ? { map, alphaTest: 0.5, side: THREE.DoubleSide } : { color: 0xff8a2a, side: THREE.DoubleSide }),
      c.capacite * c.flammes * 2,
    );
    this.flammes.name = 'feu au sol';
    this.flammes.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.flammes.frustumCulled = false;
    this.flammes.count = 0;
    scene.add(this.flammes);
  }

  /** Fait tomber les météores de l'arme sur les plus gros groupes ; faux si personne n'est à portée. */
  frapper(arme: ArmeEquipee): boolean {
    if (arme.definition.mecanique.type !== 'meteore') return false;
    const { etat, ennemis } = this.contexte;
    const j = etat.joueur;
    const s = arme.stats;
    let lance = false;
    for (let k = 0; k < s.nombre; k++) {
      if (!ennemis.plusGrosGroupe(j.x, j.z, s.portee, s.zone, groupe)) break;
      const m = this.meteores.acquerir();
      if (!m) break;
      m.arme = arme;
      m.x = groupe.x;
      m.z = groupe.z;
      m.t = -k * CONFIG.armes.meteore.decalage;
      m.prochaineTrainee = 0;
      lance = true;
    }
    return lance;
  }

  maj(dt: number): void {
    const c = CONFIG.armes.meteore;
    const { ennemis, particules, retours } = this.contexte;
    for (let i = this.meteores.nombre - 1; i >= 0; i--) {
      const m = this.meteores.elements[i];
      const arme = m.arme;
      m.t += dt;
      if (!arme) {
        this.meteores.liberer(i);
        continue;
      }
      if (m.t < 0) continue;
      if (m.t < c.duree) {
        m.prochaineTrainee -= dt;
        if (m.prochaineTrainee <= 0) {
          m.prochaineTrainee = 0.04;
          const p = m.t / c.duree;
          particules.emettre(m.x + c.departX * (1 - p), c.hauteur * (1 - p) + 0.5, m.z + c.departZ * (1 - p), GERBES.traineeFeu);
        }
        continue;
      }
      // Impact
      const n = ennemis.chercher(m.x, m.z, arme.stats.zone, this.tampon);
      for (let k = 0; k < n; k++) ennemis.toucher(ennemis.obtenir(this.tampon[k]), arme, m.x, m.z);
      particules.emettre(m.x, 0.4, m.z, GERBES.impactMeteore);
      particules.emettre(m.x, 0.4, m.z, GERBES.eboulis, this.texture);
      retours.son('explosion', 0.8);
      retours.secousse(0.3);
      const feu = this.feux.acquerir();
      if (feu) {
        feu.arme = arme;
        feu.x = m.x;
        feu.z = m.z;
        feu.temps = 0;
        feu.prochainTic = c.tic;
        feu.graine = Math.random() * 10;
      }
      m.arme = null;
      this.meteores.liberer(i);
    }

    for (let i = this.feux.nombre - 1; i >= 0; i--) {
      const f = this.feux.elements[i];
      const arme = f.arme;
      const mec = arme?.definition.mecanique;
      f.temps += dt;
      if (!arme || mec?.type !== 'meteore' || f.temps >= arme.stats.duree) {
        f.arme = null;
        this.feux.liberer(i);
        continue;
      }
      f.prochainTic -= dt;
      if (f.prochainTic <= 0) {
        f.prochainTic += c.tic;
        const degats = arme.stats.degats * mec.degatsSol;
        const n = ennemis.chercher(f.x, f.z, arme.stats.zone, this.tampon);
        for (let k = 0; k < n; k++) ennemis.blesser(ennemis.obtenir(this.tampon[k]), degats, f.x, f.z, 0, arme.index, 'brulure');
      }
    }
  }

  majVisuel(ombres: Ombres): void {
    const c = CONFIG.armes.meteore;
    this.blocs.commencer();
    for (let i = 0; i < this.meteores.nombre; i++) {
      const m = this.meteores.elements[i];
      if (!m.arme || m.t < 0) continue;
      // Chute en biais, accélérée ; l'ombre grandit sous le point d'impact
      const p = Math.min(1, m.t / c.duree);
      const reste = 1 - p * p;
      this.blocs.ajouter(m.x + c.departX * reste, c.hauteur * reste + 0.5, m.z + c.departZ * reste, c.taille, m.t * 6, m.t * 4);
      ombres.ajouter(m.x, m.z, m.arme.stats.zone * (0.3 + 0.7 * p));
    }
    this.blocs.terminer();

    let n = 0;
    for (let i = 0; i < this.feux.nombre; i++) {
      const f = this.feux.elements[i];
      if (!f.arme) continue;
      const zone = f.arme.stats.zone;
      const reste = f.arme.stats.duree - f.temps;
      const vie = Math.min(1, f.temps / 0.15, reste / 0.5);
      for (let k = 0; k < c.flammes; k++) {
        // Flammes réparties dans le disque (spirale), hauteur qui vacille
        const a = k * 2.399 + f.graine;
        const r = zone * 0.9 * Math.sqrt((k + 0.5) / c.flammes);
        const hauteur = vie * (0.75 + 0.25 * Math.sin(f.temps * 9 + k * 1.3));
        position.set(f.x + Math.sin(a) * r, 0, f.z + Math.cos(a) * r);
        echelle.set(0.95, hauteur, 0.95);
        for (let plan = 0; plan < 2; plan++) {
          orientation.setFromAxisAngle(AXE_Y, a + plan * Math.PI * 0.5 + Math.PI * 0.25);
          matrice.compose(position, orientation, echelle);
          this.flammes.setMatrixAt(n++, matrice);
        }
      }
    }
    this.flammes.count = n;
    this.flammes.instanceMatrix.needsUpdate = true;
  }
}
