/**
 * Ombre ronde au sol, comme sous les entités Minecraft (texture vanilla misc/shadow).
 * C'est un disque à opacité partielle : il est translucide, pas découpé par alphaTest.
 */
import * as THREE from 'three';
import { CONFIG } from '../config';

export function creerOmbre(texture: THREE.Texture | null, rayon: number): THREE.Mesh {
  const geometrie = texture
    ? new THREE.PlaneGeometry(rayon * 2, rayon * 2)
    : new THREE.CircleGeometry(rayon, 16);
  const materiau = new THREE.MeshBasicMaterial({
    map: texture,
    color: texture ? 0xffffff : 0x000000,
    transparent: true,
    opacity: CONFIG.ombre.opacite,
    depthWrite: false,
  });

  const ombre = new THREE.Mesh(geometrie, materiau);
  ombre.name = 'ombre';
  ombre.rotation.x = -Math.PI / 2;
  ombre.position.y = CONFIG.ombre.hauteur;
  return ombre;
}
