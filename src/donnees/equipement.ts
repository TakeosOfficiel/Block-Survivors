/**
 * Équipement de partie, trouvé dans les coffres : 2 bagues et 1 amulette au
 * plus, perdus à la fin de la partie. Chaque monde a ses objets (et sa
 * légendaire), en plus de ceux de tous les mondes. Un objet déjà porté peut
 * être renforcé (niveaux 2 et 3, voir REGLES_EQUIPEMENT.niveaux) ; les
 * légendaires ne se renforcent pas.
 *
 * Effets simples : `modif` (cumulés comme les passifs, × facteur du niveau).
 * Effets propres : `effet` (combat/talents/objets.ts).
 * Tous les objets doivent servir aux 4 classes.
 */
import type { DefinitionObjetEquipement, ReglesEquipement, RareteObjet } from './types';

const P = (x: number) => `${Math.round(x * 100)} %`;
const N = (x: number) => new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 1 }).format(x);

const definitions = {
  // --- Tous les mondes ------------------------------------------------------
  anneau_fer: {
    nom: 'Anneau de fer', emplacement: 'bague', rarete: 'commune', monde: null,
    description: (f) => `Dégâts des armes +${P(0.08 * f)}.`,
    modif: { degats: 0.08 },
    icone: { forme: 'bague', metal: 'fer', gemme: 0xc8c8c8 },
  },
  anneau_or: {
    nom: 'Anneau d\'or', emplacement: 'bague', rarete: 'commune', monde: null,
    description: (f) => `Rayon de collecte +${P(0.25 * f)}, pommes +${P(0.25 * f)}.`,
    modif: { collecte: 0.25, soinsObjets: 0.25 },
    icone: { forme: 'bague', metal: 'or', gemme: 0xffd84a },
  },
  anneau_cuivre: {
    nom: 'Anneau de cuivre', emplacement: 'bague', rarete: 'commune', monde: null,
    description: (f) => `Vitesse +${P(0.06 * f)}.`,
    modif: { vitesse: 0.06 },
    icone: { forme: 'bague', metal: 'cuivre', gemme: 0x4fd0b0 },
  },
  anneau_lapis: {
    nom: 'Anneau de lapis', emplacement: 'bague', rarete: 'rare', monde: null,
    description: (f) => `Délai des armes −${P(0.07 * f)}.`,
    modif: { cadence: -0.07 },
    icone: { forme: 'bague', metal: 'or', gemme: 0x2f5fe0 },
  },
  anneau_dragon: {
    nom: 'Anneau du dragon', emplacement: 'bague', rarete: 'epique', monde: null,
    description: (f) => `Dégâts +${P(0.1 * f)}, zone +${P(0.08 * f)}.`,
    modif: { degats: 0.1, zone: 0.08 },
    icone: { forme: 'bague', metal: 'netherite', gemme: 0xc050ff },
  },
  pendentif_eclaireur: {
    nom: 'Pendentif d\'éclaireur', emplacement: 'amulette', rarete: 'commune', monde: null,
    description: (f) => `Portée des armes +${P(0.12 * f)}.`,
    modif: { portee: 0.12 },
    icone: { forme: 'pendentif', metal: 'fer', gemme: 0x8fd0ff },
  },
  pendentif_vie: {
    nom: 'Pendentif de vie', emplacement: 'amulette', rarete: 'rare', monde: null,
    description: (f) => `Santé maximale +${N(4 * f)} PV, régénération +${N(0.1 * f)} PV/s.`,
    modif: { pvMax: 4, regeneration: 0.1 },
    icone: { forme: 'pendentif', metal: 'or', gemme: 0xff4a5a },
  },
  amulette_neant: {
    nom: 'Amulette du néant', emplacement: 'amulette', rarete: 'epique', monde: null,
    description: (f) => `Dégâts +${P(0.12 * f)}, délai des armes −${P(0.05 * f)}.`,
    modif: { degats: 0.12, cadence: -0.05 },
    icone: { forme: 'pendentif', metal: 'netherite', gemme: 0x2a1a4a },
  },

  // --- Prairie du village ---------------------------------------------------
  bague_laboureur: {
    nom: 'Bague du laboureur', emplacement: 'bague', rarete: 'commune', monde: 'prairie',
    description: (f) => `Régénération +${N(0.25 * f)} PV/s.`,
    modif: { regeneration: 0.25 },
    icone: { forme: 'bague', metal: 'cuivre', gemme: 0x7fd040 },
  },
  bague_chasseur: {
    nom: 'Bague du chasseur', emplacement: 'bague', rarete: 'rare', monde: 'prairie',
    description: (f) => `Dégâts contre les morts-vivants (zombies, squelettes) +${P(0.2 * f)}.`,
    modif: { degatsMortsVivants: 0.2 },
    icone: { forme: 'bague', metal: 'fer', gemme: 0xf0e8c0 },
  },
  collier_forgeron: {
    nom: 'Collier du forgeron', emplacement: 'amulette', rarete: 'rare', monde: 'prairie',
    description: (f) => `Chaque coup reçu fait ${N(1 * f)} dégât de moins.`,
    modif: { armure: 1 },
    icone: { forme: 'pendentif', metal: 'fer', gemme: 0x606060 },
  },
  medaillon_cloche: {
    nom: 'Médaillon de la cloche', emplacement: 'amulette', rarete: 'epique', monde: 'prairie',
    description: (f) => `Toutes les ${N(12 / f)} s, la cloche sonne : les ennemis proches sont repoussés et étourdis.`,
    effet: { type: 'cloche', intervalle: 12, rayon: 5, force: 10, etourdi: 1 },
    icone: { forme: 'pendentif', metal: 'or', gemme: 0xffc020 },
  },
  amulette_heros: {
    nom: 'Amulette du héros du village', emplacement: 'amulette', rarete: 'legendaire', monde: 'prairie',
    description: () => 'Un totem d\'immortalité en plus et +10 PV de santé maximale.',
    modif: { vies: 1, pvMax: 10 },
    icone: { forme: 'pendentif', metal: 'or', gemme: 0x30e070 },
  },

  // --- Mine souterraine -----------------------------------------------------
  bague_arpenteur: {
    nom: 'Bague de l\'arpenteur', emplacement: 'bague', rarete: 'commune', monde: 'mine',
    description: (f) => `Dégâts contre les arthropodes (araignées, poissons d'argent) +${P(0.3 * f)}.`,
    modif: { degatsArthropodes: 0.3 },
    icone: { forme: 'bague', metal: 'fer', gemme: 0xb04040 },
  },
  bague_amethyste: {
    nom: 'Bague d\'améthyste', emplacement: 'bague', rarete: 'rare', monde: 'mine',
    description: (f) => `Chance de coup critique +${P(0.06 * f)}.`,
    modif: { critique: 0.06 },
    icone: { forme: 'bague', metal: 'or', gemme: 0xa060e0 },
  },
  bague_quartz: {
    nom: 'Bague de quartz', emplacement: 'bague', rarete: 'rare', monde: 'mine',
    description: (f) => `Dégâts des coups critiques +${P(0.25 * f)}.`,
    modif: { degatsCritiques: 0.25 },
    icone: { forme: 'bague', metal: 'fer', gemme: 0xf4f0ea },
  },
  collier_toile: {
    nom: 'Collier de toile', emplacement: 'amulette', rarete: 'rare', monde: 'mine',
    description: (f) => `Ralentissements +${P(0.4 * f)} plus longs, dégâts contre les ennemis ralentis +${P(0.15 * f)}.`,
    modif: { dureeRalenti: 0.4, degatsRalentis: 0.15 },
    icone: { forme: 'pendentif', metal: 'fer', gemme: 0xe8e8f0 },
  },
  coeur_filon: {
    nom: 'Cœur du filon', emplacement: 'amulette', rarete: 'legendaire', monde: 'mine',
    description: () => '8 % des ennemis tués lâchent un éclat de diamant qui explose autour d\'eux.',
    effet: { type: 'filon', chance: 0.08, rayon: 2.4, degats: 30 },
    icone: { forme: 'pendentif', metal: 'or', gemme: 0x4ae8e0 },
  },

  // --- Zone volcanique ------------------------------------------------------
  bague_forteresse: {
    nom: 'Bague de la forteresse', emplacement: 'bague', rarete: 'commune', monde: 'nether',
    description: (f) => `Dégâts contre les créatures du Nether +${P(0.2 * f)}.`,
    modif: { degatsNether: 0.2 },
    icone: { forme: 'bague', metal: 'netherite', gemme: 0xa02020 },
  },
  bague_magma: {
    nom: 'Bague de magma', emplacement: 'bague', rarete: 'rare', monde: 'nether',
    description: (f) => `${P(0.12 * f)} des coups d'arme enflamment l'ennemi.`,
    modif: { chanceBrulure: 0.12 },
    icone: { forme: 'bague', metal: 'cuivre', gemme: 0xff7020 },
  },
  pendentif_basalte: {
    nom: 'Pendentif de basalte', emplacement: 'amulette', rarete: 'rare', monde: 'nether',
    description: (f) => `Dégâts reçus −${P(0.08 * f)}.`,
    modif: { degatsRecus: -0.08 },
    icone: { forme: 'pendentif', metal: 'netherite', gemme: 0x505058 },
  },
  amulette_bastion: {
    nom: 'Amulette du bastion', emplacement: 'amulette', rarete: 'epique', monde: 'nether',
    description: (f) => `Sous la moitié de vos PV, dégâts +${P(0.2 * f)}.`,
    effet: { type: 'bastion', seuil: 0.5, bonus: 0.2 },
    icone: { forme: 'pendentif', metal: 'or', gemme: 0xffb000 },
  },
  coeur_magma: {
    nom: 'Cœur de magma', emplacement: 'amulette', rarete: 'legendaire', monde: 'nether',
    description: () => 'Perdre 20 % de vos PV d\'un coup déclenche une nova de feu et 2 s d\'invulnérabilité (une fois toutes les 30 s).',
    effet: { type: 'magma', seuil: 0.2, rayon: 4.5, degats: 40, protection: 2, recharge: 30 },
    icone: { forme: 'pendentif', metal: 'netherite', gemme: 0xff4010 },
  },

  // --- Jardin des champignons -----------------------------------------------
  bague_mycelium: {
    nom: 'Bague de mycélium', emplacement: 'bague', rarete: 'commune', monde: 'champignons',
    description: (f) => `Régénération +${N(0.15 * f)} PV/s, vitesse +${P(0.04 * f)}.`,
    modif: { regeneration: 0.15, vitesse: 0.04 },
    icone: { forme: 'bague', metal: 'cuivre', gemme: 0x9a7aa0 },
  },
  bague_cepe: {
    nom: 'Bague du cèpe', emplacement: 'bague', rarete: 'rare', monde: 'champignons',
    description: (f) => `Santé maximale +${N(6 * f)} PV.`,
    modif: { pvMax: 6 },
    icone: { forme: 'bague', metal: 'or', gemme: 0xa06a30 },
  },
  bague_morille: {
    nom: 'Bague de la morille', emplacement: 'bague', rarete: 'rare', monde: 'champignons',
    description: (f) => `Zone des armes +${P(0.12 * f)}.`,
    modif: { zone: 0.12 },
    icone: { forme: 'bague', metal: 'fer', gemme: 0xd8b070 },
  },
  amulette_spores: {
    nom: 'Amulette de spores', emplacement: 'amulette', rarete: 'epique', monde: 'champignons',
    description: (f) => `${P(Math.min(0.5, 0.2 * f))} des ennemis tués libèrent des spores qui empoisonnent les ennemis proches.`,
    effet: { type: 'spores', chance: 0.2, rayon: 2.5, poison: 4, duree: 3 },
    icone: { forme: 'pendentif', metal: 'cuivre', gemme: 0x70d050 },
  },
  couronne_champignon: {
    nom: 'Couronne du champignon géant', emplacement: 'amulette', rarete: 'legendaire', monde: 'champignons',
    description: () => 'Toutes les 25 s, un champignon géant éclate autour de vous : gros dégâts et poison.',
    effet: { type: 'couronne', intervalle: 25, rayon: 5, degats: 50, poison: 5, duree: 4 },
    icone: { forme: 'pendentif', metal: 'or', gemme: 0xe02828 },
  },
} satisfies Record<string, DefinitionObjetEquipement>;

export type IdObjetEquipement = keyof typeof definitions;
export const OBJETS_EQUIPEMENT: Readonly<Record<IdObjetEquipement, DefinitionObjetEquipement>> = definitions;
export const IDS_OBJETS_EQUIPEMENT = Object.keys(definitions) as IdObjetEquipement[];

/** Couleur et nom de chaque rareté (cartes, HUD, Collection). */
export const RARETES_OBJETS: Readonly<Record<RareteObjet, { readonly nom: string }>> = {
  commune: { nom: 'Commune' },
  rare: { nom: 'Rare' },
  epique: { nom: 'Épique' },
  legendaire: { nom: 'Légendaire' },
};

export const REGLES_EQUIPEMENT: ReglesEquipement = {
  bagues: 2,
  amulettes: 1,
  niveaux: [1, 1.5, 2],
  propositions: 3,
  // Coffre en bois : de moins en moins de communes, de plus en plus d'épiques, jamais de légendaire
  bois: {
    poids: { commune: 60, rare: 32, epique: 8 },
    parCoffre: { commune: -5, rare: 2, epique: 3 },
    minimum: { commune: 15 },
    maximum: { rare: 50, epique: 35 },
  },
  // Coffre de l'Ender (boss) : rare au moins ; la légendaire du monde est toujours proposée tant qu'on ne l'a pas
  ender: { poids: { rare: 45, epique: 40, legendaire: 15 } },
  chanceRenfort: 0.25,
  emeraudesConsolation: 15,
  // « Ne rien prendre » : toujours proposé, bien moins que le prix d'un coffre
  emeraudesRefus: { bois: 3, ender: 6 },
};
