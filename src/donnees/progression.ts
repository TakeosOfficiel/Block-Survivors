/** Courbe d'expérience, cartes d'amélioration et orbes d'XP. */
import type { DefinitionProgression } from './types';

export const PROGRESSION: DefinitionProgression = {
  // Niveau 2 à 5 XP, puis +10 par niveau jusqu'au 20, +13 jusqu'au 40, +16 ensuite
  xpPremierNiveau: 5,
  dureePartie: 1200,     // 20:00
  paliers: [
    { jusquAuNiveau: 20, increment: 10 },
    { jusquAuNiveau: 40, increment: 13 },
    { jusquAuNiveau: Infinity, increment: 16 },
  ],
  cartesParNiveau: 3,
  relances: 3,
  emplacementsArmes: 4,
  emplacementsPassifs: 6,
  soin: {
    nom: 'Pomme dorée',
    description: 'Rien à améliorer : vous reprenez des forces.',
    icone: 'item/golden_apple',
    pv: 10,
  },
  // Seuils du jeu : plus l'orbe vaut d'XP, plus elle est grosse
  orbes: [
    { valeurMin: 1, image: 0 },
    { valeurMin: 3, image: 1 },
    { valeurMin: 7, image: 2 },
    { valeurMin: 17, image: 3 },
    { valeurMin: 37, image: 4 },
    { valeurMin: 73, image: 5 },
  ],
  // Après 15:00, chaque minute : +25 % de PV et +10 % de dégâts ennemis, groupes plus gros
  sansFin: {
    pvParMinute: 0.25,
    degatsParMinute: 0.1,
    groupeParMinute: 0.5,
    groupeMax: 12,
  },
};
