/**
 * Vent et passage du joueur, communs à tous les styles d'herbe (TSL, calculé
 * par la carte graphique). Chaque brin ou touffe porte l'attribut d'instance
 * `aRacine` (x, z, orientation, taille) ; ce module en tire son pliage.
 */
import type * as THREE from 'three';
import type { Node, UniformNode } from 'three/webgpu';
import { attribute, clamp, dot, float, length, normalize, sin, time, vec2 } from 'three/tsl';
import { CONFIG } from '../../config';

/**
 * Pliage horizontal (blocs) au sommet d'une plante : rafales qui traversent
 * le champ, frémissement propre à chaque instance, et plantes couchées autour
 * du joueur. `rafale` (0 à 1) sert aussi à éclaircir les brins couchés.
 */
export function pliageVent(joueur: UniformNode<'vec2', THREE.Vector2>, phase: Node<'float'>) {
  const v = CONFIG.herbe.vent;
  const base = attribute<'vec4'>('aRacine', 'vec4').xy;

  const direction = vec2(v.direction[0], v.direction[1]);
  const avancee = dot(base, direction).mul(0.18).sub(time.mul(v.vitesseRafales));
  const rafale = sin(avancee).mul(0.5).add(0.5).mul(sin(avancee.mul(0.37).add(1.7)).mul(0.5).add(0.5));
  const fremissement = sin(time.mul(5.1).add(phase).add(base.x.mul(2.3))).mul(0.12);
  const vent = direction.mul(rafale.mul(v.amplitude).add(v.amplitude * 0.25).add(fremissement.mul(v.amplitude)));

  const ecart = base.sub(joueur);
  const proximite = clamp(float(1).sub(length(ecart).div(v.rayonCouche)), 0, 1);
  const couche = normalize(ecart.add(vec2(0.0001, 0))).mul(proximite.mul(proximite).mul(v.couche));

  return { pliage: vent.add(couche), rafale, base };
}
