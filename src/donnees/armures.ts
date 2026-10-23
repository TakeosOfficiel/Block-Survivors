/**
 * Armures de la Collection : une par monde, 4 pièces gardées d'une partie à
 * l'autre (progression/collection.ts). Une pièce s'obtient à chaque partie
 * gagnée dans son monde ; elles se portent avant la partie (Apparence), même
 * mélangées. Chaque pièce donne un petit bonus, les 4 du même monde un bonus
 * d'ensemble. Textures du jeu : entity/equipment/humanoid(_leggings)/<matériau>.
 */
import type { CheminTexture, DefinitionArmure, IdMonde, MateriauArmure, PieceArmure } from './types';

export const PIECES_ARMURE: readonly PieceArmure[] = ['casque', 'plastron', 'jambieres', 'bottes'];
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
};

export const ARMURES: Readonly<Record<IdMonde, DefinitionArmure>> = {
  prairie: {
    nom: 'Armure du villageois',
    monde: 'prairie',
    materiau: 'chainmail',
    pieces: {
      casque: { nom: 'Coiffe du villageois', description: 'Régénération +0,05 PV/s.', modif: { regeneration: 0.05 } },
      plastron: { nom: 'Cotte du villageois', description: 'Santé maximale +3 PV.', modif: { pvMax: 3 } },
      jambieres: { nom: 'Jambières du villageois', description: 'Pommes +10 %.', modif: { soinsObjets: 0.1 } },
      bottes: { nom: 'Bottes du villageois', description: 'Vitesse +3 %.', modif: { vitesse: 0.03 } },
    },
    ensemble: { description: 'Ensemble complet : santé maximale +6 PV, régénération +0,15 PV/s.', modif: { pvMax: 6, regeneration: 0.15 } },
  },
  mine: {
    nom: 'Armure du mineur',
    monde: 'mine',
    materiau: 'iron',
    pieces: {
      casque: { nom: 'Casque du mineur', description: 'Chance de critique +2 %.', modif: { critique: 0.02 } },
      plastron: { nom: 'Plastron du mineur', description: 'Dégâts reçus −3 %.', modif: { degatsRecus: -0.03 } },
      jambieres: { nom: 'Jambières du mineur', description: 'Dégâts contre les arthropodes +10 %.', modif: { degatsArthropodes: 0.1 } },
      bottes: { nom: 'Bottes du mineur', description: 'Vitesse +3 %.', modif: { vitesse: 0.03 } },
    },
    ensemble: { description: 'Ensemble complet : chance de critique +5 %, dégâts critiques +20 %.', modif: { critique: 0.05, degatsCritiques: 0.2 } },
  },
  nether: {
    nom: 'Armure du seigneur du Nether',
    monde: 'nether',
    materiau: 'netherite',
    pieces: {
      casque: { nom: 'Heaume du seigneur', description: 'Dégâts contre le Nether +10 %.', modif: { degatsNether: 0.1 } },
      plastron: { nom: 'Cuirasse du seigneur', description: 'Dégâts reçus −4 %.', modif: { degatsRecus: -0.04 } },
      jambieres: { nom: 'Grèves du seigneur', description: '4 % des coups d\'arme enflamment.', modif: { chanceBrulure: 0.04 } },
      bottes: { nom: 'Solerets du seigneur', description: 'Dégâts du magma −50 %.', modif: { degatsSol: -0.5 } },
    },
    ensemble: { description: 'Ensemble complet : insensible au magma, dégâts +6 %.', modif: { degatsSol: -0.5, degats: 0.06 } },
  },
  champignons: {
    nom: 'Armure du mycélien',
    monde: 'champignons',
    materiau: 'leather',
    teinte: 0xb0402c,
    pieces: {
      casque: { nom: 'Chapeau du mycélien', description: 'Régénération +0,05 PV/s.', modif: { regeneration: 0.05 } },
      plastron: { nom: 'Tunique du mycélien', description: 'Santé maximale +3 PV.', modif: { pvMax: 3 } },
      jambieres: { nom: 'Pantalon du mycélien', description: 'Zone des armes +3 %.', modif: { zone: 0.03 } },
      bottes: { nom: 'Bottes du mycélien', description: 'Vitesse +3 %.', modif: { vitesse: 0.03 } },
    },
    ensemble: { description: 'Ensemble complet : régénération +0,2 PV/s, zone +6 %.', modif: { regeneration: 0.2, zone: 0.06 } },
  },
};

/** Texture de la pièce : jambières sur la seconde planche du jeu. */
export function textureArmure(materiau: MateriauArmure, piece: PieceArmure): CheminTexture {
  return piece === 'jambieres' ? `entity/equipment/humanoid_leggings/${materiau}` : `entity/equipment/humanoid/${materiau}`;
}

/** Icône d'inventaire de la pièce (item/<matériau>_<pièce>). */
export function iconeArmure(materiau: MateriauArmure, piece: PieceArmure): CheminTexture {
  const nom = { casque: 'helmet', plastron: 'chestplate', jambieres: 'leggings', bottes: 'boots' }[piece];
  return `item/${materiau}_${nom}`;
}
