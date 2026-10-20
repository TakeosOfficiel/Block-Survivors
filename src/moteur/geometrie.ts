/** Géométries de blocs et de modèles cubiques, UV mappées sur les textures vanilla. */
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
 * sur son rectangle du patron vanilla (voir donnees/modeles.ts).
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
  const rectangles: readonly (readonly [number, number, number, number])[] = [
    [u + p + l,     v + p, p, h],  // +X : côté gauche
    [u,             v + p, p, h],  // −X : côté droit
    [u + p,         v,     l, p],  // +Y : dessus
    [u + p + l,     v,     l, p],  // −Y : dessous
    [u + p,         v + p, l, h],  // +Z : avant
    [u + 2 * p + l, v + p, l, h],  // −Z : arrière
  ];

  const uvs = geometrie.getAttribute('uv');
  rectangles.forEach(([x, y, largeur, hauteur], face) => {
    // Le dessous est retourné verticalement par rapport aux autres faces, comme en jeu
    const retourne = face === FACE.ny;
    for (let sommet = 0; sommet < 4; sommet++) {
      const i = face * 4 + sommet;
      const colonne = x + uvs.getX(i) * largeur;
      const ligne = y + (retourne ? uvs.getY(i) : 1 - uvs.getY(i)) * hauteur;  // 0 = haut de l'image
      uvs.setXY(i, colonne / largeurTexture, 1 - ligne / hauteurTexture);
    }
  });
  uvs.needsUpdate = true;
  return geometrie;
}
