/** Ce que lâche un ennemi vaincu : son XP, son butin garanti, et parfois un objet de soin. */
import { IDS_OBJETS, OBJETS } from '../donnees/objets';
import type { DefinitionEnnemi } from '../donnees/types';
import type { Objets } from './objets';
import type { DepotXp } from './orbes';

export class Butin {
  private readonly orbes: DepotXp;
  private readonly objets: Objets;

  constructor(orbes: DepotXp, objets: Objets) {
    this.orbes = orbes;
    this.objets = objets;
  }

  lacher(ennemi: DefinitionEnnemi, x: number, z: number): void {
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
}
