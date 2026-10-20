/** Mécaniques des talents de chaque classe. */
import type { IdPersonnage } from '../../donnees/personnages';
import { MecaniquesGuerrier } from './guerrier';
import { MecaniquesClasse, type Talents } from './talents';

export function mecaniquesClasse(personnage: IdPersonnage): (t: Talents) => MecaniquesClasse {
  switch (personnage) {
    case 'guerrier':
      return (t) => new MecaniquesGuerrier(t);
    default:
      return (t) => new MecaniquesClasse(t);
  }
}
