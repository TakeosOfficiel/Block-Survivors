/**
 * Progression entre les parties : émeraudes gagnées en jouant, arbres de
 * talents (deux par personnage, un seul actif à la fois) et déblocages des
 * personnages et des arènes.
 *
 * Un talent s'achète rang par rang ; chaque rang ajoute ses effets. Une rangée
 * (palier) s'ouvre quand assez de rangs ont été achetés dans l'arbre.
 */
import type { IdPersonnage } from './personnages';
import type { DefinitionArbre, DefinitionDeblocage, DefinitionRecompenses } from './types';

/** Rangs à acheter dans l'arbre pour ouvrir chaque palier. */
export const SEUILS_PALIERS: readonly number[] = [0, 3, 6];

/** Coûts en émeraudes des rangs successifs, selon le palier. */
const COUTS = {
  bas: [5, 10, 15],
  milieu: [15, 25, 35],
  haut: [40, 70],
  unique: [80],
} as const;

export const ARBRES: Readonly<Record<IdPersonnage, readonly [DefinitionArbre, DefinitionArbre]>> = {
  guerrier: [
    {
      id: 'gladiateur',
      nom: 'Gladiateur',
      description: 'Frapper fort, frapper juste.',
      icone: 'item/diamond_sword',
      talents: [
        { id: 'lame', nom: 'Lame affûtée', icone: 'item/iron_sword', palier: 0, couts: COUTS.bas, effet: 'Dégâts +5 %', modif: { degats: 0.05 } },
        { id: 'precision', nom: 'Coup précis', icone: 'item/flint', palier: 0, couts: COUTS.bas, effet: 'Coups critiques +4 %', modif: { critique: 0.04 } },
        { id: 'maitre', nom: "Maître d'armes", icone: 'item/diamond_sword', palier: 1, couts: COUTS.milieu.slice(0, 2), effet: 'Épée : +1 rang au départ', arme: 'epee' },
        { id: 'fougue', nom: 'Fougue', icone: 'item/redstone', palier: 1, couts: COUTS.milieu, effet: 'Délai des armes −4 %', modif: { cadence: -0.04 } },
        { id: 'fatal', nom: 'Coup fatal', icone: 'item/netherite_sword', palier: 2, couts: COUTS.haut, effet: 'Dégâts critiques +0,25', modif: { degatsCritiques: 0.25 } },
        { id: 'tourbillon', nom: 'Tourbillon', icone: 'item/glowstone_dust', palier: 2, couts: COUTS.haut, effet: 'Zone +8 %', modif: { zone: 0.08 } },
      ],
    },
    {
      id: 'gardien',
      nom: 'Gardien',
      description: 'Encaisser, tenir, survivre.',
      icone: 'item/iron_chestplate',
      talents: [
        { id: 'robustesse', nom: 'Robustesse', icone: 'item/apple', palier: 0, couts: COUTS.bas, effet: '+1 cœur', modif: { pvMax: 2 } },
        { id: 'souffle', nom: 'Second souffle', icone: 'item/ghast_tear', palier: 0, couts: COUTS.bas, effet: '+0,05 PV par seconde', modif: { regeneration: 0.05 } },
        { id: 'bouclier', nom: 'Plates de fer', icone: 'item/iron_chestplate', palier: 1, couts: COUTS.milieu.slice(0, 2), effet: 'Dégâts reçus −1', modif: { armure: 1 } },
        { id: 'endurance', nom: 'Endurance', icone: 'item/sugar', palier: 1, couts: COUTS.milieu, effet: 'Vitesse +4 %', modif: { vitesse: 0.04 } },
        { id: 'immortel', nom: 'Seconde chance', icone: 'item/totem_of_undying', palier: 2, couts: COUTS.unique, effet: '+1 vie (totem au départ)', modif: { vies: 1 } },
        { id: 'forteresse', nom: 'Forteresse', icone: 'item/golden_apple', palier: 2, couts: COUTS.haut, effet: '+2 cœurs', modif: { pvMax: 4 } },
      ],
    },
  ],
  archer: [
    {
      id: 'elite',
      nom: "Tireuse d'élite",
      description: 'Chaque flèche compte.',
      icone: 'item/spectral_arrow',
      talents: [
        { id: 'pointes', nom: 'Pointes de fer', icone: 'item/arrow', palier: 0, couts: COUTS.bas, effet: 'Dégâts +5 %', modif: { degats: 0.05 } },
        { id: 'oeil', nom: 'Œil de faucon', icone: 'item/spyglass', palier: 0, couts: COUTS.bas, effet: 'Portée +8 %', modif: { portee: 0.08 } },
        { id: 'visee', nom: 'Visée', icone: 'item/flint', palier: 1, couts: COUTS.milieu, effet: 'Coups critiques +4 %', modif: { critique: 0.04 } },
        { id: 'arc', nom: 'Arc enchanté', icone: 'item/bow', palier: 1, couts: COUTS.milieu.slice(0, 2), effet: 'Arc : +1 rang au départ', arme: 'arc' },
        { id: 'tete', nom: 'En pleine tête', icone: 'item/spectral_arrow', palier: 2, couts: COUTS.haut, effet: 'Dégâts critiques +0,3', modif: { degatsCritiques: 0.3 } },
        { id: 'salve', nom: 'Salve', icone: 'item/crossbow_standby', palier: 2, couts: COUTS.unique, effet: '+1 projectile', modif: { projectiles: 1 } },
      ],
    },
    {
      id: 'rodeuse',
      nom: 'Rôdeuse',
      description: 'Rapide, insaisissable, prévoyante.',
      icone: 'item/rabbit_foot',
      talents: [
        { id: 'pas', nom: 'Pas légers', icone: 'item/sugar', palier: 0, couts: COUTS.bas, effet: 'Vitesse +4 %', modif: { vitesse: 0.04 } },
        { id: 'flair', nom: 'Flair', icone: 'item/ender_pearl', palier: 0, couts: COUTS.bas, effet: 'Rayon de collecte +15 %', modif: { collecte: 0.15 } },
        { id: 'ruse', nom: 'Ruse', icone: 'item/compass_16', palier: 1, couts: COUTS.milieu.slice(0, 2), effet: '+1 relance des cartes', relances: 1 },
        { id: 'cadence', nom: 'Main leste', icone: 'item/redstone', palier: 1, couts: COUTS.milieu, effet: 'Délai des armes −4 %', modif: { cadence: -0.04 } },
        { id: 'baies', nom: 'Baies sauvages', icone: 'item/sweet_berries', palier: 2, couts: COUTS.haut, effet: '+2 cœurs', modif: { pvMax: 4 } },
        { id: 'sous_bois', nom: 'Sous-bois', icone: 'item/glow_berries', palier: 2, couts: COUTS.haut, effet: '+0,1 PV par seconde', modif: { regeneration: 0.1 } },
      ],
    },
  ],
  mage: [
    {
      id: 'pyromancien',
      nom: 'Pyromancien',
      description: 'Tout brûle.',
      icone: 'item/fire_charge',
      talents: [
        { id: 'braise', nom: 'Braises', icone: 'item/blaze_powder', palier: 0, couts: COUTS.bas, effet: 'Dégâts +5 %', modif: { degats: 0.05 } },
        { id: 'souffle', nom: 'Souffle ardent', icone: 'item/glowstone_dust', palier: 0, couts: COUTS.bas, effet: 'Zone +6 %', modif: { zone: 0.06 } },
        { id: 'boules', nom: 'Charges de feu', icone: 'item/fire_charge', palier: 1, couts: COUTS.milieu.slice(0, 2), effet: 'Boules de feu : +1 rang au départ', arme: 'boules_feu' },
        { id: 'combustion', nom: 'Combustion', icone: 'item/magma_cream', palier: 1, couts: COUTS.milieu, effet: 'Coups critiques +4 %', modif: { critique: 0.04 } },
        { id: 'brasier', nom: 'Brasier', icone: 'item/blaze_rod', palier: 2, couts: COUTS.haut, effet: 'Dégâts critiques +0,3', modif: { degatsCritiques: 0.3 } },
        { id: 'meteores', nom: 'Pluie de feu', icone: 'item/dragon_breath', palier: 2, couts: COUTS.unique, effet: '+1 projectile', modif: { projectiles: 1 } },
      ],
    },
    {
      id: 'arcaniste',
      nom: 'Arcaniste',
      description: 'La foudre et le savoir.',
      icone: 'item/enchanted_book',
      talents: [
        { id: 'concentration', nom: 'Concentration', icone: 'item/amethyst_shard', palier: 0, couts: COUTS.bas, effet: 'Délai des armes −4 %', modif: { cadence: -0.04 } },
        { id: 'vigueur', nom: 'Vigueur', icone: 'item/glistering_melon_slice', palier: 0, couts: COUTS.bas, effet: '+1 cœur', modif: { pvMax: 2 } },
        { id: 'meditation', nom: 'Méditation', icone: 'item/ghast_tear', palier: 1, couts: COUTS.milieu, effet: '+0,08 PV par seconde', modif: { regeneration: 0.08 } },
        { id: 'savoir', nom: 'Savoir', icone: 'item/book', palier: 1, couts: COUTS.milieu.slice(0, 2), effet: '+1 relance des cartes', relances: 1 },
        { id: 'tempete', nom: 'Appel de la foudre', icone: 'block/lightning_rod', palier: 2, couts: COUTS.unique, effet: 'Commence avec les Éclairs', arme: 'eclairs' },
        { id: 'egide', nom: 'Égide', icone: 'item/experience_bottle', palier: 2, couts: COUTS.haut, effet: 'Dégâts reçus −1', modif: { armure: 1 } },
      ],
    },
  ],
  invocateur: [
    {
      id: 'maitre_chien',
      nom: 'Maître-chien',
      description: 'Une meute toujours plus grande.',
      icone: 'item/bone',
      talents: [
        { id: 'crocs', nom: 'Crocs aiguisés', icone: 'item/bone', palier: 0, couts: COUTS.bas, effet: 'Dégâts +5 %', modif: { degats: 0.05 } },
        { id: 'piste', nom: 'Sur la piste', icone: 'item/sugar', palier: 0, couts: COUTS.bas, effet: 'Vitesse +4 %', modif: { vitesse: 0.04 } },
        { id: 'dressage', nom: 'Dressage', icone: 'item/lead', palier: 1, couts: COUTS.milieu.slice(0, 2), effet: 'Meute de loups : +1 rang au départ', arme: 'loups' },
        { id: 'instinct', nom: 'Instinct', icone: 'item/flint', palier: 1, couts: COUTS.milieu, effet: 'Coups critiques +4 %', modif: { critique: 0.04 } },
        { id: 'alpha', nom: 'Hurlement', icone: 'item/goat_horn', palier: 2, couts: COUTS.unique, effet: '+1 allié et +1 projectile', modif: { projectiles: 1 } },
        { id: 'jugulaire', nom: 'À la gorge', icone: 'item/wolf_armor', palier: 2, couts: COUTS.haut, effet: 'Dégâts critiques +0,3', modif: { degatsCritiques: 0.3 } },
      ],
    },
    {
      id: 'evocateur',
      nom: 'Évocateur',
      description: 'Esprits et golems à son service.',
      icone: 'item/echo_shard',
      talents: [
        { id: 'robustesse', nom: 'Robustesse', icone: 'item/iron_ingot', palier: 0, couts: COUTS.bas, effet: '+1 cœur', modif: { pvMax: 2 } },
        { id: 'incantation', nom: 'Incantation', icone: 'item/amethyst_shard', palier: 0, couts: COUTS.bas, effet: 'Délai des armes −4 %', modif: { cadence: -0.04 } },
        { id: 'vex', nom: 'Esprits liés', icone: 'item/echo_shard', palier: 1, couts: COUTS.milieu.slice(0, 2), effet: 'Vex : +1 rang au départ', arme: 'vex' },
        { id: 'protection', nom: 'Garde rapprochée', icone: 'item/iron_chestplate', palier: 1, couts: COUTS.milieu.slice(0, 2), effet: 'Dégâts reçus −1', modif: { armure: 1 } },
        { id: 'golem', nom: 'Gardien de fer', icone: 'block/carved_pumpkin', palier: 2, couts: COUTS.unique, effet: 'Commence avec le Golem de fer', arme: 'golems' },
        { id: 'longevite', nom: 'Longévité', icone: 'item/glow_berries', palier: 2, couts: COUTS.haut, effet: '+0,1 PV par seconde', modif: { regeneration: 0.1 } },
      ],
    },
  ],
};

/** Émeraudes gagnées en fin de partie. */
export const RECOMPENSES: DefinitionRecompenses = {
  icone: 'item/emerald',
  parSecondes: 20,     // 1 émeraude toutes les 20 s survécues
  parTues: 40,         // 1 émeraude tous les 40 ennemis tués
  parBoss: 10,
  victoire: 25,
};

/** Ce qui n'est pas disponible dès la première partie, et comment l'obtenir. */
export const DEBLOCAGES: readonly DefinitionDeblocage[] = [
  { type: 'arene', id: 'mine', nom: 'Mine souterraine', condition: { type: 'survie', duree: 300, arene: 'prairie' }, texte: 'Survivre 5:00 dans la Prairie' },
  { type: 'arene', id: 'nether', nom: 'Zone volcanique', condition: { type: 'survie', duree: 300, arene: 'mine' }, texte: 'Survivre 5:00 dans la Mine' },
];
