/** Mécaniques des talents de chaque classe. */
import type { IdPersonnage } from '../../donnees/personnages';
import { MecaniquesArcher } from './archer';
import { MecaniquesGuerrier } from './guerrier';
import { MecaniquesMage } from './mage';
import { MecaniquesClasse, type Talents } from './talents';

export function mecaniquesClasse(personnage: IdPersonnage): (t: Talents) => MecaniquesClasse {
  switch (personnage) {
    case 'guerrier':
      return (t) => new MecaniquesGuerrier(t);
    case 'archer':
      return (t) => new MecaniquesArcher(t);
    case 'mage':
      return (t) => new MecaniquesMage(t);
    default:
      return (t) => new MecaniquesClasse(t);
  }
}
