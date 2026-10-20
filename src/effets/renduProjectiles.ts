/**
 * Rendu instancié de tous les projectiles et objets lancés (flèches, tridents,
 * haches, boules de feu), quel que soit leur propriétaire : un InstancedMesh
 * par type. Chaque image : commencer(), ajouter() par objet, terminer().
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { IDS_PROJECTILES, PROJECTILES, type IdProjectile } from '../donnees/projectiles';
import type { DefinitionProjectile } from '../donnees/types';
import { MODELES } from '../donnees/modeles';
import { creerGeometrieCroisee, creerGeometrieModele, creerGeometrieRelief } from '../moteur/geometrie';

const CAPACITE_PAR_TYPE = 256;

/**
 * Objets en relief couchés à plat : le haut du sprite part vers −X ; les icônes pointent
 * vers le coin haut-droit (ou vers le haut, la fusée) : on les tourne pour pointer vers +Z.
 */
const ORIENTATION_ICONE = Math.PI / 4;
const ORIENTATION_ICONE_HAUT = Math.PI / 2;

interface RenduType {
  readonly maillage: THREE.InstancedMesh;
  compte: number;
}

const matrice = new THREE.Matrix4();

export class RenduProjectiles {
  private readonly types: RenduType[];

  constructor(textures: BibliothequeTextures, scene: THREE.Scene) {
    this.types = IDS_PROJECTILES.map((id) => {
      const def = PROJECTILES[id];
      const map = textures.obtenir(def.texture);
      const maillage = new THREE.InstancedMesh(creerGeometrie(def, map), creerMateriau(def, map), CAPACITE_PAR_TYPE);
      maillage.name = `projectile:${id}`;
      maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
      maillage.frustumCulled = false;
      maillage.count = 0;
      scene.add(maillage);
      return { maillage, compte: 0 };
    });
  }

  /** Index de rendu d'un type de projectile (à calculer une fois, hors boucle). */
  static indexDe(id: IdProjectile): number {
    return IDS_PROJECTILES.indexOf(id);
  }

  commencer(): void {
    for (let t = 0; t < this.types.length; t++) this.types[t].compte = 0;
  }

  /** `angle` = lacet (rad) ; ignoré pour les projectiles tournés vers la caméra. */
  ajouter(indexType: number, x: number, y: number, z: number, angle: number): void {
    const type = this.types[indexType];
    if (type.compte >= CAPACITE_PAR_TYPE) return;
    matrice.makeRotationY(angle).setPosition(x, y, z);
    type.maillage.setMatrixAt(type.compte++, matrice);
  }

  terminer(): void {
    for (let t = 0; t < this.types.length; t++) {
      const type = this.types[t];
      type.maillage.count = type.compte;
      type.maillage.instanceMatrix.needsUpdate = true;
    }
  }
}

function creerGeometrie(def: DefinitionProjectile, map: THREE.Texture | null): THREE.BufferGeometry {
  if (def.forme === 'croisee') {
    return creerGeometrieCroisee(def.longueur, def.largeur, def.uv, def.tailleTexture);
  }
  if (def.forme === 'modele' && def.modele) {
    // Entité couchée dans le sens du vol : le haut du modèle (pointes) vers +Z, centrée
    const geometrie = creerGeometrieModele(MODELES[def.modele]);
    geometrie.computeBoundingBox();
    const boite = geometrie.boundingBox;
    if (boite) geometrie.translate(0, -(boite.min.y + boite.max.y) / 2, 0);
    return geometrie.rotateX(Math.PI / 2);
  }
  const relief = def.forme === 'relief' && map ? creerGeometrieRelief(map, def.longueur) : null;
  if (relief) {
    // Objet couché à plat, pointe vers l'avant (+Z)
    relief.rotateZ(Math.PI / 2);
    relief.rotateY(def.pointeEnHaut ? ORIENTATION_ICONE_HAUT : ORIENTATION_ICONE);
    return relief;
  }
  const geometrie = new THREE.PlaneGeometry(def.longueur, def.longueur);
  appliquerRectangleUv(geometrie, def.uv, def.tailleTexture);
  if (def.forme !== 'face_camera') {
    // Secours sans pixels lisibles : l'icône posée à plat
    geometrie.rotateX(-Math.PI / 2);
    geometrie.rotateY(def.pointeEnHaut ? Math.PI : (-3 * Math.PI) / 4);
  } else {
    // Face à la caméra : celle-ci ne tourne jamais, une inclinaison fixe suffit
    geometrie.rotateX(-(CONFIG.camera.inclinaisonDeg * Math.PI) / 180);
  }
  return geometrie;
}

function creerMateriau(def: DefinitionProjectile, map: THREE.Texture | null): THREE.Material {
  if (!map) return new THREE.MeshLambertMaterial({ color: def.couleurSecours, side: THREE.DoubleSide });
  // Textures découpées : alphaTest, pas de transparence. Les boules de feu brillent (non éclairées).
  return def.forme === 'face_camera'
    ? new THREE.MeshBasicMaterial({ map, alphaTest: 0.5 })
    : new THREE.MeshLambertMaterial({ map, alphaTest: 0.5, side: THREE.DoubleSide });
}

function appliquerRectangleUv(
  geometrie: THREE.BufferGeometry,
  [x, y, l, h]: readonly [number, number, number, number],
  [largeur, hauteur]: readonly [number, number],
): void {
  const uvs = geometrie.getAttribute('uv');
  for (let i = 0; i < uvs.count; i++) {
    uvs.setXY(i, (x + uvs.getX(i) * l) / largeur, 1 - (y + (1 - uvs.getY(i)) * h) / hauteur);
  }
  uvs.needsUpdate = true;
}
