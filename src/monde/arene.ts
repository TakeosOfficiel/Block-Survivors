/**
 * Construction d'une arène en blocs : sol, murs, terrain extérieur, ciel et brouillard.
 * Un InstancedMesh par type de bloc, une géométrie partagée.
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { BLOCS, type IdBloc } from '../donnees/blocs';
import type { ChoixBloc, DefinitionArene, DefinitionBloc } from '../donnees/types';
import { creerGeometrieBloc } from '../moteur/geometrie';
import { choisirPondere, hachage3 } from '../utils/maths';

/** Construit l'arène dans la scène et renvoie sa racine. */
export function construireArene(
  definition: DefinitionArene,
  textures: BibliothequeTextures,
  scene: THREE.Scene,
): THREE.Group {
  const groupe = new THREE.Group();
  groupe.name = `arene:${definition.id}`;

  const geometrie = creerGeometrieBloc();
  for (const [idBloc, positions] of listerBlocs(definition)) {
    groupe.add(creerInstancesBloc(idBloc, positions, geometrie, textures));
  }
  groupe.add(creerTerrainExterieur(definition, textures));
  appliquerAmbiance(definition, scene);

  scene.add(groupe);
  return groupe;
}

/** Centres des blocs de l'arène, regroupés par type : idBloc → [x,y,z, x,y,z, …]. */
function listerBlocs(definition: DefinitionArene): Map<IdBloc, number[]> {
  const parBloc = new Map<IdBloc, number[]>();
  const poser = (choix: readonly ChoixBloc[], x: number, y: number, z: number) => {
    const { bloc } = choisirPondere(choix, hachage3(Math.floor(x), Math.floor(y), Math.floor(z)));
    let positions = parBloc.get(bloc);
    if (!positions) parBloc.set(bloc, (positions = []));
    positions.push(x, y, z);
  };

  const demi = definition.taille / 2;

  // Sol : une couche de blocs dont la face supérieure est à y = 0
  for (let i = 0; i < definition.taille; i++) {
    for (let k = 0; k < definition.taille; k++) {
      poser(definition.sol, -demi + i + 0.5, -0.5, -demi + k + 0.5);
    }
  }

  // Murs : anneau de blocs juste autour de la zone jouable (coins compris)
  for (let h = 0; h < definition.hauteurMur; h++) {
    const y = h + 0.5;
    for (let i = -1; i <= definition.taille; i++) {
      const c = -demi + i + 0.5;
      poser(definition.murs, c, y, -demi - 0.5);    // nord
      poser(definition.murs, c, y, demi + 0.5);     // sud
      if (i >= 0 && i < definition.taille) {
        poser(definition.murs, -demi - 0.5, y, c);  // ouest
        poser(definition.murs, demi + 0.5, y, c);   // est
      }
    }
  }

  return parBloc;
}

/** Matériaux [côtés, dessus, dessous] d'un bloc ; couleur de secours pour chaque face sans texture. */
function creerMateriauxBloc(bloc: DefinitionBloc, textures: BibliothequeTextures): THREE.MeshLambertMaterial[] {
  const materiau = (chemin: string, teinte?: number) => {
    const map = textures.obtenir(chemin);
    return map
      ? new THREE.MeshLambertMaterial({ map, color: teinte ?? 0xffffff })
      : new THREE.MeshLambertMaterial({ color: bloc.couleurSecours });
  };
  return [
    materiau(bloc.faces.cotes),
    materiau(bloc.faces.haut, bloc.teinteHaut),
    materiau(bloc.faces.bas),
  ];
}

function creerInstancesBloc(
  idBloc: IdBloc,
  positions: readonly number[],
  geometrie: THREE.BufferGeometry,
  textures: BibliothequeTextures,
): THREE.InstancedMesh {
  const bloc = BLOCS[idBloc];
  const nombre = positions.length / 3;
  const materiaux = creerMateriauxBloc(bloc, textures);
  const mesh = new THREE.InstancedMesh(geometrie, materiaux, nombre);
  mesh.name = idBloc;

  // Sans aucune texture, un damier discret garde les blocs lisibles
  const sansTexture = materiaux.every((m) => m.map === null);
  const couleurBase = new THREE.Color(bloc.couleurSecours);
  const couleur = new THREE.Color();
  const matrice = new THREE.Matrix4();

  for (let i = 0; i < nombre; i++) {
    const x = positions[i * 3];
    const y = positions[i * 3 + 1];
    const z = positions[i * 3 + 2];
    const bx = Math.floor(x);
    const by = Math.floor(y);
    const bz = Math.floor(z);

    // Rotation aléatoire par quart de tour, comme le blockstate vanilla (casse la répétition)
    const quartsDeTour = bloc.rotationAleatoire ? Math.floor(hachage3(bz, bx, by) * 4) : 0;
    matrice.makeRotationY((quartsDeTour * Math.PI) / 2).setPosition(x, y, z);
    mesh.setMatrixAt(i, matrice);

    if (sansTexture) {
      const pair = (bx + by + bz) & 1;
      mesh.setColorAt(i, couleur.copy(couleurBase).multiplyScalar(pair ? 1 : CONFIG.rendu.contrasteDamier));
    }
  }
  mesh.instanceMatrix.needsUpdate = true;
  if (mesh.instanceColor) mesh.instanceColor.needsUpdate = true;
  return mesh;
}

/** Terrain décoratif hors arène : dessus du bloc répété sur un grand plan, assombri. */
function creerTerrainExterieur(definition: DefinitionArene, textures: BibliothequeTextures): THREE.Mesh {
  const { bloc: idBloc, assombrissement, taille } = definition.exterieur;
  const bloc = BLOCS[idBloc];
  const map = textures.obtenirRepetee(bloc.faces.haut, taille);
  const couleur = new THREE.Color(map ? (bloc.teinteHaut ?? 0xffffff) : bloc.couleurSecours)
    .multiplyScalar(assombrissement);

  const plan = new THREE.Mesh(
    new THREE.PlaneGeometry(taille, taille),
    new THREE.MeshLambertMaterial({ map, color: couleur }),
  );
  plan.name = 'exterieur';
  plan.rotation.x = -Math.PI / 2;
  plan.position.y = -0.02;  // juste sous le dessus du sol de l'arène
  return plan;
}

function appliquerAmbiance(definition: DefinitionArene, scene: THREE.Scene): void {
  scene.background = new THREE.Color(definition.ciel);
  scene.fog = new THREE.Fog(definition.ciel, definition.brouillard.debut, definition.brouillard.fin);
}
