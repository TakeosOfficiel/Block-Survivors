/**
 * Herbe en brins (styles « réaliste » et « stylisée ») : un brin modèle,
 * effilé et courbé, dessiné des centaines de milliers de fois (instanciation),
 * chacun avec sa position, son orientation, sa taille et sa teinte. Dégradé
 * du pied à la pointe, plaques plus sèches, vagues de lumière des rafales.
 */
import * as THREE from 'three';
import { MeshLambertNodeMaterial } from 'three/webgpu';
import { attribute, color, float, length, max, mix, pow, sin, smoothstep, uniform, vec2, vec3, cos, positionLocal } from 'three/tsl';
import type { PresetBrins } from '../../config';
import { creerAleatoire, graine } from '../../utils/aleatoire';
import { pliageVent } from './vent';
import type { TapisHerbe } from './herbe';

/** Niveaux d'un brin : 2 sommets par niveau, un seul à la pointe. */
const NIVEAUX = 5;

export class Brins implements TapisHerbe {
  readonly maillage: THREE.Mesh;
  readonly assombritSol = true;
  private readonly joueur = uniform(new THREE.Vector2());

  constructor(graineTexte: string, cases: readonly number[], preset: PresetBrins, scene: THREE.Scene) {
    this.maillage = new THREE.Mesh(creerGeometrie(graineTexte, cases, preset), this.creerMateriau(preset));
    this.maillage.name = 'herbe';
    this.maillage.frustumCulled = false;  // tout le tapis tient dans une géométrie
    this.maillage.receiveShadow = true;  // ombres du soleil (shaders)
    scene.add(this.maillage);
  }

  majVisuel(x: number, z: number): void {
    this.joueur.value.set(x, z);
  }

  private creerMateriau(p: PresetBrins): MeshLambertNodeMaterial {
    // Sommet du brin modèle : x = côté (−1, 0, 1), y = hauteur relative (0 au pied, 1 à la pointe)
    const cote = positionLocal.x;
    const t = positionLocal.y;
    // Par brin : (x, z, orientation, hauteur) et (largeur, courbure, teinte, phase)
    const racine = attribute<'vec4'>('aRacine', 'vec4');
    const forme = attribute<'vec4'>('aForme', 'vec4');
    const hauteur = racine.w;
    const { pliage: vent, rafale, base } = pliageVent(this.joueur, forme.w);

    // Repère du brin : il fait face à `devant`, sa largeur suit `droite`
    const devant = vec2(sin(racine.z), cos(racine.z));
    const droite = vec2(devant.y, devant.x.negate());

    // Pliage : courbure naturelle + vent + joueur, de plus en plus marqué vers la pointe
    const pliage = devant.mul(forme.y.mul(hauteur)).add(vent).mul(t.mul(t));
    const largeur = forme.x.mul(pow(float(1).sub(t), 0.7));  // effilé jusqu'à la pointe
    const xz = base.add(droite.mul(cote.mul(largeur))).add(pliage);
    // En pliant, le brin s'abaisse (sa longueur reste à peu près constante)
    const y = t.mul(hauteur).sub(length(pliage).mul(t).mul(0.55));
    // Matériau éclairé : le tapis reçoit la lumière et les ombres du soleil (shaders)
    const materiau = new MeshLambertNodeMaterial({ side: THREE.DoubleSide });
    materiau.positionNode = vec3(xz.x, max(y, 0), xz.y);

    // Couleur : pied dans l'ombre du tapis, milieu vert, pointe claire, plaques plus sèches
    const plaque = sin(base.x.mul(0.21).add(sin(base.y.mul(0.17)).mul(2.1))).mul(0.5).add(0.5)
      .mul(sin(base.y.mul(0.13).add(base.x.mul(0.07))).mul(0.5).add(0.5));
    const vert = mix(color(p.couleurMilieu), color(p.couleurSeche), smoothstep(0.55, 1, plaque).mul(0.7).add(forme.z.mul(0.15)));
    const pointe = mix(vert, color(p.couleurPointe), smoothstep(0.55, 1, t).mul(0.6));
    const ombre = mix(float(p.ombrePied), float(1), smoothstep(0, 0.75, t));
    // Les rafales éclaircissent les brins qu'elles couchent : des vagues de lumière parcourent le champ
    const reflet = rafale.mul(t).mul(p.refletRafales).add(1);
    materiau.colorNode = mix(color(p.couleurPied), pointe, ombre).mul(reflet).mul(forme.z.mul(0.2).add(0.9));
    return materiau;
  }
}

/** Le brin modèle, instancié pour chaque brin du tapis avec ses attributs propres. */
function creerGeometrie(graineTexte: string, cases: readonly number[], h: PresetBrins): THREE.InstancedBufferGeometry {
  const geometrie = new THREE.InstancedBufferGeometry();

  // Brin modèle : x = côté, y = hauteur relative
  const sommets: number[] = [];
  for (let n = 0; n < NIVEAUX; n++) sommets.push(-1, n / NIVEAUX, 0, 1, n / NIVEAUX, 0);
  sommets.push(0, 1, 0);
  const indices: number[] = [];
  for (let n = 0; n < NIVEAUX - 1; n++) {
    const a = n * 2;
    indices.push(a, a + 1, a + 3, a, a + 3, a + 2);
  }
  const dernier = (NIVEAUX - 1) * 2;
  indices.push(dernier, dernier + 1, dernier + 2);
  geometrie.setAttribute('position', new THREE.Float32BufferAttribute(sommets, 3));
  // Normales vers le ciel : le tapis est éclairé comme le sol, sans brins noirs à contre-jour
  geometrie.setAttribute('normal', new THREE.Float32BufferAttribute(sommets.map((_, k) => (k % 3 === 1 ? 1 : 0)), 3));
  geometrie.setIndex(indices);

  // Un brin par tirage, en touffes : certains blocs sont plus fournis que d'autres
  const alea = creerAleatoire(graine(graineTexte));
  const nombre = (cases.length / 2) * h.densite;
  const racines = new Float32Array(nombre * 4);
  const formes = new Float32Array(nombre * 4);
  let i = 0;
  for (let c = 0; c < cases.length; c += 2) {
    for (let k = 0; k < h.densite; k++, i++) {
      const hauteur = h.hauteur[0] + alea() ** 1.5 * (h.hauteur[1] - h.hauteur[0]);
      racines.set([cases[c] + alea() - 0.5, cases[c + 1] + alea() - 0.5, alea() * Math.PI * 2, hauteur], i * 4);
      formes.set([
        h.largeur * (0.7 + alea() * 0.6),   // largeur à la base
        0.15 + alea() * 0.35,               // courbure naturelle (part de la hauteur)
        alea(),                             // teinte
        alea() * Math.PI * 2,               // phase du frémissement
      ], i * 4);
    }
  }
  geometrie.setAttribute('aRacine', new THREE.InstancedBufferAttribute(racines, 4));
  geometrie.setAttribute('aForme', new THREE.InstancedBufferAttribute(formes, 4));
  geometrie.instanceCount = nombre;
  return geometrie;
}
