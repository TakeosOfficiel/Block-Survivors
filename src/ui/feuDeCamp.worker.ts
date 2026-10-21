/**
 * Worker de l'écran de chargement : dessine la scène du feu de camp sur un
 * OffscreenCanvas, loin du fil principal. La partie peut se construire (map,
 * modèles, shaders) sans que l'animation ne saccade.
 *
 * Messages reçus : voir MessageFeuDeCamp (sceneChargement.ts envoie le canvas,
 * les images et le modèle du feu, puis la taille, l'avancement, le héros,
 * démarrer / arrêter). Message envoyé : « prete » quand la scène est construite.
 */
import * as THREE from 'three';
import { FEU_DE_CAMP } from '../donnees/feuDeCamp';
import type { Quad } from '../minecraft/modeles';
import { SceneFeuDeCamp, type HerosFeuDeCamp } from './feuDeCamp3d';

/** Image vanilla envoyée par le fil principal. */
export interface ImageEnvoyee {
  readonly chemin: string;
  readonly image: ImageBitmap;
}

export type MessageFeuDeCamp =
  | { readonly type: 'init'; readonly canvas: OffscreenCanvas; readonly images: readonly ImageEnvoyee[]; readonly quads: readonly Quad[]; readonly heros: HerosFeuDeCamp; readonly largeur: number; readonly hauteur: number }
  | { readonly type: 'taille'; readonly largeur: number; readonly hauteur: number }
  | { readonly type: 'progression'; readonly part: number }
  | { readonly type: 'heros'; readonly images: readonly ImageEnvoyee[]; readonly heros: HerosFeuDeCamp }
  | { readonly type: 'demarrer' }
  | { readonly type: 'arreter' };

/** Le worker vu par lui-même (le projet est typé avec la bibliothèque DOM). */
const travailleur = self as unknown as {
  postMessage(message: unknown): void;
  onmessage: ((evenement: MessageEvent<MessageFeuDeCamp>) => void) | null;
};
const textures = new Map<string, THREE.Texture>();
const pixels = new Map<string, ImageData>();
let scene: SceneFeuDeCamp | null = null;
let actif = false;
let boucle = false;
let dernier = 0;
const debut = performance.now();

/** Textures aux règles du jeu (pixels nets, bandes animées), et pixels lisibles pour les objets en relief. */
async function ajouterImages(images: readonly ImageEnvoyee[]): Promise<void> {
  await Promise.all(images.map(async ({ chemin, image }) => {
    // Une ImageBitmap ne se retourne pas à l'envoi vers la carte graphique : on la retourne avant
    const retournee = await createImageBitmap(image, { imageOrientation: 'flipY' });
    const texture = new THREE.Texture(retournee);
    texture.flipY = false;
    texture.magFilter = THREE.NearestFilter;
    texture.minFilter = THREE.NearestFilter;
    texture.generateMipmaps = false;
    texture.colorSpace = THREE.SRGBColorSpace;
    // Texture animée vanilla = bande verticale de frames carrées : on n'en affiche qu'une
    const frames = image.height > image.width && image.height % image.width === 0 ? image.height / image.width : 1;
    if (frames > 1) {
      texture.repeat.y = 1 / frames;
      texture.offset.y = 1 - 1 / frames;
    }
    texture.needsUpdate = true;
    textures.set(chemin, texture);

    const toile = new OffscreenCanvas(image.width, image.height);
    const ctx = toile.getContext('2d', { willReadFrequently: true });
    if (ctx) {
      ctx.drawImage(image, 0, 0);
      pixels.set(chemin, ctx.getImageData(0, 0, image.width, image.height));
    }
    image.close();
  }));
}

/** Boucle d'affichage, à cadence plafonnée. */
function image(maintenant: number): void {
  if (!actif || !scene) {
    boucle = false;
    return;
  }
  if (maintenant - dernier >= 1000 / FEU_DE_CAMP.imagesParSeconde - 2 && scene.dessiner((maintenant - debut) / 1000)) {
    dernier = maintenant;
  }
  requestAnimationFrame(image);
}
function lancer(): void {
  if (boucle || !actif || !scene) return;
  boucle = true;
  requestAnimationFrame(image);
}

travailleur.onmessage = async (evenement) => {
  const m = evenement.data;
  switch (m.type) {
    case 'init':
      await ajouterImages(m.images);
      scene = new SceneFeuDeCamp(m.canvas, { textures, pixels }, m.quads, m.heros);
      scene.redimensionner(m.largeur, m.hauteur);
      // Première image tout de suite, même cachée : les shaders se compilent pendant le menu,
      // pas au moment où la partie se charge
      scene.dessiner(0);
      travailleur.postMessage({ type: 'prete' });
      lancer();
      break;
    case 'taille':
      scene?.redimensionner(m.largeur, m.hauteur);
      break;
    case 'progression':
      scene?.definirProgression(m.part);
      break;
    case 'heros':
      await ajouterImages(m.images);
      scene?.changerHeros(m.heros);
      break;
    case 'demarrer':
      actif = true;
      lancer();
      break;
    case 'arreter':
      actif = false;
      break;
  }
};
