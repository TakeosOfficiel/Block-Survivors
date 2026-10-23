/**
 * Règles et réglages du multijoueur (src/reseau) : difficulté selon le nombre
 * de joueurs, cadence du réseau, prédiction et interpolation.
 */
export const REGLES_MULTIJOUEUR = {
  /** Joueurs dans une partie, l'hôte compris. */
  joueursMax: 4,
  /** Par joueur en plus de l'hôte : apparitions plus fréquentes, ennemis et boss plus robustes. */
  parJoueur: {
    apparitions: 0.6,
    pvEnnemis: 0.25,
    pvBoss: 0.7,
  },
  reseau: {
    /** Instantanés envoyés par seconde à chaque joueur. */
    instantanesParSeconde: 15,
    /** Rayon (blocs) autour d'un joueur dans lequel l'hôte lui envoie ennemis, objets et projectiles. */
    rayonEnvoi: 34,
    /** Rayon (blocs) autour d'un joueur distant considéré comme dans son champ (apparitions). */
    rayonVue: 20,
    /** s sans entrée reçue d'un joueur avant de le considérer immobile. */
    silenceEntrees: 1,
    /** Chiffres de dégâts envoyés au plus par instantané. */
    chiffresMax: 48,
  },
  prediction: {
    /** Part de l'écart avec l'hôte corrigée à chaque instantané (le reste aux suivants). */
    correction: 0.35,
    /** Écart (blocs) au-delà duquel le joueur est replacé d'un coup. */
    teleportation: 3,
  },
} as const;

/** Multiplicateur d'un effet de difficulté pour `n` joueurs. */
export function facteurEquipe(parJoueur: number, n: number): number {
  return 1 + parJoueur * Math.max(0, n - 1);
}
