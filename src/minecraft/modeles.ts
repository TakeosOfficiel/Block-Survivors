/**
 * Modèles de blocs vanilla : lit les fichiers officiels (blockstates/*.json,
 * models/block/*.json) et les « cuit » en quadrilatères prêts à afficher,
 * dans le repère du bloc (0 à 1 sur chaque axe). C'est ce qui permet
 * d'afficher n'importe quel bloc de Minecraft sans le décrire à la main :
 * escaliers, dalles, barrières, plantes, champignons géants…
 *
 * Conventions Minecraft : x vers l'est, y vers le haut, z vers le sud ;
 * coordonnées des modèles en seizièmes de bloc, UV en pixels (v vers le bas).
 */
import { chargerJsonVanilla } from '../assets/fichiers';

export type Direction = 'down' | 'up' | 'north' | 'south' | 'west' | 'east';
type Vec3 = [number, number, number];

const DIRECTIONS: Readonly<Record<Direction, Vec3>> = {
  down: [0, -1, 0], up: [0, 1, 0], north: [0, 0, -1], south: [0, 0, 1], west: [-1, 0, 0], east: [1, 0, 0],
};

export const OPPOSEES: Readonly<Record<Direction, Direction>> = {
  down: 'up', up: 'down', north: 'south', south: 'north', west: 'east', east: 'west',
};

/** Quadrilatère cuit : coins haut-gauche, haut-droit, bas-droit, bas-gauche vus de face. */
export interface Quad {
  /** 4 coins × (x, y, z), repère du bloc (0 à 1). */
  readonly positions: Float32Array;
  /** 4 coins × (u, v) en pixels de la texture (0 à 16, v vers le bas). */
  readonly uv: Float32Array;
  readonly texture: string;
  readonly normale: Vec3;
  /** Face masquée si le voisin dans cette direction est un bloc plein opaque. */
  readonly cullface: Direction | null;
  /** Teinte de biome (herbe, feuillage, eau…). */
  readonly teinte: boolean;
  /** Faux : éclairée comme une face du dessus (plantes). */
  readonly ombre: boolean;
}

export interface ModeleCuit {
  readonly quads: readonly Quad[];
  /** Un seul cube de 0 à 16 sur chaque axe, aucune rotation : peut cacher les faces de ses voisins. */
  readonly cubePlein: boolean;
  /** Boîte englobante des éléments (seizièmes) : sert à deviner les collisions. */
  readonly boite: readonly [Vec3, Vec3] | null;
  /** Tous les éléments sont des plans sans épaisseur (plantes en croix, cultures). */
  readonly plat: boolean;
}

/** Un état de bloc : ses parties (multipart) et, pour chacune, des variantes pondérées. */
export interface BlocCuit {
  readonly etat: string;
  readonly nom: string;
  readonly proprietes: Readonly<Record<string, string>>;
  readonly parties: readonly (readonly { readonly modele: ModeleCuit; readonly poids: number }[])[];
  /** Eau ou lave (dessinées à part) ; « waterlogged » ajoute de l'eau. */
  readonly liquide: 'eau' | 'lave' | null;
  /** Aucun modèle trouvé : affiché en damier rose et noir. */
  readonly inconnu: boolean;
}

// --- Fichiers JSON --------------------------------------------------------------

interface FaceJson {
  readonly uv?: readonly [number, number, number, number];
  readonly texture: string;
  readonly cullface?: Direction;
  readonly rotation?: number;
  readonly tintindex?: number;
}

interface ElementJson {
  readonly from: Vec3;
  readonly to: Vec3;
  readonly rotation?: {
    readonly origin: Vec3;
    readonly axis?: 'x' | 'y' | 'z';
    readonly angle?: number;
    readonly rescale?: boolean;
  };
  readonly shade?: boolean;
  readonly shade_direction_override?: Direction;
  readonly faces: Partial<Record<Direction, FaceJson>>;
}

/** Référence de texture : « #variable », chemin, ou (26.x) objet { sprite, force_translucent }. */
type TextureJson = string | { readonly sprite: string };

interface ModeleJson {
  readonly parent?: string;
  readonly textures?: Readonly<Record<string, TextureJson>>;
  readonly elements?: readonly ElementJson[];
}

interface ModeleApplique {
  readonly model: string;
  readonly x?: number;
  readonly y?: number;
  readonly uvlock?: boolean;
  readonly weight?: number;
}

type Condition = Readonly<Record<string, string | boolean | readonly Condition[]>>;

interface EtatsJson {
  readonly variants?: Readonly<Record<string, ModeleApplique | readonly ModeleApplique[]>>;
  readonly multipart?: readonly { readonly when?: Condition; readonly apply: ModeleApplique | readonly ModeleApplique[] }[];
}

interface ModeleResolu {
  readonly elements: readonly ElementJson[];
  readonly textures: Readonly<Record<string, TextureJson>>;
}

/** Sépare « oak_stairs[facing=east,half=top] » en nom et propriétés. */
export function decomposerEtat(etat: string): { nom: string; proprietes: Record<string, string> } {
  const crochet = etat.indexOf('[');
  if (crochet < 0) return { nom: etat, proprietes: {} };
  const proprietes: Record<string, string> = {};
  for (const paire of etat.slice(crochet + 1, -1).split(',')) {
    const egal = paire.indexOf('=');
    if (egal > 0) proprietes[paire.slice(0, egal)] = paire.slice(egal + 1);
  }
  return { nom: etat.slice(0, crochet), proprietes };
}

/** « minecraft:block/stone » → « block/stone ». */
function cheminRessource(reference: string): string {
  return reference.replace(/^minecraft:/, '');
}

// --- Bibliothèque -----------------------------------------------------------------

export class BibliothequeModeles {
  private readonly modeles = new Map<string, Promise<ModeleResolu | null>>();

  /** Cuit chaque état de la palette (en parallèle). */
  async cuire(etats: readonly string[]): Promise<BlocCuit[]> {
    return Promise.all(etats.map((etat) => this.cuireEtat(etat)));
  }

  /** Textures utilisées par une liste de blocs cuits, liquides compris. */
  static textures(blocs: readonly BlocCuit[]): Set<string> {
    const chemins = new Set<string>();
    for (const bloc of blocs) {
      for (const partie of bloc.parties) for (const v of partie) for (const q of v.modele.quads) chemins.add(q.texture);
      if (bloc.liquide === 'eau' || bloc.proprietes.waterlogged === 'true') chemins.add(TEXTURE_EAU);
      if (bloc.liquide === 'lave') chemins.add(TEXTURE_LAVE);
    }
    return chemins;
  }

  private async cuireEtat(etat: string): Promise<BlocCuit> {
    const { nom, proprietes } = decomposerEtat(etat);
    const base = { etat, nom, proprietes };
    if (nom === 'air') return { ...base, parties: [], liquide: null, inconnu: false };
    if (nom === 'water' || nom === 'bubble_column') return { ...base, parties: [], liquide: 'eau', inconnu: false };
    if (nom === 'lava') return { ...base, parties: [], liquide: 'lave', inconnu: false };

    const secours = modeleDeSecours(nom);
    if (secours) {
      return { ...base, parties: [[{ modele: cuireModele(secours, 0, 0, false), poids: 1 }]], liquide: null, inconnu: false };
    }

    const json = await chargerJsonVanilla(`blockstates/${nom}.json`) as EtatsJson | null;
    if (!json) return { ...base, parties: [], liquide: null, inconnu: true };

    const appliques: (ModeleApplique | readonly ModeleApplique[])[] = [];
    if (json.variants) {
      // Première variante dont toutes les conditions « clé=valeur » sont remplies
      for (const [cle, valeur] of Object.entries(json.variants)) {
        if (cle === '' || cle.split(',').every((c) => {
          const [k, v] = c.split('=');
          return proprietes[k] === v;
        })) {
          appliques.push(valeur);
          break;
        }
      }
    }
    for (const partie of json.multipart ?? []) {
      if (!partie.when || conditionRemplie(partie.when, proprietes)) appliques.push(partie.apply);
    }

    const parties = await Promise.all(appliques.map(async (applique) => {
      const liste = Array.isArray(applique) ? applique : [applique as ModeleApplique];
      const variantes = await Promise.all(liste.map(async (a) => {
        const modele = await this.resoudre(cheminRessource(a.model));
        return modele ? { modele: cuireModele(modele, a.x ?? 0, a.y ?? 0, a.uvlock ?? false), poids: a.weight ?? 1 } : null;
      }));
      return variantes.filter((v): v is { modele: ModeleCuit; poids: number } => v !== null);
    }));
    const utiles = parties.filter((p) => p.length > 0);
    // Bloc à entité (coffre, panneau, bannière…) : pas de géométrie dans son modèle
    const vide = utiles.every((p) => p.every((v) => v.modele.quads.length === 0));
    return { ...base, parties: utiles, liquide: null, inconnu: utiles.length === 0 || (vide && !SANS_GEOMETRIE.test(nom)) };
  }

  /** Modèle avec ses parents fusionnés (textures du plus proche, éléments du premier qui en a). */
  private resoudre(chemin: string): Promise<ModeleResolu | null> {
    let promesse = this.modeles.get(chemin);
    if (!promesse) {
      promesse = (async () => {
        const json = await chargerJsonVanilla(`models/${chemin}.json`) as ModeleJson | null;
        if (!json) return null;
        const parent = json.parent && !json.parent.startsWith('builtin/')
          ? await this.resoudre(cheminRessource(json.parent))
          : null;
        return {
          elements: json.elements ?? parent?.elements ?? [],
          textures: { ...parent?.textures, ...json.textures },
        };
      })();
      this.modeles.set(chemin, promesse);
    }
    return promesse;
  }
}

const TEXTURE_EAU = 'block/water_still';
const TEXTURE_LAVE = 'block/lava_still';
export { TEXTURE_EAU, TEXTURE_LAVE };

/** Blocs sans géométrie par nature : leur modèle vide n'est pas une erreur. */
const SANS_GEOMETRIE = /^(air|light|barrier|structure_void|moving_piston)$|(^|_)(sign|banner|head|skull)$/;

function conditionRemplie(condition: Condition, proprietes: Readonly<Record<string, string>>): boolean {
  if (Array.isArray(condition.OR)) return (condition.OR as Condition[]).some((c) => conditionRemplie(c, proprietes));
  if (Array.isArray(condition.AND)) return (condition.AND as Condition[]).every((c) => conditionRemplie(c, proprietes));
  return Object.entries(condition).every(([cle, attendu]) => {
    const valeur = proprietes[cle];
    return String(attendu).split('|').includes(valeur ?? '');
  });
}

/**
 * Blocs dont le modèle vanilla est vide (dessinés par le jeu comme des
 * entités) : une boîte simple qui en donne l'allure.
 */
function modeleDeSecours(nom: string): ModeleResolu | null {
  const boite = (from: Vec3, to: Vec3, texture: string): ModeleResolu => ({
    textures: { t: texture },
    elements: [{
      from, to,
      faces: Object.fromEntries((Object.keys(DIRECTIONS) as Direction[]).map((d) => [d, { texture: '#t' }])),
    }],
  });
  if (nom === 'chest' || nom === 'trapped_chest') return boite([1, 0, 1], [15, 14, 15], 'block/oak_planks');
  if (nom === 'ender_chest') return boite([1, 0, 1], [15, 14, 15], 'block/obsidian');
  const lit = /^(\w+)_bed$/.exec(nom);
  if (lit) return boite([0, 0, 0], [16, 9, 16], `block/${lit[1]}_wool`);
  const shulker = /^(\w+_)?shulker_box$/.exec(nom);
  if (shulker) return boite([0, 0, 0], [16, 16, 16], `block/${nom}`);
  return null;
}

// --- Cuisson ----------------------------------------------------------------------

/** Coins (TL, TR, BR, BL) d'une face d'élément, en seizièmes. */
function coinsFace(d: Direction, f: Vec3, t: Vec3): Vec3[] {
  switch (d) {
    case 'north': return [[t[0], t[1], f[2]], [f[0], t[1], f[2]], [f[0], f[1], f[2]], [t[0], f[1], f[2]]];
    case 'south': return [[f[0], t[1], t[2]], [t[0], t[1], t[2]], [t[0], f[1], t[2]], [f[0], f[1], t[2]]];
    case 'west': return [[f[0], t[1], f[2]], [f[0], t[1], t[2]], [f[0], f[1], t[2]], [f[0], f[1], f[2]]];
    case 'east': return [[t[0], t[1], t[2]], [t[0], t[1], f[2]], [t[0], f[1], f[2]], [t[0], f[1], t[2]]];
    case 'up': return [[f[0], t[1], f[2]], [t[0], t[1], f[2]], [t[0], t[1], t[2]], [f[0], t[1], t[2]]];
    case 'down': return [[f[0], f[1], t[2]], [t[0], f[1], t[2]], [t[0], f[1], f[2]], [f[0], f[1], f[2]]];
  }
}

/** UV d'un point d'une face selon sa direction (UV par défaut de Minecraft, et « uvlock »). */
function uvProjete(d: Direction, p: Vec3): [number, number] {
  switch (d) {
    case 'up': return [p[0], p[2]];
    case 'down': return [p[0], 16 - p[2]];
    case 'north': return [16 - p[0], 16 - p[1]];
    case 'south': return [p[0], 16 - p[1]];
    case 'west': return [p[2], 16 - p[1]];
    case 'east': return [16 - p[2], 16 - p[1]];
  }
}

function tourner(p: Vec3, axe: 'x' | 'y' | 'z', angleDeg: number, centre: Vec3): Vec3 {
  const a = (angleDeg * Math.PI) / 180;
  const c = Math.cos(a);
  const s = Math.sin(a);
  const x = p[0] - centre[0];
  const y = p[1] - centre[1];
  const z = p[2] - centre[2];
  // Rotations directes (main droite), comme les quaternions de Minecraft
  if (axe === 'x') return [x + centre[0], y * c - z * s + centre[1], y * s + z * c + centre[2]];
  if (axe === 'y') return [x * c + z * s + centre[0], y + centre[1], -x * s + z * c + centre[2]];
  return [x * c - y * s + centre[0], x * s + y * c + centre[1], z + centre[2]];
}

const CENTRE: Vec3 = [8, 8, 8];
const ORIGINE: Vec3 = [0, 0, 0];

/** Rotation d'état de bloc : x puis y, par quarts de tour (sens horaire vu de l'axe). */
function tournerEtat(p: Vec3, rx: number, ry: number, centre: Vec3): Vec3 {
  let q = p;
  if (rx) q = tourner(q, 'x', -rx, centre);
  if (ry) q = tourner(q, 'y', -ry, centre);
  return q;
}

function directionDe(v: Vec3): Direction {
  let meilleure: Direction = 'up';
  let produit = -Infinity;
  for (const [d, n] of Object.entries(DIRECTIONS) as [Direction, Vec3][]) {
    const p = v[0] * n[0] + v[1] * n[1] + v[2] * n[2];
    if (p > produit) {
      produit = p;
      meilleure = d;
    }
  }
  return meilleure;
}

function resoudreTexture(reference: string, textures: Readonly<Record<string, TextureJson>>): string | null {
  let ref = reference;
  for (let i = 0; i < 8 && ref.startsWith('#'); i++) {
    const suivante = textures[ref.slice(1)];
    if (suivante === undefined) return null;
    ref = typeof suivante === 'string' ? suivante : suivante.sprite;
  }
  return ref.startsWith('#') ? null : cheminRessource(ref);
}

function cuireModele(modele: ModeleResolu, rx: number, ry: number, uvlock: boolean): ModeleCuit {
  const quads: Quad[] = [];
  let min: Vec3 | null = null;
  let max: Vec3 | null = null;
  let plat = modele.elements.length > 0;

  for (const element of modele.elements) {
    const { from: f, to: t } = element;
    min = min ? [Math.min(min[0], f[0]), Math.min(min[1], f[1]), Math.min(min[2], f[2])] : [...f];
    max = max ? [Math.max(max[0], t[0]), Math.max(max[1], t[1]), Math.max(max[2], t[2])] : [...t];
    if (f[0] !== t[0] && f[1] !== t[1] && f[2] !== t[2]) plat = false;

    // Rotation de l'élément (et mise à l'échelle « rescale » des deux autres axes)
    const r = element.rotation;
    const angle = r?.angle ?? 0;
    const axe = r?.axis ?? 'y';
    const echelle = r?.rescale && angle ? 1 / Math.cos((Math.abs(angle) * Math.PI) / 180) : 1;
    const transformer = (p: Vec3): Vec3 => {
      let q = p;
      if (r && angle) {
        q = tourner(q, axe, angle, r.origin);
        if (echelle !== 1) {
          q = [
            axe === 'x' ? q[0] : r.origin[0] + (q[0] - r.origin[0]) * echelle,
            axe === 'y' ? q[1] : r.origin[1] + (q[1] - r.origin[1]) * echelle,
            axe === 'z' ? q[2] : r.origin[2] + (q[2] - r.origin[2]) * echelle,
          ];
        }
      }
      return tournerEtat(q, rx, ry, CENTRE);
    };

    for (const [d, face] of Object.entries(element.faces) as [Direction, FaceJson][]) {
      const texture = resoudreTexture(face.texture, modele.textures);
      if (!texture) continue;
      const coins = coinsFace(d, f, t).map(transformer);
      let normale = DIRECTIONS[d];
      if (r && angle) normale = tourner(normale, axe, angle, ORIGINE);
      normale = tournerEtat(normale, rx, ry, ORIGINE);
      const directionFinale = directionDe(normale);

      const uv = new Float32Array(8);
      if (uvlock) {
        for (let i = 0; i < 4; i++) [uv[i * 2], uv[i * 2 + 1]] = uvProjete(directionFinale, coins[i]);
      } else {
        const [u0, v0, u1, v1] = face.uv ?? uvParDefaut(d, f, t);
        const base = [u0, v0, u1, v0, u1, v1, u0, v1];
        // Rotation de la texture sur la face, par quarts de tour (sens horaire)
        const k = (((face.rotation ?? 0) / 90) % 4 + 4) % 4;
        for (let i = 0; i < 4; i++) {
          const j = (i - k + 4) % 4;
          uv[i * 2] = base[j * 2];
          uv[i * 2 + 1] = base[j * 2 + 1];
        }
      }

      const positions = new Float32Array(12);
      coins.forEach((p, i) => positions.set([p[0] / 16, p[1] / 16, p[2] / 16], i * 3));
      const cullface = face.cullface ? directionDe(tournerEtat(DIRECTIONS[face.cullface], rx, ry, ORIGINE)) : null;
      quads.push({
        positions,
        uv,
        texture,
        normale,
        cullface,
        teinte: face.tintindex !== undefined,
        ombre: element.shade !== false && element.shade_direction_override === undefined,
      });
    }
  }

  const cubePlein = modele.elements.length === 1 && !modele.elements[0].rotation?.angle
    && modele.elements[0].from.every((v) => v === 0) && modele.elements[0].to.every((v) => v === 16)
    && Object.keys(modele.elements[0].faces).length === 6;
  return { quads, cubePlein, boite: min && max ? [min, max] : null, plat };
}

function uvParDefaut(d: Direction, f: Vec3, t: Vec3): [number, number, number, number] {
  switch (d) {
    case 'down': return [f[0], 16 - t[2], t[0], 16 - f[2]];
    case 'up': return [f[0], f[2], t[0], t[2]];
    case 'north': return [16 - t[0], 16 - t[1], 16 - f[0], 16 - f[1]];
    case 'south': return [f[0], 16 - t[1], t[0], 16 - f[1]];
    case 'west': return [f[2], 16 - t[1], t[2], 16 - f[1]];
    case 'east': return [16 - t[2], 16 - t[1], 16 - f[2], 16 - f[1]];
  }
}
