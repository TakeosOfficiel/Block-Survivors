/**
 * Skins importées par le joueur, ramenées au format du jeu avant d'être
 * gardées : patron carré 64×64 ou HD (128, 256, 512, 1024…). Une image carrée
 * d'une autre taille est remise à la taille HD la plus proche (pixels nets) ;
 * l'ancien format 2:1 (64×32 et ses HD), sans membres gauches, est converti
 * comme le fait le jeu (membres gauches = droits en miroir).
 *
 * Gardée aussi légère que possible (plusieurs profils partagent ~5 Mo de
 * stockage) : une skin « HD » qui n'est qu'une skin agrandie (blocs de pixels
 * identiques) est gardée à sa vraie taille, sans aucune perte ; une skin
 * encore trop lourde est réduite de moitié jusqu'à tenir.
 */

/** Taille du patron du jeu (les skins HD en sont des multiples). */
const PATRON = 64;
/** Plus grande skin gardée (16 fois le patron). */
const TAILLE_MAX = 1024;
/** Au-delà de ce poids, une skin gardée avant l'allègement est ramenée à sa forme légère au démarrage. */
export const SKIN_LEGERE = 20_000;
/** Poids maximal de la skin dans la sauvegarde (caractères de la data URL ; le stockage local fait ~5 Mo en tout). */
const POIDS_MAX = 1_200_000;

/** `reduite` : détails perdus pour tenir dans le stockage (une réduction sans perte ne compte pas). */
export type SkinNormalisee = { readonly adresse: string; readonly taille: number; readonly reduite: boolean } | { readonly erreur: string };

/** Lit une image quelconque (data URL), ou null si elle est illisible. */
export function lireImage(adresse: string): Promise<HTMLImageElement | null> {
  return new Promise((resoudre) => {
    const image = new Image();
    image.onload = () => resoudre(image);
    image.onerror = () => resoudre(null);
    image.src = adresse;
  });
}

/** Ramène l'image au patron du jeu (voir l'en-tête) et l'encode en PNG assez léger pour être gardé. */
export function normaliserSkin(image: HTMLImageElement): SkinNormalisee {
  const l = image.width;
  const h = image.height;
  const carre = l === h;
  const ancien = l === h * 2;
  if ((!carre && !ancien) || l < PATRON) {
    return { erreur: `Skin refusée : l'image fait ${l}×${h}. Il faut une skin du jeu, carrée (64×64, ou HD : 128, 256, 512, 1024…) ou à l'ancien format (64×32 et ses HD).` };
  }
  // Taille HD la plus proche : 64 × une puissance de 2, au plus TAILLE_MAX
  let taille = PATRON * 2 ** Math.round(Math.log2(l / PATRON));
  taille = Math.min(TAILLE_MAX, Math.max(PATRON, taille));
  let toile = dessinerPatron(image, taille, ancien);
  // Agrandie sans détail en plus : gardée à sa vraie taille (rendu identique, pixels nets)
  const reelle = tailleReelle(toile);
  if (reelle < taille) {
    taille = reelle;
    toile = reduire(toile, taille);
  }
  let adresse = toile.toDataURL('image/png');
  let reduite = false;
  while (adresse.length > POIDS_MAX && taille > PATRON) {
    reduite = true;
    taille /= 2;
    toile = dessinerPatron(image, taille, ancien);
    adresse = toile.toDataURL('image/png');
  }
  if (adresse.length > POIDS_MAX) return { erreur: 'Skin refusée : l\'image est trop lourde pour être gardée.' };
  return { adresse, taille, reduite };
}

/** Plus petite taille (64 × une puissance de 2) où chaque bloc de pixels de la skin est d'une seule couleur. */
function tailleReelle(toile: HTMLCanvasElement): number {
  const taille = toile.width;
  const ctx = toile.getContext('2d');
  if (!ctx) return taille;
  const px = new Uint32Array(ctx.getImageData(0, 0, taille, taille).data.buffer);
  let facteur = taille / PATRON;
  while (facteur > 1) {
    if (blocsUnis(px, taille, facteur)) return taille / facteur;
    facteur /= 2;
  }
  return taille;
}

function blocsUnis(px: Uint32Array, taille: number, facteur: number): boolean {
  for (let y = 0; y < taille; y++) {
    const ligneBloc = (y - (y % facteur)) * taille;
    for (let x = 0; x < taille; x++) {
      if (px[y * taille + x] !== px[ligneBloc + x - (x % facteur)]) return false;
    }
  }
  return true;
}

/** Copie réduite à `taille` (pixels nets). */
function reduire(source: HTMLCanvasElement, taille: number): HTMLCanvasElement {
  const toile = document.createElement('canvas');
  toile.width = taille;
  toile.height = taille;
  const ctx = toile.getContext('2d');
  if (!ctx) return toile;
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(source, 0, 0, taille, taille);
  return toile;
}

/** Patron carré de `taille` px ; l'ancien format reçoit ses membres gauches. */
function dessinerPatron(image: HTMLImageElement, taille: number, ancien: boolean): HTMLCanvasElement {
  const toile = document.createElement('canvas');
  toile.width = taille;
  toile.height = taille;
  const ctx = toile.getContext('2d');
  if (!ctx) return toile;
  ctx.imageSmoothingEnabled = false;
  if (!ancien) {
    ctx.drawImage(image, 0, 0, taille, taille);
    return toile;
  }
  // Ancien format : la moitié haute telle quelle, puis jambe et bras gauches
  ctx.drawImage(image, 0, 0, image.width, image.height, 0, 0, taille, taille / 2);
  const k = taille / PATRON;
  membreMiroir(ctx, k, 0, 16, 16, 48);   // jambe droite → jambe gauche
  membreMiroir(ctx, k, 40, 16, 32, 48);  // bras droit → bras gauche
  return toile;
}

/**
 * Membre de 4×12×4 (patron du jeu, en px de 64) recopié en miroir de (u, v) vers (du, dv) :
 * chaque face retournée, faces droite et gauche échangées.
 */
function membreMiroir(ctx: CanvasRenderingContext2D, k: number, u: number, v: number, du: number, dv: number): void {
  // [x source, y, largeur, hauteur, x destination] des 6 faces, en px du patron
  const faces: readonly (readonly [number, number, number, number, number])[] = [
    [4, 0, 4, 4, 4],    // dessus
    [8, 0, 4, 4, 8],    // dessous
    [0, 4, 4, 12, 8],   // côté extérieur → l'autre côté
    [4, 4, 4, 12, 4],   // devant
    [8, 4, 4, 12, 0],   // côté intérieur → l'autre côté
    [12, 4, 4, 12, 12], // dos
  ];
  const source = ctx.getImageData(0, 0, ctx.canvas.width, ctx.canvas.height);
  const toile = document.createElement('canvas');
  toile.width = ctx.canvas.width;
  toile.height = ctx.canvas.height;
  toile.getContext('2d')?.putImageData(source, 0, 0);
  for (const [sx, sy, l, h, dx] of faces) {
    ctx.save();
    ctx.translate((du + dx + l) * k, (dv + sy) * k);
    ctx.scale(-1, 1);
    ctx.drawImage(toile, (u + sx) * k, (v + sy) * k, l * k, h * k, 0, 0, l * k, h * k);
    ctx.restore();
  }
}
