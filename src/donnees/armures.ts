/**
 * Armures de la Collection : une par monde, 7 pièces gardées d'une partie à
 * l'autre (progression/collection.ts). Les boss en lâchent parfois (BUTIN_BOSS :
 * chances et pièces selon le rang du boss dans la partie), jamais en double ;
 * elles se portent avant la partie (Apparence), même mélangées. Chaque pièce
 * donne un petit bonus, les 7 du même monde un bonus d'ensemble.
 * Casque, plastron, jambières, bottes : textures du jeu
 * (entity/equipment/humanoid(_leggings)/<matériau>) ; bouclier, cape et
 * talisman : dessinés au lancement aux couleurs du monde (ui/iconesObjets.ts).
 */
import type { ButinBoss, CheminTexture, DefinitionArmure, IdMonde, MateriauArmure, PieceArmure, PieceArmureJeu } from './types';

export const PIECES_ARMURE: readonly PieceArmure[] = ['casque', 'plastron', 'jambieres', 'bottes', 'bouclier', 'cape', 'talisman'];
export const PIECES_JEU: readonly PieceArmureJeu[] = ['casque', 'plastron', 'jambieres', 'bottes'];

export function estPieceJeu(piece: PieceArmure): piece is PieceArmureJeu {
  return (PIECES_JEU as readonly PieceArmure[]).includes(piece);
}

/**
 * Butin des boss : le 1er donne surtout le bas de l'armure, le 2e surtout le plastron et les pièces
 * rares, les suivants (mode sans fin) de tout, plus souvent. Une pièce déjà gagnée n'est jamais redonnée.
 */
export const BUTIN_BOSS: readonly ButinBoss[] = [
  { chance: 0.25, chanceSeconde: 0.08, poids: { bottes: 30, jambieres: 30, casque: 25, bouclier: 10, cape: 5 } },
  { chance: 0.3, chanceSeconde: 0.1, poids: { plastron: 30, bouclier: 20, cape: 20, casque: 10, talisman: 10, jambieres: 5, bottes: 5 } },
  { chance: 0.35, chanceSeconde: 0.12, poids: { plastron: 15, casque: 10, jambieres: 10, bottes: 10, bouclier: 15, cape: 20, talisman: 20 } },
];
export const MONDES: readonly IdMonde[] = ['prairie', 'mine', 'nether', 'champignons'];

export const NOMS_MONDES: Readonly<Record<IdMonde, string>> = {
  prairie: 'Prairie du village',
  mine: 'Mine souterraine',
  nether: 'Zone volcanique',
  champignons: 'Jardin des champignons',
};

export const NOMS_PIECES: Readonly<Record<PieceArmure, string>> = {
  casque: 'Casque',
  plastron: 'Plastron',
  jambieres: 'Jambières',
  bottes: 'Bottes',
  bouclier: 'Bouclier',
  cape: 'Cape',
  talisman: 'Talisman',
};

export const ARMURES: Readonly<Record<IdMonde, DefinitionArmure>> = {
  prairie: {
    nom: 'Armure du villageois',
    monde: 'prairie',
    materiau: 'chainmail',
    couleurs: { champ: 0x2f8a3a, bordure: 0x7a5a30, embleme: 0xf0d060 },
    pieces: {
      casque: { nom: 'Coiffe du villageois', description: 'Régénération +0,05 PV/s.', modif: { regeneration: 0.05 } },
      plastron: { nom: 'Cotte du villageois', description: 'Santé maximale +3 PV.', modif: { pvMax: 3 } },
      jambieres: { nom: 'Jambières du villageois', description: 'Pommes +10 %.', modif: { soinsObjets: 0.1 } },
      bottes: { nom: 'Bottes du villageois', description: 'Vitesse +3 %.', modif: { vitesse: 0.03 } },
      bouclier: { nom: 'Bouclier du village', description: 'Dégâts reçus −3 %.', modif: { degatsRecus: -0.03 } },
      cape: { nom: 'Cape du berger', description: 'Rayon de collecte +10 %.', modif: { collecte: 0.1 } },
      talisman: { nom: 'Cloche du village', description: 'Santé maximale +4 PV.', modif: { pvMax: 4 } },
    },
    ensemble: { description: 'Ensemble complet : santé maximale +6 PV, régénération +0,15 PV/s.', modif: { pvMax: 6, regeneration: 0.15 } },
  },
  mine: {
    nom: 'Armure du mineur',
    monde: 'mine',
    materiau: 'iron',
    couleurs: { champ: 0x4a4e58, bordure: 0x9a9aa4, embleme: 0xa060e0 },
    pieces: {
      casque: { nom: 'Casque du mineur', description: 'Chance de critique +2 %.', modif: { critique: 0.02 } },
      plastron: { nom: 'Plastron du mineur', description: 'Dégâts reçus −3 %.', modif: { degatsRecus: -0.03 } },
      jambieres: { nom: 'Jambières du mineur', description: 'Dégâts contre les arthropodes +10 %.', modif: { degatsArthropodes: 0.1 } },
      bottes: { nom: 'Bottes du mineur', description: 'Vitesse +3 %.', modif: { vitesse: 0.03 } },
      bouclier: { nom: 'Bouclier du mineur', description: 'Dégâts reçus −3 %.', modif: { degatsRecus: -0.03 } },
      cape: { nom: 'Cape des profondeurs', description: 'Dégâts critiques +10 %.', modif: { degatsCritiques: 0.1 } },
      talisman: { nom: 'Géode d\'améthyste', description: 'Chance de critique +3 %.', modif: { critique: 0.03 } },
    },
    ensemble: { description: 'Ensemble complet : chance de critique +5 %, dégâts critiques +20 %.', modif: { critique: 0.05, degatsCritiques: 0.2 } },
  },
  nether: {
    nom: 'Armure du seigneur du Nether',
    monde: 'nether',
    materiau: 'netherite',
    couleurs: { champ: 0x5a1414, bordure: 0x2a2424, embleme: 0xffa020 },
    pieces: {
      casque: { nom: 'Heaume du seigneur', description: 'Dégâts contre le Nether +10 %.', modif: { degatsNether: 0.1 } },
      plastron: { nom: 'Cuirasse du seigneur', description: 'Dégâts reçus −4 %.', modif: { degatsRecus: -0.04 } },
      jambieres: { nom: 'Grèves du seigneur', description: '4 % des coups d\'arme enflamment.', modif: { chanceBrulure: 0.04 } },
      bottes: { nom: 'Solerets du seigneur', description: 'Dégâts du magma −50 %.', modif: { degatsSol: -0.5 } },
      bouclier: { nom: 'Pavois du seigneur', description: 'Dégâts reçus −4 %.', modif: { degatsRecus: -0.04 } },
      cape: { nom: 'Cape de cendres', description: 'Dégâts +3 %.', modif: { degats: 0.03 } },
      talisman: { nom: 'Larme de ghast', description: 'Régénération +0,15 PV/s.', modif: { regeneration: 0.15 } },
    },
    ensemble: { description: 'Ensemble complet : insensible au magma, dégâts +6 %.', modif: { degatsSol: -0.5, degats: 0.06 } },
  },
  champignons: {
    nom: 'Armure du mycélien',
    monde: 'champignons',
    materiau: 'leather',
    teinte: 0xb0402c,
    couleurs: { champ: 0xc02a20, bordure: 0xe8dcc8, embleme: 0xf4f0e8 },
    pieces: {
      casque: { nom: 'Chapeau du mycélien', description: 'Régénération +0,05 PV/s.', modif: { regeneration: 0.05 } },
      plastron: { nom: 'Tunique du mycélien', description: 'Santé maximale +3 PV.', modif: { pvMax: 3 } },
      jambieres: { nom: 'Pantalon du mycélien', description: 'Zone des armes +3 %.', modif: { zone: 0.03 } },
      bottes: { nom: 'Bottes du mycélien', description: 'Vitesse +3 %.', modif: { vitesse: 0.03 } },
      bouclier: { nom: 'Chapeau-bouclier', description: 'Dégâts reçus −3 %.', modif: { degatsRecus: -0.03 } },
      cape: { nom: 'Cape de spores', description: 'Vitesse +3 %.', modif: { vitesse: 0.03 } },
      talisman: { nom: 'Spore dorée', description: 'Dégâts du poison +15 %.', modif: { degatsPoison: 0.15 } },
    },
    ensemble: { description: 'Ensemble complet : régénération +0,2 PV/s, zone +6 %.', modif: { regeneration: 0.2, zone: 0.06 } },
  },
};

/** Texture de la pièce : jambières sur la seconde planche du jeu. */
export function textureArmure(materiau: MateriauArmure, piece: PieceArmureJeu): CheminTexture {
  return piece === 'jambieres' ? `entity/equipment/humanoid_leggings/${materiau}` : `entity/equipment/humanoid/${materiau}`;
}

/** Icône d'inventaire de la pièce (item/<matériau>_<pièce>). */
export function iconeArmure(materiau: MateriauArmure, piece: PieceArmureJeu): CheminTexture {
  const nom = { casque: 'helmet', plastron: 'chestplate', jambieres: 'leggings', bottes: 'boots' }[piece];
  return `item/${materiau}_${nom}`;
}

/** Où chercher une pièce (Collection, Apparence) : le boss qui la lâche le plus souvent. */
export function provenancePiece(monde: IdMonde, piece: PieceArmure): string {
  const premier = BUTIN_BOSS[0].poids[piece] ?? 0;
  const second = BUTIN_BOSS[1].poids[piece] ?? 0;
  const boss = premier >= second ? 'le 1er boss' : premier === 0 ? 'le 2e boss (ou les suivants, en mode sans fin)' : 'le 2e boss';
  return `Lâchée parfois par ${boss} de « ${NOMS_MONDES[monde]} ».`;
}
