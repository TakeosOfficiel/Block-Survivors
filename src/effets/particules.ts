/**
 * Particules vanilla (explosions, nuages de mort, flammes, paillettes du
 * totem…) : pool + un seul InstancedMesh de carrés tournés vers la caméra.
 * Toutes les images sont rassemblées dans un atlas au démarrage ; chaque
 * instance lit sa case (et sa teinte) dans des attributs par instance.
 */
import * as THREE from 'three';
import { MeshBasicNodeMaterial } from 'three/webgpu';
import { attribute, texture, uv, vec4 } from 'three/tsl';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { IDS_PARTICULES, PARTICULES } from '../donnees/particules';
import type { CheminTexture, DefinitionGerbe, DefinitionParticule } from '../donnees/types';
import { Pool } from '../utils/pool';

interface Particule {
  definition: DefinitionParticule;
  /** Première case de la série dans l'atlas. */
  premiereCase: number;
  x: number; y: number; z: number;
  vx: number; vy: number; vz: number;
  vie: number;
  duree: number;
  taille: number;
  r: number; g: number; b: number;
  /** Morceau d'icône : coin du quart choisi (fraction de la case). */
  morceauU: number;
  morceauV: number;
}

/** Côté d'une case de l'atlas (px) : les explosions du jeu font 32 px. */
const CASE = 32;
const COLONNES = 16;
const COULEUR = new THREE.Color();

// Temporaires réutilisés
const matrice = new THREE.Matrix4();
const position = new THREE.Vector3();
const echelle = new THREE.Vector3();

export class Particules {
  private readonly pool = new Pool<Particule>(CONFIG.particules.capacite, () => ({
    definition: PARTICULES.poof, premiereCase: 0,
    x: 0, y: 0, z: 0, vx: 0, vy: 0, vz: 0, vie: 0, duree: 1, taille: 0.1,
    r: 1, g: 1, b: 1, morceauU: 0, morceauV: 0,
  }));
  private readonly maillage: THREE.InstancedMesh;
  private readonly camera: THREE.Camera;
  private readonly cases: THREE.InstancedBufferAttribute;
  private readonly teintes: THREE.InstancedBufferAttribute;
  /** Case de chaque image (séries des particules, icônes des objets). */
  private readonly casesImages = new Map<CheminTexture, number>();
  private readonly lignes: number;

  constructor(scene: THREE.Scene, camera: THREE.Camera, textures: BibliothequeTextures, icones: Iterable<CheminTexture>) {
    this.camera = camera;
    const images = new Set<CheminTexture>(icones);
    for (const id of IDS_PARTICULES) for (const image of PARTICULES[id].images) images.add(image);
    this.lignes = Math.ceil(images.size / COLONNES);
    const atlas = this.construireAtlas([...images], textures);

    const capacite = CONFIG.particules.capacite;
    const geometrie = new THREE.PlaneGeometry(1, 1);
    this.cases = new THREE.InstancedBufferAttribute(new Float32Array(capacite * 4), 4);
    this.teintes = new THREE.InstancedBufferAttribute(new Float32Array(capacite * 3), 3);
    this.cases.setUsage(THREE.DynamicDrawUsage);
    this.teintes.setUsage(THREE.DynamicDrawUsage);
    geometrie.setAttribute('aCase', this.cases);
    geometrie.setAttribute('aTeinte', this.teintes);

    // Case (u0, v0, largeur, hauteur) dans l'atlas, teinte multipliée ; détourage net comme en jeu
    const materiau = new MeshBasicNodeMaterial({ side: THREE.DoubleSide, alphaTest: 0.1 });
    const c = attribute('aCase', 'vec4');
    const image = texture(atlas, uv().mul(c.zw).add(c.xy));
    materiau.colorNode = vec4(image.rgb.mul(attribute('aTeinte', 'vec3')), image.a);

    this.maillage = new THREE.InstancedMesh(geometrie, materiau, capacite);
    this.maillage.name = 'particules';
    this.maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.maillage.frustumCulled = false;
    this.maillage.count = 0;
    scene.add(this.maillage);
  }

  get actives(): number {
    return this.pool.nombre;
  }

  /** Gerbe autour d'un point ; `icone` donne les morceaux des particules `morceau`. */
  emettre(x: number, y: number, z: number, gerbe: DefinitionGerbe, icone?: CheminTexture): void {
    for (const salve of gerbe) {
      const def = PARTICULES[salve.particule];
      const premiereCase = def.animation === 'morceau'
        ? (icone !== undefined ? this.casesImages.get(icone) : undefined)
        : this.casesImages.get(def.images[0]);
      if (premiereCase === undefined) continue;
      for (let i = 0; i < salve.nombre; i++) {
        const p = this.pool.acquerir();
        if (!p) return;
        p.definition = def;
        p.premiereCase = premiereCase;
        const angle = Math.random() * Math.PI * 2;
        const rayon = Math.sqrt(Math.random()) * salve.dispersion;
        p.x = x + Math.cos(angle) * rayon;
        p.z = z + Math.sin(angle) * rayon;
        p.y = y + (Math.random() - 0.5) * salve.dispersion * 0.6;
        const vitesse = salve.vitesse * (0.3 + Math.random() * 0.7);
        const direction = Math.random() * Math.PI * 2;
        p.vx = Math.cos(direction) * vitesse;
        p.vz = Math.sin(direction) * vitesse;
        p.vy = salve.montee * (0.4 + Math.random() * 0.6);
        p.duree = tirer(def.duree);
        p.vie = p.duree;
        p.taille = tirer(def.taille);
        if (def.teintes) {
          COULEUR.setHex(def.teintes[Math.floor(Math.random() * def.teintes.length)]);
          p.r = COULEUR.r; p.g = COULEUR.g; p.b = COULEUR.b;
        } else {
          const gris = def.gris ? tirer(def.gris) : 1;
          p.r = gris; p.g = gris; p.b = gris;
        }
        // Morceau d'objet : un quart de l'icône, pris au hasard (comme les miettes en jeu)
        p.morceauU = Math.random() * 0.75;
        p.morceauV = Math.random() * 0.75;
      }
    }
  }

  maj(dt: number): void {
    for (let i = this.pool.nombre - 1; i >= 0; i--) {
      const p = this.pool.elements[i];
      p.vie -= dt;
      if (p.vie <= 0) {
        this.pool.liberer(i);
        continue;
      }
      const def = p.definition;
      const frein = 1 - def.frottement * dt;
      p.vx *= frein;
      p.vz *= frein;
      p.vy = p.vy * frein - def.gravite * dt;
      p.x += p.vx * dt;
      p.y += p.vy * dt;
      p.z += p.vz * dt;
      const sol = p.taille * 0.3;
      if (p.y < sol) {  // posée au sol
        p.y = sol;
        p.vy = 0;
        p.vx *= 0.8;
        p.vz *= 0.8;
      }
    }
  }

  majVisuel(): void {
    const n = this.pool.nombre;
    const quaternion = this.camera.quaternion;
    const cases = this.cases.array as Float32Array;
    const teintes = this.teintes.array as Float32Array;
    const largeurCase = 1 / COLONNES;
    const hauteurCase = 1 / this.lignes;
    for (let i = 0; i < n; i++) {
      const p = this.pool.elements[i];
      const def = p.definition;
      const age = 1 - p.vie / p.duree;
      let s = p.taille;
      if (def.retrecit) s *= 1 - age * age * 0.6;
      matrice.compose(position.set(p.x, p.y, p.z), quaternion, echelle.set(s, s, s));
      this.maillage.setMatrixAt(i, matrice);

      let indexCase = p.premiereCase;
      if (def.animation === 'age') indexCase += Math.min(def.images.length - 1, Math.floor(age * def.images.length));
      const u0 = (indexCase % COLONNES) * largeurCase;
      const v0 = 1 - (Math.floor(indexCase / COLONNES) + 1) * hauteurCase;
      if (def.animation === 'morceau') {
        cases[i * 4] = u0 + p.morceauU * largeurCase;
        cases[i * 4 + 1] = v0 + p.morceauV * hauteurCase;
        cases[i * 4 + 2] = largeurCase / 4;
        cases[i * 4 + 3] = hauteurCase / 4;
      } else {
        cases[i * 4] = u0;
        cases[i * 4 + 1] = v0;
        cases[i * 4 + 2] = largeurCase;
        cases[i * 4 + 3] = hauteurCase;
      }
      teintes[i * 3] = p.r;
      teintes[i * 3 + 1] = p.g;
      teintes[i * 3 + 2] = p.b;
    }
    this.maillage.count = n;
    this.maillage.instanceMatrix.needsUpdate = true;
    this.cases.needsUpdate = true;
    this.teintes.needsUpdate = true;
  }

  /** Une case par image, agrandie sans lissage ; une image absente devient un carré blanc. */
  private construireAtlas(images: readonly CheminTexture[], textures: BibliothequeTextures): THREE.Texture {
    const toile = document.createElement('canvas');
    toile.width = COLONNES * CASE;
    toile.height = this.lignes * CASE;
    const ctx = toile.getContext('2d')!;
    ctx.imageSmoothingEnabled = false;
    images.forEach((chemin, index) => {
      this.casesImages.set(chemin, index);
      const x = (index % COLONNES) * CASE;
      const y = Math.floor(index / COLONNES) * CASE;
      const source = textures.obtenir(chemin)?.image as CanvasImageSource | undefined;
      if (source) {
        ctx.drawImage(source, x, y, CASE, CASE);
      } else {
        ctx.fillStyle = '#ffffff';
        ctx.fillRect(x + CASE / 4, y + CASE / 4, CASE / 2, CASE / 2);
      }
    });
    const atlas = new THREE.CanvasTexture(toile);
    atlas.magFilter = THREE.NearestFilter;
    atlas.minFilter = THREE.NearestFilter;
    atlas.generateMipmaps = false;
    atlas.colorSpace = THREE.SRGBColorSpace;
    return atlas;
  }
}

function tirer(bornes: readonly [number, number]): number {
  return bornes[0] + Math.random() * (bornes[1] - bornes[0]);
}
