/**
 * Mécanique « pluie » (Pluie de flèches) : pendant la durée de l'arme, des
 * flèches tombent du ciel en des points au hasard du plus gros groupe
 * d'ennemis ; chacune blesse autour de son point d'impact, reste plantée un
 * moment dans le sol, puis s'y enfonce. Les flèches sont les entités du jeu
 * (plans croisés de la texture de la flèche), pointe en bas, chacune penchée
 * dans une direction au hasard. Un cercle doré marque la zone au sol et
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

/** Côté de la texture du cercle (pixels) : lisse, filtrée, nette même quand la zone grandit. */
const COTE_CERCLE = 256;

/**
 * Cercle au sol, façon marqueur de zone : anneau doré fin bordé d'un liseré
 * sombre (lisible sur l'herbe comme sur la pierre), reflet clair à
 * l'intérieur et léger dégradé doré qui monte vers le bord. Centre vide.
 */
function textureCercle(): THREE.CanvasTexture {
  const k = CONFIG.armes.pluie.cercle;
  const toile = document.createElement('canvas');
  toile.width = COTE_CERCLE;
  toile.height = COTE_CERCLE;
  const ctx = toile.getContext('2d');
  const c = COTE_CERCLE / 2;
  const r = c - 3;
  if (ctx) {
    const voile = ctx.createRadialGradient(c, c, r * 0.5, c, c, r);
    voile.addColorStop(0, k.voile.replace('ALPHA', '0'));
    voile.addColorStop(1, k.voile.replace('ALPHA', String(k.opaciteVoile)));
    ctx.fillStyle = voile;
    ctx.beginPath();
    ctx.arc(c, c, r, 0, Math.PI * 2);
    ctx.fill();
    const anneau = (rayon: number, epaisseur: number, couleurTrait: string) => {
      ctx.strokeStyle = couleurTrait;
      ctx.lineWidth = epaisseur;
      ctx.beginPath();
      ctx.arc(c, c, rayon, 0, Math.PI * 2);
      ctx.stroke();
    };
    anneau(r - 3, 8, k.contour);
    anneau(r - 3, 5, k.or);
    anneau(r - 8, 1.5, k.reflet);
  }
  const texture = new THREE.CanvasTexture(toile);
  texture.colorSpace = THREE.SRGBColorSpace;
  texture.anisotropy = 4;
  return texture;
}

export class Pluies {
  private readonly contexte: ContexteCombat;
  private readonly tampon = new Int32Array(64);
  private readonly zones: Pool<Zone>;
  private readonly fleches: Pool<FlecheTombante>;
  private readonly maillage: THREE.InstancedMesh;
  /** Un cercle par zone possible (chacun son opacité pour s'effacer). */
  private readonly cercles: THREE.Mesh<THREE.PlaneGeometry, THREE.MeshBasicMaterial>[] = [];

  constructor(contexte: ContexteCombat, textures: BibliothequeTextures, scene: THREE.Scene) {
    const c = CONFIG.armes.pluie;
    this.contexte = contexte;
    this.zones = new Pool<Zone>(c.zones, () => ({ arme: null, x: 0, z: 0, rayon: 0, duree: 0, temps: 0, prochaineFleche: 0 }));
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
    const motif = textureCercle();
    for (let i = 0; i < c.zones; i++) {
      const cercle = new THREE.Mesh(disque, new THREE.MeshBasicMaterial({ map: motif, transparent: true, depthWrite: false }));
      cercle.name = 'cercle de pluie';
      cercle.visible = false;
      cercle.renderOrder = 1;  // après les ombres rondes
      this.cercles.push(cercle);
      scene.add(cercle);
    }
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
        // Chaque flèche penche de son côté : la pluie a l'air désordonnée, pas une rangée au garde-à-vous
        f.azimut = Math.random() * Math.PI * 2;
        f.inclinaison = c.inclinaison[0] + Math.random() * (c.inclinaison[1] - c.inclinaison[0]);
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
      // Pointe en bas, penchée vers son azimut (elle arrive du côté opposé, en ligne droite)
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
    for (let i = 0; i < this.cercles.length; i++) {
      const cercle = this.cercles[i];
      cercle.visible = i < this.zones.nombre;
      if (!cercle.visible) continue;
      const z = this.zones.elements[i];
      const fin = z.duree + c.chute + k.reste;
      const apparition = Math.min(1, z.temps / k.apparition);
      const diametre = z.rayon * 2 * (0.85 + 0.15 * apparition);
      cercle.material.opacity = apparition * Math.max(0, Math.min(1, 1 - (z.temps - fin) / k.fondu));
      cercle.position.set(z.x, k.hauteur, z.z);
      cercle.scale.set(diametre, 1, diametre);
    }
  }
}
