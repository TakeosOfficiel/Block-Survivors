/**
 * Ce que les fichiers de modèles ne disent pas : teinte de biome des faces
 * teintées, collisions et effets au sol de chaque bloc. Couleurs du biome
 * « plaines », comme les arènes du jeu.
 */
import type { BlocCuit } from './modeles';

const HERBE = 0x91bd59;
const FEUILLAGE = 0x77ab2f;

const TEINTES: readonly [RegExp, number][] = [
  [/^birch_leaves$/, 0x80a755],
  [/^spruce_leaves$/, 0x619961],
  [/^(oak|jungle|acacia|dark_oak|mangrove)_leaves$|^vine$/, FEUILLAGE],
  [/^lily_pad$/, 0x208030],
  [/^(water|water_cauldron|bubble_column)$/, 0x3f76e4],
  [/^redstone_wire$/, 0xa00000],
  [/^(melon|pumpkin)_stem$|^attached_(melon|pumpkin)_stem$/, 0xbfb030],
];

/** Couleur des faces teintées du bloc. */
export function teinteDe(nom: string): number {
  for (const [motif, couleur] of TEINTES) if (motif.test(nom)) return couleur;
  return HERBE;
}

/** « plein » bloque la marche et les tirs ; « liquide » bloque seulement la marche. */
export type Collision = 'vide' | 'plein' | 'liquide';

export interface Physique {
  readonly collision: Collision;
  /** Multiplicateur de vitesse (sol ou bloc traversé). */
  readonly ralentit?: number;
  /** Seules les araignées ignorent ce ralentissement. */
  readonly toile?: boolean;
  /** Dégâts par seconde (sol ou bloc traversé). */
  readonly degats?: number;
}

/** Blocs qu'on traverse en jeu, malgré un modèle épais. */
const TRAVERSABLES = /torch|button|lever|pressure_plate|tripwire|(^|_)rail$|redstone_wire|(^|_)sign$|banner$|ladder|vine|lichen|scaffolding|cobweb|petals|leaf_litter|wildflowers|lily_pad|sugar_cane|kelp|seagrass|(^|_)fire$|structure_void|^light$|^air$|portal$|^end_gateway$|_sapling$|propagule|bush$|^(short|tall)_grass$|fern$|roots$|sprouts$|carpet$|^snow$|pointed_dripstone|amethyst_bud$|^small_dripleaf$/;

const EFFETS: readonly [RegExp, Omit<Physique, 'collision'>][] = [
  [/^cobweb$/, { ralentit: 0.3, toile: true }],
  [/^sweet_berry_bush$/, { ralentit: 0.5, degats: 1 }],
  [/^(soul_sand|honey_block)$/, { ralentit: 0.6 }],
  [/^magma_block$/, { degats: 2 }],
  [/^(fire|soul_fire|campfire|soul_campfire)$/, { degats: 2 }],
  [/^lava$/, { degats: 8 }],
];

export function physiqueDe(bloc: BlocCuit): Physique {
  const effets = EFFETS.find(([motif]) => motif.test(bloc.nom))?.[1] ?? {};
  if (bloc.liquide) return { collision: 'liquide', ...effets };
  if (bloc.nom === 'barrier') return { collision: 'plein' };
  if (bloc.inconnu) return { collision: 'plein' };
  if (TRAVERSABLES.test(bloc.nom)) return { collision: 'vide', ...effets };
  if (bloc.nom.endsWith('fence_gate') && bloc.proprietes.open === 'true') return { collision: 'vide' };

  // D'après la forme : rien, des plans sans épaisseur ou une fine couche au sol → on passe
  const modeles = bloc.parties.flatMap((p) => p.map((v) => v.modele));
  const boites = modeles.map((m) => m.boite).filter((b) => b !== null);
  if (boites.length === 0) return { collision: 'vide', ...effets };
  if (modeles.every((m) => m.plat || m.quads.length === 0)) return { collision: 'vide', ...effets };
  const hauteur = Math.max(...boites.map(([, max]) => max[1])) - Math.min(...boites.map(([min]) => min[1]));
  if (hauteur <= 3) return { collision: 'vide', ...effets };
  return { collision: 'plein', ...effets };
}
