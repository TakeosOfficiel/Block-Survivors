/** Alliés invoqués par les armes. Ajouter un allié = ajouter une entrée. */
import type { ApparenceEntite, DefinitionAllie } from './types';

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
    sonBlessure: 'golem_blesse',
    sonMort: 'golem_mort',
    pv: 120,
    armure: 2,
    quota: 2,
    renforts: false,
    provocation: 7,   // il attire la horde loin du joueur
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
    sonBlessure: 'loup_blesse',
    sonMort: 'loup_mort',
    pv: 24,
    quota: 6,
    renforts: true,
    provocation: 3,
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
    sonBlessure: 'vex_blesse',
    sonMort: 'vex_mort',
    pv: 14,
    quota: 6,
    renforts: true,
    provocation: 2,
    vol: 1.1,
  },
  esprit_blaze: {
    nom: 'Esprit de blaze',
    modele: 'blaze',
    skin: 'entity/blaze',
    couleursSecours: {},
    couleurSecoursParDefaut: 0xf0b030,
    rayonOmbre: 0.45,
    vitesse: 5,
    rayon: 0.4,
    cadenceCoups: 2,
    detection: 14,
    sonCoup: 'boule_feu',
    sonBlessure: 'blaze_blesse',
    sonMort: 'blaze_mort',
    pv: 20,
    quota: 3,
    renforts: true,
    provocation: 2,
    vol: 1.4,
    tir: { projectile: 'boule_feu_blaze', salve: 3, ecart: 0.15, distance: 6 },
  },
  // Totem en blocs de palétuvier : immobile, il attire les ennemis et les retient
  totem_racines: {
    nom: 'Totem des racines',
    modele: 'vex',
    skin: 'block/mangrove_log',
    couleursSecours: {},
    couleurSecoursParDefaut: 0x6b3a2a,
    rayonOmbre: 0.7,
    vitesse: 0,
    rayon: 0.5,
    cadenceCoups: 1,
    detection: 0,
    sonCoup: 'racines',
    sonBlessure: 'bois',
    sonMort: 'racines',
    pv: 60,
    quota: 2,
    renforts: false,
    provocation: 4,
    aura: { intervalle: 0.5, enracinement: 1, delai: 3 },
    blocs: { empilement: ['block/mangrove_log', 'block/mangrove_log'], racines: 'block/mangrove_roots_side' },
    remplacement: true,
  },
  // L'allay du jeu (même patron que le vex) : soutien, il ne combat pas
  allay: {
    nom: 'Esprit protecteur',
    modele: 'vex',
    skin: 'entity/allay/allay',
    couleursSecours: {},
    couleurSecoursParDefaut: 0x6fc8ff,
    rayonOmbre: 0.25,
    vitesse: 6,
    rayon: 0.25,
    cadenceCoups: 1,
    detection: 0,
    sonCoup: 'allay',
    sonBlessure: 'allay_blesse',
    sonMort: 'allay_mort',
    pv: 20,
    quota: 2,
    renforts: false,
    provocation: 0,
    vol: 1.3,
    soutien: true,
  },
} satisfies Record<string, DefinitionAllie>;

export type IdAllie = keyof typeof definitions;

export const ALLIES: Readonly<Record<IdAllie, DefinitionAllie>> = definitions;

export const IDS_ALLIES = Object.keys(definitions) as IdAllie[];

/**
 * Créatures des talents de l'Invocateur, hors armes : le ravageur
 * (Évocation supérieure) et les crocs d'évocateur (Crocs de l'évocateur).
 */
export const CREATURES_TALENTS = {
  ravageur: {
    modele: 'ravageur', skin: 'entity/illager/ravager', couleursSecours: {}, couleurSecoursParDefaut: 0x5b5652, rayonOmbre: 1.3,
  },
  crocs: {
    modele: 'crocs', skin: 'entity/illager/evoker_fangs', couleursSecours: {}, couleurSecoursParDefaut: 0x8a8070, rayonOmbre: 0.4,
  },
} as const satisfies Record<string, ApparenceEntite>;
