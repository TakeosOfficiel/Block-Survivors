/**
 * Profil du joueur, conservé dans le navigateur : pseudo (affiché au-dessus
 * de son personnage) et apparence. L'apparence ne change que le modèle et la
 * skin ; la classe choisie garde ses statistiques, ses armes et son objet en main.
 */
import { SKINS_VANILLA, type NomSkin } from './donnees/skins';
import type { ApparenceEntite, CheminTexture } from './donnees/types';
import { ecrireStockage, lireStockage } from './options';

/** Skin de la classe, skin importée, ou une des skins par défaut du jeu. */
export type ChoixSkin = 'classe' | 'importe' | NomSkin;

export interface Profil {
  pseudo: string;
  skin: ChoixSkin;
  /** Modèle aux bras fins (3 pixels, comme Alex). */
  brasFins: boolean;
  /** Skin importée (PNG en data URL), ou null. */
  skinImporte: string | null;
}

export const LONGUEUR_PSEUDO = 16;

/** Chemin sous lequel la skin importée est enregistrée dans la bibliothèque de textures. */
export const CHEMIN_SKIN_IMPORTE: CheminTexture = 'profil/skin';

const CLE = 'block-survivors.profil';

export function chargerProfil(): Profil {
  const profil: Profil = { pseudo: 'Joueur', skin: 'classe', brasFins: false, skinImporte: null };
  const lu = lireStockage<Partial<Profil>>(CLE);
  if (!lu) return profil;
  if (typeof lu.pseudo === 'string') profil.pseudo = nettoyerPseudo(lu.pseudo);
  if (typeof lu.brasFins === 'boolean') profil.brasFins = lu.brasFins;
  if (typeof lu.skinImporte === 'string' && lu.skinImporte.startsWith('data:image/')) profil.skinImporte = lu.skinImporte;
  if (lu.skin === 'classe' || (lu.skin === 'importe' && profil.skinImporte) || SKINS_VANILLA.includes(lu.skin as NomSkin)) {
    profil.skin = lu.skin as ChoixSkin;
  }
  return profil;
}

export function enregistrerProfil(profil: Profil): void {
  ecrireStockage(CLE, profil);
}

/** Pseudo sans espaces superflus, limité à LONGUEUR_PSEUDO caractères. */
export function nettoyerPseudo(pseudo: string): string {
  return pseudo.replace(/\s+/g, ' ').trim().slice(0, LONGUEUR_PSEUDO);
}

/** Skin d'une des skins par défaut, au gabarit choisi. */
export function cheminSkinVanilla(nom: NomSkin, brasFins: boolean): CheminTexture {
  return `entity/player/${brasFins ? 'slim' : 'wide'}/${nom}`;
}

/** Apparence du personnage joué : celle de la classe, ou la skin du profil. */
export function apparenceJoueur<T extends ApparenceEntite>(classe: T, profil: Profil): T {
  if (profil.skin === 'classe') return classe;
  const modele = profil.brasFins ? 'humanoide_fin' : 'humanoide';
  const skin = profil.skin === 'importe' ? CHEMIN_SKIN_IMPORTE : cheminSkinVanilla(profil.skin, profil.brasFins);
  return { ...classe, modele, skin };
}

const DEBUTS = ['Creeper', 'Diamant', 'Redstone', 'Ender', 'Pixel', 'Blaze', 'Golem', 'Nether', 'Obsi', 'Lapis', 'Silex', 'Cubo'];
const FINS = ['Chasseur', 'Furtif', 'Doré', 'Sombre', 'Rapide', 'Mineur', 'Lunaire', 'Royal', 'Sauvage', 'Brave'];

/** Pseudo au hasard, dans l'esprit du jeu (ex. « RedstoneFurtif42 »). */
export function pseudoAleatoire(): string {
  const tirer = (liste: readonly string[]) => liste[Math.floor(Math.random() * liste.length)];
  const nombre = Math.random() < 0.6 ? String(Math.floor(Math.random() * 100)) : '';
  return nettoyerPseudo(`${tirer(DEBUTS)}${tirer(FINS)}${nombre}`);
}
