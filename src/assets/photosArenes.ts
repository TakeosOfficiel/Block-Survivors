/**
 * Photos des arènes pour l'écran de chargement (comme les images de zone de
 * WoW). Le jeu en prend une à chaque fin de chargement (Jeu.photographier) et
 * la garde sur l'ordinateur : le chargement suivant de cette arène l'affiche.
 * Avant la toute première partie sur une arène, on montre celle fournie avec
 * le jeu (assets/chargement/<id>.webp), s'il y en a une.
 */
import { CONFIG } from '../config';
import { ecrireCache, lireCache } from './cacheLocal';

const FOURNIES = import.meta.glob('./chargement/*.webp', { eager: true, query: '?url', import: 'default' }) as Record<string, string>;

const cle = (idArene: string) => `block-survivors://photo-arene/${idArene}`;
let derniere: string | null = null;

/** Adresse de l'image à afficher pour cette arène, ou null s'il n'y en a aucune. */
export async function photoArene(idArene: string): Promise<string | null> {
  const gardee = await lireCache(cle(idArene));
  if (gardee) {
    if (derniere) URL.revokeObjectURL(derniere);
    derniere = URL.createObjectURL(gardee);
    return derniere;
  }
  return FOURNIES[`./chargement/${idArene}.webp`] ?? null;
}

/** Garde la photo pour les prochains chargements de cette arène (sans attendre). */
export function memoriserPhoto(idArene: string, photo: HTMLCanvasElement): void {
  photo.toBlob((image) => {
    if (image) ecrireCache(cle(idArene), image);
  }, 'image/webp', CONFIG.chargement.photo.qualite);
}
