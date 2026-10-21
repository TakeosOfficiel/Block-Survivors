/**
 * Feuillage et plantes qui ondulent au vent quand les shaders sont actifs,
 * comme les « waving plants » des packs de shaders du jeu : les feuilles
 * frémissent, le haut des herbes et des fleurs se balance. Les vagues de vent
 * suivent la position de chaque bloc : elles traversent la prairie.
 *
 * Les matériaux animés remplacent ceux d'origine dès la construction ; sans
 * shaders, la force du vent est simplement nulle (rien à recompiler). Les
 * maps importées (un seul maillage fusionné) portent leur ondulation dans
 * l'attribut `aVent` de chaque sommet (voir minecraft/maillage.ts).
 */
import * as THREE from 'three';
import { MeshLambertNodeMaterial, type UniformNode } from 'three/webgpu';
import { attribute, cos, dot, positionLocal, sin, step, time, uniform, vec2, vec3 } from 'three/tsl';
import { CONFIG } from '../config';
import { BLOCS, IDS_BLOCS } from '../donnees/blocs';
import { VENT_FEUILLAGE, VENT_PLANTE } from '../minecraft/maillage';

export class Vegetation {
  /** 0 = immobile (shaders désactivés), 1 = vent normal. */
  private readonly force = uniform(0);

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

      const convertir = (m: THREE.Material) => creerMateriauOndulant(m as THREE.MeshLambertMaterial, mode, this.force);
      maillage.geometry = geometrie;
      maillage.material = Array.isArray(maillage.material) ? maillage.material.map(convertir) : convertir(maillage.material);
    }
    // Maps importées : feuilles et plantes repérées sommet par sommet
    scene.traverse((objet) => {
      const maillage = objet as THREE.Mesh;
      if (!maillage.isMesh || !maillage.geometry.getAttribute('aVent') || Array.isArray(maillage.material)) return;
      maillage.material = creerMateriauVentImporte(maillage.material as THREE.MeshLambertMaterial, this.force);
    });
  }

  definirShaders(actifs: boolean): void {
    this.force.value = actifs ? 1 : 0;
  }
}

function creerMateriauOndulant(
  origine: THREE.MeshLambertMaterial,
  mode: 'feuillage' | 'plante',
  force: UniformNode<'float', number>,
): MeshLambertNodeMaterial {
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
  materiau.positionNode = p.add(decalage.mul(force));
  return materiau;
}

/**
 * Même vent pour une map importée : chaque sommet sait s'il appartient à un
 * feuillage ou à une plante, où est le centre de son bloc et le pied de la plante.
 */
function creerMateriauVentImporte(origine: THREE.MeshLambertMaterial, force: UniformNode<'float', number>): MeshLambertNodeMaterial {
  const v = CONFIG.shaders.vent;
  const materiau = new MeshLambertNodeMaterial({
    map: origine.map, vertexColors: origine.vertexColors, alphaTest: origine.alphaTest, side: origine.side,
    transparent: origine.transparent, depthWrite: origine.depthWrite,
  });
  const vent = attribute<'vec4'>('aVent', 'vec4');
  const mode = vent.w;
  const feuillage = step(VENT_FEUILLAGE - 0.5, mode).mul(step(mode, VENT_FEUILLAGE + 0.5));
  const plante = step(VENT_PLANTE - 0.5, mode);
  const direction = vec2(v.direction[0], v.direction[1]);
  const phase = dot(vent.xz, direction).mul(0.6).sub(time.mul(v.vitesse));
  const p = positionLocal;
  // Plante : pied fixe, sommet qui se balance (deux fois plus haut pour la moitié haute d'une grande plante)
  const hauteur = p.y.sub(vent.y);
  const balancement = sin(phase).mul(v.plantes).add(sin(time.mul(4.3).add(vent.x.mul(3.1))).mul(v.plantes * 0.25));
  const decalagePlante = vec3(direction.x.mul(balancement), 0, direction.y.mul(balancement)).mul(hauteur);
  // Feuillage : tout le bloc frémit, chaque sommet un peu différemment
  const decalageFeuillage = vec3(
    sin(phase.add(p.y.mul(4))),
    sin(phase.mul(1.3).add(p.x.mul(4))).mul(0.5),
    cos(phase.add(p.z.mul(4))),
  ).mul(v.feuillage);
  materiau.positionNode = p.add(decalagePlante.mul(plante).add(decalageFeuillage.mul(feuillage)).mul(force));
  return materiau;
}
