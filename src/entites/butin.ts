/**
 * Ce que lâche un ennemi vaincu : son XP, son butin garanti, et parfois un
 * objet (soin, émeraude). Un mini-boss lâche aussi des émeraudes et un coffre,
 * un boss des émeraudes et un coffre de l'Ender. Mort au-dessus de la lave,
 * dans un mur ou hors d'atteinte (blaze, boss volants), il lâche tout sur la
 * case accessible la plus proche.
 */
import { IDS_OBJETS, OBJETS } from '../donnees/objets';
import type { DefinitionEnnemi } from '../donnees/types';
import type { Terrain } from '../monde/navigation';
import type { Coffres } from './coffres';
import type { Objets } from './objets';
import type { DepotXp } from './orbes';

export class Butin {
  private readonly orbes: DepotXp;
  private readonly objets: Objets;
  private readonly terrain: Terrain;
  private coffres: Coffres | null = null;
  // Résultat de placer (évite d'allouer un objet)
  private x = 0;
  private z = 0;

  constructor(orbes: DepotXp, objets: Objets, terrain: Terrain) {
    this.orbes = orbes;
    this.objets = objets;
    this.terrain = terrain;
  }

  /** Les coffres des mini-boss et des boss (créés après le butin). */
  brancherCoffres(coffres: Coffres): void {
    this.coffres = coffres;
  }

  lacher(ennemi: DefinitionEnnemi, xMort: number, zMort: number): void {
    this.placer(xMort, zMort);
    const x = this.x;
    const z = this.z;
    this.orbes.deposer(x, z, ennemi.xp);
    // Émeraudes en couronne autour du corps
    const n = ennemi.emeraudes ?? 0;
    for (let k = 0; k < n; k++) {
      const angle = (k / n) * Math.PI * 2;
      this.placer(x + Math.sin(angle) * 1.2, z + Math.cos(angle) * 1.2);
      this.objets.poser('emeraude', this.x, this.z);
    }
    if (ennemi.miniBoss) this.coffres?.poser('coffre', x, z);
    if (ennemi.boss) this.coffres?.poser('coffre_ender', x, z);
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
