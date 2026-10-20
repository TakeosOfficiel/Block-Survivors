/**
 * Génération déterministe d'une arène à partir de sa définition : sol et
 * taches, murs, structures placées ou dispersées, puis carte de collision.
 * Même arène = même monde à chaque partie (graine tirée de son identifiant).
 */
import { BLOCS, bloqueLePassage, type IdBloc } from '../donnees/blocs';
import { STRUCTURES } from '../donnees/structures';
import type { DefinitionArene, DefinitionStructure, Rectangle } from '../donnees/types';
import { creerAleatoire, graine } from '../utils/aleatoire';
import { choisirPondere, hachage3 } from '../utils/maths';
import { BLOQUE_MARCHE, BLOQUE_TIR, CarteMonde } from './carte';

/** Couches de décor possibles au-dessus du sol. */
const HAUTEUR_MAX = 8;

export interface MondeGenere {
  /** Centres des blocs à afficher, par type : idBloc → [x,y,z, x,y,z, …]. */
  readonly blocs: Map<IdBloc, number[]>;
  readonly carte: CarteMonde;
}

export function genererMonde(definition: DefinitionArene): MondeGenere {
  return new Generateur(definition).generer();
}

class Generateur {
  private readonly def: DefinitionArene;
  private readonly taille: number;
  private readonly demi: number;
  private readonly aleatoire: () => number;
  /** Bloc de sol de chaque case. */
  private readonly sol: IdBloc[];
  /** Blocs de décor : clé = (couche × taille + iz) × taille + ix. */
  private readonly volume = new Map<number, IdBloc>();
  /** Cases dont la colonne contient déjà du décor. */
  private readonly occupe: Uint8Array;

  constructor(definition: DefinitionArene) {
    this.def = definition;
    this.taille = definition.taille;
    this.demi = definition.taille / 2;
    this.aleatoire = creerAleatoire(graine(definition.id));
    this.sol = new Array<IdBloc>(this.taille * this.taille);
    this.occupe = new Uint8Array(this.taille * this.taille);
  }

  generer(): MondeGenere {
    this.poserSol();
    for (const tache of this.def.taches) {
      if (tache.type === 'rectangle') this.remplirRectangle(tache.bloc, tache.rect);
      else this.poserTaches(tache.bloc, tache.nombre, tache.rayon, tache.zone);
    }
    for (const element of this.def.decor) {
      const structure = STRUCTURES[element.structure];
      if (element.type === 'structure') {
        this.poserStructure(structure, Math.floor(element.x + this.demi), Math.floor(element.z + this.demi));
      } else {
        this.disperser(structure, element.nombre, element.espacement, element.zone);
      }
    }
    return { blocs: this.listerBlocs(), carte: this.construireCarte() };
  }

  // --- Sol ---------------------------------------------------------------

  private poserSol(): void {
    for (let iz = 0; iz < this.taille; iz++) {
      for (let ix = 0; ix < this.taille; ix++) {
        this.sol[iz * this.taille + ix] = choisirPondere(this.def.sol, hachage3(ix, 0, iz)).bloc;
      }
    }
  }

  private remplirRectangle(bloc: IdBloc, [x0, z0, x1, z1]: Rectangle): void {
    for (let iz = 0; iz < this.taille; iz++) {
      for (let ix = 0; ix < this.taille; ix++) {
        const x = ix - this.demi + 0.5;
        const z = iz - this.demi + 0.5;
        if (x >= x0 && x < x1 && z >= z0 && z < z1) this.sol[iz * this.taille + ix] = bloc;
      }
    }
  }

  /** Taches rondes au hasard dans la zone, jamais sur la zone de départ. */
  private poserTaches(bloc: IdBloc, nombre: number, [rMin, rMax]: readonly [number, number], zone?: Rectangle): void {
    for (let n = 0; n < nombre; n++) {
      const rayon = rMin + this.aleatoire() * (rMax - rMin);
      for (let essai = 0; essai < 30; essai++) {
        const [cx, cz] = this.pointDans(zone);
        if (Math.hypot(cx, cz) < this.def.zoneDepart + rayon + 1) continue;
        for (let iz = 0; iz < this.taille; iz++) {
          for (let ix = 0; ix < this.taille; ix++) {
            if (Math.hypot(ix - this.demi + 0.5 - cx, iz - this.demi + 0.5 - cz) <= rayon) {
              this.sol[iz * this.taille + ix] = bloc;
            }
          }
        }
        break;
      }
    }
  }

  // --- Décor -------------------------------------------------------------

  /** Pose une structure dont le coin nord-ouest est la case (ix0, iz0). */
  private poserStructure(structure: DefinitionStructure, ix0: number, iz0: number): void {
    structure.couches.forEach((lignes, couche) => {
      if (couche >= HAUTEUR_MAX) return;
      lignes.forEach((ligne, r) => {
        for (let c = 0; c < ligne.length; c++) {
          const bloc = structure.legende[ligne[c]];
          const ix = ix0 + c;
          const iz = iz0 + r;
          if (!bloc || ix < 0 || iz < 0 || ix >= this.taille || iz >= this.taille) continue;
          this.volume.set((couche * this.taille + iz) * this.taille + ix, bloc);
          this.occupe[iz * this.taille + ix] = 1;
        }
      });
    });
  }

  /** Disperse des structures au hasard là où il y a de la place. */
  private disperser(structure: DefinitionStructure, nombre: number, espacement: number, zone?: Rectangle): void {
    const largeur = Math.max(...structure.couches.flatMap((lignes) => lignes.map((l) => l.length)));
    const profondeur = Math.max(...structure.couches.map((lignes) => lignes.length));
    let poses = 0;
    for (let essai = 0; essai < nombre * 40 && poses < nombre; essai++) {
      const [x, z] = this.pointDans(zone);
      const ix0 = Math.floor(x + this.demi - largeur / 2);
      const iz0 = Math.floor(z + this.demi - profondeur / 2);
      if (!this.estLibre(ix0 - espacement, iz0 - espacement, largeur + 2 * espacement, profondeur + 2 * espacement)) continue;
      this.poserStructure(structure, ix0, iz0);
      poses++;
    }
  }

  /** Rectangle de cases entièrement dans l'arène, sans décor, sans liquide, hors zone de départ. */
  private estLibre(ix0: number, iz0: number, largeur: number, profondeur: number): boolean {
    for (let iz = iz0; iz < iz0 + profondeur; iz++) {
      for (let ix = ix0; ix < ix0 + largeur; ix++) {
        if (ix < 0 || iz < 0 || ix >= this.taille || iz >= this.taille) return false;
        const c = iz * this.taille + ix;
        if (this.occupe[c] || BLOCS[this.sol[c]].forme === 'liquide') return false;
        if (Math.hypot(ix - this.demi + 0.5, iz - this.demi + 0.5) < this.def.zoneDepart) return false;
      }
    }
    return true;
  }

  /** Point au hasard dans la zone (toute l'arène par défaut, à 1 bloc des murs). */
  private pointDans(zone?: Rectangle): [number, number] {
    const m = this.demi - 1;
    const [x0, z0, x1, z1] = zone ?? [-m, -m, m, m];
    return [
      Math.max(-m, Math.min(m, x0 + this.aleatoire() * (x1 - x0))),
      Math.max(-m, Math.min(m, z0 + this.aleatoire() * (z1 - z0))),
    ];
  }

  // --- Résultats ---------------------------------------------------------

  private listerBlocs(): Map<IdBloc, number[]> {
    const parBloc = new Map<IdBloc, number[]>();
    const ajouter = (bloc: IdBloc, x: number, y: number, z: number) => {
      let positions = parBloc.get(bloc);
      if (!positions) parBloc.set(bloc, (positions = []));
      positions.push(x, y, z);
    };

    // Sol : face supérieure à y = 0
    for (let iz = 0; iz < this.taille; iz++) {
      for (let ix = 0; ix < this.taille; ix++) {
        ajouter(this.sol[iz * this.taille + ix], ix - this.demi + 0.5, -0.5, iz - this.demi + 0.5);
      }
    }

    // Décor
    for (const [cle, bloc] of this.volume) {
      const ix = cle % this.taille;
      const iz = Math.floor(cle / this.taille) % this.taille;
      const couche = Math.floor(cle / (this.taille * this.taille));
      ajouter(bloc, ix - this.demi + 0.5, couche + 0.5, iz - this.demi + 0.5);
    }

    // Murs : anneau de blocs juste autour de la zone jouable (coins compris)
    const demi = this.demi;
    const mur = (x: number, y: number, z: number) =>
      ajouter(choisirPondere(this.def.murs, hachage3(Math.floor(x), Math.floor(y), Math.floor(z))).bloc, x, y, z);
    for (let h = 0; h < this.def.hauteurMur; h++) {
      const y = h + 0.5;
      for (let i = -1; i <= this.taille; i++) {
        const c = -demi + i + 0.5;
        mur(c, y, -demi - 0.5);
        mur(c, y, demi + 0.5);
        if (i >= 0 && i < this.taille) {
          mur(-demi - 0.5, y, c);
          mur(demi + 0.5, y, c);
        }
      }
    }
    return parBloc;
  }

  /**
   * Couches 0 et 1 (hauteur du corps) : blocs pleins et liquides bloquent la
   * marche ; la couche 1 (hauteur de tir) arrête les projectiles. Les sols et
   * les croix (toiles) donnent leurs effets.
   */
  private construireCarte(): CarteMonde {
    const carte = new CarteMonde(this.taille);
    for (let c = 0; c < this.taille * this.taille; c++) {
      const sol = BLOCS[this.sol[c]];
      if (sol.forme === 'liquide') carte.drapeaux[c] |= BLOQUE_MARCHE;
      carte.ralentissement[c] = sol.ralentit ?? 1;
      carte.ralentissementHorsToiles[c] = carte.ralentissement[c];
      carte.degats[c] = sol.degats ?? 0;

      for (let couche = 0; couche <= 1; couche++) {
        const id = this.volume.get(couche * this.taille * this.taille + c);
        if (!id) continue;
        const bloc = BLOCS[id];
        if (bloqueLePassage(bloc)) {
          carte.drapeaux[c] |= BLOQUE_MARCHE;
          if (couche === 1 && bloc.forme !== 'liquide') carte.drapeaux[c] |= BLOQUE_TIR;
        } else if (bloc.ralentit !== undefined) {
          carte.ralentissement[c] = Math.min(carte.ralentissement[c], bloc.ralentit);
          if (!bloc.toile) carte.ralentissementHorsToiles[c] = Math.min(carte.ralentissementHorsToiles[c], bloc.ralentit);
        }
      }
    }
    return carte;
  }
}
