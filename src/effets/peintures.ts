/**
 * Skins peintes pour les modèles 3D d'objets qui n'ont pas d'entité dans le jeu
 * (la fusée d'artifice en vol n'y est qu'une icône) : les couleurs viennent des
 * pixels de l'item vanilla, posées sur le patron des cubes du modèle.
 */
import type { IdPeinture } from '../donnees/types';

type Couleur = readonly [number, number, number];

/** Pixel de l'item vanilla et couleur de secours si l'image est illisible. */
interface Echantillon {
  readonly x: number;
  readonly y: number;
  readonly secours: Couleur;
}

/** Couleurs de item/firework_rocket (16x16). */
const COULEURS_FUSEE = {
  rougeClair: { x: 8, y: 3, secours: [0xd6, 0x2a, 0x2a] },
  rouge:      { x: 8, y: 2, secours: [0xb9, 0x29, 0x29] },
  rougeFonce: { x: 7, y: 3, secours: [0x99, 0x29, 0x29] },
  rougeNoir:  { x: 9, y: 3, secours: [0x6d, 0x17, 0x17] },
  blanc:      { x: 7, y: 7, secours: [0xf4, 0xf4, 0xf4] },
  blancOmbre: { x: 7, y: 6, secours: [0xb2, 0xcc, 0xd1] },
  meche:      { x: 8, y: 13, secours: [0x2d, 0x14, 0x09] },
} satisfies Record<string, Echantillon>;

type Face = 'droite' | 'avant' | 'gauche' | 'arriere' | 'dessus' | 'dessous';

/** Remplit le patron d'un cube (même dépliage que creerGeometrieCube) pixel par pixel. */
function peindreCube(
  pixels: ImageData,
  [u, v]: readonly [number, number],
  [l, h, p]: readonly [number, number, number],
  couleur: (face: Face, colonne: number, ligne: number) => Couleur,
): void {
  const faces: readonly (readonly [Face, number, number, number, number])[] = [
    ['droite', u, v + p, p, h],
    ['avant', u + p, v + p, l, h],
    ['gauche', u + p + l, v + p, p, h],
    ['arriere', u + 2 * p + l, v + p, l, h],
    ['dessus', u + p, v, l, p],
    ['dessous', u + p + l, v, l, p],
  ];
  for (const [face, x0, y0, largeur, hauteur] of faces) {
    for (let y = 0; y < hauteur; y++) {
      for (let x = 0; x < largeur; x++) {
        // Côtés : colonne continue tout autour du cube (pour les rayures en spirale)
        const colonne = face === 'dessus' || face === 'dessous' ? x : x0 - u + x;
        const [r, g, b] = couleur(face, colonne, y);
        const i = ((y0 + y) * pixels.width + x0 + x) * 4;
        pixels.data[i] = r;
        pixels.data[i + 1] = g;
        pixels.data[i + 2] = b;
        pixels.data[i + 3] = 255;
      }
    }
  }
}

/** Lit les couleurs dans l'image de l'item (repli sur les couleurs de secours). */
function lireCouleurs<T extends Record<string, Echantillon>>(
  image: CanvasImageSource | null,
  echantillons: T,
): Record<keyof T, Couleur> {
  const resultat = {} as Record<keyof T, Couleur>;
  let pixels: ImageData | null = null;
  if (image) {
    try {
      const toile = document.createElement('canvas');
      toile.width = 16;
      toile.height = 16;
      const ctx = toile.getContext('2d', { willReadFrequently: true });
      if (ctx) {
        ctx.drawImage(image, 0, 0, 16, 16);
        pixels = ctx.getImageData(0, 0, 16, 16);
      }
    } catch {
      pixels = null;  // image d'une autre origine : couleurs de secours
    }
  }
  for (const cle of Object.keys(echantillons) as (keyof T)[]) {
    const e = echantillons[cle];
    const i = (e.y * 16 + e.x) * 4;
    resultat[cle] = pixels && pixels.data[i + 3] > 0
      ? [pixels.data[i], pixels.data[i + 1], pixels.data[i + 2]]
      : e.secours;
  }
  return resultat;
}

/** Fusée : tube à rayures blanches en spirale, coiffe rouge en pointe, tuyère noircie. */
function peindreFusee(pixels: ImageData, image: CanvasImageSource | null): void {
  const c = lireCouleurs(image, COULEURS_FUSEE);
  peindreCube(pixels, [0, 0], [4, 9, 4], (face, colonne, ligne) => {
    if (face === 'dessus') return c.rougeFonce;
    if (face === 'dessous') return c.rougeNoir;
    const bande = (colonne + ligne) % 6;
    return bande === 0 ? c.blanc : bande === 1 ? c.blancOmbre : bande === 5 ? c.rougeFonce : c.rouge;
  });
  peindreCube(pixels, [0, 13], [3, 2, 3], (face, _colonne, ligne) =>
    face === 'dessus' ? c.rouge : face === 'dessous' ? c.rougeFonce : ligne === 0 ? c.rougeClair : c.rouge);
  peindreCube(pixels, [16, 0], [2, 2, 2], (face, colonne) =>
    face === 'dessous' ? c.rougeFonce : colonne % 2 === 0 ? c.rougeClair : c.rouge);
  peindreCube(pixels, [16, 4], [1, 1, 1], () => c.rougeClair);
  peindreCube(pixels, [16, 6], [2, 1, 2], () => c.meche);
}

const PEINTURES: Readonly<Record<IdPeinture, (pixels: ImageData, image: CanvasImageSource | null) => void>> = {
  fusee: peindreFusee,
};

/** Skin du modèle, à la taille de texture qu'il attend, peinte d'après l'image de l'item. */
export function peindreSkin(id: IdPeinture, image: CanvasImageSource | null, [largeur, hauteur]: readonly [number, number]): HTMLCanvasElement {
  const toile = document.createElement('canvas');
  toile.width = largeur;
  toile.height = hauteur;
  const ctx = toile.getContext('2d');
  if (!ctx) return toile;
  const pixels = ctx.createImageData(largeur, hauteur);
  PEINTURES[id](pixels, image);
  ctx.putImageData(pixels, 0, 0);
  return toile;
}
