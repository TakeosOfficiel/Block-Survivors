/**
 * Portraits : visage découpé dans la skin d'une entité (face avant de la
 * partie « tete » de son modèle, ou de plusieurs parties vues de face), avec
 * la couche de chapeau des skins de joueur. Servent au menu et au bilan des
 * ennemis vaincus.
 */
import type { BibliothequeTextures } from '../assets/textures';
import { MODELES } from '../donnees/modeles';
import type { ApparenceEntite, CubeModele } from '../donnees/types';

const cache = new Map<string, string | null>();

/** Oublie les visages découpés dans cette skin (elle a changé sous le même chemin). */
export function oublierPortraits(skin: string): void {
  for (const cle of cache.keys()) if (cle.endsWith(`|${skin}`)) cache.delete(cle);
}

/** Adresse d'une image du visage (pixels d'origine), ou null si la skin manque. */
export function portrait(textures: BibliothequeTextures, apparence: ApparenceEntite): string | null {
  const cle = `${apparence.modele}|${apparence.skin}`;
  const connu = cache.get(cle);
  if (connu !== undefined) return connu;

  const image = textures.obtenir(apparence.skin)?.image as HTMLImageElement | undefined;
  const modele = MODELES[apparence.modele];
  // Parties du portrait vues de face, à leur place dans le modèle (la tête, le plus souvent)
  const parties = (modele.portrait ?? ['tete']).map((nom) => modele.parties[nom]).filter((p) => p !== undefined);
  let url: string | null = null;
  if (image && parties.length > 0) {
    // Cadre englobant des faces avant (le haut du modèle en haut de l'image)
    const gauche = (p: CubeModele) => p.pivot[0] + p.centre[0] - p.taille[0] / 2;
    const haut = (p: CubeModele) => p.pivot[1] + p.centre[1] + p.taille[1] / 2;
    const avant = (p: CubeModele) => p.pivot[2] + p.centre[2] + p.taille[2] / 2;
    const xMin = Math.min(...parties.map(gauche));
    const xMax = Math.max(...parties.map((p) => gauche(p) + p.taille[0]));
    const yMax = Math.max(...parties.map(haut));
    const yMin = Math.min(...parties.map((p) => haut(p) - p.taille[1]));
    const echelle = image.width / modele.tailleTexture[0];
    const canvas = document.createElement('canvas');
    canvas.width = Math.ceil(xMax - xMin);
    canvas.height = Math.ceil(yMax - yMin);
    // Canvas logiciel : toDataURL ne fait pas un aller-retour vers la carte graphique (lent)
    const ctx = canvas.getContext('2d', { willReadFrequently: true });
    if (ctx) {
      ctx.imageSmoothingEnabled = false;
      // De l'arrière vers l'avant : les parties proches recouvrent les autres
      for (const partie of [...parties].sort((a, b) => avant(a) - avant(b))) {
        // Patron d'une boîte : la face avant commence à (u + profondeur, v + profondeur)
        const [l, h, p] = partie.taille;
        const decouper = (u: number, v: number) => ctx.drawImage(
          image,
          (u + p) * echelle, (v + p) * echelle, l * echelle, h * echelle,
          Math.round(gauche(partie) - xMin), Math.round(yMax - haut(partie)), l, h,
        );
        decouper(partie.uv[0], partie.uv[1]);
        // Couche de chapeau des skins de joueur 64x64 (32 pixels à droite)
        if (parties.length === 1 && modele.tailleTexture[1] === 64 && modele.tailleTexture[0] === 64) decouper(partie.uv[0] + 32, partie.uv[1]);
      }
      try {
        url = canvas.toDataURL();
      } catch {
        url = null;  // image sans CORS : canvas illisible
      }
    }
  }
  cache.set(cle, url);
  return url;
}
