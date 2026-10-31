/**
 * Objets ramassables posés au sol (soins, émeraudes) : icône d'objet en relief
 * qui tourne et flotte doucement. Un soin se ramasse au contact du joueur
 * (miettes et bruit de bouche) ; un tas d'émeraudes est aspiré vers lui comme
 * les orbes d'XP, dès qu'il entre dans son rayon de collecte.
 * Pool + un InstancedMesh par type d'objet.
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { GERBES } from '../donnees/particules';
import { JOUEUR_BASE } from '../donnees/joueur';
import { IDS_OBJETS, OBJETS, type IdObjet } from '../donnees/objets';
import { creerGeometrieRelief } from '../moteur/geometrie';
import type { Particules } from '../effets/particules';
import type { Retours } from '../effets/retours';
import type { EtatPartie } from '../etat';
import { CONFIG } from '../config';
import { longueur } from '../utils/maths';
import { Pool } from '../utils/pool';
import type { EquipeJoueurs } from './types';
import { alea } from '../utils/aleatoire';

interface Objet {
  indexType: number;
  x: number;
  z: number;
  phase: number;
  /** Émeraudes du tas (0 pour un soin). */
  valeur: number;
  /** Tas d'émeraudes aspiré vers le joueur, et sa vitesse (blocs/s). */
  aimante: boolean;
  vitesse: number;
}

/** Grossissement d'un tas d'émeraudes par émeraude au-delà de la première. */
const GROSSISSEMENT_TAS = 0.15;

const CAPACITE = 192;

const matrice = new THREE.Matrix4();
const echelle = new THREE.Vector3();

export class Objets {
  private readonly etat: EtatPartie;
  private readonly particules: Particules;
  private readonly retours: Retours;
  /** Public pour le multijoueur : l'hôte le lit, le client le remplit d'après l'hôte. */
  readonly pool = new Pool<Objet>(CAPACITE, () => ({ indexType: 0, x: 0, z: 0, phase: 0, valeur: 0, aimante: false, vitesse: 0 }));
  private readonly maillages: THREE.InstancedMesh[];
  private readonly comptes: Int32Array;
  /** Multijoueur (hôte) : chaque objet va au joueur vivant le plus proche. */
  equipe: EquipeJoueurs | null = null;

  constructor(etat: EtatPartie, particules: Particules, retours: Retours, scene: THREE.Scene, textures: BibliothequeTextures) {
    this.etat = etat;
    this.particules = particules;
    this.retours = retours;

    // Comme les objets au sol du jeu : en relief (un pixel d'épaisseur), qui tournent sur eux-mêmes
    this.maillages = IDS_OBJETS.map((id) => {
      const def = OBJETS[id];
      const map = textures.obtenir(def.icone);
      const relief = map ? creerGeometrieRelief(map, def.taille) : null;
      const geometrie = relief ?? new THREE.PlaneGeometry(def.taille, def.taille).rotateX(-Math.PI / 4);
      const materiau = map
        ? new THREE.MeshLambertMaterial({ map, alphaTest: 0.5, side: THREE.DoubleSide })
        : new THREE.MeshBasicMaterial({ color: def.couleurSecours });
      const maillage = new THREE.InstancedMesh(geometrie, materiau, CAPACITE);
      maillage.name = `objet:${id}`;
      maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
      maillage.frustumCulled = false;
      maillage.count = 0;
      scene.add(maillage);
      return maillage;
    });
    this.comptes = new Int32Array(this.maillages.length);
  }

  get actifs(): number {
    return this.pool.nombre;
  }

  poser(id: IdObjet, x: number, z: number): void {
    const o = this.pool.acquerir();
    if (!o) return;
    o.indexType = IDS_OBJETS.indexOf(id);
    o.x = x;
    o.z = z;
    o.phase = alea() * 10;
    o.aimante = false;
    o.vitesse = 0;
    const tas = OBJETS[id].emeraudes;
    o.valeur = tas ? tas[0] + Math.floor(alea() * (tas[1] - tas[0] + 1)) : 0;
  }

  maj(dt: number): void {
    let j = this.etat.joueur;
    const equipe = this.equipe;
    if (!equipe && j.mort) return;
    const portee = JOUEUR_BASE.rayonRamassage;
    const c = CONFIG.orbes;
    const collecte = JOUEUR_BASE.collecte * this.etat.modificateurs.collecte;
    for (let i = this.pool.nombre - 1; i >= 0; i--) {
      const o = this.pool.elements[i];
      if (equipe) {
        const k = equipe.plusProche(o.x, o.z);
        if (k < 0) return;
        j = equipe.joueur(k);
      }
      const def = OBJETS[IDS_OBJETS[o.indexType]];
      const dx = j.x - o.x;
      const dz = j.z - o.z;
      const d = longueur(dx, dz);
      if (def.emeraudes) {
        // Aspiré comme une orbe d'XP : droit sur le joueur, de plus en plus vite
        if (d < collecte) o.aimante = true;
        if (!o.aimante) continue;
        o.vitesse = Math.min(c.vitesseMax, o.vitesse + c.attraction * dt);
        const pas = o.vitesse * dt;
        if (d > c.distanceCollecte && pas < d) {
          o.x += (dx / d) * pas;
          o.z += (dz / d) * pas;
          continue;
        }
        const emeraudes = Math.round(o.valeur * (1 + this.etat.modificateurs.emeraudes));
        this.etat.emeraudes += emeraudes;
        this.etat.stats.emeraudesRamassees += emeraudes;
        this.retours.son('emeraude');
        this.pool.liberer(i);
        continue;
      }
      if (d > portee) continue;
      const avant = j.pv;
      j.pv = Math.min(j.pvMax, j.pv + def.soin * (1 + this.etat.modificateurs.soinsObjets));
      this.etat.stats.pvSoignes += j.pv - avant;
      this.retours.son('manger');
      this.particules.emettre(o.x, 0.8, o.z, GERBES.ramassage, def.icone);
      this.pool.liberer(i);
    }
  }

  majVisuel(temps: number): void {
    this.comptes.fill(0);
    for (let i = 0; i < this.pool.nombre; i++) {
      const o = this.pool.elements[i];
      // Balancement et rotation des objets au sol du jeu (un tour toutes les ~6 s)
      matrice.makeRotationY(temps + o.phase).setPosition(o.x, 0.55 + Math.sin(temps * 2 + o.phase) * 0.1, o.z);
      // Un tas d'émeraudes est d'autant plus gros qu'il en vaut
      if (o.valeur > 1) matrice.scale(echelle.setScalar(1 + GROSSISSEMENT_TAS * (o.valeur - 1)));
      this.maillages[o.indexType].setMatrixAt(this.comptes[o.indexType]++, matrice);
    }
    for (let t = 0; t < this.maillages.length; t++) {
      this.maillages[t].count = this.comptes[t];
      this.maillages[t].instanceMatrix.needsUpdate = true;
    }
  }
}
