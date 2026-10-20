/**
 * Options du joueur, conservées dans le navigateur (localStorage). Le stockage
 * peut être refusé (navigation privée, file:// sous certains navigateurs) :
 * on retombe alors sur les valeurs par défaut, sans erreur.
 */

export interface Options {
  /** Volume des sons, de 0 à 1. */
  volume: number;
  afficherFps: boolean;
  secousses: boolean;
}

const CLE = 'block-survivors.options';

const PAR_DEFAUT: Readonly<Options> = { volume: 0.7, afficherFps: false, secousses: true };

export function chargerOptions(): Options {
  const options = { ...PAR_DEFAUT };
  const lu = lireStockage<Partial<Options>>(CLE);
  if (lu) {
    if (typeof lu.volume === 'number' && Number.isFinite(lu.volume)) options.volume = Math.min(1, Math.max(0, lu.volume));
    if (typeof lu.afficherFps === 'boolean') options.afficherFps = lu.afficherFps;
    if (typeof lu.secousses === 'boolean') options.secousses = lu.secousses;
  }
  return options;
}

export function enregistrerOptions(options: Options): void {
  ecrireStockage(CLE, options);
}

export function lireStockage<T>(cle: string): T | null {
  try {
    const brut = window.localStorage.getItem(cle);
    return brut ? (JSON.parse(brut) as T) : null;
  } catch {
    return null;
  }
}

export function ecrireStockage(cle: string, valeur: unknown): void {
  try {
    window.localStorage.setItem(cle, JSON.stringify(valeur));
  } catch {
    // Stockage indisponible : l'option ne vaut que pour cette session
  }
}
