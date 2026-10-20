/**
 * Types de blocs. La clé reprend l'identifiant officiel Minecraft.
 * Ajouter un bloc = ajouter une entrée.
 */
import type { CheminTexture, DefinitionBloc } from './types';

/** Même texture sur les six faces. */
const partout = (texture: CheminTexture) => ({ haut: texture, cotes: texture, bas: texture });

/** Teintes de biome « plaines » appliquées par le jeu aux textures grises. */
const TEINTE_HERBE = 0x91bd59;
const TEINTE_FEUILLAGE = 0x77ab2f;
const TEINTE_EAU = 0x3f76e4;

const definitions = {
  // --- Prairie ---------------------------------------------------------------
  grass_block: {
    nom: "Bloc d'herbe",
    faces: { haut: 'block/grass_block_top', cotes: 'block/grass_block_side', bas: 'block/dirt' },
    teinteHaut: TEINTE_HERBE,
    rotationAleatoire: true,
    couleurSecours: 0x91bd59,
  },
  dirt: { nom: 'Terre', faces: partout('block/dirt'), couleurSecours: 0x866043 },
  dirt_path: {
    nom: 'Chemin de terre',
    faces: { haut: 'block/dirt_path_top', cotes: 'block/dirt_path_side', bas: 'block/dirt' },
    couleurSecours: 0x947a45,
  },
  gravel: { nom: 'Gravier', faces: partout('block/gravel'), rotationAleatoire: true, couleurSecours: 0x857f7d },
  cobblestone: { nom: 'Pierre', faces: partout('block/cobblestone'), couleurSecours: 0x7f7f7f },
  mossy_cobblestone: { nom: 'Pierre moussue', faces: partout('block/mossy_cobblestone'), couleurSecours: 0x6f7a62 },
  oak_planks: { nom: 'Planches de chêne', faces: partout('block/oak_planks'), couleurSecours: 0xa2834f },
  oak_log: {
    nom: 'Bûche de chêne',
    faces: { haut: 'block/oak_log_top', cotes: 'block/oak_log', bas: 'block/oak_log_top' },
    couleurSecours: 0x6b5232,
  },
  oak_leaves: {
    nom: 'Feuilles de chêne',
    faces: partout('block/oak_leaves'),
    decoupe: true,
    teinte: TEINTE_FEUILLAGE,
    couleurSecours: 0x4f8a2a,
  },
  hay_block: {
    nom: 'Botte de foin',
    faces: { haut: 'block/hay_block_top', cotes: 'block/hay_block_side', bas: 'block/hay_block_top' },
    couleurSecours: 0xc8a22a,
  },
  water: {
    nom: 'Eau',
    faces: partout('block/water_still'),
    forme: 'liquide',
    teinte: TEINTE_EAU,
    animation: { imagesParSeconde: 10 },
    couleurSecours: 0x3f76e4,
  },
  short_grass: {
    nom: 'Herbes',
    faces: partout('block/short_grass'),
    forme: 'croix',
    decoupe: true,
    teinte: TEINTE_HERBE,
    couleurSecours: 0x6f9a3a,
  },
  dandelion: { nom: 'Pissenlit', faces: partout('block/dandelion'), forme: 'croix', decoupe: true, couleurSecours: 0xffd23a },
  poppy: { nom: 'Coquelicot', faces: partout('block/poppy'), forme: 'croix', decoupe: true, couleurSecours: 0xd0302a },

  // --- Mine -------------------------------------------------------------------
  stone: { nom: 'Roche', faces: partout('block/stone'), couleurSecours: 0x7d7d7d },
  andesite: { nom: 'Andésite', faces: partout('block/andesite'), couleurSecours: 0x888888 },
  stone_bricks: { nom: 'Briques de pierre', faces: partout('block/stone_bricks'), couleurSecours: 0x7a7a7a },
  mossy_stone_bricks: { nom: 'Briques de pierre moussues', faces: partout('block/mossy_stone_bricks'), couleurSecours: 0x6f7a62 },
  coal_ore: { nom: 'Minerai de charbon', faces: partout('block/coal_ore'), couleurSecours: 0x5a5a5a },
  iron_ore: { nom: 'Minerai de fer', faces: partout('block/iron_ore'), couleurSecours: 0x9a8a7a },
  diamond_ore: { nom: 'Minerai de diamant', faces: partout('block/diamond_ore'), couleurSecours: 0x6acfd0 },
  bedrock: { nom: 'Bedrock', faces: partout('block/bedrock'), couleurSecours: 0x404040 },
  glowstone: { nom: 'Pierre lumineuse', faces: partout('block/glowstone'), lumineux: true, couleurSecours: 0xf0c060 },
  cobweb: {
    nom: "Toile d'araignée",
    faces: partout('block/cobweb'),
    forme: 'croix',
    decoupe: true,
    ralentit: 0.35,
    couleurSecours: 0xdddddd,
  },

  // --- Nether ------------------------------------------------------------------
  netherrack: { nom: 'Netherrack', faces: partout('block/netherrack'), couleurSecours: 0x6f3434 },
  nether_bricks: { nom: 'Briques du Nether', faces: partout('block/nether_bricks'), couleurSecours: 0x2c1418 },
  soul_sand: { nom: 'Sable des âmes', faces: partout('block/soul_sand'), ralentit: 0.6, couleurSecours: 0x51402f },
  magma: {
    nom: 'Bloc de magma',
    faces: partout('block/magma'),
    lumineux: true,
    degats: 1,
    animation: { imagesParSeconde: 2.5 },
    couleurSecours: 0x8a3a10,
  },
  lava: {
    nom: 'Lave',
    faces: partout('block/lava_still'),
    forme: 'liquide',
    lumineux: true,
    animation: { imagesParSeconde: 10, allerRetour: true },
    couleurSecours: 0xd85a10,
  },
  obsidian: { nom: 'Obsidienne', faces: partout('block/obsidian'), couleurSecours: 0x14101e },
  crying_obsidian: { nom: 'Obsidienne pleureuse', faces: partout('block/crying_obsidian'), lumineux: true, couleurSecours: 0x2a0a50 },
} satisfies Record<string, DefinitionBloc>;

export type IdBloc = keyof typeof definitions;

export const BLOCS: Readonly<Record<IdBloc, DefinitionBloc>> = definitions;

/** Un bloc bloque-t-il le passage ? (cubes pleins et liquides) */
export function bloqueLePassage(bloc: DefinitionBloc): boolean {
  const forme = bloc.forme ?? 'cube';
  if (forme === 'croix') return false;
  if (forme === 'liquide') return true;
  return bloc.solide ?? true;
}
