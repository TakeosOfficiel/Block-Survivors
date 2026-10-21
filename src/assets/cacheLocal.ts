/**
 * Cache local des fichiers téléchargés (textures, modèles de blocs, police) :
 * gardés sur l'ordinateur dans IndexedDB, rangés par adresse. L'adresse
 * contient la version de Minecraft, donc un changement de version les
 * retélécharge tout seul. Après le premier lancement, plus rien ne passe par
 * internet : le chargement est plus rapide, et le jeu marche hors ligne.
 *
 * Toute erreur (stockage indisponible, navigation privée…) revient à « pas en
 * cache » : le fichier est alors simplement téléchargé comme avant.
 */
const BASE = 'block-survivors-cache';
const MAGASIN = 'fichiers';

let base: Promise<IDBDatabase | null> | null = null;

function ouvrir(): Promise<IDBDatabase | null> {
  base ??= new Promise((resoudre) => {
    try {
      const demande = indexedDB.open(BASE, 1);
      demande.onupgradeneeded = () => demande.result.createObjectStore(MAGASIN);
      demande.onsuccess = () => resoudre(demande.result);
      demande.onerror = () => resoudre(null);
      demande.onblocked = () => resoudre(null);
    } catch {
      resoudre(null);
    }
  });
  return base;
}

/** Contenu gardé pour cette adresse, ou null. */
export async function lireCache(adresse: string): Promise<Blob | null> {
  const db = await ouvrir();
  if (!db) return null;
  return new Promise((resoudre) => {
    try {
      const demande = db.transaction(MAGASIN, 'readonly').objectStore(MAGASIN).get(adresse);
      demande.onsuccess = () => resoudre(demande.result instanceof Blob ? demande.result : null);
      demande.onerror = () => resoudre(null);
    } catch {
      resoudre(null);
    }
  });
}

/** Garde ce contenu pour la prochaine fois (sans attendre ni échouer). */
export function ecrireCache(adresse: string, contenu: Blob): void {
  void ouvrir().then((db) => {
    try {
      db?.transaction(MAGASIN, 'readwrite').objectStore(MAGASIN).put(contenu, adresse);
    } catch {
      // Stockage plein ou indisponible : le fichier sera retéléchargé
    }
  });
}

/**
 * Le fichier à cette adresse : depuis le cache, sinon téléchargé (puis gardé).
 * Rejette si le téléchargement échoue ou dépasse `delaiMs`.
 */
export async function obtenirFichier(adresse: string, delaiMs: number): Promise<Blob> {
  const garde = await lireCache(adresse);
  if (garde) return garde;
  const abandon = new AbortController();
  const minuterie = window.setTimeout(() => abandon.abort(), delaiMs);
  try {
    const reponse = await fetch(adresse, { signal: abandon.signal });
    if (!reponse.ok) throw new Error(`${reponse.status} : ${adresse}`);
    const contenu = await reponse.blob();
    ecrireCache(adresse, contenu);
    return contenu;
  } finally {
    window.clearTimeout(minuterie);
  }
}
