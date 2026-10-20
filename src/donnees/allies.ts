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
    cadenceCoups: 0.8,
    detection: 20,
    sonCoup: 'golem_coup',
  },
  loup: {
    nom: 'Loup',
    modele: 'loup',
    skin: 'entity/wolf/wolf_tame',
    couleursSecours: {},
    couleurSecoursParDefaut: 0xd8d4d0,
    rayonOmbre: 0.45,
    vitesse: 6.2,
    rayon: 0.35,
    cadenceCoups: 0.6,
    detection: 16,
    sonCoup: 'loup_morsure',
  },
  vex: {
    nom: 'Vex',
    modele: 'vex',
    skin: 'entity/illager/vex',
    couleursSecours: {},
    couleurSecoursParDefaut: 0x8fa4c0,
    rayonOmbre: 0.25,
    vitesse: 7.5,
    rayon: 0.25,
    cadenceCoups: 0.5,
    detection: 18,
    sonCoup: 'vex_charge',
    vol: 1.1,
  },
} satisfies Record<string, DefinitionAllie>;

export type IdAllie = keyof typeof definitions;

export const ALLIES: Readonly<Record<IdAllie, DefinitionAllie>> = definitions;

export const IDS_ALLIES = Object.keys(definitions) as IdAllie[];
