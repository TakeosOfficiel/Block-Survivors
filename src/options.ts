/**
 * Options du joueur, conservées dans le navigateur (localStorage). Le stockage
 * peut être refusé (navigation privée, file:// sous certains navigateurs) :
 * on retombe alors sur les valeurs par défaut, sans erreur.
 */
import type { CategorieSon } from './donnees/types';

export interface Options {
  /** Volume général, de 0 à 1. */
  volume: number;
  /** Volume de chaque famille de sons (multiplié par le général). */
  volumes: Record<CategorieSon, number>;
  afficherFps: boolean;
  secousses: boolean;
  /** Ombres du soleil et végétation au vent. */
  shaders: NiveauShaders;
  /** Chiffres des dégâts au-dessus des ennemis. */
  chiffresDegats: boolean;
  /** Panneau des dégâts par arme en jeu (touche Tab). */
  compteurDegats: boolean;
  /** Vue de la caméra (touche F5). */
  vue: ModeVue;
}

export type ModeVue = 'dessus' | 'troisieme' | 'premiere';
export const MODES_VUE: readonly ModeVue[] = ['dessus', 'troisieme', 'premiere'];

export type NiveauShaders = 'aucun' | 'legers' | 'complets';
export const NIVEAUX_SHADERS: readonly NiveauShaders[] = ['aucun', 'legers', 'complets'];

const CLE = 'block-survivors.options';

export const CATEGORIES_SONS: readonly CategorieSon[] = ['joueur', 'armes', 'creatures', 'interface'];

const PAR_DEFAUT: Readonly<Options> = {
  volume: 0.7,
  volumes: { joueur: 1, armes: 1, creatures: 1, interface: 1 },
  afficherFps: false,
  secousses: true,
  shaders: 'legers',
  chiffresDegats: true,
  compteurDegats: true,
  vue: 'dessus',
};

/**
 * Version du format enregistré. En 2, les shaders (devenus ombres + vent, sans
 * coût de changement) repartent activés pour tous, même désactivés avant.
 */
const VERSION = 2;

export function chargerOptions(): Options {
  const options = { ...PAR_DEFAUT, volumes: { ...PAR_DEFAUT.volumes } };
  const lu = lireStockage<Partial<Options> & { version?: number }>(CLE);
  if (lu) {
    if (typeof lu.volume === 'number' && Number.isFinite(lu.volume)) options.volume = Math.min(1, Math.max(0, lu.volume));
    for (const categorie of CATEGORIES_SONS) {
      const v = lu.volumes?.[categorie];
      if (typeof v === 'number' && Number.isFinite(v)) options.volumes[categorie] = Math.min(1, Math.max(0, v));
    }
    if (typeof lu.afficherFps === 'boolean') options.afficherFps = lu.afficherFps;
    if (typeof lu.secousses === 'boolean') options.secousses = lu.secousses;
    if (lu.version === VERSION && NIVEAUX_SHADERS.includes(lu.shaders as NiveauShaders)) options.shaders = lu.shaders as NiveauShaders;
    if (typeof lu.chiffresDegats === 'boolean') options.chiffresDegats = lu.chiffresDegats;
    if (typeof lu.compteurDegats === 'boolean') options.compteurDegats = lu.compteurDegats;
    if (MODES_VUE.includes(lu.vue as ModeVue)) options.vue = lu.vue as ModeVue;
  }
  return options;
}

export function enregistrerOptions(options: Options): void {
  ecrireStockage(CLE, { ...options, version: VERSION });
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
