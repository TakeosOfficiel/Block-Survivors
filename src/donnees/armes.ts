/**
 * Armes et pouvoirs du joueur, tous automatiques. Ajouter une arme = ajouter
 * une entrée qui réutilise une mécanique existante (voir MecaniqueArme).
 *
 * Usage des statistiques par mécanique :
 *   balayage   degats, cadence, zone (rayon de l'arc), recul
 *   tir        degats, cadence, nombre, vitesse, portee, perforation, recul
 *   retour     degats, cadence, nombre, vitesse, portee (distance avant le retour), recul
 *   orbite     degats, cadence (délai avant de retoucher un ennemi), nombre, vitesse (rad/s), zone (rayon), recul
 *   explosif   degats, cadence, nombre, vitesse, portee, zone (rayon d'explosion), recul
 *   foudre     degats, cadence, nombre, portee (rayon de recherche), zone (rayon d'impact), recul
 *   invocation degats (par coup), cadence, nombre (alliés simultanés), duree, zone (allonge), recul
 */
import type { DefinitionArme, StatsArme } from './types';

/** Statistiques neutres : chaque arme ne précise que ce qu'elle utilise. */
const BASE: StatsArme = {
  degats: 0, cadence: 1, nombre: 1, portee: 0, vitesse: 0, zone: 0, duree: 0, perforation: 1, recul: 0,
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
    stats: { ...BASE, degats: 8, cadence: 1.2, vitesse: 22, portee: 15, perforation: 1, recul: 3 },
    niveaux: [
      { description: '+1 flèche', modif: { nombre: 1 } },
      { description: 'Dégâts +4', modif: { degats: 4 } },
      { description: 'Les flèches traversent 1 ennemi de plus', modif: { perforation: 1 } },
      { description: 'Tire 25 % plus souvent', modif: { cadence: -0.3 } },
      { description: '+2 flèches', modif: { nombre: 2 } },
    ],
  },
  trident: {
    son: 'trident',
    nom: 'Trident',
    description: 'Lancé à travers les ennemis, il revient dans votre main.',
    icone: 'item/trident',
    mecanique: { type: 'retour', projectile: 'trident', dispersion: 0.5 },
    stats: { ...BASE, degats: 12, cadence: 2.4, vitesse: 13, portee: 9, recul: 4 },
    niveaux: [
      { description: 'Dégâts +6', modif: { degats: 6 } },
      { description: '+1 trident', modif: { nombre: 1 } },
      { description: 'Portée +3 blocs', modif: { portee: 3 } },
      { description: 'Lancé 25 % plus souvent', modif: { cadence: -0.6 } },
      { description: '+1 trident, dégâts +6', modif: { nombre: 1, degats: 6 } },
    ],
  },
  haches: {
    nom: 'Haches tournoyantes',
    description: 'Des haches tournent autour de vous.',
    icone: 'item/iron_axe',
    mecanique: { type: 'orbite', projectile: 'hache' },
    stats: { ...BASE, degats: 6, cadence: 0.6, nombre: 2, vitesse: 3.2, zone: 2.2, recul: 4 },
    niveaux: [
      { description: '+1 hache', modif: { nombre: 1 } },
      { description: 'Dégâts +4', modif: { degats: 4 } },
      { description: 'Rayon +0,6 bloc', modif: { zone: 0.6 } },
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
    stats: { ...BASE, degats: 14, cadence: 2.2, vitesse: 10, portee: 13, zone: 2.2, recul: 6 },
    niveaux: [
      { description: 'Rayon d\'explosion +0,6 bloc', modif: { zone: 0.6 } },
      { description: '+1 boule de feu', modif: { nombre: 1 } },
      { description: 'Dégâts +8', modif: { degats: 8 } },
      { description: 'Lancées 25 % plus souvent', modif: { cadence: -0.5 } },
      { description: '+1 boule de feu, rayon +0,6', modif: { nombre: 1, zone: 0.6 } },
    ],
  },
  eclairs: {
    son: 'tonnerre',
    nom: 'Éclairs',
    description: 'La foudre frappe des ennemis au hasard.',
    icone: 'block/lightning_rod',
    mecanique: { type: 'foudre', couleur: 0xdfeaff },
    stats: { ...BASE, degats: 18, cadence: 2.6, nombre: 2, portee: 12, zone: 1.2, recul: 2 },
    niveaux: [
      { description: '+1 éclair', modif: { nombre: 1 } },
      { description: 'Dégâts +8', modif: { degats: 8 } },
      { description: 'Zone d\'impact +0,5 bloc', modif: { zone: 0.5 } },
      { description: 'Frappe 25 % plus souvent', modif: { cadence: -0.6 } },
      { description: '+2 éclairs', modif: { nombre: 2 } },
    ],
  },
  golems: {
    son: 'golem_invocation',
    nom: 'Golem de fer',
    description: 'Invoque un golem qui écrase les ennemis.',
    icone: 'block/carved_pumpkin',
    mecanique: { type: 'invocation', allie: 'golem_fer' },
    stats: { ...BASE, degats: 20, cadence: 12, nombre: 1, duree: 15, zone: 1.4, recul: 10 },
    niveaux: [
      { description: 'Le golem reste 5 s de plus', modif: { duree: 5 } },
      { description: 'Dégâts +10', modif: { degats: 10 } },
      { description: '+1 golem', modif: { nombre: 1 } },
      { description: 'Invoqué 25 % plus souvent', modif: { cadence: -3 } },
      { description: '+1 golem, dégâts +10', modif: { nombre: 1, degats: 10 } },
    ],
  },
} satisfies Record<string, DefinitionArme>;

export type IdArme = keyof typeof definitions;

export const ARMES: Readonly<Record<IdArme, DefinitionArme>> = definitions;

export const IDS_ARMES = Object.keys(definitions) as IdArme[];
