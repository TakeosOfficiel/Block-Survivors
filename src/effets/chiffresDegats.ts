/**
 * Chiffres des dégâts qui jaillissent au-dessus des ennemis touchés, comme
 * dans les MMO : blancs pour un coup normal, dorés et plus gros (avec un
 * « pop ») pour un coup critique, petits et orange ou verts pour les dégâts
 * d'une brûlure ou d'un poison. Police pixel du jeu, chaque chiffre est un
 * carré d'un InstancedMesh tourné vers la caméra ; anneau de taille fixe : un
 * nouveau chiffre remplace le plus ancien quand tout est occupé.
 */
import * as THREE from 'three';
import { MeshBasicNodeMaterial } from 'three/webgpu';
import { attribute, texture, uv, vec2, vec4 } from 'three/tsl';
import { NOM_POLICE } from '../assets/police';
import { CONFIG } from '../config';

export type StyleChiffre = 'normal' | 'critique' | 'feu' | 'poison';

interface Chiffre {
  actif: boolean;
  x: number; y: number; z: number;
  /** Dérive horizontale (blocs/s, le long de l'écran). */
  derive: number;
  valeur: number;
  nbChiffres: number;
  style: StyleChiffre;
  age: number;
}

/** Atlas : une case par chiffre, police du jeu à 4 px par pixel de police. */
const CASE_L = 32;
const CASE_H = 40;
const AVANCE = 24 / CASE_L;   // avance d'un chiffre (6 px de police) en largeur de case

// Temporaires réutilisés
const matrice = new THREE.Matrix4();
const position = new THREE.Vector3();
const echelle = new THREE.Vector3();
const droite = new THREE.Vector3();

export class ChiffresDegats {
  /** Désactivable dans les options. */
  actif = true;
  private readonly chiffres: Chiffre[];
  private prochain = 0;
  private readonly maillage: THREE.InstancedMesh;
  private readonly camera: THREE.Camera;
  private readonly couleurs: THREE.InstancedBufferAttribute;
  private readonly cases: THREE.InstancedBufferAttribute;
  /** Couleur de chaque style, convertie en linéaire. */
  private readonly teintes: Record<StyleChiffre, THREE.Color> = {
    normal: new THREE.Color(CONFIG.chiffresDegats.normal.couleur),
    critique: new THREE.Color(CONFIG.chiffresDegats.critique.couleur),
    feu: new THREE.Color(CONFIG.chiffresDegats.feu.couleur),
    poison: new THREE.Color(CONFIG.chiffresDegats.poison.couleur),
  };

  constructor(scene: THREE.Scene, camera: THREE.Camera) {
    const c = CONFIG.chiffresDegats;
    this.camera = camera;
    this.chiffres = Array.from({ length: c.capacite }, () => ({
      actif: false, x: 0, y: 0, z: 0, derive: 0, valeur: 0, nbChiffres: 1, style: 'normal', age: 0,
    }));

    const instances = c.capacite * c.chiffresMax;
    const geometrie = new THREE.PlaneGeometry(1, CASE_H / CASE_L);
    this.cases = new THREE.InstancedBufferAttribute(new Float32Array(instances * 2), 2);
    this.couleurs = new THREE.InstancedBufferAttribute(new Float32Array(instances * 4), 4);
    this.cases.setUsage(THREE.DynamicDrawUsage);
    this.couleurs.setUsage(THREE.DynamicDrawUsage);
    geometrie.setAttribute('aCase', this.cases);
    geometrie.setAttribute('aCouleur', this.couleurs);

    // Toujours lisibles : par-dessus le décor, sans écrire la profondeur
    const materiau = new MeshBasicNodeMaterial({ transparent: true, depthTest: false, depthWrite: false });
    const image = texture(creerAtlas(), uv().mul(vec2(0.1, 1)).add(attribute('aCase', 'vec2')));
    const couleur = attribute('aCouleur', 'vec4');
    materiau.colorNode = vec4(image.rgb.mul(couleur.rgb), image.a.mul(couleur.a));

    this.maillage = new THREE.InstancedMesh(geometrie, materiau, instances);
    this.maillage.name = 'chiffres';
    this.maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.maillage.frustumCulled = false;
    this.maillage.renderOrder = 10;
    this.maillage.count = 0;
    scene.add(this.maillage);
  }

  ajouter(x: number, y: number, z: number, valeur: number, style: StyleChiffre): void {
    if (!this.actif) return;
    const ch = this.chiffres[this.prochain];
    this.prochain = (this.prochain + 1) % this.chiffres.length;
    ch.actif = true;
    // Un peu d'écart : les coups simultanés sur une foule ne s'empilent pas
    const e = CONFIG.chiffresDegats.ecart;
    ch.x = x + (Math.random() - 0.5) * e;
    ch.y = y + Math.random() * e * 0.6;
    ch.z = z + (Math.random() - 0.5) * e;
    ch.derive = (Math.random() - 0.5) * CONFIG.chiffresDegats.derive;
    ch.valeur = Math.max(1, Math.round(valeur));
    ch.nbChiffres = Math.min(CONFIG.chiffresDegats.chiffresMax, String(ch.valeur).length);
    ch.style = style;
    ch.age = 0;
  }

  maj(dt: number): void {
    const c = CONFIG.chiffresDegats;
    for (const ch of this.chiffres) {
      if (!ch.actif) continue;
      ch.age += dt;
      const style = c[ch.style];
      if (ch.age >= style.duree) ch.actif = false;
    }
  }

  majVisuel(): void {
    const c = CONFIG.chiffresDegats;
    const quaternion = this.camera.quaternion;
    droite.set(1, 0, 0).applyQuaternion(quaternion);
    const cases = this.cases.array as Float32Array;
    const couleurs = this.couleurs.array as Float32Array;
    let n = 0;
    for (const ch of this.chiffres) {
      if (!ch.actif) continue;
      const style = c[ch.style];
      const t = ch.age / style.duree;
      // Montée qui ralentit, « pop » des critiques au début, fondu à la fin
      const montee = style.montee * (1 - (1 - t) * (1 - t));
      const pop = 1 + style.pop * Math.max(0, 1 - ch.age / c.dureePop);
      const taille = style.taille * pop;
      const alpha = t < 1 - c.fondu ? 1 : (1 - t) / c.fondu;
      const couleur = this.teintes[ch.style];
      const decalage = ch.derive * ch.age;
      // Chiffres centrés, de gauche à droite (valeur lue en base 10 depuis la droite)
      let reste = ch.valeur;
      for (let k = ch.nbChiffres - 1; k >= 0; k--) {
        const chiffre = reste % 10;
        reste = Math.floor(reste / 10);
        const cote = (k - (ch.nbChiffres - 1) / 2) * AVANCE * taille + decalage;
        position.set(ch.x, ch.y + montee, ch.z).addScaledVector(droite, cote);
        matrice.compose(position, quaternion, echelle.set(taille, taille, taille));
        this.maillage.setMatrixAt(n, matrice);
        cases[n * 2] = chiffre / 10;
        cases[n * 2 + 1] = 0;
        couleurs[n * 4] = couleur.r;
        couleurs[n * 4 + 1] = couleur.g;
        couleurs[n * 4 + 2] = couleur.b;
        couleurs[n * 4 + 3] = alpha;
        n++;
      }
    }
    this.maillage.count = n;
    this.maillage.instanceMatrix.needsUpdate = true;
    this.cases.needsUpdate = true;
    this.couleurs.needsUpdate = true;
  }
}

/** Chiffres 0 à 9 en blanc avec l'ombre portée du texte du jeu (quart de la couleur, 1 pixel plus bas à droite). */
function creerAtlas(): THREE.Texture {
  const toile = document.createElement('canvas');
  toile.width = CASE_L * 10;
  toile.height = CASE_H;
  const ctx = toile.getContext('2d')!;
  ctx.font = `32px "${NOM_POLICE}", monospace`;
  ctx.textBaseline = 'alphabetic';
  for (let i = 0; i < 10; i++) {
    const x = i * CASE_L + 2;
    ctx.fillStyle = '#3f3f3f';
    ctx.fillText(String(i), x + 4, 32);
    ctx.fillStyle = '#ffffff';
    ctx.fillText(String(i), x, 28);
  }
  const atlas = new THREE.CanvasTexture(toile);
  atlas.magFilter = THREE.NearestFilter;
  atlas.minFilter = THREE.NearestFilter;
  atlas.generateMipmaps = false;
  atlas.colorSpace = THREE.SRGBColorSpace;
  return atlas;
}
