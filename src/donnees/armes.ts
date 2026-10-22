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
 *   tornade    degats, cadence, nombre, duree, zone (rayon), recul (négatif : aspire)
 *   nova       degats, cadence, zone (rayon final), recul, ralenti et duree (ralentissement)
 *   meteore    degats, cadence, nombre, portee (recherche du groupe), zone (rayon), duree (feu au sol), recul
 *   orbes      degats, cadence, nombre, portee, zone (rayon d'orbite), duree (d'orbite), vitesse (de vol)
 *   ricochet   degats, cadence, nombre, vitesse, portee, perforation (rebonds), recul
 *   pluie      degats (par flèche), cadence, nombre (zones), portee (recherche du groupe), zone (rayon), duree, recul
 *   piege      degats, cadence, nombre (pièges en même temps), duree, zone (rayon), ralenti
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
    famille: 'melee',
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
    eveils: [
      { nom: 'Tranchant', description: 'Dégâts +40 %.', icone: 'item/netherite_sword', part: { degats: 0.4 } },
      { nom: 'Aura de feu', description: 'Chaque coup enflamme (4 dégâts/s pendant 3 s) ; dégâts +10 %.', icone: 'item/blaze_powder', part: { degats: 0.1 }, etat: { type: 'brulure', valeur: 4, duree: 3 } },
    ],
  },
  arc: {
    son: 'arc',
    nom: 'Arc',
    description: "Tire sur l'ennemi le plus proche.",
    icone: 'item/bow',
    famille: 'projectile',
    mecanique: { type: 'tir', projectile: 'fleche', dispersion: 0.18 },
    stats: { ...BASE, degats: 10, cadence: 1.1, vitesse: 22, portee: 15, perforation: 1, recul: 3 },
    niveaux: [
      { description: '+1 flèche', modif: { nombre: 1 } },
      { description: 'Dégâts +4', modif: { degats: 4 } },
      { description: 'Les flèches traversent 1 ennemi de plus', modif: { perforation: 1 } },
      { description: 'Tire 25 % plus souvent', modif: { cadence: -0.25 } },
      { description: '+1 flèche, dégâts +4', modif: { nombre: 1, degats: 4 } },
    ],
    eveils: [
      { nom: 'Frappe', description: 'Dégâts +25 %, recul doublé.', icone: 'item/slime_ball', part: { degats: 0.25, recul: 1 } },
      { nom: 'Flamme', description: 'Les flèches enflamment (4 dégâts/s pendant 3 s).', icone: 'item/blaze_rod', etat: { type: 'brulure', valeur: 4, duree: 3 } },
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
    eveils: [
      { nom: 'Empalement', description: 'Dégâts +50 %.', icone: 'item/prismarine_shard', part: { degats: 0.5 } },
      { nom: 'Canalisation', description: '+1 trident ; chaque coup étourdit 0,4 s (pas les boss).', icone: 'item/heart_of_the_sea', ajout: { nombre: 1 }, etat: { type: 'etourdi', valeur: 0.4, duree: 0.4 } },
    ],
  },
  haches: {
    nom: 'Haches tournoyantes',
    description: 'Des haches tournent autour de vous.',
    icone: 'item/iron_axe',
    famille: 'melee',
    mecanique: { type: 'orbite', projectile: 'hache' },
    stats: { ...BASE, degats: 6, cadence: 0.6, nombre: 2, vitesse: 3.4, zone: 1.5, recul: 4 },
    niveaux: [
      { description: '+1 hache', modif: { nombre: 1 } },
      { description: 'Dégâts +4', modif: { degats: 4 } },
      { description: 'Rayon +0,4 bloc', modif: { zone: 0.4 } },
      { description: '+1 hache, rotation plus rapide', modif: { nombre: 1, vitesse: 1 } },
      { description: 'Dégâts +6', modif: { degats: 6 } },
    ],
    eveils: [
      { nom: 'Haches fendeuses', description: '+1 hache, dégâts +15 %.', icone: 'item/netherite_axe', ajout: { nombre: 1 }, part: { degats: 0.15 } },
      { nom: 'Haches givrées', description: 'Ralentissent de 35 % pendant 1,5 s ; rayon +20 %.', icone: 'block/packed_ice', part: { zone: 0.2 }, etat: { type: 'ralenti', valeur: 0.35, duree: 1.5 } },
    ],
  },
  boules_feu: {
    son: 'boule_feu',
    nom: 'Boules de feu',
    description: 'Explosent au contact et brûlent la zone.',
    icone: 'item/fire_charge',
    famille: 'feu',
    mecanique: { type: 'explosif', projectile: 'boule_feu' },
    stats: { ...BASE, degats: 12, cadence: 2.2, vitesse: 10, portee: 13, zone: 2, recul: 6 },
    niveaux: [
      { description: 'Rayon d\'explosion +0,5 bloc', modif: { zone: 0.5 } },
      { description: '+1 boule de feu', modif: { nombre: 1 } },
      { description: 'Dégâts +6', modif: { degats: 6 } },
      { description: 'Lancées 20 % plus souvent', modif: { cadence: -0.45 } },
      { description: 'Rayon +0,5 bloc, dégâts +4', modif: { zone: 0.5, degats: 4 } },
    ],
    eveils: [
      { nom: 'Boule de ghast', description: 'Explosion +40 % de zone, dégâts +20 %.', icone: 'item/ghast_tear', part: { zone: 0.4, degats: 0.2 } },
      { nom: 'Rafale de blaze', description: '+2 boules de feu, dégâts −20 %.', icone: 'item/blaze_powder', ajout: { nombre: 2 }, part: { degats: -0.2 } },
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
    eveils: [
      { nom: 'Canalisation', description: '+2 éclairs.', icone: 'block/lightning_rod', ajout: { nombre: 2 } },
      { nom: 'Foudre en boule', description: 'Zone +50 % ; étourdit 0,3 s (pas les boss).', icone: 'item/glowstone_dust', part: { zone: 0.5 }, etat: { type: 'etourdi', valeur: 0.3, duree: 0.3 } },
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
    eveils: [
      { nom: 'Faille', description: 'Portée +40 %, +1 fissure.', icone: 'block/cracked_deepslate_bricks', ajout: { nombre: 1 }, part: { portee: 0.4 } },
      { nom: 'Secousse', description: 'Étourdit 0,6 s (pas les boss) ; dégâts +10 %.', icone: 'item/goat_horn', part: { degats: 0.1 }, etat: { type: 'etourdi', valeur: 0.6, duree: 0.6 } },
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
    eveils: [
      { nom: 'Lame de netherite', description: 'Dégâts +50 %, zone +15 %.', icone: 'item/netherite_ingot', part: { degats: 0.5, zone: 0.15 } },
      { nom: 'Pluie d\'acier', description: '+1 lame, dégâts −10 %.', icone: 'item/iron_ingot', ajout: { nombre: 1 }, part: { degats: -0.1 } },
    ],
  },
  tourbillon_lames: {
    nom: 'Tourbillon de lames',
    description: 'Tourne sur lui-même et frappe tout autour, plusieurs fois.',
    icone: 'item/golden_sword',
    famille: 'melee',
    mecanique: { type: 'tourbillon', intervalle: 0.3, effet: Array.from({ length: 8 }, (_, i) => `particle/sweep_${i}`) },
    stats: { ...BASE, degats: 5, cadence: 5, zone: 2.2, duree: 1.2, recul: 2 },
    niveaux: [
      { description: 'Dégâts +3 par frappe', modif: { degats: 3 } },
      { description: 'Rayon +0,5 bloc', modif: { zone: 0.5 } },
      { description: 'Tourne 0,6 s de plus (2 frappes)', modif: { duree: 0.6 } },
      { description: 'Tourne 20 % plus souvent', modif: { cadence: -1 } },
      { description: 'Dégâts +3 ; aspire les ennemis vers lui', modif: { degats: 3, recul: -5 } },
    ],
    eveils: [
      { nom: 'Cyclone', description: 'Dure 1 s de plus, rayon +20 %.', icone: 'item/breeze_rod', ajout: { duree: 1 }, part: { zone: 0.2 } },
      { nom: 'Lames ardentes', description: 'Chaque coup enflamme (3 dégâts/s pendant 3 s).', icone: 'item/fire_charge', etat: { type: 'brulure', valeur: 3, duree: 3 } },
    ],
  },
  heurt_bouclier: {
    son: 'bouclier',
    nom: 'Heurt de bouclier',
    description: 'Coup de bouclier devant lui : repousse fort et interrompt les attaques.',
    icone: 'block/iron_trapdoor',
    famille: 'melee',
    mecanique: { type: 'heurt', angle: 1.6, renvoi: 6 },
    stats: { ...BASE, degats: 8, cadence: 3, zone: 1.8, recul: 12 },
    niveaux: [
      { description: 'Repousse plus fort', modif: { recul: 6 } },
      { description: 'Dégâts +6', modif: { degats: 6 } },
      { description: 'Étourdit 0,5 s', modif: { etourdi: 0.5 } },
      { description: 'Frappe 20 % plus souvent', modif: { cadence: -0.6 } },
      { description: 'Dégâts +6 ; renvoie les projectiles ennemis', modif: { degats: 6 } },
    ],
    eveils: [
      { nom: 'Bouclier à pointes', description: 'Dégâts +50 %.', icone: 'item/flint', part: { degats: 0.5 } },
      { nom: 'Bouclier tonnerre', description: 'Étourdit 0,5 s de plus, cône +20 %.', icone: 'item/bell', ajout: { etourdi: 0.5 }, part: { zone: 0.2 } },
    ],
  },
  tornade_feu: {
    son: 'tornade',
    nom: 'Tornade de feu',
    description: 'Une colonne de braises tourbillonne vers les ennemis et les aspire.',
    icone: 'item/blaze_powder',
    famille: 'feu',
    mecanique: {
      type: 'tornade', texture: 'block/magma', feu: 'block/fire_0', brulure: 'block/soul_soil',
      vitesse: 3.5, intervalle: 0.4,
    },
    stats: { ...BASE, degats: 6, cadence: 8, duree: 4, zone: 1.5, recul: -2 },
    niveaux: [
      { description: 'La tornade dure 2 s de plus', modif: { duree: 2 } },
      { description: 'Dégâts +3', modif: { degats: 3 } },
      { description: 'Rayon +0,4 bloc', modif: { zone: 0.4 } },
      { description: 'Lancée 20 % plus souvent', modif: { cadence: -1.6 } },
      { description: '+1 tornade', modif: { nombre: 1 } },
    ],
    eveils: [
      { nom: 'Tornade dévorante', description: 'Rayon +30 %, dure 1 s de plus.', icone: 'item/fire_charge', ajout: { duree: 1 }, part: { zone: 0.3 } },
      { nom: 'Tornades jumelles', description: '+1 tornade.', icone: 'item/wind_charge', ajout: { nombre: 1 } },
    ],
  },
  nova_glace: {
    son: 'nova',
    nom: 'Nova de glace',
    description: "Un anneau d'éclats de glace jaillit autour de lui, blesse et ralentit.",
    icone: 'block/packed_ice',
    mecanique: { type: 'nova', texture: 'block/packed_ice', expansion: 0.4, sol: 'block/powder_snow', solAuRang: 6, dureeSol: 2 },
    stats: { ...BASE, degats: 10, cadence: 5, zone: 4, recul: 2, ralenti: 0.35, duree: 2 },
    niveaux: [
      { description: 'Rayon +1 bloc', modif: { zone: 1 } },
      { description: 'Dégâts +6', modif: { degats: 6 } },
      { description: 'Ralentit de 45 % pendant 2,5 s', modif: { ralenti: 0.1, duree: 0.5 } },
      { description: 'Lancée 20 % plus souvent', modif: { cadence: -1 } },
      { description: 'Dégâts +4 ; laisse de la neige poudreuse qui ralentit 2 s', modif: { degats: 4 } },
    ],
    eveils: [
      { nom: 'Glace bleue', description: 'Ralentissement +20 points, zone +15 %.', icone: 'block/blue_ice', ajout: { ralenti: 0.2 }, part: { zone: 0.15 } },
      { nom: 'Éclats de givre', description: 'Dégâts +50 %.', icone: 'item/snowball', part: { degats: 0.5 } },
    ],
  },
  meteore: {
    son: 'boule_feu',
    nom: 'Météore',
    description: 'Un bloc de magma tombe sur un groupe et laisse le sol en feu.',
    icone: 'block/magma',
    famille: 'feu',
    mecanique: { type: 'meteore', texture: 'block/magma', feu: 'block/fire_0', degatsSol: 0.15 },
    stats: { ...BASE, degats: 35, cadence: 9, portee: 12, zone: 2.5, duree: 3, recul: 6 },
    niveaux: [
      { description: 'Dégâts +15', modif: { degats: 15 } },
      { description: 'Le feu au sol dure 2 s de plus', modif: { duree: 2 } },
      { description: "Rayon d'impact +0,5 bloc", modif: { zone: 0.5 } },
      { description: 'Tombe 20 % plus souvent', modif: { cadence: -1.8 } },
      { description: '+1 météore', modif: { nombre: 1 } },
    ],
    eveils: [
      { nom: 'Comète', description: 'Dégâts +40 %, zone +15 %.', icone: 'item/nether_star', part: { degats: 0.4, zone: 0.15 } },
      { nom: 'Essaim de météores', description: '+1 météore.', icone: 'item/magma_cream', ajout: { nombre: 1 } },
    ],
  },
  orbes_arcaniques: {
    son: 'orbes',
    nom: 'Orbes arcaniques',
    description: "Des orbes d'améthyste tournent autour de lui puis foncent sur les ennemis.",
    icone: 'item/amethyst_shard',
    mecanique: { type: 'orbes', texture: 'block/amethyst_block', contactAuRang: 4, contact: 0.45 },
    stats: { ...BASE, degats: 9, cadence: 4, nombre: 3, portee: 10, zone: 1.3, duree: 2, vitesse: 12 },
    niveaux: [
      { description: '+1 orbe', modif: { nombre: 1 } },
      { description: 'Dégâts +4', modif: { degats: 4 } },
      { description: "Orbitent 1 s de plus et blessent au contact", modif: { duree: 1 } },
      { description: 'Lancées 20 % plus souvent', modif: { cadence: -0.8 } },
      { description: '+2 orbes', modif: { nombre: 2 } },
    ],
    eveils: [
      { nom: 'Orbes du néant', description: 'Dégâts +40 %.', icone: 'item/ender_pearl', part: { degats: 0.4 } },
      { nom: 'Constellation', description: '+2 orbes.', icone: 'item/amethyst_shard', ajout: { nombre: 2 } },
    ],
  },
  salve: {
    son: 'arbalete',
    nom: 'Salve en éventail',
    description: 'Tire une gerbe de flèches en éventail vers les ennemis.',
    icone: 'item/crossbow_standby',
    famille: 'projectile',
    mecanique: { type: 'tir', projectile: 'fleche', dispersion: 0.26 },
    stats: { ...BASE, degats: 7, cadence: 2.5, nombre: 5, vitesse: 20, portee: 9, perforation: 1, recul: 2 },
    niveaux: [
      { description: '+2 flèches', modif: { nombre: 2 } },
      { description: 'Dégâts +3', modif: { degats: 3 } },
      { description: 'Portée +3 blocs', modif: { portee: 3 } },
      { description: 'Tire 20 % plus souvent', modif: { cadence: -0.5 } },
      { description: '+2 flèches, elles traversent 1 ennemi de plus', modif: { nombre: 2, perforation: 1 } },
    ],
    eveils: [
      { nom: 'Multitir', description: '+2 flèches.', icone: 'item/arrow', ajout: { nombre: 2 } },
      { nom: 'Flèches perforantes', description: 'Traversent 2 ennemis de plus, dégâts +10 %.', icone: 'item/spectral_arrow', ajout: { perforation: 2 }, part: { degats: 0.1 } },
    ],
  },
  pluie_fleches: {
    son: 'arc',
    nom: 'Pluie de flèches',
    description: 'Des flèches tombent du ciel sur le plus gros groupe.',
    icone: 'item/arrow',
    famille: 'projectile',
    mecanique: { type: 'pluie', projectile: 'fleche', impactsParSeconde: 7.5 },
    stats: { ...BASE, degats: 6, cadence: 6, portee: 14, zone: 2.5, duree: 1.6, recul: 1 },
    niveaux: [
      { description: 'Rayon +0,5 bloc', modif: { zone: 0.5 } },
      { description: 'Dégâts +3', modif: { degats: 3 } },
      { description: 'Dure 0,8 s de plus', modif: { duree: 0.8 } },
      { description: 'Tombe 20 % plus souvent', modif: { cadence: -1.2 } },
      { description: '+1 zone', modif: { nombre: 1 } },
    ],
    eveils: [
      { nom: 'Déluge', description: 'Dure 1,5 s de plus, zone +20 %.', icone: 'item/feather', ajout: { duree: 1.5 }, part: { zone: 0.2 } },
      { nom: 'Flèches de lenteur', description: 'Ralentissent de 40 % pendant 1,5 s.', icone: 'block/cobweb', etat: { type: 'ralenti', valeur: 0.4, duree: 1.5 } },
    ],
  },
  fleche_ricochet: {
    son: 'arc',
    nom: 'Flèche ricocheuse',
    description: "Une flèche spectrale rebondit d'ennemi en ennemi.",
    icone: 'item/spectral_arrow',
    famille: 'projectile',
    mecanique: { type: 'ricochet', projectile: 'fleche_spectrale', perte: 0.15, distanceRebond: 5 },
    stats: { ...BASE, degats: 12, cadence: 2, nombre: 1, vitesse: 22, portee: 12, perforation: 3, recul: 2 },
    niveaux: [
      { description: '+1 rebond', modif: { perforation: 1 } },
      { description: 'Dégâts +5', modif: { degats: 5 } },
      { description: '+1 rebond', modif: { perforation: 1 } },
      { description: 'Tire 20 % plus souvent', modif: { cadence: -0.4 } },
      { description: '+1 flèche', modif: { nombre: 1 } },
    ],
    eveils: [
      { nom: 'Rebond sans fin', description: '2 rebonds de plus.', icone: 'item/echo_shard', ajout: { perforation: 2 } },
      { nom: 'Flèche du chasseur', description: 'Dégâts +45 %.', icone: 'item/ender_eye', part: { degats: 0.45 } },
    ],
  },
  fusee: {
    son: 'fusee',
    nom: 'Fusée explosive',
    description: "Une fusée de feu d'artifice qui explose en gerbe colorée.",
    icone: 'item/firework_rocket',
    famille: 'projectile',
    mecanique: { type: 'explosif', projectile: 'fusee', gerbe: 'artifice', son: 'artifice', eclatsAuRang: 6 },
    stats: { ...BASE, degats: 18, cadence: 3.5, vitesse: 14, portee: 13, zone: 2, recul: 5 },
    niveaux: [
      { description: "Rayon d'explosion +0,5 bloc", modif: { zone: 0.5 } },
      { description: 'Dégâts +8', modif: { degats: 8 } },
      { description: '+1 fusée', modif: { nombre: 1 } },
      { description: 'Tirée 20 % plus souvent', modif: { cadence: -0.7 } },
      { description: "L'explosion se divise en 4 petites explosions", modif: {} },
    ],
    eveils: [
      { nom: 'Étoile de feu', description: 'Explosion +30 % de zone, qui enflamme (3 dégâts/s pendant 3 s).', icone: 'item/firework_star', part: { zone: 0.3 }, etat: { type: 'brulure', valeur: 3, duree: 3 } },
      { nom: 'Bouquet', description: '+1 fusée.', icone: 'item/gunpowder', ajout: { nombre: 1 } },
    ],
  },
  piege_ronces: {
    son: 'ronces',
    nom: 'Piège de ronces',
    description: 'Pose des buissons de baies qui blessent et ralentissent.',
    icone: 'item/sweet_berries',
    mecanique: { type: 'piege', image: 'block/sweet_berry_bush_stage3', intervalle: 0.5, enracineAuRang: 6 },
    stats: { ...BASE, degats: 4, cadence: 5, nombre: 3, duree: 8, zone: 1.5, ralenti: 0.4 },
    niveaux: [
      { description: '+1 piège en même temps', modif: { nombre: 1 } },
      { description: 'Dégâts +3', modif: { degats: 3 } },
      { description: 'Rayon +0,5 bloc, dure 4 s de plus', modif: { zone: 0.5, duree: 4 } },
      { description: 'Posé 20 % plus souvent', modif: { cadence: -1 } },
      { description: 'Ralentit de 55 % et enracine un instant à l\u2019entrée', modif: { ralenti: 0.15 } },
    ],
    eveils: [
      { nom: 'Ronces empoisonnées', description: 'Empoisonnent (3 dégâts/s pendant 4 s).', icone: 'item/poisonous_potato', etat: { type: 'poison', valeur: 3, duree: 4 } },
      { nom: 'Grand roncier', description: '+1 piège, zone +30 %.', icone: 'item/sweet_berries', ajout: { nombre: 1 }, part: { zone: 0.3 } },
    ],
  },
  esprit_blaze: {
    son: 'tornade',
    nom: 'Esprit de blaze',
    description: 'Un blaze allié flotte près de lui et tire des salves de boules de feu.',
    icone: 'item/blaze_rod',
    famille: 'allie',
    mecanique: { type: 'invocation', allie: 'esprit_blaze', brulureAuRang: 6 },
    stats: { ...BASE, degats: 6, cadence: 10, nombre: 1, duree: 12, portee: 10, vitesse: 14, recul: 1 },
    niveaux: [
      { description: '+1 blaze par invocation', modif: { nombre: 1 } },
      { description: 'Dégâts +3', modif: { degats: 3 } },
      { description: 'Les blazes restent 4 s de plus', modif: { duree: 4 } },
      { description: 'Invoqués 20 % plus souvent', modif: { cadence: -2 } },
      { description: '+1 blaze ; leurs tirs enflamment', modif: { nombre: 1 } },
    ],
    eveils: [
      { nom: 'Blaze ardent', description: 'Les tirs enflamment (4 dégâts/s pendant 3 s).', icone: 'item/blaze_powder', etat: { type: 'brulure', valeur: 4, duree: 3 } },
      { nom: 'Blaze tireur', description: 'Portée +30 %, dégâts +20 %.', icone: 'item/blaze_rod', part: { portee: 0.3, degats: 0.2 } },
    ],
  },
  totem_racines: {
    son: 'racines',
    nom: 'Totem des racines',
    description: 'Un totem de palétuvier planté dans la horde attire, ralentit et enracine.',
    icone: 'block/mangrove_roots_side',
    famille: 'allie',
    mecanique: { type: 'invocation', allie: 'totem_racines' },
    stats: { ...BASE, degats: 0, cadence: 10, nombre: 1, duree: 8, zone: 3, ralenti: 0.4 },
    niveaux: [
      { description: 'Rayon +0,7 bloc', modif: { zone: 0.7 } },
      { description: 'Le totem reste 4 s de plus', modif: { duree: 4 } },
      { description: 'Les racines blessent : 3 dégâts par seconde', modif: { degats: 1.5 } },
      { description: 'Planté 20 % plus souvent', modif: { cadence: -2 } },
      { description: '+1 totem', modif: { nombre: 1 } },
    ],
    eveils: [
      { nom: 'Racines profondes', description: 'Zone +30 %, reste 2 s de plus.', icone: 'block/mangrove_roots_side', ajout: { duree: 2 }, part: { zone: 0.3 } },
      { nom: 'Racines épineuses', description: 'Dégâts +4 à chaque pulsation.', icone: 'item/sweet_berries', ajout: { degats: 4 } },
    ],
  },
  esprit_protecteur: {
    son: 'allay',
    nom: 'Esprit protecteur',
    description: 'Un allay le suit, lui donne un bouclier et soigne les alliés blessés.',
    icone: 'block/amethyst_cluster',
    mecanique: { type: 'invocation', allie: 'allay' },
    // Dégâts = PV de bouclier donnés ; zone = part des PV soignée à un allié ; délai = entre deux soutiens
    stats: { ...BASE, degats: 3, cadence: 10, nombre: 1, duree: 9999, zone: 0.15 },
    libelles: { degats: 'Bouclier' },
    statsAffichees: ['degats', 'cadence', 'nombre'],
    niveaux: [
      { description: '+1 PV de bouclier', modif: { degats: 1 } },
      { description: 'Soigne les alliés de 10 % de plus', modif: { zone: 0.1 } },
      { description: 'Soutien 20 % plus fréquent', modif: { cadence: -2 } },
      { description: '+1 PV de bouclier', modif: { degats: 1 } },
      { description: '+1 allay', modif: { nombre: 1 } },
    ],
    eveils: [
      { nom: 'Chant protecteur', description: 'Bouclier +1 PV à chaque chant.', icone: 'block/note_block', ajout: { degats: 1 } },
      { nom: 'Chant rapide', description: 'Chante 20 % plus souvent.', icone: 'item/clock_00', part: { cadence: -0.2 } },
    ],
  },
  golems: {
    son: 'golem_invocation',
    nom: 'Golem de fer',
    description: 'Invoque un golem qui écrase les ennemis.',
    icone: 'block/carved_pumpkin',
    famille: 'allie',
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
    eveils: [
      { nom: 'Colosse', description: 'Dégâts +50 %, recul +50 %.', icone: 'block/iron_block', part: { degats: 0.5, recul: 0.5 } },
      { nom: 'Golem vigilant', description: 'Reste 4 s de plus, dégâts +15 %.', icone: 'block/poppy', ajout: { duree: 4 }, part: { degats: 0.15 } },
    ],
  },
  loups: {
    son: 'loup_aboiement',
    nom: 'Meute de loups',
    description: 'Des loups apprivoisés traquent et mordent les ennemis.',
    icone: 'item/bone',
    famille: 'allie',
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
    eveils: [
      { nom: 'Meute affamée', description: 'Dégâts +35 %.', icone: 'item/wolf_armor', part: { degats: 0.35 } },
      { nom: 'Loups des neiges', description: 'Les morsures ralentissent de 30 % pendant 1 s.', icone: 'block/powder_snow', etat: { type: 'ralenti', valeur: 0.3, duree: 1 } },
    ],
  },
  vex: {
    son: 'vex_invocation',
    nom: 'Vex',
    description: 'Des esprits volants foncent sur les ennemis à travers le décor.',
    icone: 'item/echo_shard',
    famille: 'allie',
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
    eveils: [
      { nom: 'Vex furieux', description: 'Dégâts +40 %.', icone: 'item/iron_sword', part: { degats: 0.4 } },
      { nom: 'Essaim de vex', description: '+2 vex.', icone: 'item/totem_of_undying', ajout: { nombre: 2 } },
    ],
  },
} satisfies Record<string, DefinitionArme>;

export type IdArme = keyof typeof definitions;

export const ARMES: Readonly<Record<IdArme, DefinitionArme>> = definitions;

export const IDS_ARMES = Object.keys(definitions) as IdArme[];
