/**
 * Version du jeu : numéro et commit injectés à la construction. Pour prévenir
 * le joueur d'une mise à jour sans rien publier du code, le jeu lit un petit
 * fichier public (version.json) qui ne contient que le numéro de la dernière
 * version et l'adresse où la télécharger ; le workflow le met à jour à chaque
 * version publiée.
 */

export const VERSION = __VERSION__;
export const COMMIT = __COMMIT__;
export const DATE_CONSTRUCTION = new Date(__DATE_CONSTRUCTION__);

/** Fichier de version public (dépôt public qui ne contient que ce fichier). */
const URL_FICHIER_VERSION = 'https://raw.githubusercontent.com/TakeosOfficiel/Block-Survivors-version/main/version.json';

/** Dernière version publiée, d'après le fichier public. */
export interface VersionPubliee {
  readonly version: string;
  /** Page où la télécharger. */
  readonly telechargement: string;
}

/** « v0.8.0 (4ee10c8) » */
export function libelleVersion(): string {
  return `v${VERSION} (${COMMIT.slice(0, 7)})`;
}

/**
 * La version publiée si elle est plus récente que celle-ci, sinon null
 * (à jour, hors ligne, fichier absent ou illisible : on ne prévient pas).
 */
export async function miseAJourDisponible(): Promise<VersionPubliee | null> {
  try {
    const reponse = await fetch(URL_FICHIER_VERSION, { cache: 'no-store' });
    if (!reponse.ok) return null;
    const lu = (await reponse.json()) as Partial<VersionPubliee>;
    if (typeof lu.version !== 'string' || typeof lu.telechargement !== 'string') return null;
    if (!/^https:\/\//.test(lu.telechargement)) return null;
    return plusRecente(lu.version, VERSION) ? { version: lu.version, telechargement: lu.telechargement } : null;
  } catch {
    return null;
  }
}

/** « 0.10.0 » est plus récente que « 0.9.2 » (comparaison nombre par nombre). */
function plusRecente(a: string, b: string): boolean {
  const pa = a.split('.').map(Number);
  const pb = b.split('.').map(Number);
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const x = pa[i] ?? 0;
    const y = pb[i] ?? 0;
    if (!Number.isFinite(x) || !Number.isFinite(y)) return false;
    if (x !== y) return x > y;
  }
  return false;
}
