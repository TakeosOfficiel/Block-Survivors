/** Fonctions mathématiques pures, sans allocation. */

/** Facteur d'approche exponentielle indépendant du framerate : 0 = immobile, 1 = cible atteinte. */
export function facteurLissage(raideur: number, dt: number): number {
  return 1 - Math.exp(-raideur * dt);
}

export function borner(valeur: number, min: number, max: number): number {
  return valeur < min ? min : valeur > max ? max : valeur;
}

/** Écart angulaire signé le plus court pour aller de `depuis` à `vers` (rad). */
export function ecartAngle(depuis: number, vers: number): number {
  return Math.atan2(Math.sin(vers - depuis), Math.cos(vers - depuis));
}

/** Pseudo-aléatoire déterministe dans [0, 1) à partir d'une position entière. */
export function hachage3(x: number, y: number, z: number): number {
  let h = Math.imul(x | 0, 73856093) ^ Math.imul(y | 0, 19349663) ^ Math.imul(z | 0, 83492791);
  h = Math.imul(h ^ (h >>> 13), 0x5bd1e995);
  h ^= h >>> 15;
  return (h >>> 0) / 4294967296;
}

/** Élément d'une liste pondérée, choisi par un tirage `alea` dans [0, 1). */
export function choisirPondere<T extends { readonly poids: number }>(choix: readonly T[], alea: number): T {
  let total = 0;
  for (const c of choix) total += c.poids;
  let seuil = alea * total;
  for (const c of choix) {
    seuil -= c.poids;
    if (seuil < 0) return c;
  }
  const dernier = choix[choix.length - 1];
  if (dernier === undefined) throw new Error('choisirPondere : liste vide');
  return dernier;
}
