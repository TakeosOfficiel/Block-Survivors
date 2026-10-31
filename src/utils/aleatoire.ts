/** Générateur pseudo-aléatoire déterministe (même graine = même arène). */

/** Graine numérique tirée d'un texte (FNV-1a). */
export function graine(texte: string): number {
  let h = 2166136261;
  for (let i = 0; i < texte.length; i++) {
    h ^= texte.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

/** Fonction qui renvoie un nombre dans [0, 1) à chaque appel (mulberry32). */
export function creerAleatoire(depart: number): () => number {
  let etat = depart >>> 0;
  return () => {
    etat = (etat + 0x6d2b79f5) >>> 0;
    let t = etat;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/**
 * Aléatoire de la simulation (combat, ennemis, butin, cartes) : tout tirage qui change
 * l'état de la partie passe par `alea()`. Graine au hasard pour une partie normale ;
 * le robot d'équilibrage la fixe (`semerSimulation`) pour rejouer une partie à l'identique.
 * Les effets purement visuels (particules, tremblements) gardent `Math.random`.
 */
let tirageSimulation = creerAleatoire((Math.random() * 4294967296) >>> 0);

/** Nombre dans [0, 1) tiré par la simulation. */
export function alea(): number {
  return tirageSimulation();
}

/** Fixe la graine de la simulation (avant de créer la partie). */
export function semerSimulation(depart: number): void {
  tirageSimulation = creerAleatoire(depart);
}
