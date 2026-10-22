/**
 * Mécanique « pluie » (Pluie de flèches) : pendant la durée de l'arme, des
 * flèches tombent du ciel en des points au hasard du plus gros groupe
 * d'ennemis ; chacune blesse autour de son point d'impact, reste plantée un
 * moment dans le sol, puis s'y enfonce. Les flèches sont les entités du jeu
 * (plans croisés de la texture de la flèche), pointe en bas, penchées comme
 * une volée tirée depuis le joueur. Un cercle doré marque la zone au sol et
 * reste un peu après la dernière flèche avant de s'effacer.
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
  rayon: number;
  /** Durée de la pluie (s), fixée au lancement. */
  duree: number;
  /** Direction d'où viennent les flèches (rad, depuis le joueur). */
  azimut: number;
  temps: number;
  prochaineFleche: number;
}

interface FlecheTombante {
  arme: ArmeEquipee | null;
  x: number;
  z: number;
  age: number;
  azimut: number;
  inclinaison: number;
  touche: boolean;
}

const groupe: Point = { x: 0, z: 0 };
const matrice = new THREE.Matrix4();
const position = new THREE.Vector3();
const orientation = new THREE.Quaternion();
const euler = new THREE.Euler(0, 0, 0, 'YXZ');
const echelle = new THREE.Vector3();
const axe = new THREE.Vector3();
const BAS = new THREE.Vector3(0, -1, 0);
const couleur = new THREE.Color();

/** Côté de la texture du cercle (pixels) : environ 16 pixels par bloc, comme les blocs du jeu. */
const COTE_CERCLE = 80;

/**
 * Cercle au sol en pixel art (blanc, teinté par instance) : anneau plein au
 * bord, anneau en tirets à l'intérieur, fond très léger. L'alpha sert
 * d'intensité (mélange additif).
 */
function textureCercle(): THREE.CanvasTexture {
  const toile = document.createElement('canvas');
  toile.width = COTE_CERCLE;
  toile.height = COTE_CERCLE;
  const ctx = toile.getContext('2d');
  const c = COTE_CERCLE / 2;
  if (ctx) {
    for (let y = 0; y < COTE_CERCLE; y++) {
      for (let x = 0; x < COTE_CERCLE; x++) {
        const dx = x + 0.5 - c;
        const dy = y + 0.5 - c;
        const d = Math.hypot(dx, dy);
        const tiret = Math.floor(((Math.atan2(dy, dx) + Math.PI) / (Math.PI * 2)) * 32) % 2 === 0;
        let alpha = 0;
        if (d <= c && d > c - 3) alpha = 1;
        else if (d <= c - 3 && d > c - 4) alpha = 0.45;
        else if (d <= c - 8 && d > c - 10 && tiret) alpha = 0.55;
        else if (d <= c - 3) alpha = 0.1;
        if (alpha === 0) continue;
        ctx.fillStyle = `rgba(255,255,255,${alpha})`;
        ctx.fillRect(x, y, 1, 1);
      }
    }
  }
  const texture = new THREE.CanvasTexture(toile);
  texture.magFilter = THREE.NearestFilter;
  texture.minFilter = THREE.NearestFilter;
  texture.generateMipmaps = false;
  texture.colorSpace = THREE.SRGBColorSpace;
  return texture;
}

export class Pluies {
  private readonly contexte: ContexteCombat;
  private readonly tampon = new Int32Array(64);
  private readonly zones: Pool<Zone>;
  private readonly fleches: Pool<FlecheTombante>;
  private readonly maillage: THREE.InstancedMesh;
  private readonly cercles: THREE.InstancedMesh;

  constructor(contexte: ContexteCombat, textures: BibliothequeTextures, scene: THREE.Scene) {
    const c = CONFIG.armes.pluie;
    this.contexte = contexte;
    this.zones = new Pool<Zone>(c.zones, () => ({ arme: null, x: 0, z: 0, rayon: 0, duree: 0, azimut: 0, temps: 0, prochaineFleche: 0 }));
    this.fleches = new Pool<FlecheTombante>(c.fleches, () => ({ arme: null, x: 0, z: 0, age: 0, azimut: 0, inclinaison: 0, touche: false }));
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

    const disque = new THREE.PlaneGeometry(1, 1).rotateX(-Math.PI / 2);
    const materiauCercle = new THREE.MeshBasicMaterial({
      map: textureCercle(), transparent: true, depthWrite: false, blending: THREE.AdditiveBlending,
    });
    this.cercles = new THREE.InstancedMesh(disque, materiauCercle, c.zones);
    this.cercles.name = 'cercles de pluie';
    this.cercles.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.cercles.setColorAt(0, couleur.set(c.cercle.couleur));  // crée l'attribut de couleur
    this.cercles.frustumCulled = false;
    this.cercles.count = 0;
    scene.add(this.cercles);
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
      z.rayon = arme.stats.zone;
      z.duree = arme.stats.duree;
      // Les flèches arrivent du côté du joueur, qui les a tirées en l'air
      z.azimut = Math.atan2(j.x - groupe.x, j.z - groupe.z);
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
      // Le cercle reste après la dernière flèche, le temps de s'effacer
      if (!arme || m?.type !== 'pluie' || z.temps > z.duree + c.chute + c.cercle.reste + c.cercle.fondu) {
        z.arme = null;
        this.zones.liberer(i);
        continue;
      }
      if (z.temps > z.duree) continue;
      z.prochaineFleche -= dt;
      while (z.prochaineFleche <= 0) {
        z.prochaineFleche += 1 / m.impactsParSeconde;
        const f = this.fleches.acquerir();
        if (!f) break;
        // Point au hasard dans le disque (uniforme)
        const a = Math.random() * Math.PI * 2;
        const r = z.rayon * Math.sqrt(Math.random());
        f.arme = arme;
        f.x = z.x + Math.sin(a) * r;
        f.z = z.z + Math.cos(a) * r;
        f.age = 0;
        f.azimut = z.azimut + (Math.random() * 2 - 1) * c.ecartAzimut;
        f.inclinaison = c.inclinaison + (Math.random() * 2 - 1) * c.ecartInclinaison;
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
      if (f.age >= c.chute + c.plantee + c.enfoncement) {
        f.arme = null;
        this.fleches.liberer(i);
      }
    }
  }

  majVisuel(): void {
    const c = CONFIG.armes.pluie;
    const longueur = PROJECTILES.fleche.longueur * c.taille;
    echelle.setScalar(c.taille);
    let n = 0;
    for (let i = 0; i < this.fleches.nombre; i++) {
      const f = this.fleches.elements[i];
      // Pointe en bas, penchée : elle file à l'opposé du joueur, queue relevée de son côté
      orientation.setFromEuler(euler.set(f.inclinaison, f.azimut, 0));
      axe.copy(BAS).applyQuaternion(orientation);  // sens de la chute, vers la pointe
      // Recule le long de son axe : arrive en ligne droite, puis plantée (pointe enfoncée de
      // 20 % de sa longueur), puis s'enfonce dans le sol avant de disparaître
      const p = Math.min(1, f.age / c.chute);
      const enfoncement = Math.max(0, f.age - c.chute - c.plantee) / c.enfoncement;
      const recul = (c.hauteur / -axe.y) * (1 - p) + longueur * (0.3 - 0.6 * Math.min(1, enfoncement));
      position.set(f.x, 0, f.z).addScaledVector(axe, -recul);
      matrice.compose(position, orientation, echelle);
      this.maillage.setMatrixAt(n++, matrice);
    }
    this.maillage.count = n;
    this.maillage.instanceMatrix.needsUpdate = true;

    // Cercles : grandissent à l'apparition, restent, puis s'effacent
    const k = c.cercle;
    let m = 0;
    for (let i = 0; i < this.zones.nombre; i++) {
      const z = this.zones.elements[i];
      const fin = z.duree + c.chute + k.reste;
      const apparition = Math.min(1, z.temps / k.apparition);
      const intensite = apparition * Math.max(0, Math.min(1, 1 - (z.temps - fin) / k.fondu));
      const diametre = z.rayon * 2 * (0.7 + 0.3 * apparition);
      matrice.makeScale(diametre, 1, diametre).setPosition(z.x, k.hauteur, z.z);
      this.cercles.setMatrixAt(m, matrice);
      this.cercles.setColorAt(m++, couleur.set(k.couleur).multiplyScalar(intensite));
    }
    this.cercles.count = m;
    this.cercles.instanceMatrix.needsUpdate = true;
    if (this.cercles.instanceColor) this.cercles.instanceColor.needsUpdate = true;
  }
}
