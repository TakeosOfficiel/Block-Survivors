/**
 * Version du jeu : numéro et commit injectés à la construction, comparés à la
 * dernière version publiée (release « derniere-version » du dépôt) pour
 * prévenir le joueur qu'une mise à jour existe.
 */

export const VERSION = __VERSION__;
export const COMMIT = __COMMIT__;
export const DATE_CONSTRUCTION = new Date(__DATE_CONSTRUCTION__);

const DEPOT = 'TakeosOfficiel/Block-Survivors';
const RELEASE = 'derniere-version';
/** Page où télécharger la dernière version. */
export const URL_TELECHARGEMENT = `https://github.com/${DEPOT}/releases/tag/${RELEASE}`;

/** « v0.4.0 (4e06a10) » */
export function libelleVersion(): string {
  return `v${VERSION} (${COMMIT.slice(0, 7)})`;
}

/**
 * Vrai si la dernière version publiée vient d'un autre commit que celle-ci.
 * Faux si on ne peut pas savoir (hors ligne, API indisponible, build local).
 */
export async function miseAJourDisponible(): Promise<boolean> {
  if (!/^[0-9a-f]{40}$/.test(COMMIT)) return false;
  try {
    const reponse = await fetch(`https://api.github.com/repos/${DEPOT}/releases/tags/${RELEASE}`, {
      headers: { Accept: 'application/vnd.github+json' },
    });
    if (!reponse.ok) return false;
    const release = (await reponse.json()) as { target_commitish?: string };
    const publie = release.target_commitish ?? '';
    return /^[0-9a-f]{40}$/.test(publie) && publie !== COMMIT;
  } catch {
    return false;
  }
}
