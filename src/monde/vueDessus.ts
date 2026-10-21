/**
 * Vue de dessus d'une arène, comme une carte de Minecraft : une couleur par
 * colonne (celle du bloc le plus haut, moyenne de la texture de son dessus),
 * assombrie quand le sol descend vers le sud pour faire sentir le relief.
 * L'écran de chargement la dévoile chunk par chunk.
 */
import { lirePixels } from '../assets/pixels';
import type { BibliothequeTextures } from '../assets/textures';
import { BLOCS, type IdBloc } from '../donnees/blocs';
import type { DefinitionArene } from '../donnees/types';
import type { MondeGenere } from './generation';

export interface VueDessus {
  /** Colonnes d'ouest en est, et du nord au sud. */
  readonly largeur: number;
  readonly profondeur: number;
  /** RGBA par colonne (ligne par ligne, du nord au sud). */
  readonly couleurs: Uint8ClampedArray;
  /** Colonne où le joueur apparaît, ou null. */
  readonly depart: { readonly x: number; readonly z: number } | null;
}

/** Relief façon carte de Minecraft : plus haut que la colonne au nord = clair, plus bas = sombre. */
const OMBRES = { plusHaut: 1, egal: 0.88, plusBas: 0.74 } as const;

const moyennes = new Map<string, number | null>();

/** Couleur moyenne (0xRRGGBB) des pixels visibles de la première image d'une texture ; null si illisible. */
export function couleurMoyenne(textures: BibliothequeTextures, chemin: string): number | null {
  if (moyennes.has(chemin)) return moyennes.get(chemin) ?? null;
  let couleur: number | null = null;
  const image = textures.obtenir(chemin)?.image as (CanvasImageSource & { width: number; height: number }) | undefined;
  if (image) {
    try {
      const { data, width } = lirePixels(image);
      let r = 0, v = 0, b = 0, n = 0;
      // Bande animée : seulement la première image (carrée, en haut)
      for (let i = 0; i < width * width; i++) {
        if (data[i * 4 + 3] < 128) continue;
        r += data[i * 4]; v += data[i * 4 + 1]; b += data[i * 4 + 2]; n++;
      }
      if (n > 0) couleur = (Math.round(r / n) << 16) | (Math.round(v / n) << 8) | Math.round(b / n);
    } catch {
      // Image sans CORS : couleur de secours
    }
  }
  moyennes.set(chemin, couleur);
  return couleur;
}

/** Multiplie deux couleurs 0xRRGGBB, comme une teinte de biome sur une texture grise. */
export function teinter(couleur: number, teinte: number): number {
  const c = (decalage: number) => Math.round((((couleur >> decalage) & 255) * ((teinte >> decalage) & 255)) / 255);
  return (c(16) << 16) | (c(8) << 8) | c(0);
}

/**
 * Remplit les couleurs à partir de la couleur et de la hauteur de chaque
 * colonne (couleur −1 : colonne vide, laissée transparente).
 */
export function colorierVue(largeur: number, profondeur: number, couleur: Int32Array, hauteur: Float32Array): Uint8ClampedArray {
  const couleurs = new Uint8ClampedArray(largeur * profondeur * 4);
  for (let z = 0; z < profondeur; z++) {
    for (let x = 0; x < largeur; x++) {
      const i = z * largeur + x;
      if (couleur[i] < 0) continue;
      const nord = z > 0 && couleur[i - largeur] >= 0 ? hauteur[i - largeur] : hauteur[i];
      const ombre = hauteur[i] > nord ? OMBRES.plusHaut : hauteur[i] < nord ? OMBRES.plusBas : OMBRES.egal;
      couleurs[i * 4] = ((couleur[i] >> 16) & 255) * ombre;
      couleurs[i * 4 + 1] = ((couleur[i] >> 8) & 255) * ombre;
      couleurs[i * 4 + 2] = (couleur[i] & 255) * ombre;
      couleurs[i * 4 + 3] = 255;
    }
  }
  return couleurs;
}

/** Textures du dessus des blocs d'un monde généré (à charger avant vueDessusMonde). */
export function texturesDessus(monde: MondeGenere): Set<string> {
  return new Set([...monde.blocs.keys()].map((id) => BLOCS[id].faces.haut));
}

/** Vue de dessus d'une arène générée : le bloc le plus haut de chaque colonne de la zone jouable. */
export function vueDessusMonde(monde: MondeGenere, arene: DefinitionArene, textures: BibliothequeTextures): VueDessus {
  const { largeur, profondeur, demiX, demiZ } = monde.carte;
  const couleur = new Int32Array(largeur * profondeur).fill(-1);
  const hauteur = new Float32Array(largeur * profondeur).fill(-Infinity);
  const couleurBloc = new Map<IdBloc, number>();
  for (const [id, positions] of monde.blocs) {
    const bloc = BLOCS[id];
    const moyenne = couleurMoyenne(textures, bloc.faces.haut);
    const teinte = bloc.teinteHaut ?? bloc.teinte;
    couleurBloc.set(id, moyenne === null ? bloc.couleurSecours : teinte !== undefined ? teinter(moyenne, teinte) : moyenne);
    for (let k = 0; k < positions.length; k += 3) {
      const x = Math.floor(positions[k] + demiX), z = Math.floor(positions[k + 2] + demiZ);
      if (x < 0 || z < 0 || x >= largeur || z >= profondeur) continue;
      const i = z * largeur + x;
      if (positions[k + 1] <= hauteur[i]) continue;
      hauteur[i] = positions[k + 1];
      couleur[i] = couleurBloc.get(id) ?? 0;
    }
  }
  return {
    largeur,
    profondeur,
    couleurs: colorierVue(largeur, profondeur, couleur, hauteur),
    depart: { x: Math.floor((arene.depart?.x ?? 0) + demiX), z: Math.floor((arene.depart?.z ?? 0) + demiZ) },
  };
}
