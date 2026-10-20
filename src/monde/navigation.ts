/**
 * Champ de flux vers le joueur : distance en cases depuis sa position
 * (parcours en largeur sur les cases franchissables), recalculée quand il
 * change de case. Chaque ennemi descend ce champ pour contourner les obstacles.
 */
import { BLOQUE_MARCHE, type CarteMonde, type Point } from './carte';

const INATTEIGNABLE = -1;
const VOISINS_X = [1, -1, 0, 0, 1, 1, -1, -1];
const VOISINS_Z = [0, 0, 1, -1, 1, -1, 1, -1];

const centre: Point = { x: 0, z: 0 };

/** Ce que les entités consultent pour se déplacer dans l'arène. */
export interface Terrain {
  readonly carte: CarteMonde;
  readonly flux: ChampDeFlux;
}

export class ChampDeFlux {
  private readonly carte: CarteMonde;
  private readonly distances: Int32Array;
  private readonly file: Int32Array;
  private celluleCible = -2;

  constructor(carte: CarteMonde) {
    this.carte = carte;
    this.distances = new Int32Array(carte.taille * carte.taille).fill(INATTEIGNABLE);
    this.file = new Int32Array(carte.taille * carte.taille);
  }

  /** Recalcule le champ si la cible a changé de case. */
  maj(x: number, z: number): void {
    const cible = this.carte.cellule(x, z);
    if (cible === this.celluleCible || cible < 0) return;
    this.celluleCible = cible;

    const { taille, drapeaux } = this.carte;
    this.distances.fill(INATTEIGNABLE);
    this.distances[cible] = 0;
    let debut = 0;
    let fin = 0;
    this.file[fin++] = cible;
    while (debut < fin) {
      const c = this.file[debut++];
      const ix = c % taille;
      const iz = (c - ix) / taille;
      const d = this.distances[c] + 1;
      for (let v = 0; v < 4; v++) {
        const nx = ix + VOISINS_X[v];
        const nz = iz + VOISINS_Z[v];
        if (nx < 0 || nz < 0 || nx >= taille || nz >= taille) continue;
        const n = nz * taille + nx;
        if (this.distances[n] !== INATTEIGNABLE || (drapeaux[n] & BLOQUE_MARCHE) !== 0) continue;
        this.distances[n] = d;
        this.file[fin++] = n;
      }
    }
  }

  /** Le joueur est-il atteignable à pied depuis (x, z) ? */
  atteignable(x: number, z: number): boolean {
    const c = this.carte.cellule(x, z);
    return c >= 0 && this.distances[c] !== INATTEIGNABLE;
  }

  /**
   * Direction unitaire à suivre depuis (x, z) : vers la case voisine la plus
   * proche du joueur (diagonale seulement si elle ne coupe pas un angle).
   * Renvoie faux si aucune direction n'est connue.
   */
  direction(x: number, z: number, sortie: Point): boolean {
    const c = this.carte.cellule(x, z);
    if (c < 0 || this.distances[c] === INATTEIGNABLE) return false;
    const { taille, drapeaux } = this.carte;
    const ix = c % taille;
    const iz = (c - ix) / taille;
    let meilleure = -1;
    let meilleureDistance = this.distances[c];
    for (let v = 0; v < 8; v++) {
      const nx = ix + VOISINS_X[v];
      const nz = iz + VOISINS_Z[v];
      if (nx < 0 || nz < 0 || nx >= taille || nz >= taille) continue;
      const n = nz * taille + nx;
      const d = this.distances[n];
      if (d === INATTEIGNABLE || d >= meilleureDistance) continue;
      if (v >= 4 && ((drapeaux[iz * taille + nx] & BLOQUE_MARCHE) !== 0 || (drapeaux[nz * taille + ix] & BLOQUE_MARCHE) !== 0)) continue;
      meilleure = n;
      meilleureDistance = d;
    }
    if (meilleure < 0) return false;
    this.carte.centre(meilleure, centre);
    const dx = centre.x - x;
    const dz = centre.z - z;
    const longueur = Math.sqrt(dx * dx + dz * dz) || 1;
    sortie.x = dx / longueur;
    sortie.z = dz / longueur;
    return true;
  }
}
