/**
 * Portraits : visage découpé dans la skin d'une entité (face avant de la
 * partie « tete » de son modèle), avec la couche de chapeau des skins de
 * joueur. Servent au menu et au bilan des ennemis vaincus.
 */
import type { BibliothequeTextures } from '../assets/textures';
import { MODELES } from '../donnees/modeles';
import type { ApparenceEntite } from '../donnees/types';

const cache = new Map<string, string | null>();

/** Oublie les visages déjà découpés (une skin a changé sous le même chemin). */
export function oublierPortraits(): void {
  cache.clear();
}

/** Adresse d'une image du visage (pixels d'origine), ou null si la skin manque. */
export function portrait(textures: BibliothequeTextures, apparence: ApparenceEntite): string | null {
  const cle = `${apparence.modele}|${apparence.skin}`;
  const connu = cache.get(cle);
  if (connu !== undefined) return connu;

  const image = textures.obtenir(apparence.skin)?.image as HTMLImageElement | undefined;
  const modele = MODELES[apparence.modele];
  const tete = modele.parties['tete'];
  let url: string | null = null;
  if (image && tete) {
    // Patron d'une boîte : la face avant commence à (u + profondeur, v + profondeur)
    const [largeur, hauteur, profondeur] = tete.taille;
    const echelle = image.width / modele.tailleTexture[0];
    const canvas = document.createElement('canvas');
    canvas.width = largeur;
    canvas.height = hauteur;
    const ctx = canvas.getContext('2d');
    if (ctx) {
      ctx.imageSmoothingEnabled = false;
      const decouper = (u: number, v: number) => ctx.drawImage(
        image,
        (u + profondeur) * echelle, (v + profondeur) * echelle, largeur * echelle, hauteur * echelle,
        0, 0, largeur, hauteur,
      );
      decouper(tete.uv[0], tete.uv[1]);
      // Couche de chapeau des skins de joueur 64x64 (32 pixels à droite)
      if (modele.tailleTexture[1] === 64 && modele.tailleTexture[0] === 64) decouper(tete.uv[0] + 32, tete.uv[1]);
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
