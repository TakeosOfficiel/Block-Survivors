/**
 * Coffres de la partie : un coffre ordinaire apparaît sur la carte tous les
 * tant d'ennemis tués et à la mort de chaque mini-boss ; il s'ouvre en
 * marchant dessus contre des émeraudes (ramassées sur les ennemis), de plus
 * en plus cher. Le coffre de l'Ender, lâché par les boss, est gratuit et
 * offre deux cartes. Ajouter un coffre = ajouter une entrée.
 */
import type { DefinitionCoffre, ReglesCoffres } from './types';

const definitions = {
  coffre: {
    nom: 'Coffre',
    titre: 'Coffre !',
    apparence: { modele: 'coffre', skin: 'entity/chest/normal', couleursSecours: {}, couleurSecoursParDefaut: 0x9a6a2a, rayonOmbre: 0.6 },
    payant: true,
    choix: 1,
    son: 'coffre',
  },
  coffre_ender: {
    nom: "Coffre de l'Ender",
    titre: "Coffre de l'Ender !",
    apparence: { modele: 'coffre', skin: 'entity/chest/ender', couleursSecours: {}, couleurSecoursParDefaut: 0x1d3a3a, rayonOmbre: 0.6 },
    payant: false,
    choix: 2,
    son: 'coffre_ender',
  },
} satisfies Record<string, DefinitionCoffre>;

export type IdCoffre = keyof typeof definitions;

export const COFFRES: Readonly<Record<IdCoffre, DefinitionCoffre>> = definitions;

export const IDS_COFFRES = Object.keys(definitions) as IdCoffre[];

export const REGLES_COFFRES: ReglesCoffres = {
  capacite: 8,
  // Une dizaine de coffres par partie complète (≈ 3 300 ennemis tués), plus un par mini-boss
  tues: { premier: 100, intervalle: 150, croissance: 50 },
  // 8, 13, 18… : une douzaine de coffres ouverts sur une partie complète
  prix: { base: 8, increment: 5 },
  distance: [7, 12],
  rayonOuverture: 1.1,
  dureeOuverture: 0.5,
  resteOuvert: 1.2,
};
