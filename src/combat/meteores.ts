/**
 * Mécanique « météore » : un bloc de magma tombe en biais sur le plus gros
 * groupe d'ennemis (son ombre annonce l'impact) en laissant une traînée de
 * feu ; à l'impact, dégâts de zone, lave et fumée, puis le sol brûle (feu
 * vanilla animé) et blesse ceux qui s'y trouvent toutes les 0,5 s.
 * Talents : Pluie de feu (3 petits météores), Lave en fusion (feu plus long et
 * plus fort), Tempête de feu (un météore tombé dans une tornade la renforce).
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
  /** Part des dégâts (Pluie de feu, Écho) et taille (petits météores). */
  facteur: number;
  taille: number;
}

interface Feu {
  arme: ArmeEquipee | null;
  x: number;
  z: number;
  temps: number;
  prochainTic: number;
  graine: number;
  facteur: number;
  zone: number;
  duree: number;
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
    this.meteores = new Pool<Meteore>(c.capacite, () => ({ arme: null, x: 0, z: 0, t: 0, prochaineTrainee: 0, facteur: 1, taille: 1 }));
    this.feux = new Pool<Feu>(c.capacite, () => ({ arme: null, x: 0, z: 0, temps: 0, prochainTic: 0, graine: 0, facteur: 1, zone: 1, duree: 0 }));
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
    const c = CONFIG.armes.meteore;
    const talents = this.contexte.talents;
    const facteur = talents.facteurDeclenchement;
    const pluie = talents.rang('pluie_feu') > 0;
    let lance = false;
    for (let k = 0; k < s.nombre; k++) {
      if (!ennemis.plusGrosGroupe(j.x, j.z, s.portee, s.zone, groupe)) break;
      if (!pluie) {
        lance = this.lancer(arme, groupe.x, groupe.z, -k * c.decalage, facteur, 1) || lance;
        continue;
      }
      // Pluie de feu : 3 météores plus petits autour du groupe
      for (let q = 0; q < c.pluie.nombre; q++) {
        const a = Math.random() * Math.PI * 2;
        const r = q === 0 ? 0 : c.pluie.ecart * (0.6 + Math.random() * 0.4);
        const depart = -(k * c.pluie.nombre + q) * c.pluie.decalage;
        lance = this.lancer(arme, groupe.x + Math.sin(a) * r, groupe.z + Math.cos(a) * r, depart, facteur * c.pluie.degats, c.pluie.taille) || lance;
      }
    }
    return lance;
  }

  private lancer(arme: ArmeEquipee, x: number, z: number, depart: number, facteur: number, taille: number): boolean {
    const m = this.meteores.acquerir();
    if (!m) return false;
    m.arme = arme;
    m.x = x;
    m.z = z;
    m.t = depart;
    m.prochaineTrainee = 0;
    m.facteur = facteur;
    m.taille = taille;
    return true;
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
      const talents = this.contexte.talents;
      const zone = arme.stats.zone * m.taille;
      const n = ennemis.chercher(m.x, m.z, zone, this.tampon);
      for (let k = 0; k < n; k++) ennemis.toucher(ennemis.obtenir(this.tampon[k]), arme, m.x, m.z, arme.stats.recul, m.facteur);
      // Tempête de feu : un météore tombé dans une tornade la change en tornade de magma
      if (talents.rang('tempete_feu') > 0) talents.systemes.tornades.renforcer(m.x, m.z);
      particules.emettre(m.x, 0.4, m.z, GERBES.impactMeteore);
      particules.emettre(m.x, 0.4, m.z, GERBES.eboulis, this.texture);
      retours.son('explosion', 0.8);
      retours.secousse(0.3);
      const feu = this.feux.acquerir();
      if (feu) {
        // Lave en fusion : le feu au sol dure plus et brûle plus fort
        const lave = talents.rang('lave_fusion') > 0;
        feu.arme = arme;
        feu.x = m.x;
        feu.z = m.z;
        feu.temps = 0;
        feu.prochainTic = c.tic;
        feu.graine = Math.random() * 10;
        feu.facteur = m.facteur * (lave ? 1 + c.laveFusion.degats : 1);
        feu.zone = m.taille;
        feu.duree = arme.stats.duree + (lave ? c.laveFusion.duree : 0);
      }
      m.arme = null;
      this.meteores.liberer(i);
    }

    for (let i = this.feux.nombre - 1; i >= 0; i--) {
      const f = this.feux.elements[i];
      const arme = f.arme;
      const mec = arme?.definition.mecanique;
      f.temps += dt;
      if (!arme || mec?.type !== 'meteore' || f.temps >= f.duree) {
        f.arme = null;
        this.feux.liberer(i);
        continue;
      }
      f.prochainTic -= dt;
      if (f.prochainTic <= 0) {
        f.prochainTic += c.tic;
        const degats = arme.stats.degats * mec.degatsSol * f.facteur;
        const n = ennemis.chercher(f.x, f.z, arme.stats.zone * f.zone, this.tampon);
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
      this.blocs.ajouter(m.x + c.departX * reste, c.hauteur * reste + 0.5, m.z + c.departZ * reste, c.taille * m.taille, m.t * 6, m.t * 4);
      ombres.ajouter(m.x, m.z, m.arme.stats.zone * m.taille * (0.3 + 0.7 * p));
    }
    this.blocs.terminer();

    let n = 0;
    for (let i = 0; i < this.feux.nombre; i++) {
      const f = this.feux.elements[i];
      if (!f.arme) continue;
      const zone = f.arme.stats.zone * f.zone;
      const reste = f.duree - f.temps;
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
