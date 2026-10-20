/**
 * Lave et eau « réelles » quand les shaders sont actifs : matériaux calculés
 * par la carte graphique (TSL) à la place des textures animées du jeu.
 *  - Lave : croûte sombre qui dérive, fissures en fusion qui palpitent ; le
 *    cœur dépasse le blanc et rayonne avec la lueur.
 *  - Eau : vagues, reflet du ciel selon l'angle de vue (Fresnel), éclats du
 *    soleil, légère transparence.
 * Sans shaders, les blocs gardent leurs textures Minecraft d'origine.
 */
import * as THREE from 'three';
import { MeshBasicNodeMaterial } from 'three/webgpu';
import {
  cameraPosition, color, dot, float, max, mix, mx_fractal_noise_float, mx_worley_noise_vec2, normalize, positionWorld,
  pow, reflect, sin, smoothstep, time, uniform, vec2, vec3,
} from 'three/tsl';
import { CONFIG } from '../config';

/** Maillage d'un type de liquide et ses matériaux d'origine (textures). */
interface Liquide {
  readonly maillage: THREE.Mesh;
  readonly origine: THREE.Material | THREE.Material[];
  readonly shader: THREE.Material;
}

export class Liquides {
  private readonly liquides: Liquide[] = [];
  private readonly ciel = uniform(new THREE.Color());

  constructor(scene: THREE.Scene, couleurCiel: number) {
    this.ciel.value.set(couleurCiel);
    const lave = scene.getObjectByName('lava') as THREE.Mesh | undefined;
    if (lave) this.liquides.push({ maillage: lave, origine: lave.material, shader: creerLave() });
    const eau = scene.getObjectByName('water') as THREE.Mesh | undefined;
    if (eau) this.liquides.push({ maillage: eau, origine: eau.material, shader: this.creerEau() });
  }

  definirShaders(actifs: boolean): void {
    for (const l of this.liquides) l.maillage.material = actifs ? l.shader : l.origine;
  }

  private creerEau(): MeshBasicNodeMaterial {
    const e = CONFIG.liquides.eau;
    const p = positionWorld.xz;
    // Hauteur des vagues : deux couches de bruit qui dérivent en sens contraires
    const vagues = (q: typeof p) => mx_fractal_noise_float(vec3(q.mul(0.9).add(vec2(time.mul(0.35), time.mul(0.2))), time.mul(0.25)), 3, 2, 0.5)
      .add(mx_fractal_noise_float(vec3(q.mul(2.3).sub(vec2(time.mul(0.5), 0)), time.mul(0.4)), 2, 2, 0.5).mul(0.35));
    const h = vagues(p);
    const d = 0.08;
    const pente = vec2(vagues(p.add(vec2(d, 0))).sub(h), vagues(p.add(vec2(0, d))).sub(h)).div(d);
    const normale = normalize(vec3(pente.x.mul(e.relief).negate(), 1, pente.y.mul(e.relief).negate()));

    const vue = normalize(cameraPosition.sub(positionWorld));
    const fresnel = float(0.06).add(pow(float(1).sub(max(dot(normale, vue), 0)), 5).mul(0.94));
    const profondeur = mix(color(e.couleurProfonde), color(e.couleurClaire), h.mul(0.5).add(0.5));
    const reflet = mix(profondeur, this.ciel, fresnel.add(e.refletCiel).min(1));
    const soleil = normalize(vec3(...CONFIG.rendu.directionSoleil));
    const eclat = pow(max(dot(reflect(vue.negate(), normale), soleil), 0), e.brillance).mul(e.eclatSoleil);

    const materiau = new MeshBasicNodeMaterial({ transparent: true, opacity: e.opacite });
    materiau.colorNode = reflet.add(eclat);
    return materiau;
  }
}

function creerLave(): MeshBasicNodeMaterial {
  const l = CONFIG.liquides.lave;
  const p = positionWorld.xz;
  const derive = vec2(time.mul(0.1), time.mul(0.04));
  // Remous lents : décident où la croûte flotte (zones froides) et où la lave est à nu
  const remous = mx_fractal_noise_float(vec3(p.mul(0.3).add(derive), time.mul(0.08)), 4, 2, 0.5);
  // Coulée en fusion : orange vif, veines jaunes qui ondulent
  const veines = mx_fractal_noise_float(vec3(p.mul(1.4).sub(derive.mul(2)), time.mul(0.35)), 3, 2, 0.5);
  const pulsation = sin(time.mul(1.6).add(remous.mul(6))).mul(0.12).add(1);
  const fusion = mix(color(l.couleurFusion), color(l.couleurCoeur).mul(l.intensite), smoothstep(0, 0.45, veines)).mul(pulsation);

  // Îlots de croûte sombre, découpés en plaques par des fissures incandescentes
  const distances = mx_worley_noise_vec2(vec3(p.mul(l.taillePlaques).sub(derive.mul(0.8)), time.mul(0.12)));
  const fissure = float(1).sub(smoothstep(0, l.largeurFissures, distances.y.sub(distances.x)));
  const ilots = smoothstep(l.couvertureCroute + 0.12, l.couvertureCroute - 0.12, remous);
  const croute = mix(color(l.couleurCroute), color(l.couleurFusion).mul(0.35), smoothstep(-0.6, 0.1, remous));
  const croutePercee = mix(croute, fusion, fissure);

  const materiau = new MeshBasicNodeMaterial();
  materiau.colorNode = mix(fusion, croutePercee, ilots);
  return materiau;
}
