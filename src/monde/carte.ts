/**
 * Carte de l'arène, une case par bloc au sol : ce qui bloque la marche, ce
 * qui arrête les projectiles, et les effets de sol (ralentissement, dégâts).
 * Fournit les collisions cercle / blocs et la ligne de vue.
 */

/** Case infranchissable à pied (bloc plein ou liquide à hauteur de corps). */
export const BLOQUE_MARCHE = 1;
/** Case qui arrête les projectiles (bloc plein, même un muret d'un bloc). */
export const BLOQUE_TIR = 2;

export interface Point {
  x: number;
  z: number;
}

export class CarteMonde {
  /** Cases d'ouest en est, et du nord au sud. */
  readonly largeur: number;
  readonly profondeur: number;
  /** Demi-côtés : l'arène va de −demiX à demiX et de −demiZ à demiZ. */
  readonly demiX: number;
  readonly demiZ: number;
  readonly drapeaux: Uint8Array;
  readonly ralentissement: Float32Array;
  /** Même chose sans les toiles (pour les araignées). */
  readonly ralentissementHorsToiles: Float32Array;
  readonly degats: Float32Array;

  constructor(largeur: number, profondeur: number) {
    this.largeur = largeur;
    this.profondeur = profondeur;
    this.demiX = largeur / 2;
    this.demiZ = profondeur / 2;
    const cases = largeur * profondeur;
    this.drapeaux = new Uint8Array(cases);
    this.ralentissement = new Float32Array(cases).fill(1);
    this.ralentissementHorsToiles = new Float32Array(cases).fill(1);
    this.degats = new Float32Array(cases);
  }

  /** Index de la case contenant (x, z), ou −1 hors de l'arène. */
  cellule(x: number, z: number): number {
    const ix = Math.floor(x + this.demiX);
    const iz = Math.floor(z + this.demiZ);
    if (ix < 0 || iz < 0 || ix >= this.largeur || iz >= this.profondeur) return -1;
    return iz * this.largeur + ix;
  }

  /** Centre de la case `index`. */
  centre(index: number, sortie: Point): Point {
    sortie.x = (index % this.largeur) - this.demiX + 0.5;
    sortie.z = Math.floor(index / this.largeur) - this.demiZ + 0.5;
    return sortie;
  }

  bloqueMarche(x: number, z: number): boolean {
    const c = this.cellule(x, z);
    return c < 0 || (this.drapeaux[c] & BLOQUE_MARCHE) !== 0;
  }

  bloqueTir(x: number, z: number): boolean {
    const c = this.cellule(x, z);
    return c < 0 || (this.drapeaux[c] & BLOQUE_TIR) !== 0;
  }

  /** Multiplicateur de vitesse du sol (1 = normal). */
  /** Multiplicateur de vitesse sur cette case ; `traverseToiles` : sans tenir compte des toiles. */
  vitesseSol(x: number, z: number, traverseToiles = false): number {
    const c = this.cellule(x, z);
    if (c < 0) return 1;
    return traverseToiles ? this.ralentissementHorsToiles[c] : this.ralentissement[c];
  }

  degatsSol(x: number, z: number): number {
    const c = this.cellule(x, z);
    return c < 0 ? 0 : this.degats[c];
  }

  /**
   * Repousse un cercle hors des cases infranchissables voisines et le garde
   * dans l'arène ; il glisse le long des obstacles. Écrit dans `sortie`.
   */
  resoudreCollision(x: number, z: number, rayon: number, sortie: Point): Point {
    const limiteX = this.demiX - rayon;
    const limiteZ = this.demiZ - rayon;
    let px = x < -limiteX ? -limiteX : x > limiteX ? limiteX : x;
    let pz = z < -limiteZ ? -limiteZ : z > limiteZ ? limiteZ : z;

    const ix0 = Math.max(0, Math.floor(px - rayon + this.demiX));
    const ix1 = Math.min(this.largeur - 1, Math.floor(px + rayon + this.demiX));
    const iz0 = Math.max(0, Math.floor(pz - rayon + this.demiZ));
    const iz1 = Math.min(this.profondeur - 1, Math.floor(pz + rayon + this.demiZ));
    for (let iz = iz0; iz <= iz1; iz++) {
      for (let ix = ix0; ix <= ix1; ix++) {
        if ((this.drapeaux[iz * this.largeur + ix] & BLOQUE_MARCHE) === 0) continue;
        // Point de la case le plus proche du centre du cercle
        const x0 = ix - this.demiX;
        const z0 = iz - this.demiZ;
        const qx = px < x0 ? x0 : px > x0 + 1 ? x0 + 1 : px;
        const qz = pz < z0 ? z0 : pz > z0 + 1 ? z0 + 1 : pz;
        const dx = px - qx;
        const dz = pz - qz;
        const d2 = dx * dx + dz * dz;
        if (d2 >= rayon * rayon) continue;
        if (d2 > 1e-10) {
          const d = Math.sqrt(d2);
          px += (dx / d) * (rayon - d);
          pz += (dz / d) * (rayon - d);
        } else {
          // Centre dans la case : sortie par le côté le plus proche
          const gauche = px - x0;
          const droite = x0 + 1 - px;
          const haut = pz - z0;
          const bas = z0 + 1 - pz;
          const m = Math.min(gauche, droite, haut, bas);
          if (m === gauche) px = x0 - rayon;
          else if (m === droite) px = x0 + 1 + rayon;
          else if (m === haut) pz = z0 - rayon;
          else pz = z0 + 1 + rayon;
        }
      }
    }
    sortie.x = px;
    sortie.z = pz;
    return sortie;
  }

  /**
   * Vrai si un cercle de rayon `rayon` en (x, z) touche une case qui arrête
   * les tirs (hors de l'arène : faux, la sortie est gérée à part).
   */
  toucheTir(x: number, z: number, rayon: number): boolean {
    const ix0 = Math.max(0, Math.floor(x - rayon + this.demiX));
    const ix1 = Math.min(this.largeur - 1, Math.floor(x + rayon + this.demiX));
    const iz0 = Math.max(0, Math.floor(z - rayon + this.demiZ));
    const iz1 = Math.min(this.profondeur - 1, Math.floor(z + rayon + this.demiZ));
    for (let iz = iz0; iz <= iz1; iz++) {
      for (let ix = ix0; ix <= ix1; ix++) {
        if ((this.drapeaux[iz * this.largeur + ix] & BLOQUE_TIR) === 0) continue;
        const x0 = ix - this.demiX;
        const z0 = iz - this.demiZ;
        const dx = x - (x < x0 ? x0 : x > x0 + 1 ? x0 + 1 : x);
        const dz = z - (z < z0 ? z0 : z > z0 + 1 ? z0 + 1 : z);
        if (dx * dx + dz * dz < rayon * rayon) return true;
      }
    }
    return false;
  }

  /** Vrai si un projectile tiré de (x0, z0) vers (x1, z1) n'est arrêté par aucun bloc. */
  ligneDeTir(x0: number, z0: number, x1: number, z1: number, rayon: number): boolean {
    const dx = x1 - x0;
    const dz = z1 - z0;
    const pas = Math.ceil(Math.sqrt(dx * dx + dz * dz) * 4);
    for (let i = 1; i < pas; i++) {
      const t = i / pas;
      if (this.toucheTir(x0 + dx * t, z0 + dz * t, rayon)) return false;
    }
    return true;
  }

  /** Vrai si aucune case infranchissable ne coupe le segment (pas de 0,5 bloc). */
  ligneDeVue(x0: number, z0: number, x1: number, z1: number): boolean {
    const dx = x1 - x0;
    const dz = z1 - z0;
    const pas = Math.ceil(Math.sqrt(dx * dx + dz * dz) * 2);
    for (let i = 1; i < pas; i++) {
      const t = i / pas;
      if (this.bloqueMarche(x0 + dx * t, z0 + dz * t)) return false;
    }
    return true;
  }
}
