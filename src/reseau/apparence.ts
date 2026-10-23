/** Apparence et place de départ des joueurs d'une partie à plusieurs (mêmes calculs chez l'hôte et les joueurs). */
import { PERSONNAGES } from '../donnees/personnages';
import type { ApparenceEntite, DefinitionArene } from '../donnees/types';
import { apparenceJoueur } from '../profil';
import type { InfoJoueur } from './protocole';

/** Écart (blocs) entre les joueurs au départ. */
const ECART_DEPART = 1.6;

/** Skin choisie par le joueur (une skin importée n'est pas transmise : celle de la classe la remplace). */
export function apparenceDistante(info: InfoJoueur): ApparenceEntite {
  const skin = info.skin === 'importe' ? 'classe' : info.skin;
  return apparenceJoueur(PERSONNAGES[info.personnage], { pseudo: info.pseudo, skin, brasFins: info.brasFins, skinImporte: null });
}

/** Place de départ du joueur `index` : autour du départ de l'arène, l'hôte au centre. */
export function positionDepart(arene: DefinitionArene, index: number, sortie: { x: number; z: number }): void {
  const x = arene.depart?.x ?? 0;
  const z = arene.depart?.z ?? 0;
  const angle = (index - 1) * (Math.PI / 2);
  sortie.x = index === 0 ? x : x + Math.cos(angle) * ECART_DEPART;
  sortie.z = index === 0 ? z : z + Math.sin(angle) * ECART_DEPART;
}
