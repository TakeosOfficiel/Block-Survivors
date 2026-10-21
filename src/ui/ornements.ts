/**
 * Ornements de l'écran de chargement, dessinés en pixel art au lancement :
 * un losange d'or avec une émeraude sertie au centre (coins du cadre, bouts
 * de la barre). Il est affiché exactement 3 fois plus grand, comme le liseré
 * du cadre (3 pixels par trait) : tous les pixels ont la même taille, et sa
 * rangée du milieu recouvre exactement le trait d'or.
 */

/** Côté du losange, en pixels du dessin (impair : un trait de 1 pixel passe par sa rangée du milieu). */
export const COTE_PLAQUE = 13;

/** Couleurs de l'or et de l'émeraude. */
const COULEURS = {
  contour: '#1a1206',
  orClair: '#ffe58a',
  or: '#e0b040',
  orSombre: '#8a6418',
  reflet: '#c8ffd8',
  emeraude: '#17dd62',
  emeraudeSombre: '#0a8a3a',
} as const;

/**
 * Couleur du pixel (x, y) du losange, par anneaux depuis le bord (distance
 * « en losange » au centre) : contour sombre, biseau éclairé en haut à
 * gauche, or, puis l'émeraude au centre (reflet en haut à
 * gauche, ombre en bas à droite). Hors du losange : transparent.
 */
function pixelPlaque(x: number, y: number): string | null {
  const c = (COTE_PLAQUE - 1) / 2;
  const anneau = c - (Math.abs(x - c) + Math.abs(y - c));
  if (anneau < 0) return null;
  const hautGauche = x + y < 2 * c;   // au-dessus de la diagonale qui monte vers la droite
  if (anneau === 0) return COULEURS.contour;
  if (anneau === 1) return hautGauche ? COULEURS.orClair : COULEURS.orSombre;
  if (anneau === 2) return COULEURS.or;
  // Émeraude (losange de 7 pixels de haut)
  if (x === c - 1 && y === c - 1) return COULEURS.reflet;
  if (x === c && y === c - 2) return COULEURS.reflet;
  return hautGauche || (x + y === 2 * c && x < c) ? COULEURS.emeraude : COULEURS.emeraudeSombre;
}

/** Le losange, en image PNG (adresse data:) de COTE_PLAQUE pixels de côté. */
export function dessinerPlaque(): string {
  const toile = document.createElement('canvas');
  toile.width = COTE_PLAQUE;
  toile.height = COTE_PLAQUE;
  const ctx = toile.getContext('2d');
  if (!ctx) return '';
  for (let y = 0; y < COTE_PLAQUE; y++) {
    for (let x = 0; x < COTE_PLAQUE; x++) {
      const couleur = pixelPlaque(x, y);
      if (!couleur) continue;
      ctx.fillStyle = couleur;
      ctx.fillRect(x, y, 1, 1);
    }
  }
  return toile.toDataURL();
}
