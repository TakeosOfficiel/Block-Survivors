/**
 * Ornements de l'écran de chargement, dessinés en pixel art au lancement :
 * une plaque d'or carrée avec une émeraude sertie au centre (coins du cadre,
 * bouts de la barre). Elle est affichée exactement 3 fois plus grande, comme
 * le liseré du cadre (3 pixels par trait) : tous les pixels ont la même taille.
 */

/** Côté de la plaque, en pixels du dessin (impair : un trait de 1 pixel passe par sa rangée du milieu). */
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
 * Couleur du pixel (x, y) de la plaque, par anneaux depuis le bord : contour
 * sombre (coins arrondis), biseau éclairé en haut à gauche, or, sertissure
 * sombre, puis l'émeraude au centre (reflet en haut à gauche, ombre en bas à droite).
 */
function pixelPlaque(x: number, y: number): string | null {
  const n = COTE_PLAQUE - 1;
  const anneau = Math.min(x, y, n - x, n - y);
  if (anneau === 0) {
    const coin = (x === 0 || x === n) && (y === 0 || y === n);
    return coin ? null : COULEURS.contour;
  }
  if (anneau === 1) {
    const hautGauche = x === 1 || y === 1;
    const basDroite = x === n - 1 || y === n - 1;
    if (hautGauche && basDroite) return COULEURS.or;   // coins du biseau
    return hautGauche ? COULEURS.orClair : COULEURS.orSombre;
  }
  if (anneau === 2) return COULEURS.or;
  if (anneau === 3) return COULEURS.contour;
  // Émeraude (5 x 5)
  const u = x - 4, v = y - 4;
  if (u + v <= 1) return COULEURS.reflet;
  if (u === 4 || v === 4) return COULEURS.emeraudeSombre;
  return COULEURS.emeraude;
}

/** La plaque, en image PNG (adresse data:) de COTE_PLAQUE pixels de côté. */
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
