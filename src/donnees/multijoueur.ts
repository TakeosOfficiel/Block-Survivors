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
  /** Joueur tombé à 0 PV (après le totem) : à terre, un allié peut le relever. */
  reanimation: {
    /** s à terre avant d'être mort pour de bon. */
    aTerre: 30,
    /** Distance (blocs) et temps (s) qu'un allié doit rester près de lui. */
    rayon: 1.6,
    duree: 3,
    /** Part des PV max rendus, puis s d'invincibilité. */
    pv: 0.3,
    protection: 2,
  },
  /** Joueur qui choisit ses cartes : la partie continue pour les autres, lui est protégé. */
  choixCartes: {
    /** s de protection renouvelées à chaque pas tant qu'il choisit. */
    protection: 0.25,
    /** s entre deux gerbes de particules autour de lui (son bouclier). */
    periodeBouclier: 0.35,
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
    /** s entre deux envois du bilan de chaque joueur (son compteur de dégâts). */
    periodeStats: 1,
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
