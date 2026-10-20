/** Géométries de blocs, de modèles cubiques et de projectiles, UV mappées sur les textures vanilla. */
import * as THREE from 'three';
import type { CubeModele } from '../donnees/types';

export const PIXELS_PAR_BLOC = 16;

/** Ordre des faces d'une BoxGeometry three.js. */
const FACE = { px: 0, nx: 1, py: 2, ny: 3, pz: 4, nz: 5 } as const;

/** Index des matériaux d'un bloc. */
export const GROUPE_BLOC = { cotes: 0, haut: 1, bas: 2 } as const;

/**
 * Cube unitaire dont les faces sont regroupées en trois groupes de matériaux
 * (côtés, dessus, dessous) : 3 appels de dessin par type de bloc au lieu de 6.
 */
export function creerGeometrieBloc(): THREE.BoxGeometry {
  const geometrie = new THREE.BoxGeometry(1, 1, 1);
  const index = geometrie.getIndex();
  if (!index) throw new Error('BoxGeometry sans index');

  const source = Array.from(index.array);
  const ordre = [FACE.px, FACE.nx, FACE.pz, FACE.nz, FACE.py, FACE.ny];
  const indices: number[] = [];
  for (const face of ordre) indices.push(...source.slice(face * 6, face * 6 + 6));

  geometrie.setIndex(indices);
  geometrie.clearGroups();
  geometrie.addGroup(0, 24, GROUPE_BLOC.cotes);
  geometrie.addGroup(24, 6, GROUPE_BLOC.haut);
  geometrie.addGroup(30, 6, GROUPE_BLOC.bas);
  return geometrie;
}

/**
 * Boîte d'un modèle cubique, en blocs (1 px = 1/16), dont chaque face pointe
 * sur son rectangle du patron vanilla (voir donnees/modeles.ts). La boîte est
 * décalée de `cube.centre` : son origine est l'articulation de la partie.
 * Le modèle regarde vers +Z : +Z = avant, −X = côté droit, +X = côté gauche.
 */
export function creerGeometrieCube(
  cube: CubeModele,
  [largeurTexture, hauteurTexture]: readonly [number, number],
): THREE.BoxGeometry {
  const [l, h, p] = cube.taille;
  const [u, v] = cube.uv;
  const geometrie = new THREE.BoxGeometry(l / PIXELS_PAR_BLOC, h / PIXELS_PAR_BLOC, p / PIXELS_PAR_BLOC);

  // [x, y, largeur, hauteur] en pixels, dans l'ordre des faces de BoxGeometry
  const gauche = [u + p + l, v + p, p, h] as const;
  const droite = [u, v + p, p, h] as const;
  const rectangles: readonly (readonly [number, number, number, number])[] = [
    cube.miroir ? droite : gauche, // +X : côté gauche (en miroir, les côtés s'échangent)
    cube.miroir ? gauche : droite, // −X : côté droit
    [u + p, v, l, p],              // +Y : dessus
    [u + p + l, v, l, p],          // −Y : dessous
    [u + p, v + p, l, h],          // +Z : avant
    [u + 2 * p + l, v + p, l, h],  // −Z : arrière
  ];

  const uvs = geometrie.getAttribute('uv');
  rectangles.forEach(([x, y, largeur, hauteur], face) => {
    // Le dessous est retourné verticalement par rapport aux autres faces, comme en jeu
    const retourne = face === FACE.ny;
    for (let sommet = 0; sommet < 4; sommet++) {
      const i = face * 4 + sommet;
      const su = cube.miroir ? 1 - uvs.getX(i) : uvs.getX(i);
      const colonne = x + su * largeur;
      const ligne = y + (retourne ? uvs.getY(i) : 1 - uvs.getY(i)) * hauteur;  // 0 = haut de l'image
      uvs.setXY(i, colonne / largeurTexture, 1 - ligne / hauteurTexture);
    }
  });
  uvs.needsUpdate = true;

  const [cx, cy, cz] = cube.centre;
  geometrie.translate(cx / PIXELS_PAR_BLOC, cy / PIXELS_PAR_BLOC, cz / PIXELS_PAR_BLOC);
  return geometrie;
}

/**
 * Deux plans croisés le long de +Z (un vertical, un horizontal), comme les
 * flèches en jeu. `uv` = rectangle de la vue de profil dans la texture (pixels).
 */
export function creerGeometrieCroisee(
  longueur: number,
  largeur: number,
  [x, y, l, h]: readonly [number, number, number, number],
  [largeurTexture, hauteurTexture]: readonly [number, number],
): THREE.BufferGeometry {
  const u0 = x / largeurTexture;
  const u1 = (x + l) / largeurTexture;
  const v0 = 1 - (y + h) / hauteurTexture;
  const v1 = 1 - y / hauteurTexture;
  const z0 = -longueur / 2;
  const z1 = longueur / 2;
  const e = largeur / 2;

  // Plan vertical (x = 0) puis plan horizontal (y = 0) ; u va de l'arrière (−Z) à la pointe (+Z)
  const positions = [
    0, -e, z0,   0, -e, z1,   0, e, z1,   0, e, z0,
    -e, 0, z0,   -e, 0, z1,   e, 0, z1,   e, 0, z0,
  ];
  const uvs = [
    u0, v0,   u1, v0,   u1, v1,   u0, v1,
    u0, v0,   u1, v0,   u1, v1,   u0, v1,
  ];
  const geometrie = new THREE.BufferGeometry();
  geometrie.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  geometrie.setAttribute('uv', new THREE.Float32BufferAttribute(uvs, 2));
  geometrie.setIndex([0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7]);
  geometrie.computeVertexNormals();
  return geometrie;
}
