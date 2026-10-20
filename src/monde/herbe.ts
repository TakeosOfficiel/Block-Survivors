/**
 * Herbe réaliste (test WebGPU) : des dizaines de milliers de brins effilés
 * plantés sur les blocs d'herbe libres de l'arène, dans une seule géométrie.
 * Tout le mouvement est calculé par la carte graphique (TSL, le langage de
 * shaders de three.js : WebGPU, ou WebGL2 en repli) : le vent ondule en
 * vagues sur la prairie et les brins se couchent autour du joueur.
 */
import * as THREE from 'three';
import { MeshLambertNodeMaterial } from 'three/webgpu';
import {
  attribute, clamp, color, dot, float, length, mix, normalize, positionLocal, sin, time, uniform, vec2, vec3,
} from 'three/tsl';
import { CONFIG } from '../config';
import { creerAleatoire, graine } from '../utils/aleatoire';
import type { MondeGenere } from './generation';

/** Niveaux d'un brin : 2 sommets par niveau, la pointe en a un seul. */
const NIVEAUX = 4;
const SOMMETS_PAR_BRIN = NIVEAUX * 2 + 1;
const TRIANGLES_PAR_BRIN = (NIVEAUX - 1) * 2 + 1;

export class Herbe {
  readonly maillage: THREE.Mesh;
  private readonly joueur = uniform(new THREE.Vector2());

  /** Crée l'herbe de l'arène, ou null si elle n'a aucun bloc d'herbe. */
  static creer(idArene: string, monde: MondeGenere, scene: THREE.Scene): Herbe | null {
    const cases = casesHerbe(monde);
    return cases.length > 0 ? new Herbe(idArene, cases, scene) : null;
  }

  private constructor(idArene: string, cases: readonly number[], scene: THREE.Scene) {
    this.maillage = new THREE.Mesh(creerGeometrie(idArene, cases), this.creerMateriau());
    this.maillage.name = 'herbe';
    this.maillage.frustumCulled = false;  // une seule géométrie pour toute l'arène
    scene.add(this.maillage);
  }

  set visible(visible: boolean) {
    this.maillage.visible = visible;
  }

  /** Position du joueur, autour de laquelle les brins se couchent. */
  majVisuel(x: number, z: number): void {
    this.joueur.value.set(x, z);
  }

  private creerMateriau(): MeshLambertNodeMaterial {
    const h = CONFIG.herbe;
    const base = attribute<'vec2'>('aBase', 'vec2');
    const hauteur = attribute<'float'>('aHauteur', 'float');
    const teinte = attribute<'float'>('aTeinte', 'float');

    // Vent : deux ondes qui traversent la prairie, plus un léger frémissement
    const phase = dot(base, vec2(0.35, 0.22));
    const force = sin(time.mul(1.6).add(phase)).mul(0.5).add(0.6)
      .add(sin(time.mul(4.3).add(base.x.mul(1.7))).mul(0.12));
    const vent = vec2(h.directionVent[0], h.directionVent[1]).mul(force.mul(h.vent));

    // Le joueur couche les brins proches, vers l'extérieur
    const ecart = base.sub(this.joueur);
    const proximite = clamp(float(1).sub(length(ecart).div(h.rayonCouche)), 0, 1);
    const couche = normalize(ecart.add(vec2(0.0001, 0))).mul(proximite.mul(h.couche));

    // Plus on monte dans le brin, plus il plie (courbe quadratique) ; il s'abaisse un peu en pliant
    const pliage = vent.add(couche).mul(hauteur.mul(hauteur));
    const abaissement = length(pliage).mul(0.45);
    const materiau = new MeshLambertNodeMaterial({ side: THREE.DoubleSide });
    materiau.positionNode = positionLocal.add(vec3(pliage.x, abaissement.negate(), pliage.y));

    // Couleur : pied sombre, pointe claire, chaque brin un peu différent
    const variation = teinte.mul(0.3).add(0.85);
    materiau.colorNode = mix(color(h.couleurPied), color(h.couleurPointe), hauteur).mul(variation);
    return materiau;
  }
}

/** Centres (x, z) des blocs d'herbe du sol sur lesquels on peut marcher. */
function casesHerbe(monde: MondeGenere): number[] {
  const positions = monde.blocs.get('grass_block') ?? [];
  const cases: number[] = [];
  for (let i = 0; i < positions.length; i += 3) {
    const [x, y, z] = [positions[i], positions[i + 1], positions[i + 2]];
    if (y < 0 && !monde.carte.bloqueMarche(x, z)) cases.push(x, z);
  }
  return cases;
}

/** Tous les brins, en coordonnées du monde (une seule géométrie, déterministe). */
function creerGeometrie(idArene: string, cases: readonly number[]): THREE.BufferGeometry {
  const h = CONFIG.herbe;
  const alea = creerAleatoire(graine(`herbe:${idArene}`));
  const nombre = (cases.length / 2) * h.densite;
  const positions = new Float32Array(nombre * SOMMETS_PAR_BRIN * 3);
  const normales = new Float32Array(nombre * SOMMETS_PAR_BRIN * 3);
  const bases = new Float32Array(nombre * SOMMETS_PAR_BRIN * 2);
  const hauteurs = new Float32Array(nombre * SOMMETS_PAR_BRIN);
  const teintes = new Float32Array(nombre * SOMMETS_PAR_BRIN);
  const indices = new Uint32Array(nombre * TRIANGLES_PAR_BRIN * 3);

  let s = 0;  // sommet courant
  let t = 0;  // index courant
  for (let c = 0; c < cases.length; c += 2) {
    for (let k = 0; k < h.densite; k++) {
      const bx = cases[c] + alea() - 0.5;
      const bz = cases[c + 1] + alea() - 0.5;
      const lacet = alea() * Math.PI * 2;
      const hauteur = h.hauteur[0] + alea() * (h.hauteur[1] - h.hauteur[0]);
      const penche = (alea() - 0.5) * 0.25;  // légère inclinaison naturelle
      const teinte = alea();
      const cosL = Math.cos(lacet);
      const sinL = Math.sin(lacet);
      const premier = s;

      for (let n = 0; n <= NIVEAUX; n++) {
        const r = n / NIVEAUX;
        const demiLargeur = (h.largeur / 2) * (1 - r);
        const decalage = penche * r * r * hauteur;
        const cotes = n === NIVEAUX ? [0] : [-demiLargeur, demiLargeur];
        for (const d of cotes) {
          positions.set([bx + cosL * d + sinL * decalage, r * hauteur, bz - sinL * d + cosL * decalage], s * 3);
          normales.set([0, 1, 0], s * 3);  // éclairées comme le sol : pas de brins noirs à contre-jour
          bases.set([bx, bz], s * 2);
          hauteurs[s] = r;
          teintes[s] = teinte;
          s++;
        }
      }
      for (let n = 0; n < NIVEAUX - 1; n++) {
        const a = premier + n * 2;
        indices.set([a, a + 1, a + 3, a, a + 3, a + 2], t);
        t += 6;
      }
      const dernier = premier + (NIVEAUX - 1) * 2;
      indices.set([dernier, dernier + 1, dernier + 2], t);
      t += 3;
    }
  }

  const geometrie = new THREE.BufferGeometry();
  geometrie.setAttribute('position', new THREE.BufferAttribute(positions, 3));
  geometrie.setAttribute('normal', new THREE.BufferAttribute(normales, 3));
  geometrie.setAttribute('aBase', new THREE.BufferAttribute(bases, 2));
  geometrie.setAttribute('aHauteur', new THREE.BufferAttribute(hauteurs, 1));
  geometrie.setAttribute('aTeinte', new THREE.BufferAttribute(teintes, 1));
  geometrie.setIndex(new THREE.BufferAttribute(indices, 1));
  return geometrie;
}
