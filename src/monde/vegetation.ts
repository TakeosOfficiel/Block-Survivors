/**
 * Feuillage et plantes qui ondulent au vent quand les shaders sont actifs,
 * comme les « waving plants » des packs de shaders du jeu : les feuilles
 * frémissent, le haut des herbes et des fleurs se balance. Les vagues de vent
 * suivent la position de chaque bloc : elles traversent la prairie.
 * Sans shaders, les blocs gardent leurs matériaux d'origine, immobiles.
 */
import * as THREE from 'three';
import { MeshLambertNodeMaterial } from 'three/webgpu';
import { attribute, cos, dot, positionLocal, sin, time, vec2, vec3 } from 'three/tsl';
import { CONFIG } from '../config';
import { BLOCS, IDS_BLOCS } from '../donnees/blocs';

interface Ondulant {
  readonly maillage: THREE.InstancedMesh;
  readonly origine: { geometrie: THREE.BufferGeometry; materiau: THREE.Material | THREE.Material[] };
  readonly shader: { geometrie: THREE.BufferGeometry; materiau: THREE.Material | THREE.Material[] };
}

export class Vegetation {
  private readonly ondulants: Ondulant[] = [];

  constructor(scene: THREE.Scene) {
    for (const id of IDS_BLOCS) {
      const mode = BLOCS[id].ondule;
      const maillage = scene.getObjectByName(id) as THREE.InstancedMesh | undefined;
      if (!mode || !maillage?.isInstancedMesh) continue;

      // Centre de chaque bloc, pour que les vagues de vent soient cohérentes d'un bloc à l'autre
      const centres = new Float32Array(maillage.count * 3);
      const matrices = maillage.instanceMatrix.array;
      for (let i = 0; i < maillage.count; i++) centres.set([matrices[i * 16 + 12], matrices[i * 16 + 13], matrices[i * 16 + 14]], i * 3);
      const geometrie = maillage.geometry.clone();
      geometrie.setAttribute('aCentre', new THREE.InstancedBufferAttribute(centres, 3));

      const convertir = (m: THREE.Material) => creerMateriauOndulant(m as THREE.MeshLambertMaterial, mode);
      const materiau = Array.isArray(maillage.material) ? maillage.material.map(convertir) : convertir(maillage.material);
      this.ondulants.push({
        maillage,
        origine: { geometrie: maillage.geometry, materiau: maillage.material },
        shader: { geometrie, materiau },
      });
    }
  }

  definirShaders(actifs: boolean): void {
    for (const o of this.ondulants) {
      const etat = actifs ? o.shader : o.origine;
      o.maillage.geometry = etat.geometrie;
      o.maillage.material = etat.materiau;
    }
  }
}

function creerMateriauOndulant(origine: THREE.MeshLambertMaterial, mode: 'feuillage' | 'plante'): MeshLambertNodeMaterial {
  const v = CONFIG.shaders.vent;
  const materiau = new MeshLambertNodeMaterial({
    map: origine.map, color: origine.color, alphaTest: origine.alphaTest, side: origine.side,
  });
  const centre = attribute<'vec3'>('aCentre', 'vec3');
  const direction = vec2(v.direction[0], v.direction[1]);
  // Vague qui avance dans la direction du vent, plus un frémissement propre à chaque bloc
  const phase = dot(centre.xz, direction).mul(0.6).sub(time.mul(v.vitesse));
  const p = positionLocal;
  let decalage;
  if (mode === 'plante') {
    // Pied fixe (y = −0,5), sommet qui se balance
    const hauteur = p.y.add(0.5);
    const balancement = sin(phase).mul(v.plantes).add(sin(time.mul(4.3).add(centre.x.mul(3.1))).mul(v.plantes * 0.25));
    decalage = vec3(direction.x.mul(balancement), 0, direction.y.mul(balancement)).mul(hauteur);
  } else {
    // Tout le bloc frémit, chaque sommet un peu différemment
    decalage = vec3(
      sin(phase.add(p.y.mul(4))),
      sin(phase.mul(1.3).add(p.x.mul(4))).mul(0.5),
      cos(phase.add(p.z.mul(4))),
    ).mul(v.feuillage);
  }
  materiau.positionNode = p.add(decalage);
  return materiau;
}
