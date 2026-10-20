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
