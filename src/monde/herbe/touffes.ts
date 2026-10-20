/**
 * Touffes ondulantes : le sprite d'herbe de Minecraft (short_grass, en
 * croix), posé en nombre sur les blocs d'herbe et qui ondule au vent, comme
 * les « waving plants » des packs de shaders. Reste fidèle au pixel art.
 */
import * as THREE from 'three';
import { MeshLambertNodeMaterial } from 'three/webgpu';
import { attribute, color, cos, length, max, positionLocal, sin, texture, uniform, uv, vec3, vec4 } from 'three/tsl';
import { CONFIG } from '../../config';
import { creerAleatoire, graine } from '../../utils/aleatoire';
import type { TapisHerbe } from './herbe';
import { pliageVent } from './vent';

export class Touffes implements TapisHerbe {
  readonly maillage: THREE.Mesh;
  readonly assombritSol = false;
  private readonly joueur = uniform(new THREE.Vector2());

  constructor(graineTexte: string, cases: readonly number[], carte: THREE.Texture, scene: THREE.Scene) {
    this.maillage = new THREE.Mesh(creerGeometrie(graineTexte, cases), this.creerMateriau(carte));
    this.maillage.name = 'herbe';
    this.maillage.frustumCulled = false;
    this.maillage.receiveShadow = true;  // ombres du soleil (shaders)
    scene.add(this.maillage);
  }

  majVisuel(x: number, z: number): void {
    this.joueur.value.set(x, z);
  }

  private creerMateriau(carte: THREE.Texture): MeshLambertNodeMaterial {
    const racine = attribute<'vec4'>('aRacine', 'vec4');
    const forme = attribute<'vec4'>('aForme', 'vec4');
    const taille = racine.w;
    const t = positionLocal.y;
    const { pliage: vent, base } = pliageVent(this.joueur, forme.y);

    // Croix tournée selon l'instance ; seul le haut de la touffe plie
    const c = cos(racine.z);
    const s = sin(racine.z);
    const x = positionLocal.x.mul(c).sub(positionLocal.z.mul(s));
    const z = positionLocal.x.mul(s).add(positionLocal.z.mul(c));
    const pliage = vent.mul(t.mul(t));
    const materiau = new MeshLambertNodeMaterial({ side: THREE.DoubleSide, alphaTest: 0.5 });
    materiau.positionNode = vec3(
      base.x.add(x.mul(taille)).add(pliage.x),
      max(t.mul(taille).sub(length(pliage).mul(t).mul(0.3)), 0),
      base.y.add(z.mul(taille)).add(pliage.y),
    );

    // Sprite vanilla teinté comme en jeu, chaque touffe un peu différente
    const pixel = texture(carte, uv());
    const teinte = color(CONFIG.herbe.touffes.teinte).mul(forme.x.mul(0.25).add(0.88));
    materiau.colorNode = vec4(pixel.rgb.mul(teinte), pixel.a);
    return materiau;
  }
}

/** Deux plans en croix (diagonales du bloc), le pied à y = 0, comme les plantes du jeu. */
function creerGeometrie(graineTexte: string, cases: readonly number[]): THREE.InstancedBufferGeometry {
  const t = CONFIG.herbe.touffes;
  const geometrie = new THREE.InstancedBufferGeometry();
  const d = 0.5;
  geometrie.setAttribute('position', new THREE.Float32BufferAttribute([
    -d, 0, -d, d, 0, d, d, 1, d, -d, 1, -d,
    -d, 0, d, d, 0, -d, d, 1, -d, -d, 1, d,
  ], 3));
  geometrie.setAttribute('uv', new THREE.Float32BufferAttribute([0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1], 2));
  geometrie.setAttribute('normal', new THREE.Float32BufferAttribute(Array.from({ length: 8 }, () => [0, 1, 0]).flat(), 3));
  geometrie.setIndex([0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7]);

  const alea = creerAleatoire(graine(graineTexte));
  const racines: number[] = [];
  const formes: number[] = [];
  for (let c = 0; c < cases.length; c += 2) {
    // En moyenne `parBloc` touffes par bloc, placées n'importe où sur le bloc
    const nombre = Math.floor(t.parBloc) + (alea() < t.parBloc % 1 ? 1 : 0);
    for (let k = 0; k < nombre; k++) {
      racines.push(cases[c] + (alea() - 0.5) * 0.8, cases[c + 1] + (alea() - 0.5) * 0.8, alea() * Math.PI, t.taille[0] + alea() * (t.taille[1] - t.taille[0]));
      formes.push(alea(), alea() * Math.PI * 2, 0, 0);
    }
  }
  geometrie.setAttribute('aRacine', new THREE.InstancedBufferAttribute(new Float32Array(racines), 4));
  geometrie.setAttribute('aForme', new THREE.InstancedBufferAttribute(new Float32Array(formes), 4));
  geometrie.instanceCount = racines.length / 4;
  return geometrie;
}
