/** Géométries de blocs, de modèles cubiques et de projectiles, UV mappées sur les textures vanilla. */
import * as THREE from 'three';
import { mergeGeometries } from 'three/examples/jsm/utils/BufferGeometryUtils.js';
import { lirePixels } from '../assets/pixels';
import type { CubeModele, DefinitionModele } from '../donnees/types';

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
  const g = 2 * (cube.gonflement ?? 0);
  const geometrie = new THREE.BoxGeometry((l + g) / PIXELS_PAR_BLOC, (h + g) / PIXELS_PAR_BLOC, (p + g) / PIXELS_PAR_BLOC);

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

/**
 * Objet tenu en main, extrudé comme en jeu : le sprite 2D (pixels RGBA,
 * `largeur` x `hauteur`) devient une plaque d'un pixel d'épaisseur — une face
 * avant, une face arrière, et un petit rebord pour chaque pixel opaque au bord
 * d'un pixel transparent, coloré par ce pixel.
 *
 * Repère : plaque dans le plan x = 0, centrée sur l'origine ; la droite du
 * sprite va vers +Z, son haut vers +Y. `pixel` = taille d'un pixel en blocs.
 */
export function creerGeometrieObjet(
  pixels: Uint8ClampedArray,
  largeur: number,
  hauteur: number,
  pixel: number,
): THREE.BufferGeometry {
  const opaque = (x: number, y: number) =>
    x >= 0 && y >= 0 && x < largeur && y < hauteur && pixels[(y * largeur + x) * 4 + 3] >= 128;
  const positions: number[] = [];
  const normales: number[] = [];
  const uvs: number[] = [];
  const indices: number[] = [];
  const e = pixel / 2;
  const z = (colonne: number) => (colonne - largeur / 2) * pixel;
  const y = (ligne: number) => (hauteur / 2 - ligne) * pixel;

  /** Quadrilatère de 4 sommets [x, y, z] avec une normale et 4 UV. */
  const quad = (sommets: readonly number[], normale: readonly number[], coords: readonly number[]) => {
    const base = positions.length / 3;
    positions.push(...sommets);
    for (let k = 0; k < 4; k++) normales.push(...normale);
    uvs.push(...coords);
    indices.push(base, base + 1, base + 2, base, base + 2, base + 3);
  };

  // Faces avant (+X) et arrière (−X) : tout le sprite, la découpe se fait par alphaTest
  const [z0, z1, y0, y1] = [z(0), z(largeur), y(hauteur), y(0)];
  quad([e, y0, z1, e, y0, z0, e, y1, z0, e, y1, z1], [1, 0, 0], [1, 0, 0, 0, 0, 1, 1, 1]);
  quad([-e, y0, z0, -e, y0, z1, -e, y1, z1, -e, y1, z0], [-1, 0, 0], [0, 0, 1, 0, 1, 1, 0, 1]);

  // Rebords : UV ramenées au centre du pixel, qui colore toute la tranche
  for (let ligne = 0; ligne < hauteur; ligne++) {
    for (let colonne = 0; colonne < largeur; colonne++) {
      if (!opaque(colonne, ligne)) continue;
      const u = (colonne + 0.5) / largeur;
      const v = 1 - (ligne + 0.5) / hauteur;
      const c = [u, v, u, v, u, v, u, v];
      const [za, zb, ya, yb] = [z(colonne), z(colonne + 1), y(ligne + 1), y(ligne)];
      if (!opaque(colonne, ligne - 1)) quad([-e, yb, za, e, yb, za, e, yb, zb, -e, yb, zb], [0, 1, 0], c);
      if (!opaque(colonne, ligne + 1)) quad([-e, ya, zb, e, ya, zb, e, ya, za, -e, ya, za], [0, -1, 0], c);
      if (!opaque(colonne - 1, ligne)) quad([-e, ya, za, e, ya, za, e, yb, za, -e, yb, za], [0, 0, -1], c);
      if (!opaque(colonne + 1, ligne)) quad([e, ya, zb, -e, ya, zb, -e, yb, zb, e, yb, zb], [0, 0, 1], c);
    }
  }

  const geometrie = new THREE.BufferGeometry();
  geometrie.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  geometrie.setAttribute('normal', new THREE.Float32BufferAttribute(normales, 3));
  geometrie.setAttribute('uv', new THREE.Float32BufferAttribute(uvs, 2));
  geometrie.setIndex(indices);
  return geometrie;
}

/**
 * Objet en relief tiré d'une texture d'objet (fusée, hache, épée, pomme) :
 * la plaque d'un pixel d'épaisseur de `creerGeometrieObjet`, de côté `taille`
 * blocs. Null si les pixels de l'image ne peuvent pas être lus.
 */
export function creerGeometrieRelief(texture: THREE.Texture, taille: number): THREE.BufferGeometry | null {
  const image = texture.image as (CanvasImageSource & { width: number; height: number }) | undefined;
  if (!image) return null;
  try {
    const pixels = lirePixels(image);
    return creerGeometrieObjet(pixels.data, image.width, image.height, taille / image.width);
  } catch {
    return null;
  }
}

/** Modèle cubique fusionné en une seule géométrie (parties immobiles : trident lancé). */
export function creerGeometrieModele(definition: DefinitionModele): THREE.BufferGeometry {
  const morceaux = Object.values(definition.parties).map((cube) => {
    const g = creerGeometrieCube(cube, definition.tailleTexture);
    if (cube.rotation) g.rotateZ(cube.rotation[2]).rotateY(cube.rotation[1]).rotateX(cube.rotation[0]);
    return g.translate(cube.pivot[0] / PIXELS_PAR_BLOC, cube.pivot[1] / PIXELS_PAR_BLOC, cube.pivot[2] / PIXELS_PAR_BLOC);
  });
  const geometrie = mergeGeometries(morceaux);
  if (!geometrie) throw new Error('Modèle impossible à fusionner');
  return geometrie;
}
