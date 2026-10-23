/** Apparence et place de départ des joueurs d'une partie à plusieurs (mêmes calculs chez l'hôte et les joueurs). */
import { REGLES_MULTIJOUEUR } from '../donnees/multijoueur';
import { PERSONNAGES } from '../donnees/personnages';
import type { EtatJoueur } from '../etat';
import type { ApparenceEntite, CheminTexture, DefinitionArene } from '../donnees/types';
import { apparenceJoueur } from '../profil';
import { lireImage } from '../assets/skinsImportees';
import type { InfoJoueur } from './protocole';

/** Écart (blocs) entre les joueurs au départ. */
const ECART_DEPART = 1.6;

/** Texture de la skin importée d'un joueur de la partie (enregistrée au chargement, voir main.ts). */
export function cheminSkinJoueur(index: number): CheminTexture {
  return `profil/skin-${index}`;
}

/** Skin choisie par le joueur, sa skin importée comprise. */
export function apparenceDistante(info: InfoJoueur): ApparenceEntite {
  const classe = PERSONNAGES[info.personnage];
  if (info.skin === 'importe' && info.skinImporte) {
    return { ...classe, modele: info.brasFins ? 'humanoide_fin' : 'humanoide', skin: cheminSkinJoueur(info.index) };
  }
  const skin = info.skin === 'importe' ? 'classe' : info.skin;
  return apparenceJoueur(classe, { pseudo: info.pseudo, skin, brasFins: info.brasFins, skinImporte: null });
}

/** Place de départ du joueur `index` : autour du départ de l'arène, l'hôte au centre. */
export function positionDepart(arene: DefinitionArene, index: number, sortie: { x: number; z: number }): void {
  const x = arene.depart?.x ?? 0;
  const z = arene.depart?.z ?? 0;
  const angle = (index - 1) * (Math.PI / 2);
  sortie.x = index === 0 ? x : x + Math.cos(angle) * ECART_DEPART;
  sortie.z = index === 0 ? z : z + Math.sin(angle) * ECART_DEPART;
}

/**
 * Ce que dit l'étiquette d'un joueur, codé en un nombre (situation × 1000 +
 * valeur) : le texte n'est reconstruit que quand ce code change.
 */
export function codeEtiquette(j: EtatJoueur, enChoix: boolean, reanimation: number): number {
  const r = REGLES_MULTIJOUEUR.reanimation;
  if (j.mort) {
    if (j.tempsMort >= r.aTerre) return 1000;
    if (reanimation > 0) return 2000 + Math.floor(reanimation * 10) * 10;
    return 3000 + Math.ceil(r.aTerre - j.tempsMort);
  }
  return enChoix ? 4000 : 0;
}

/** Étiquette d'un joueur : son pseudo, et ce qui lui arrive (cartes, à terre, réanimation). */
export function texteEtiquette(pseudo: string, code: number): string {
  const valeur = code % 1000;
  switch (Math.floor(code / 1000)) {
    case 1: return `${pseudo} (mort)`;
    case 2: return `${pseudo} : réanimation ${valeur} %`;
    case 3: return `${pseudo} : à terre (${valeur} s)`;
    case 4: return `${pseudo} (choisit ses cartes)`;
    default: return pseudo;
  }
}

/** Taille des skins envoyées aux autres joueurs (le patron du jeu) : une skin HD y est réduite. */
const TAILLE_SKIN_RESEAU = 64;

/**
 * Skin importée à envoyer aux autres joueurs : ramenée au patron 64×64 (pixels
 * nets) pour rester légère, quelle que soit sa taille gardée ; null si illisible.
 */
export async function skinPourReseau(adresse: string): Promise<string | null> {
  const image = await lireImage(adresse);
  if (!image) return null;
  if (image.width === TAILLE_SKIN_RESEAU && image.height === TAILLE_SKIN_RESEAU) return adresse;
  const toile = document.createElement('canvas');
  toile.width = TAILLE_SKIN_RESEAU;
  toile.height = TAILLE_SKIN_RESEAU;
  const ctx = toile.getContext('2d');
  if (!ctx) return null;
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(image, 0, 0, TAILLE_SKIN_RESEAU, TAILLE_SKIN_RESEAU);
  return toile.toDataURL('image/png');
}
