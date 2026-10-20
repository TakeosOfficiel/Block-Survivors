/**
 * Ce que lâche un ennemi vaincu : son XP, son butin garanti, et parfois un
 * objet de soin. Mort au-dessus de la lave, dans un mur ou hors d'atteinte
 * (blaze, boss volants), il lâche tout sur la case accessible la plus proche.
 */
import { IDS_OBJETS, OBJETS } from '../donnees/objets';
import type { DefinitionEnnemi } from '../donnees/types';
import type { Terrain } from '../monde/navigation';
import type { Objets } from './objets';
import type { DepotXp } from './orbes';

export class Butin {
  private readonly orbes: DepotXp;
  private readonly objets: Objets;
  private readonly terrain: Terrain;
  // Résultat de placer (évite d'allouer un objet)
  private x = 0;
  private z = 0;

  constructor(orbes: DepotXp, objets: Objets, terrain: Terrain) {
    this.orbes = orbes;
    this.objets = objets;
    this.terrain = terrain;
  }

  lacher(ennemi: DefinitionEnnemi, xMort: number, zMort: number): void {
    this.placer(xMort, zMort);
    const x = this.x;
    const z = this.z;
    this.orbes.deposer(x, z, ennemi.xp);
    if (ennemi.butin) {
      this.objets.poser(ennemi.butin, x, z);
      return;
    }
    // Un seul tirage : chaque objet occupe sa part de la probabilité
    let tirage = Math.random();
    for (const id of IDS_OBJETS) {
      tirage -= OBJETS[id].chance;
      if (tirage < 0) {
        this.objets.poser(id, x, z);
        return;
      }
    }
  }

  /** Case où le joueur peut marcher sans se blesser et qu'il peut atteindre : là, ou la plus proche autour. */
  private placer(x: number, z: number): void {
    this.x = x;
    this.z = z;
    if (this.accessible(x, z)) return;
    const cx = Math.floor(x) + 0.5;
    const cz = Math.floor(z) + 0.5;
    let meilleure = Infinity;
    for (let r = 1; r <= RAYON_RECHERCHE && meilleure === Infinity; r++) {
      for (let dx = -r; dx <= r; dx++) {
        for (let dz = -r; dz <= r; dz++) {
          if (Math.max(Math.abs(dx), Math.abs(dz)) !== r) continue;  // seulement l'anneau
          const px = cx + dx;
          const pz = cz + dz;
          if (!this.accessible(px, pz)) continue;
          const d = (px - x) ** 2 + (pz - z) ** 2;
          if (d < meilleure) {
            meilleure = d;
            this.x = px;
            this.z = pz;
          }
        }
      }
    }
  }

  private accessible(x: number, z: number): boolean {
    const { carte, flux } = this.terrain;
    return !carte.bloqueMarche(x, z) && carte.degatsSol(x, z) === 0 && flux.atteignable(x, z);
  }
}

/** Distance maximale (cases) où chercher une case accessible. */
const RAYON_RECHERCHE = 8;
