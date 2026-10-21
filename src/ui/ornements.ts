/**
 * Ornements de l'écran de chargement, dessinés en pixel art au lancement :
 * un joyau émeraude serti d'or (coins du cadre, bouts de la barre).
 * Rien d'importé : quelques formes simples, un pixel = un carré.
 */

/** Couleurs de la monture dorée et de l'émeraude. */
const OR = { contour: '#1a1206', clair: '#ffe58a', moyen: '#e0b040', sombre: '#8a6418' };
const EMERAUDE = { clair: '#b4ffd0', moyen: '#17dd62', sombre: '#0a8a3a' };

/**
 * Joyau de `cote` pixels (impair) : un losange d'or au bord sombre, éclairé en
 * haut à gauche, et au centre une émeraude taillée avec un reflet. Image PNG
 * (adresse data:) à afficher en pixels nets.
 */
export function dessinerJoyau(cote = 15): string {
  const toile = document.createElement('canvas');
  toile.width = cote;
  toile.height = cote;
  const ctx = toile.getContext('2d');
  if (!ctx) return '';
  const c = (cote - 1) / 2;
  for (let y = 0; y < cote; y++) {
    for (let x = 0; x < cote; x++) {
      const d = Math.abs(x - c) + Math.abs(y - c);   // distance « losange » au centre
      const hautGauche = x + y < cote - 1;
      if (d > c) continue;  // hors du losange : transparent
      let couleur: string | null = null;
      if (d === c) couleur = OR.contour;
      else if (d === c - 1) couleur = hautGauche ? OR.clair : OR.sombre;
      else if (d >= c - 3) couleur = OR.moyen;
      else if (d === c - 4) couleur = OR.contour;
      else if (d <= c - 5) couleur = hautGauche ? (d <= 1 ? EMERAUDE.clair : EMERAUDE.moyen) : EMERAUDE.sombre;
      if (!couleur) continue;
      ctx.fillStyle = couleur;
      ctx.fillRect(x, y, 1, 1);
    }
  }
  // Reflet de l'émeraude
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(c - 1, c - 1, 1, 1);
  return toile.toDataURL();
}
