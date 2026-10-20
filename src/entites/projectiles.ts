/**
 * Projectiles des ennemis (flèches) : pool, vol en ligne droite, collision avec
 * le joueur. Un InstancedMesh par type de projectile.
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { JOUEUR_BASE } from '../donnees/joueur';
import { IDS_PROJECTILES, PROJECTILES, type IdProjectile } from '../donnees/projectiles';
import type { DefinitionProjectile } from '../donnees/types';
import type { EtatPartie } from '../etat';
import { creerGeometrieCroisee } from '../moteur/geometrie';
import { Pool } from '../utils/pool';
import type { CibleJoueur, SourceEntites, Tireur } from './types';

interface Projectile {
  indexType: number;
  definition: DefinitionProjectile;
  x: number;
  z: number;
  vx: number;
  vz: number;
  vie: number;
  degats: number;
}

/** Rendu d'un type de projectile. */
interface RenduType {
  readonly maillage: THREE.InstancedMesh;
  compte: number;
}

const matrice = new THREE.Matrix4();

export class Projectiles implements SourceEntites, Tireur {
  private readonly etat: EtatPartie;
  private readonly cible: CibleJoueur;
  private readonly pool = new Pool<Projectile>(CONFIG.projectiles.capacite, () => ({
    indexType: 0, definition: PROJECTILES[IDS_PROJECTILES[0]], x: 0, z: 0, vx: 0, vz: 0, vie: 0, degats: 0,
  }));
  /** Un rendu par type, dans l'ordre de IDS_PROJECTILES. */
  private readonly rendus: RenduType[];

  constructor(etat: EtatPartie, cible: CibleJoueur, scene: THREE.Scene, textures: BibliothequeTextures) {
    this.etat = etat;
    this.cible = cible;
    this.rendus = IDS_PROJECTILES.map((id) => {
      const def = PROJECTILES[id];
      const map = textures.obtenir(def.texture);
      const maillage = new THREE.InstancedMesh(
        creerGeometrieCroisee(def.longueur, map ? def.largeur : def.largeur * 0.6, def.uv, def.tailleTexture),
        // Texture découpée : alphaTest, pas de transparence
        map
          ? new THREE.MeshLambertMaterial({ map, alphaTest: 0.5, side: THREE.DoubleSide })
          : new THREE.MeshLambertMaterial({ color: def.couleurSecours, side: THREE.DoubleSide }),
        CONFIG.projectiles.capacite,
      );
      maillage.name = `projectile:${id}`;
      maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
      maillage.frustumCulled = false;
      maillage.count = 0;
      scene.add(maillage);
      return { maillage, compte: 0 };
    });
  }

  get actifs(): number {
    return this.pool.nombre;
  }

  tirer(type: IdProjectile, x: number, z: number, dirX: number, dirZ: number, degats: number): void {
    const p = this.pool.acquerir();
    if (!p) return;
    const def = PROJECTILES[type];
    p.indexType = IDS_PROJECTILES.indexOf(type);
    p.definition = def;
    p.x = x;
    p.z = z;
    p.vx = dirX * def.vitesse;
    p.vz = dirZ * def.vitesse;
    p.vie = def.duree;
    p.degats = degats;
  }

  maj(dt: number): void {
    const j = this.etat.joueur;
    const limite = this.etat.arene.taille / 2;
    for (let i = this.pool.nombre - 1; i >= 0; i--) {
      const p = this.pool.elements[i];
      p.vie -= dt;
      p.x += p.vx * dt;
      p.z += p.vz * dt;

      const dx = p.x - j.x;
      const dz = p.z - j.z;
      const portee = p.definition.rayon + JOUEUR_BASE.rayon;
      const touche = !j.mort && dx * dx + dz * dz < portee * portee;
      if (touche) this.cible.subirDegats(p.degats);

      const sortie = p.x < -limite || p.x > limite || p.z < -limite || p.z > limite;
      if (touche || sortie || p.vie <= 0) this.pool.liberer(i);
    }
  }

  majVisuel(): void {
    for (let t = 0; t < this.rendus.length; t++) this.rendus[t].compte = 0;

    for (let i = 0; i < this.pool.nombre; i++) {
      const p = this.pool.elements[i];
      const rendu = this.rendus[p.indexType];
      matrice.makeRotationY(Math.atan2(p.vx, p.vz)).setPosition(p.x, p.definition.hauteur, p.z);
      rendu.maillage.setMatrixAt(rendu.compte++, matrice);
    }

    for (let t = 0; t < this.rendus.length; t++) {
      const rendu = this.rendus[t];
      rendu.maillage.count = rendu.compte;
      rendu.maillage.instanceMatrix.needsUpdate = true;
    }
  }
}
