/**
 * Photos des arènes pour l'écran de chargement (comme les images de zone de
 * WoW) et le menu d'accueil (fond, vignettes). Le jeu en prend une à chaque fin
 * de chargement (Jeu.photographier) et la garde sur l'ordinateur : la suivante
 * affichera celle-là. Avant la toute première partie sur une arène, on montre
 * celle fournie avec le jeu (assets/chargement/<id>.webp), s'il y en a une.
 */
import { CONFIG } from '../config';
import { ecrireCache, lireCache } from './cacheLocal';

const FOURNIES = import.meta.glob('./chargement/*.webp', { eager: true, query: '?url', import: 'default' }) as Record<string, string>;

const cle = (idArene: string) => `block-survivors://photo-arene/${idArene}`;
/** Adresses déjà prêtes, par arène (la page est rechargée à chaque partie : elles restent justes). */
const adresses = new Map<string, Promise<string | null>>();

/** Adresse de l'image à afficher pour cette arène, ou null s'il n'y en a aucune. */
export function photoArene(idArene: string): Promise<string | null> {
  let adresse = adresses.get(idArene);
  if (!adresse) {
    adresse = lireCache(cle(idArene)).then((gardee) => (gardee ? URL.createObjectURL(gardee) : FOURNIES[`./chargement/${idArene}.webp`] ?? null));
    adresses.set(idArene, adresse);
  }
  return adresse;
}

/** Garde la photo pour les prochains chargements de cette arène (sans attendre). */
export function memoriserPhoto(idArene: string, photo: HTMLCanvasElement): void {
  photo.toBlob((image) => {
    if (image) ecrireCache(cle(idArene), image);
  }, 'image/webp', CONFIG.chargement.photo.qualite);
}
