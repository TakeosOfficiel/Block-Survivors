/** Personnage ou créature construit en boîtes depuis une DefinitionModele et un skin. */
import * as THREE from 'three';
import type { DefinitionModele } from '../donnees/types';
import { creerGeometrieCube, PIXELS_PAR_BLOC } from '../moteur/geometrie';

export class ModeleCubique {
  /** Racine du modèle : à positionner et orienter. */
  readonly racine = new THREE.Group();
  /** Articulation de chaque partie, par nom : c'est elles qu'on anime. */
  readonly articulations = new Map<string, THREE.Object3D>();

  constructor(
    definition: DefinitionModele,
    skin: THREE.Texture | null,
    couleursSecours: Readonly<Record<string, number>>,
  ) {
    const materiauSkin = skin ? new THREE.MeshLambertMaterial({ map: skin }) : null;

    for (const [nom, cube] of Object.entries(definition.parties)) {
      const articulation = new THREE.Group();
      articulation.name = nom;
      articulation.position.set(...versBlocs(cube.pivot));

      const materiau = materiauSkin
        ?? new THREE.MeshLambertMaterial({ color: couleursSecours[nom] ?? 0xffffff });
      const boite = new THREE.Mesh(creerGeometrieCube(cube, definition.tailleTexture), materiau);
      boite.position.set(...versBlocs(cube.centre));

      articulation.add(boite);
      this.racine.add(articulation);
      this.articulations.set(nom, articulation);
    }
  }
}

function versBlocs([x, y, z]: readonly [number, number, number]): [number, number, number] {
  return [x / PIXELS_PAR_BLOC, y / PIXELS_PAR_BLOC, z / PIXELS_PAR_BLOC];
}
