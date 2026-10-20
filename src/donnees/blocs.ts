/**
 * Types de blocs. La clé reprend l'identifiant officiel Minecraft.
 * Ajouter un bloc = ajouter une entrée.
 */
import type { CheminTexture, DefinitionBloc } from './types';

/** Même texture sur les six faces. */
const partout = (texture: CheminTexture) => ({ haut: texture, cotes: texture, bas: texture });

const definitions = {
  grass_block: {
    nom: "Bloc d'herbe",
    faces: { haut: 'block/grass_block_top', cotes: 'block/grass_block_side', bas: 'block/dirt' },
    teinteHaut: 0x91bd59,          // teinte du biome « plaines »
    rotationAleatoire: true,
    couleurSecours: 0x91bd59,
  },
  cobblestone: {
    nom: 'Pierre',
    faces: partout('block/cobblestone'),
    couleurSecours: 0x7f7f7f,
  },
  mossy_cobblestone: {
    nom: 'Pierre moussue',
    faces: partout('block/mossy_cobblestone'),
    couleurSecours: 0x6f7a62,
  },
} satisfies Record<string, DefinitionBloc>;

export type IdBloc = keyof typeof definitions;

export const BLOCS: Readonly<Record<IdBloc, DefinitionBloc>> = definitions;
