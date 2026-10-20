/**
 * Armes et pouvoirs du joueur, tous automatiques. Ajouter une arme = ajouter
 * une entrée qui réutilise une mécanique existante (voir MecaniqueArme).
 *
 * Usage des statistiques par mécanique :
 *   balayage   degats, cadence, zone (rayon de l'arc), recul
 *   tir        degats, cadence, nombre, vitesse, portee, perforation, recul
 *   retour     degats, cadence, nombre, vitesse, portee (distance avant le retour), recul ;
 *              ne revient qu'à partir du rang `loyaute` (sinon disparaît au bout de sa portée)
 *   orbite     degats, cadence (délai avant de retoucher un ennemi), nombre, vitesse (rad/s), zone (rayon), recul
 *   explosif   degats, cadence, nombre, vitesse, portee, zone (rayon d'explosion), recul
 *   foudre     degats, cadence, nombre, portee (rayon de recherche), zone (rayon d'impact), recul
 *   invocation degats (par coup), cadence, nombre (alliés par invocation), duree, zone (allonge), recul ;
 *              le nombre présent est limité par le quota de l'allié (allies.ts)
 *   fissure    degats, cadence, nombre (fissures en éventail), portee (longueur), zone (largeur), recul,
 *              ralenti et duree (ralentissement des ennemis touchés)
 *   chute      degats, cadence, nombre, portee (recherche du groupe), zone (rayon d'impact), recul, ralenti, duree
 *   tourbillon degats (par frappe), cadence, zone (rayon), duree (de rotation), recul (négatif : aspire)
 *   heurt      degats, cadence, zone (portée du cône), recul, etourdi
 */
import type { DefinitionArme, StatsArme } from './types';

/** Statistiques neutres : chaque arme ne précise que ce qu'elle utilise. */
const BASE: StatsArme = {
  degats: 0, cadence: 1, nombre: 1, portee: 0, vitesse: 0, zone: 0, duree: 0, perforation: 1, recul: 0, ralenti: 0, etourdi: 0,
};

const definitions = {
  epee: {
    son: 'balayage',
    nom: 'Épée en fer',
    description: "Frappe en arc de cercle l'ennemi le plus proche.",
    icone: 'item/iron_sword',
    mecanique: {
      type: 'balayage',
      angle: 2.2,
      effet: Array.from({ length: 8 }, (_, i) => `particle/sweep_${i}`),
    },
    stats: { ...BASE, degats: 10, cadence: 1.1, zone: 2.6, recul: 6 },
    niveaux: [
      { description: 'Dégâts +5', modif: { degats: 5 } },
      { description: 'Portée du coup +0,6 bloc', modif: { zone: 0.6 } },
      { description: 'Frappe 20 % plus souvent', modif: { cadence: -0.2 } },
      { description: 'Dégâts +8', modif: { degats: 8 } },
      { description: 'Portée +0,8 bloc, recul accru', modif: { zone: 0.8, recul: 4 } },
    ],
  },
  arc: {
    son: 'arc',
    nom: 'Arc',
    description: "Tire sur l'ennemi le plus proche.",
    icone: 'item/bow',
    mecanique: { type: 'tir', projectile: 'fleche', dispersion: 0.18 },
    stats: { ...BASE, degats: 10, cadence: 1.1, vitesse: 22, portee: 15, perforation: 2, recul: 3 },
    niveaux: [
      { description: '+1 flèche', modif: { nombre: 1 } },
      { description: 'Dégâts +4', modif: { degats: 4 } },
      { description: 'Les flèches traversent 1 ennemi de plus', modif: { perforation: 1 } },
      { description: 'Tire 25 % plus souvent', modif: { cadence: -0.25 } },
      { description: '+1 flèche, dégâts +4', modif: { nombre: 1, degats: 4 } },
    ],
  },
  trident: {
    son: 'trident',
    nom: 'Trident',
    description: 'Lancé à travers les ennemis. Avec Loyauté, il revient dans votre main.',
    icone: 'item/trident',
    mecanique: { type: 'retour', projectile: 'trident', dispersion: 0.5, loyaute: 3 },
    stats: { ...BASE, degats: 14, cadence: 2.4, vitesse: 13, portee: 9, recul: 4 },
    niveaux: [
      { description: 'Dégâts +6', modif: { degats: 6 } },
      { description: 'Loyauté : le trident revient dans votre main', modif: {} },
      { description: '+1 trident', modif: { nombre: 1 } },
      { description: 'Portée +3 blocs, lancé 20 % plus souvent', modif: { portee: 3, cadence: -0.5 } },
      { description: '+1 trident', modif: { nombre: 1 } },
    ],
  },
  haches: {
    nom: 'Haches tournoyantes',
    description: 'Des haches tournent autour de vous.',
    icone: 'item/iron_axe',
    mecanique: { type: 'orbite', projectile: 'hache' },
    stats: { ...BASE, degats: 6, cadence: 0.6, nombre: 2, vitesse: 3.4, zone: 1.5, recul: 4 },
    niveaux: [
      { description: '+1 hache', modif: { nombre: 1 } },
      { description: 'Dégâts +4', modif: { degats: 4 } },
      { description: 'Rayon +0,4 bloc', modif: { zone: 0.4 } },
      { description: '+1 hache, rotation plus rapide', modif: { nombre: 1, vitesse: 1 } },
      { description: 'Dégâts +6', modif: { degats: 6 } },
    ],
  },
  boules_feu: {
    son: 'boule_feu',
    nom: 'Boules de feu',
    description: 'Explosent au contact et brûlent la zone.',
    icone: 'item/fire_charge',
    mecanique: { type: 'explosif', projectile: 'boule_feu' },
    stats: { ...BASE, degats: 12, cadence: 2.2, vitesse: 10, portee: 13, zone: 2, recul: 6 },
    niveaux: [
      { description: 'Rayon d\'explosion +0,5 bloc', modif: { zone: 0.5 } },
      { description: '+1 boule de feu', modif: { nombre: 1 } },
      { description: 'Dégâts +6', modif: { degats: 6 } },
      { description: 'Lancées 20 % plus souvent', modif: { cadence: -0.45 } },
      { description: 'Rayon +0,5 bloc, dégâts +4', modif: { zone: 0.5, degats: 4 } },
    ],
  },
  eclairs: {
    son: 'tonnerre',
    nom: 'Éclairs',
    description: 'La foudre frappe des ennemis au hasard.',
    icone: 'block/lightning_rod',
    mecanique: { type: 'foudre', couleur: 0xdfeaff },
    stats: { ...BASE, degats: 16, cadence: 3, nombre: 1, portee: 12, zone: 1.1, recul: 2 },
    niveaux: [
      { description: '+1 éclair', modif: { nombre: 1 } },
      { description: 'Dégâts +6', modif: { degats: 6 } },
      { description: 'Zone d\'impact +0,4 bloc', modif: { zone: 0.4 } },
      { description: 'Frappe 15 % plus souvent', modif: { cadence: -0.45 } },
      { description: '+1 éclair', modif: { nombre: 1 } },
    ],
  },
  onde_choc: {
    son: 'fissure',
    nom: 'Onde de choc',
    description: 'Frappe le sol : une fissure soulève les blocs vers le plus gros groupe.',
    icone: 'block/cracked_stone_bricks',
    mecanique: { type: 'fissure', ecart: 0.5, vitesse: 14 },
    stats: { ...BASE, degats: 14, cadence: 4, portee: 6, zone: 1.5, recul: 3, duree: 1 },
    niveaux: [
      { description: 'Dégâts +4', modif: { degats: 4 } },
      { description: 'Fissure plus longue : +2 blocs', modif: { portee: 2 } },
      { description: '+1 fissure en éventail', modif: { nombre: 1 } },
      { description: 'Frappe 20 % plus souvent', modif: { cadence: -0.8 } },
      { description: '+1 fissure ; les ennemis touchés sont ralentis de 30 % pendant 1 s', modif: { nombre: 1, ralenti: 0.3 } },
    ],
  },
  lame_colossale: {
    nom: 'Lame colossale',
    description: "Une épée géante s'abat sur le plus gros groupe d'ennemis.",
    icone: 'item/netherite_sword',
    mecanique: { type: 'chute', image: 'item/netherite_sword', taille: 5 },
    stats: { ...BASE, degats: 40, cadence: 7, portee: 10, zone: 2.5, recul: 9, duree: 2 },
    niveaux: [
      { description: 'Dégâts +15', modif: { degats: 15 } },
      { description: "Rayon d'impact +0,7 bloc", modif: { zone: 0.7 } },
      { description: 'Tombe 20 % plus souvent', modif: { cadence: -1.4 } },
      { description: '+1 lame', modif: { nombre: 1 } },
      { description: 'Dégâts +25 ; le sol fissuré ralentit de 20 % pendant 2 s', modif: { degats: 25, ralenti: 0.2 } },
    ],
  },
  tourbillon_lames: {
    nom: 'Tourbillon de lames',
    description: 'Tourne sur lui-même et frappe tout autour, plusieurs fois.',
    icone: 'item/golden_sword',
    mecanique: { type: 'tourbillon', intervalle: 0.3, effet: Array.from({ length: 8 }, (_, i) => `particle/sweep_${i}`) },
    stats: { ...BASE, degats: 5, cadence: 5, zone: 2.2, duree: 1.2, recul: 2 },
    niveaux: [
      { description: 'Dégâts +3 par frappe', modif: { degats: 3 } },
      { description: 'Rayon +0,5 bloc', modif: { zone: 0.5 } },
      { description: 'Tourne 0,6 s de plus (2 frappes)', modif: { duree: 0.6 } },
      { description: 'Tourne 20 % plus souvent', modif: { cadence: -1 } },
      { description: 'Dégâts +3 ; aspire les ennemis vers lui', modif: { degats: 3, recul: -5 } },
    ],
  },
  heurt_bouclier: {
    son: 'bouclier',
    nom: 'Heurt de bouclier',
    description: 'Coup de bouclier devant lui : repousse fort et interrompt les attaques.',
    icone: 'block/iron_trapdoor',
    mecanique: { type: 'heurt', angle: 1.6, renvoi: 6 },
    stats: { ...BASE, degats: 8, cadence: 3, zone: 1.8, recul: 12 },
    niveaux: [
      { description: 'Repousse plus fort', modif: { recul: 6 } },
      { description: 'Dégâts +6', modif: { degats: 6 } },
      { description: 'Étourdit 0,5 s', modif: { etourdi: 0.5 } },
      { description: 'Frappe 20 % plus souvent', modif: { cadence: -0.6 } },
      { description: 'Dégâts +6 ; renvoie les projectiles ennemis', modif: { degats: 6 } },
    ],
  },
  golems: {
    son: 'golem_invocation',
    nom: 'Golem de fer',
    description: 'Invoque un golem qui écrase les ennemis.',
    icone: 'block/carved_pumpkin',
    mecanique: { type: 'invocation', allie: 'golem_fer' },
    // Revient toutes les 16 s et reste 10 s : permanent à partir du rang 5
    stats: { ...BASE, degats: 30, cadence: 16, nombre: 1, duree: 10, zone: 1.4, recul: 10 },
    niveaux: [
      { description: 'Le golem reste 3 s de plus', modif: { duree: 3 } },
      { description: 'Dégâts +10', modif: { degats: 10 } },
      { description: 'Invoqué 20 % plus souvent', modif: { cadence: -3.2 } },
      { description: 'Le golem reste 3 s de plus', modif: { duree: 3 } },
      { description: '+1 golem, dégâts +10', modif: { nombre: 1, degats: 10 } },
    ],
  },
  loups: {
    son: 'loup_aboiement',
    nom: 'Meute de loups',
    description: 'Des loups apprivoisés traquent et mordent les ennemis.',
    icone: 'item/bone',
    mecanique: { type: 'invocation', allie: 'loup' },
    // 2 loups toutes les 7 s qui vivent 10 s : environ 3 présents au rang 1 (quota 6)
    stats: { ...BASE, degats: 5, cadence: 7, nombre: 2, duree: 10, zone: 0.6, recul: 2 },
    niveaux: [
      { description: 'Dégâts +3', modif: { degats: 3 } },
      { description: '+1 loup par invocation', modif: { nombre: 1 } },
      { description: 'Les loups restent 4 s de plus', modif: { duree: 4 } },
      { description: 'Dégâts +3', modif: { degats: 3 } },
      { description: '+1 loup par invocation, dégâts +3', modif: { nombre: 1, degats: 3 } },
    ],
  },
  vex: {
    son: 'vex_invocation',
    nom: 'Vex',
    description: 'Des esprits volants foncent sur les ennemis à travers le décor.',
    icone: 'item/echo_shard',
    mecanique: { type: 'invocation', allie: 'vex' },
    // 2 vex toutes les 6 s qui vivent 8 s : environ 3 présents au rang 1 (quota 6)
    stats: { ...BASE, degats: 4, cadence: 6, nombre: 2, duree: 8, zone: 0.4, recul: 1 },
    niveaux: [
      { description: '+1 vex par invocation', modif: { nombre: 1 } },
      { description: 'Dégâts +3', modif: { degats: 3 } },
      { description: 'Invoqués 25 % plus souvent', modif: { cadence: -1.5 } },
      { description: 'Dégâts +2', modif: { degats: 2 } },
      { description: '+1 vex par invocation, restent 3 s de plus', modif: { nombre: 1, duree: 3 } },
    ],
  },
} satisfies Record<string, DefinitionArme>;

export type IdArme = keyof typeof definitions;

export const ARMES: Readonly<Record<IdArme, DefinitionArme>> = definitions;

export const IDS_ARMES = Object.keys(definitions) as IdArme[];
