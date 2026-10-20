/**
 * Affichage d'une arène générée : un InstancedMesh par type de bloc (cubes,
 * croix, liquides), le terrain décoratif hors arène, et les textures animées.
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { BLOCS, type IdBloc } from '../donnees/blocs';
import type { DefinitionArene, DefinitionBloc } from '../donnees/types';
import { creerGeometrieBloc } from '../moteur/geometrie';
import { hachage3 } from '../utils/maths';
import type { MondeGenere } from './generation';

/** Un liquide est un cube abaissé de 2 pixels, comme en jeu. */
const HAUTEUR_LIQUIDE = 14 / 16;

/** Construit l'arène dans la scène et renvoie sa racine. */
export function construireArene(
  definition: DefinitionArene,
  monde: MondeGenere,
  textures: BibliothequeTextures,
  scene: THREE.Scene,
): THREE.Group {
  const groupe = new THREE.Group();
  groupe.name = `arene:${definition.id}`;

  const geometries = { cube: creerGeometrieBloc(), croix: creerGeometrieCroix() };
  for (const [idBloc, positions] of monde.blocs) {
    const bloc = BLOCS[idBloc];
    groupe.add(creerInstancesBloc(idBloc, bloc, positions, geometries, textures));
    if (bloc.animation) {
      for (const chemin of new Set(Object.values(bloc.faces))) {
        textures.animer(chemin, bloc.animation.imagesParSeconde, bloc.animation.allerRetour);
      }
    }
  }
  groupe.add(creerTerrainExterieur(definition, textures));

  scene.add(groupe);
  return groupe;
}

/** Matériaux [côtés, dessus, dessous] d'un bloc ; couleur de secours pour chaque face sans texture. */
function creerMateriauxBloc(bloc: DefinitionBloc, textures: BibliothequeTextures): THREE.Material[] {
  const croix = bloc.forme === 'croix';
  const materiau = (chemin: string, teinte?: number): THREE.Material => {
    const map = textures.obtenir(chemin);
    const parametres = {
      map,
      color: map ? (teinte ?? bloc.teinte ?? 0xffffff) : bloc.couleurSecours,
      // Feuilles, toiles, fleurs : découpe nette (alphaTest), jamais de transparence
      alphaTest: map && bloc.decoupe ? 0.5 : 0,
      side: croix || bloc.decoupe ? THREE.DoubleSide : THREE.FrontSide,
    };
    // Blocs lumineux : pleine lumière, même dans le noir (et rayonnent avec les shaders)
    if (!bloc.lumineux) return new THREE.MeshLambertMaterial(parametres);
    const materiau = new THREE.MeshBasicMaterial(parametres);
    materiau.userData['lumineux'] = true;
    return materiau;
  };
  if (croix) return [materiau(bloc.faces.cotes)];
  return [
    materiau(bloc.faces.cotes),
    materiau(bloc.faces.haut, bloc.teinteHaut),
    materiau(bloc.faces.bas),
  ];
}

function creerInstancesBloc(
  idBloc: IdBloc,
  bloc: DefinitionBloc,
  positions: readonly number[],
  geometries: { readonly cube: THREE.BufferGeometry; readonly croix: THREE.BufferGeometry },
  textures: BibliothequeTextures,
): THREE.InstancedMesh {
  const nombre = positions.length / 3;
  const materiaux = creerMateriauxBloc(bloc, textures);
  const forme = bloc.forme ?? 'cube';
  const geometrie = forme === 'croix' ? geometries.croix : geometries.cube;
  const mesh = new THREE.InstancedMesh(geometrie, materiaux.length === 1 ? materiaux[0] : materiaux, nombre);
  mesh.name = idBloc;

  // Sans aucune texture, un damier discret garde les blocs lisibles
  const sansTexture = materiaux.every((m) => (m as THREE.MeshBasicMaterial).map === null);
  const couleurBase = new THREE.Color(bloc.couleurSecours);
  const couleur = new THREE.Color();
  const matrice = new THREE.Matrix4();
  const echelle = new THREE.Matrix4();

  for (let i = 0; i < nombre; i++) {
    const x = positions[i * 3];
    const y = positions[i * 3 + 1];
    const z = positions[i * 3 + 2];
    const bx = Math.floor(x);
    const by = Math.floor(y);
    const bz = Math.floor(z);

    // Rotation aléatoire par quart de tour, comme le blockstate vanilla (casse la répétition)
    const quartsDeTour = bloc.rotationAleatoire ? Math.floor(hachage3(bz, bx, by) * 4) : 0;
    matrice.makeRotationY((quartsDeTour * Math.PI) / 2);
    if (forme === 'liquide') {
      matrice.multiply(echelle.makeScale(1, HAUTEUR_LIQUIDE, 1));
      matrice.setPosition(x, y - (1 - HAUTEUR_LIQUIDE) / 2, z);
    } else {
      matrice.setPosition(x, y, z);
    }
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

/** Deux plans verticaux croisés en diagonale du bloc (fleurs, herbes, toiles). */
function creerGeometrieCroix(): THREE.BufferGeometry {
  const d = 0.45;  // demi-diagonale, légèrement rentrée comme en jeu
  const positions = [
    -d, -0.5, -d,   d, -0.5, d,   d, 0.5, d,   -d, 0.5, -d,
    -d, -0.5, d,    d, -0.5, -d,  d, 0.5, -d,  -d, 0.5, d,
  ];
  const uvs = [0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1];
  const geometrie = new THREE.BufferGeometry();
  geometrie.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  geometrie.setAttribute('uv', new THREE.Float32BufferAttribute(uvs, 2));
  geometrie.setIndex([0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7]);
  geometrie.computeVertexNormals();
  return geometrie;
}

/**
 * Terrain décoratif hors arène : dessus du bloc répété, assombri, sur un grand
 * plan troué à la taille de l'arène (il ne recouvre ni le sol ni les liquides).
 * Les UV d'une ShapeGeometry valent ses coordonnées : une texture par bloc.
 */
function creerTerrainExterieur(definition: DefinitionArene, textures: BibliothequeTextures): THREE.Mesh {
  const { bloc: idBloc, assombrissement, taille } = definition.exterieur;
  const bloc = BLOCS[idBloc];
  const map = textures.obtenirRepetee(bloc.faces.haut, 1);
  const couleur = new THREE.Color(map ? (bloc.teinteHaut ?? 0xffffff) : bloc.couleurSecours)
    .multiplyScalar(assombrissement);

  const exterieur = carre(taille / 2);
  exterieur.holes.push(carre(definition.taille / 2));
  const plan = new THREE.Mesh(
    new THREE.ShapeGeometry(exterieur),
    new THREE.MeshLambertMaterial({ map, color: couleur }),
  );
  plan.name = 'exterieur';
  plan.rotation.x = -Math.PI / 2;
  return plan;
}

/** Carré centré de demi-côté `d`. */
function carre(d: number): THREE.Shape {
  return new THREE.Shape([
    new THREE.Vector2(-d, -d), new THREE.Vector2(d, -d), new THREE.Vector2(d, d), new THREE.Vector2(-d, d),
  ]);
}
