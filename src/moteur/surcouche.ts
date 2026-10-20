/**
 * Surcouche de couleur par instance, comme l'« overlay » des entités Minecraft :
 * rouge quand une entité est blessée, blanc quand un creeper va exploser.
 *
 * Chaque instance porte un vec4 (r, g, b, force) ; la couleur finale du
 * matériau est mélangée vers cette couleur (en TSL, le langage de shaders de
 * three.js, valable en WebGPU comme en WebGL2). Une multiplication de couleur
 * ne permettrait pas d'éclaircir jusqu'au blanc.
 */
import * as THREE from 'three';
import { MeshLambertNodeMaterial } from 'three/webgpu';
import { attribute, mix, output, vec4 } from 'three/tsl';

export const NOM_ATTRIBUT_SURCOUCHE = 'aSurcouche';

/** Attribut d'instance à partager entre toutes les géométries d'un même modèle. */
export function creerAttributSurcouche(capacite: number): THREE.InstancedBufferAttribute {
  const attribut = new THREE.InstancedBufferAttribute(new Float32Array(capacite * 4), 4);
  attribut.setUsage(THREE.DynamicDrawUsage);
  return attribut;
}

/** Matériau Lambert dont la couleur finale est mélangée vers la surcouche de l'instance. */
export function creerMateriauSurcouche(parametres: THREE.MeshLambertMaterialParameters): MeshLambertNodeMaterial {
  const materiau = new MeshLambertNodeMaterial(parametres);
  const surcouche = attribute<'vec4'>(NOM_ATTRIBUT_SURCOUCHE, 'vec4');
  materiau.outputNode = vec4(mix(output.rgb, surcouche.rgb, surcouche.a), output.a);
  return materiau;
}
