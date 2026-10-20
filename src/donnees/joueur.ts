/** Statistiques et apparence de base du joueur. */
import type { DefinitionJoueur } from './types';

export const JOUEUR_BASE: DefinitionJoueur = {
  pv: 20,
  invulnerabilite: 0.5,
  rayon: 0.4,
  vitesse: 5,
  reactivite: 25,
  vitesseRotation: 14,
  modele: 'humanoide',
  skin: 'entity/player/wide/steve',
  couleursSecours: {
    tete: 0xa8805e,
    corps: 0x2fa8a8,
    brasDroit: 0xa8805e,
    brasGauche: 0xa8805e,
    jambeDroite: 0x3b3b9c,
    jambeGauche: 0x3b3b9c,
  },
  rayonOmbre: 0.5,
};
