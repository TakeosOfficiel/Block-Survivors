/**
 * Police pixel de Minecraft, construite à l'exécution depuis les bitmaps
 * vanilla (font/ascii.png, accented.png, nonlatin_european.png) et la table de
 * caractères du jeu (font/include/default.json). Chaque pixel allumé devient
 * un carré vectoriel : la police reste nette à tout multiple de 8 px.
 * En cas d'échec, l'interface garde sa police de secours.
 */
import opentype from 'opentype.js';
import { SOURCES_FICHIERS, SOURCES_TEXTURES } from '../config';

export const NOM_POLICE = 'Minecraft';

/** Unités de police par pixel de texture (8 pixels par em). */
const U = 128;
/** Avance de l'espace, en pixels (fournisseur « space » du jeu). */
const AVANCE_ESPACE = 4;

interface FournisseurBitmap {
  readonly type: string;
  readonly file?: string;
  readonly ascent?: number;
  readonly height?: number;
  readonly chars?: readonly string[];
}

/** Charge la police et l'enregistre dans le document ; renvoie faux si elle est indisponible. */
export async function chargerPolice(): Promise<boolean> {
  try {
    const table = await chargerJson('font/include/default.json') as { providers: FournisseurBitmap[] };
    const glyphes = new Map<number, opentype.Glyph>();
    for (const fournisseur of table.providers) {
      if (fournisseur.type !== 'bitmap' || !fournisseur.file || !fournisseur.chars) continue;
      const chemin = `font/${fournisseur.file.replace('minecraft:font/', '').replace(/\.png$/, '')}`;
      const image = await chargerImage(chemin);
      ajouterGlyphes(fournisseur, image, glyphes);
    }
    if (glyphes.size === 0) return false;

    const police = new opentype.Font({
      familyName: NOM_POLICE,
      styleName: 'Regular',
      unitsPerEm: 8 * U,
      ascender: 7 * U,
      descender: -1 * U,
      glyphs: [
        new opentype.Glyph({ name: '.notdef', unicode: 0, advanceWidth: 6 * U, path: new opentype.Path() }),
        new opentype.Glyph({ name: 'space', unicode: 32, advanceWidth: AVANCE_ESPACE * U, path: new opentype.Path() }),
        ...glyphes.values(),
      ],
    });
    const face = new FontFace(NOM_POLICE, police.toArrayBuffer());
    await face.load();
    document.fonts.add(face);
    return true;
  } catch {
    return false;
  }
}

/** Découpe la planche en cases et vectorise chaque caractère (le premier fournisseur l'emporte, comme en jeu). */
function ajouterGlyphes(fournisseur: FournisseurBitmap, image: HTMLImageElement, glyphes: Map<number, opentype.Glyph>): void {
  const lignes = fournisseur.chars ?? [];
  const colonnes = [...(lignes[0] ?? '')].length;
  if (colonnes === 0) return;
  const largeurCase = image.width / colonnes;
  const hauteurCase = image.height / lignes.length;
  const echelle = (fournisseur.height ?? 8) / hauteurCase;
  const ascent = fournisseur.ascent ?? 7;
  const pixels = lirePixels(image);

  lignes.forEach((ligne, rangee) => {
    [...ligne].forEach((caractere, colonne) => {
      const code = caractere.codePointAt(0) ?? 0;
      if (code === 0 || code === 32 || glyphes.has(code)) return;
      const x0 = colonne * largeurCase;
      const y0 = rangee * hauteurCase;
      const allume = (x: number, y: number) => pixels.data[((y0 + y) * image.width + x0 + x) * 4 + 3] > 0;

      // Largeur = dernière colonne allumée + 1 ; avance = largeur + 1 pixel d'espacement
      let largeur = 0;
      for (let x = largeurCase - 1; x >= 0 && largeur === 0; x--) {
        for (let y = 0; y < hauteurCase; y++) if (allume(x, y)) { largeur = x + 1; break; }
      }
      // Un rectangle par suite de pixels allumés sur une ligne
      const chemin = new opentype.Path();
      for (let y = 0; y < hauteurCase; y++) {
        let debut = -1;
        for (let x = 0; x <= largeur; x++) {
          const plein = x < largeur && allume(x, y);
          if (plein && debut < 0) debut = x;
          if (!plein && debut >= 0) {
            rectangle(chemin, debut * echelle, (ascent - y * echelle), (x - debut) * echelle, echelle);
            debut = -1;
          }
        }
      }
      glyphes.set(code, new opentype.Glyph({
        name: `u${code.toString(16)}`,
        unicode: code,
        advanceWidth: Math.round((largeur * echelle + 1) * U),
        path: chemin,
      }));
    });
  });
}

/** Rectangle de largeur `l` et hauteur `h` pixels sous le point (x, haut), sens horaire. */
function rectangle(chemin: opentype.Path, x: number, haut: number, l: number, h: number): void {
  chemin.moveTo(x * U, haut * U);
  chemin.lineTo((x + l) * U, haut * U);
  chemin.lineTo((x + l) * U, (haut - h) * U);
  chemin.lineTo(x * U, (haut - h) * U);
  chemin.close();
}

function lirePixels(image: HTMLImageElement): ImageData {
  const canvas = document.createElement('canvas');
  canvas.width = image.width;
  canvas.height = image.height;
  const ctx = canvas.getContext('2d', { willReadFrequently: true });
  if (!ctx) throw new Error('Canvas 2D indisponible');
  ctx.drawImage(image, 0, 0);
  return ctx.getImageData(0, 0, image.width, image.height);
}

async function chargerJson(chemin: string): Promise<unknown> {
  for (const source of SOURCES_FICHIERS) {
    try {
      const reponse = await fetch(source(chemin));
      if (reponse.ok) return await reponse.json();
    } catch {
      // Source suivante
    }
  }
  throw new Error(`Introuvable : ${chemin}`);
}

async function chargerImage(chemin: string): Promise<HTMLImageElement> {
  for (const source of SOURCES_TEXTURES) {
    try {
      const image = new Image();
      image.crossOrigin = 'anonymous';
      image.src = source.url(chemin);
      await image.decode();
      return image;
    } catch {
      // Source suivante
    }
  }
  throw new Error(`Introuvable : ${chemin}`);
}
