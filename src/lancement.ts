/**
 * Choix d'une partie (personnage et arène) et navigation entre le menu et le
 * jeu. Chaque partie démarre sur une page neuve : « Rejouer » et « Menu
 * principal » rechargent la page, ce qui remet tout à zéro proprement.
 *
 * Adresse : ?personnage=archer&arene=mine&jouer=1 (jouer=1 saute le menu).
 */
import { ARENE_PAR_DEFAUT, ARENES } from './donnees/arenes';
import { IDS_PERSONNAGES, PERSONNAGE_PAR_DEFAUT, type IdPersonnage } from './donnees/personnages';
import type { DefinitionArene } from './donnees/types';
import { ecrireStockage, lireStockage } from './options';
import { cleProfil } from './profils';

export interface ChoixPartie {
  readonly personnage: IdPersonnage;
  readonly arene: DefinitionArene;
}

/** Choix demandé dans l'adresse, sinon le dernier joué, sinon celui par défaut. */
export function choixDemande(): ChoixPartie {
  const adresse = new URLSearchParams(window.location.search);
  const memorise = lireStockage<{ personnage?: string; arene?: string }>(cleProfil('choix'));
  const idPersonnage = adresse.get('personnage') ?? memorise?.personnage;
  const idArene = adresse.get('arene') ?? memorise?.arene ?? ARENE_PAR_DEFAUT;
  return {
    personnage: IDS_PERSONNAGES.find((id) => id === idPersonnage) ?? PERSONNAGE_PAR_DEFAUT,
    arene: ARENES.find((a) => a.id === idArene) ?? ARENES[0],
  };
}

/** Vrai si l'adresse demande de lancer la partie sans passer par le menu. */
export function partieDemandee(): boolean {
  return new URLSearchParams(window.location.search).get('jouer') === '1';
}

/** Retient le choix pour le prochain passage au menu. */
export function memoriserChoix(choix: ChoixPartie): void {
  ecrireStockage(cleProfil('choix'), { personnage: choix.personnage, arene: choix.arene.id });
}

/** Relance une partie avec ce choix (Rejouer, Recommencer, debug). */
export function lancerPartie(choix: ChoixPartie): void {
  naviguer(choix, true);
}

/** Retourne au menu principal, choix présélectionné. */
export function retournerAuMenu(choix: ChoixPartie): void {
  naviguer(choix, false);
}

function naviguer(choix: ChoixPartie, jouer: boolean): void {
  memoriserChoix(choix);
  const adresse = new URL(window.location.href);
  adresse.searchParams.set('personnage', choix.personnage);
  adresse.searchParams.set('arene', choix.arene.id);
  if (jouer) adresse.searchParams.set('jouer', '1');
  else adresse.searchParams.delete('jouer');
  window.location.href = adresse.href;
}
