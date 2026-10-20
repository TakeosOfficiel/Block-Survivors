/** Fichiers JSON vanilla (police, états et modèles de blocs), cherchés dans SOURCES_FICHIERS. */
import { SOURCES_FICHIERS } from '../config';

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
      const reponse = await fetch(source(chemin));
      if (reponse.ok) return await reponse.json();
    } catch {
      // Source suivante
    }
  }
  return null;
}
