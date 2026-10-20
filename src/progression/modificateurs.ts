/** Modificateurs globaux du joueur, cumulés depuis ses bonus passifs. */
import type { Modificateurs } from '../donnees/types';

export function creerModificateurs(): Modificateurs {
  return {
    degats: 1, cadence: 1, zone: 1, portee: 1, projectiles: 0, allies: 0,
    vitesse: 1, pvMax: 0, armure: 0, regeneration: 0, collecte: 1, vies: 0,
    critique: 0.05, degatsCritiques: 1.5,  // comme les coups critiques du jeu : ×1,5
    degatsMelee: 0, zoneMelee: 0, degatsProjectiles: 0, vitesseProjectiles: 0, degatsFeu: 0, cadenceFeu: 0,
    degatsAllies: 0, vitesseAllies: 0, pvAllies: 0, dureeAllies: 0, recul: 1,
    degatsBrulure: 0, dureeBrulure: 0, cumulsBrulure: 0, degatsPoison: 0, dureeRalenti: 0, degatsRalentis: 0,
    regenerationPart: 0, soinsObjets: 0,
  };
}

/** Ajoute `modif` à `cible`, champ par champ (un champ absent de `cible` part de 0). */
export function cumulerModificateurs(cible: Partial<Modificateurs>, modif: Partial<Modificateurs>): void {
  for (const cle of Object.keys(modif) as (keyof Modificateurs)[]) {
    cible[cle] = (cible[cle] ?? 0) + (modif[cle] ?? 0);
  }
}
