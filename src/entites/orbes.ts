/**
 * Orbes d'expérience : lâchées par les ennemis, attirées par le joueur dans
 * son rayon de collecte, ramassées au contact. Pool + un InstancedMesh par
 * taille d'orbe ; la teinte vert-jaune pulse comme en jeu.
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { JOUEUR_BASE } from '../donnees/joueur';
import { PROGRESSION } from '../donnees/progression';
import type { Retours } from '../effets/retours';
import type { EtatPartie } from '../etat';
import { gagnerXp } from '../progression/niveaux';
import { longueur } from '../utils/maths';
import { Pool } from '../utils/pool';
import type { EquipeJoueurs, SourceEntites } from './types';
import { alea } from '../utils/aleatoire';

interface Orbe {
  x: number;
  z: number;
  vitesse: number;      // blocs/s, vers le joueur
  aimantee: boolean;    // entrée une fois dans le rayon de collecte : suit le joueur jusqu'au bout
  valeur: number;
  phase: number;        // décale la pulsation et le flottement
}

/** Ce qui reçoit l'expérience des ennemis vaincus. */
export interface DepotXp {
  deposer(x: number, z: number, valeur: number): void;
}

const TAILLE_IMAGE = 16;
const TAILLE_TEXTURE = 64;

const matrice = new THREE.Matrix4();
const position = new THREE.Vector3();
const echelle = new THREE.Vector3(1, 1, 1);
const couleur = new THREE.Color();

export class Orbes implements SourceEntites, DepotXp {
  private readonly etat: EtatPartie;
  private readonly retours: Retours;
  /** Public pour le multijoueur : l'hôte le lit, le client le remplit d'après l'hôte. */
  readonly pool = new Pool<Orbe>(CONFIG.orbes.capacite, () => ({ x: 0, z: 0, vitesse: 0, aimantee: false, valeur: 0, phase: 0 }));
  /** Un maillage par palier de taille (PROGRESSION.orbes). */
  private readonly maillages: THREE.InstancedMesh[];
  private readonly comptes: Int32Array;
  /** Les orbes sont toujours de face : chaque instance prend l'orientation de la caméra. */
  private readonly camera: THREE.Camera;
  /** Multijoueur (hôte) : chaque orbe va au joueur vivant le plus proche (l'XP reste commune). */
  equipe: EquipeJoueurs | null = null;

  constructor(etat: EtatPartie, retours: Retours, scene: THREE.Scene, textures: BibliothequeTextures, camera: THREE.Camera) {
    this.etat = etat;
    this.retours = retours;
    this.camera = camera;
    const map = textures.obtenir(CONFIG.orbes.texture);

    this.maillages = PROGRESSION.orbes.map((palier, i) => {
      const taille = CONFIG.orbes.taille * (1 + i * 0.15);
      const geometrie = new THREE.PlaneGeometry(taille, taille);
      if (map) cadrerImage(geometrie, palier.image);
      // Texture en niveaux de gris teintée par instance ; découpée (alphaTest), non éclairée
      const materiau = map
        ? new THREE.MeshBasicMaterial({ map, alphaTest: 0.5 })
        : new THREE.MeshBasicMaterial({ color: 0xb8ff40 });
      const maillage = new THREE.InstancedMesh(geometrie, materiau, CONFIG.orbes.capacite);
      maillage.name = `orbes:${i}`;
      maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
      maillage.setColorAt(0, couleur.setRGB(1, 1, 1));  // crée l'attribut de couleur
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

  deposer(x: number, z: number, valeur: number): void {
    const o = this.pool.acquerir();
    if (!o) {
      this.fusionnerAvecLaPlusProche(x, z, valeur);
      return;
    }
    o.x = x;
    o.z = z;
    o.vitesse = 0;
    o.aimantee = false;
    o.valeur = valeur;
    o.phase = alea() * 100;
  }

  /** Attire vers le joueur les orbes à moins de `rayon` de (x, z) (Allay chanteur). */
  aimanter(x: number, z: number, rayon: number): void {
    for (let i = 0; i < this.pool.nombre; i++) {
      const o = this.pool.elements[i];
      if ((o.x - x) ** 2 + (o.z - z) ** 2 <= rayon * rayon) o.aimantee = true;
    }
  }

  maj(dt: number): void {
    let j = this.etat.joueur;
    const equipe = this.equipe;
    if (!equipe && j.mort) return;
    const c = CONFIG.orbes;
    const rayon = JOUEUR_BASE.collecte * this.etat.modificateurs.collecte;

    for (let i = this.pool.nombre - 1; i >= 0; i--) {
      const o = this.pool.elements[i];
      if (equipe) {
        const k = equipe.plusProche(o.x, o.z);
        if (k < 0) return;
        j = equipe.joueur(k);
      }
      const dx = j.x - o.x;
      const dz = j.z - o.z;
      const d = longueur(dx, dz);
      if (d < rayon) o.aimantee = true;
      if (!o.aimantee) continue;

      // Guidage direct, de plus en plus rapide : toujours droit sur le joueur,
      // jamais en orbite autour de lui (aucune vitesse latérale conservée)
      o.vitesse = Math.min(c.vitesseMax, o.vitesse + c.attraction * dt);
      const pas = o.vitesse * dt;
      if (d < c.distanceCollecte || pas >= d) {
        gagnerXp(this.etat, o.valeur);
        this.retours.son('orbe');
        this.pool.liberer(i);
        continue;
      }
      o.x += (dx / d) * pas;
      o.z += (dz / d) * pas;
    }
  }

  majVisuel(temps: number): void {
    const comptes = this.comptes;
    comptes.fill(0);
    for (let i = 0; i < this.pool.nombre; i++) {
      const o = this.pool.elements[i];
      const palier = this.palier(o.valeur);
      const maillage = this.maillages[palier];
      const t = temps * 10 + o.phase;
      position.set(o.x, CONFIG.orbes.hauteur + Math.sin(t * 0.3) * 0.06, o.z);
      matrice.compose(position, this.camera.quaternion, echelle);
      maillage.setMatrixAt(comptes[palier], matrice);
      // Pulsation du jeu : rouge et bleu oscillent, vert plein
      couleur.setRGB((Math.sin(t / 2) + 1) * 0.5, 1, (Math.sin(t / 2 + 4.18879) + 1) * 0.1, THREE.SRGBColorSpace);
      maillage.setColorAt(comptes[palier], couleur);
      comptes[palier]++;
    }
    for (let p = 0; p < this.maillages.length; p++) {
      const maillage = this.maillages[p];
      maillage.count = comptes[p];
      maillage.instanceMatrix.needsUpdate = true;
      if (maillage.instanceColor) maillage.instanceColor.needsUpdate = true;
    }
  }

  /** Index du palier de taille correspondant à une valeur d'XP. */
  private palier(valeur: number): number {
    const paliers = PROGRESSION.orbes;
    for (let p = paliers.length - 1; p > 0; p--) if (valeur >= paliers[p].valeurMin) return p;
    return 0;
  }

  /** Pool plein : la valeur rejoint l'orbe la plus proche (les orbes fusionnent aussi en jeu). */
  private fusionnerAvecLaPlusProche(x: number, z: number, valeur: number): void {
    let meilleure: Orbe | null = null;
    let meilleureD2 = Infinity;
    for (let i = 0; i < this.pool.nombre; i++) {
      const o = this.pool.elements[i];
      const d2 = (o.x - x) ** 2 + (o.z - z) ** 2;
      if (d2 < meilleureD2) {
        meilleureD2 = d2;
        meilleure = o;
      }
    }
    if (meilleure) meilleure.valeur += valeur;
  }
}

/** Cadre les UV du plan sur l'image `index` de l'atlas 4x4 des orbes. */
function cadrerImage(geometrie: THREE.PlaneGeometry, index: number): void {
  const x = (index % 4) * TAILLE_IMAGE;
  const y = Math.floor(index / 4) * TAILLE_IMAGE;
  const uvs = geometrie.getAttribute('uv');
  for (let i = 0; i < uvs.count; i++) {
    uvs.setXY(
      i,
      (x + uvs.getX(i) * TAILLE_IMAGE) / TAILLE_TEXTURE,
      1 - (y + (1 - uvs.getY(i)) * TAILLE_IMAGE) / TAILLE_TEXTURE,
    );
  }
  uvs.needsUpdate = true;
}
