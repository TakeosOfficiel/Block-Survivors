/**
 * Fichiers JSON vanilla (police, états et modèles de blocs), cherchés dans
 * SOURCES_FICHIERS et gardés dans le cache local après le premier téléchargement.
 */
import { CONFIG, SOURCES_FICHIERS } from '../config';
import { obtenirFichier } from './cacheLocal';

const cache = new Map<string, Promise<unknown>>();

/** Fichier relatif à assets/minecraft/ (ex. « blockstates/stone.json ») ; null s'il n'existe nulle part. */
export function chargerJsonVanilla(chemin: string): Promise<unknown> {
  let promesse = cache.get(chemin);
  if (!promesse) {
    promesse = chercher(chemin);
    cache.set(chemin, promesse);
  }
  return promesse;
}

async function chercher(chemin: string): Promise<unknown> {
  for (const source of SOURCES_FICHIERS) {
    try {
      return JSON.parse(await (await obtenirFichier(source(chemin), CONFIG.textures.delaiMaxMs)).text());
    } catch {
      // Source suivante
    }
  }
  return null;
}
