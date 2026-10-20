/**
 * Surcouche de couleur par instance, comme l'« overlay » des entités Minecraft :
 * rouge quand une entité est blessée, blanc quand un creeper va exploser.
 *
 * Chaque instance porte un vec4 (r, g, b, force) ; le fragment final est
 * mélangé vers cette couleur. Une multiplication de couleur ne permettrait pas
 * d'éclaircir jusqu'au blanc.
 */
import * as THREE from 'three';

export const NOM_ATTRIBUT_SURCOUCHE = 'aSurcouche';

/** Attribut d'instance à partager entre toutes les géométries d'un même modèle. */
export function creerAttributSurcouche(capacite: number): THREE.InstancedBufferAttribute {
  const attribut = new THREE.InstancedBufferAttribute(new Float32Array(capacite * 4), 4);
  attribut.setUsage(THREE.DynamicDrawUsage);
  return attribut;
}

/** Ajoute la surcouche au shader d'un matériau three.js standard. */
export function activerSurcouche(materiau: THREE.Material): void {
  materiau.onBeforeCompile = (shader) => {
    shader.vertexShader = shader.vertexShader
      .replace('#include <common>', `#include <common>\nattribute vec4 ${NOM_ATTRIBUT_SURCOUCHE};\nvarying vec4 vSurcouche;`)
      .replace('#include <begin_vertex>', `#include <begin_vertex>\nvSurcouche = ${NOM_ATTRIBUT_SURCOUCHE};`);
    shader.fragmentShader = shader.fragmentShader
      .replace('#include <common>', '#include <common>\nvarying vec4 vSurcouche;')
      // Mélange après la conversion sRGB : même rendu qu'en jeu, où la teinte s'applique sur l'image finale
      .replace(
        '#include <colorspace_fragment>',
        '#include <colorspace_fragment>\ngl_FragColor.rgb = mix(gl_FragColor.rgb, vSurcouche.rgb, vSurcouche.a);',
      );
  };
  materiau.customProgramCacheKey = () => 'surcouche';
}
