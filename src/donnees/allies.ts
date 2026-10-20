/** Alliés invoqués par les armes. Ajouter un allié = ajouter une entrée. */
import type { DefinitionAllie } from './types';

const definitions = {
  golem_fer: {
    nom: 'Golem de fer',
    modele: 'golem_fer',
    skin: 'entity/iron_golem/iron_golem',
    couleursSecours: {},
    couleurSecoursParDefaut: 0xc8c0b8,
    rayonOmbre: 1,
    vitesse: 3.5,
    rayon: 0.7,
    cadenceCoups: 1,
    detection: 20,
  },
} satisfies Record<string, DefinitionAllie>;

export type IdAllie = keyof typeof definitions;

export const ALLIES: Readonly<Record<IdAllie, DefinitionAllie>> = definitions;

export const IDS_ALLIES = Object.keys(definitions) as IdAllie[];
