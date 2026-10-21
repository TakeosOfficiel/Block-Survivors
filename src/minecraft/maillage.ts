/**
 * Affichage d'une schématique : toutes ses textures dans un atlas, puis trois
 * maillages fusionnés (opaque, découpé, translucide). Les faces cachées par un
 * bloc plein voisin ne sont pas dessinées, comme dans Minecraft.
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import type { BlocCuit, Direction, ModeleCuit, Quad } from './modeles';
import { TEXTURE_EAU, TEXTURE_LAVE } from './modeles';
import { teinteDe } from './proprietes';
import type { Schematique } from './schematique';

/**
 * Ondulation au vent d'un bloc (attribut `aVent` de chaque sommet : centre x, pied y, centre z, mode),
 * reprise par monde/vegetation.ts : feuillage qui frémit, plantes dont le sommet se balance.
 */
export const VENT_AUCUN = 0;
export const VENT_FEUILLAGE = 1;
export const VENT_PLANTE = 2;

/** Plantes qui se balancent (fleurs, herbes, pousses, cultures, lianes…). */
const PLANTES = new RegExp('(_sapling|_tulip)$|^(short_grass|grass|tall_grass|fern|large_fern|dandelion|poppy|blue_orchid|allium|azure_bluet'
  + '|oxeye_daisy|cornflower|lily_of_the_valley|wither_rose|torchflower|sunflower|lilac|rose_bush|peony|sweet_berry_bush|dead_bush'
  + '|bush|firefly_bush|short_dry_grass|tall_dry_grass|wildflowers|pink_petals|leaf_litter|cactus_flower|open_eyeblossom'
  + '|closed_eyeblossom|wheat|carrots|potatoes|beetroots|seagrass|tall_seagrass|kelp|kelp_plant|vine|cave_vines|cave_vines_plant'
  + '|weeping_vines|weeping_vines_plant|twisting_vines|twisting_vines_plant|hanging_roots|brown_mushroom|red_mushroom'
  + '|crimson_roots|warped_roots|nether_sprouts|spore_blossom|pitcher_plant|small_dripleaf)$');

function modeVent(bloc: BlocCuit): number {
  if (bloc.nom.endsWith('_leaves')) return VENT_FEUILLAGE;
  return PLANTES.test(bloc.nom) ? VENT_PLANTE : VENT_AUCUN;
}

/** Rendu d'une texture : opaque, découpée (feuilles, plantes, vitres) ou translucide (verre teinté, eau). */
type RenduTexture = 'opaque' | 'decoupe' | 'translucide';
const RENDUS: readonly RenduTexture[] = ['opaque', 'decoupe', 'translucide'];

const CASE = 16;
/** Bordure autour de chaque texture, recopiée de son bord : pas de fuite de la texture voisine. */
const BORD = 1;
const PAS = CASE + 2 * BORD;
const COLONNES = 32;

interface CaseAtlas {
  readonly colonne: number;
  readonly ligne: number;
  readonly rendu: RenduTexture;
}

export class Atlas {
  readonly texture: THREE.Texture;
  private readonly cases = new Map<string, CaseAtlas>();
  private readonly lignes: number;

  /** Textures déjà chargées dans `textures` ; une absente devient un damier rose et noir. */
  constructor(chemins: Iterable<string>, textures: BibliothequeTextures) {
    const liste = [...new Set(chemins)];
    liste.push(MANQUANTE);
    this.lignes = Math.max(1, Math.ceil(liste.length / COLONNES));
    const canvas = document.createElement('canvas');
    canvas.width = COLONNES * PAS;
    canvas.height = this.lignes * PAS;
    const ctx = canvas.getContext('2d', { willReadFrequently: true });
    if (!ctx) throw new Error('Canvas 2D indisponible');
    ctx.imageSmoothingEnabled = false;

    liste.forEach((chemin, i) => {
      const colonne = i % COLONNES;
      const ligne = Math.floor(i / COLONNES);
      const x = colonne * PAS + BORD;
      const y = ligne * PAS + BORD;
      const image = chemin === MANQUANTE ? null : textures.obtenir(chemin)?.image as HTMLImageElement | undefined;
      if (image) {
        // Texture animée = bande verticale d'images carrées : la première seulement
        ctx.drawImage(image, 0, 0, image.width, image.width, x, y, CASE, CASE);
      } else {
        for (let py = 0; py < 2; py++) {
          for (let px = 0; px < 2; px++) {
            ctx.fillStyle = (px + py) % 2 === 0 ? '#f800f8' : '#000000';
            ctx.fillRect(x + px * 8, y + py * 8, 8, 8);
          }
        }
      }
      this.cases.set(chemin, { colonne, ligne, rendu: classer(ctx.getImageData(x, y, CASE, CASE)) });
      // Bordure : lignes et colonnes du bord recopiées vers l'extérieur
      ctx.drawImage(canvas, x, y, CASE, 1, x, y - 1, CASE, 1);
      ctx.drawImage(canvas, x, y + CASE - 1, CASE, 1, x, y + CASE, CASE, 1);
      ctx.drawImage(canvas, x, y - 1, 1, CASE + 2, x - 1, y - 1, 1, CASE + 2);
      ctx.drawImage(canvas, x + CASE - 1, y - 1, 1, CASE + 2, x + CASE, y - 1, 1, CASE + 2);
    });

    this.texture = new THREE.CanvasTexture(canvas);
    this.texture.magFilter = THREE.NearestFilter;
    this.texture.minFilter = THREE.NearestFilter;
    this.texture.generateMipmaps = false;
    this.texture.colorSpace = THREE.SRGBColorSpace;
  }

  private case(chemin: string): CaseAtlas {
    return this.cases.get(chemin) ?? this.cases.get(MANQUANTE)!;
  }

  rendu(chemin: string): RenduTexture {
    return this.case(chemin).rendu;
  }

  /** (u, v) en pixels de la texture (v vers le bas) → coordonnées dans l'atlas (v vers le haut). */
  uv(chemin: string, u: number, v: number, sortie: number[]): void {
    const c = this.case(chemin);
    sortie.push(
      (c.colonne * PAS + BORD + u) / (COLONNES * PAS),
      1 - (c.ligne * PAS + BORD + v) / (this.lignes * PAS),
    );
  }
}

const MANQUANTE = '#manquante';

function classer(pixels: ImageData): RenduTexture {
  let partiels = 0;
  let transparents = 0;
  for (let i = 3; i < pixels.data.length; i += 4) {
    const a = pixels.data[i];
    if (a < 128) transparents++;
    if (a > 20 && a < 235) partiels++;
  }
  if (partiels > 0) return 'translucide';
  return transparents > 0 ? 'decoupe' : 'opaque';
}

// --- Maillage -----------------------------------------------------------------------

/**
 * Tableau de nombres qui s'agrandit tout seul (capacité doublée) : les sommets
 * sont écrits directement dans le tableau typé envoyé à la carte graphique,
 * sans passer par une liste JavaScript à convertir ensuite (bien plus lent).
 */
class Croissant<T extends Float32Array | Uint32Array> {
  donnees: T;
  longueur = 0;
  private readonly creer: (taille: number) => T;

  constructor(creer: (taille: number) => T) {
    this.creer = creer;
    this.donnees = creer(4096);
  }

  private reserver(n: number): void {
    if (this.longueur + n <= this.donnees.length) return;
    let taille = this.donnees.length * 2;
    while (taille < this.longueur + n) taille *= 2;
    const plus = this.creer(taille);
    plus.set(this.donnees.subarray(0, this.longueur));
    this.donnees = plus;
  }

  ajouter2(a: number, b: number): void {
    this.reserver(2);
    const d = this.donnees;
    d[this.longueur++] = a;
    d[this.longueur++] = b;
  }

  ajouter3(a: number, b: number, c: number): void {
    this.reserver(3);
    const d = this.donnees;
    d[this.longueur++] = a;
    d[this.longueur++] = b;
    d[this.longueur++] = c;
  }

  ajouter4(a: number, b: number, c: number, e: number): void {
    this.reserver(4);
    const d = this.donnees;
    d[this.longueur++] = a;
    d[this.longueur++] = b;
    d[this.longueur++] = c;
    d[this.longueur++] = e;
  }

  /** Le contenu, sans la place réservée en trop. */
  get contenu(): T {
    return this.donnees.subarray(0, this.longueur) as T;
  }
}

const flottants = (taille: number) => new Float32Array(taille);

interface Tampons {
  readonly positions: Croissant<Float32Array>;
  readonly normales: Croissant<Float32Array>;
  readonly uvs: Croissant<Float32Array>;
  readonly couleurs: Croissant<Float32Array>;
  readonly vent: Croissant<Float32Array>;
  readonly indices: Croissant<Uint32Array>;
  /** Vrai si au moins un sommet ondule au vent. */
  ondule: boolean;
}

const VOISINS: readonly [Direction, number, number, number][] = [
  ['down', 0, -1, 0], ['up', 0, 1, 0], ['north', 0, 0, -1], ['south', 0, 0, 1], ['west', -1, 0, 0], ['east', 1, 0, 0],
];
const DECALAGES = Object.fromEntries(VOISINS.map(([d, x, y, z]) => [d, [x, y, z]])) as unknown as Record<Direction, readonly [number, number, number]>;

/**
 * Maillages de la schématique. `origine` : coin de la schématique placé à
 * (0, 0, 0) dans le jeu, en blocs (fractionnaire si besoin).
 */
export function construireMaillage(
  schem: Schematique,
  blocs: readonly BlocCuit[],
  atlas: Atlas,
  origine: { readonly x: number; readonly y: number; readonly z: number },
): THREE.Group {
  const { largeur: L, hauteur: H, profondeur: P } = schem;
  const index = (x: number, y: number, z: number) => (y * P + z) * L + x;

  // Ce que chaque état de la palette cache chez ses voisins
  const opaque = blocs.map((b) => {
    const m = modeleUnique(b);
    return m !== null && m.cubePlein && m.quads.every((q) => atlas.rendu(q.texture) === 'opaque');
  });
  // Vitres, glace… : un bloc plein non opaque cache les faces d'un voisin identique (pas les feuilles)
  const fusionne = blocs.map((b) => {
    const m = modeleUnique(b);
    return m !== null && m.cubePlein && !b.nom.endsWith('_leaves');
  });
  const eau = blocs.map((b) => b.liquide === 'eau' || b.proprietes.waterlogged === 'true');
  const lave = blocs.map((b) => b.liquide === 'lave');
  const teintes = blocs.map((b) => new THREE.Color(teinteDe(b.liquide === 'eau' ? 'water' : b.nom)));
  const vents = blocs.map(modeVent);
  const blanc = new THREE.Color(0xffffff);

  const tampons: Record<RenduTexture, Tampons> = {
    opaque: nouveauxTampons(), decoupe: nouveauxTampons(), translucide: nouveauxTampons(),
  };
  const uv: number[] = [];
  const venti: Vent = { mode: VENT_AUCUN, x: 0, pied: 0, z: 0 };

  const cache = (x: number, y: number, z: number, d: Direction, ici: number): boolean => {
    const [dx, dy, dz] = DECALAGES[d];
    const nx = x + dx;
    const ny = y + dy;
    const nz = z + dz;
    if (ny < 0) return true;  // dessous de la schématique : jamais vu
    if (nx < 0 || nz < 0 || nx >= L || ny >= H || nz >= P) return false;
    const voisin = schem.blocs[index(nx, ny, nz)];
    return opaque[voisin] || (voisin === ici && fusionne[ici]);
  };

  for (let y = 0; y < H; y++) {
    for (let z = 0; z < P; z++) {
      for (let x = 0; x < L; x++) {
        const i = schem.blocs[index(x, y, z)];
        if (i === 0) continue;
        const bloc = blocs[i];
        const ox = x - origine.x;
        const oy = y - origine.y;
        const oz = z - origine.z;

        // Vent : centre du bloc, et pied de la plante (celui du bloc du bas pour la moitié haute d'une grande plante)
        venti.mode = vents[i];
        venti.x = ox + 0.5;
        venti.z = oz + 0.5;
        venti.pied = oy - (bloc.proprietes.half === 'upper' ? 1 : 0);
        for (const partie of bloc.parties) {
          const modele = choisir(partie, x, y, z);
          for (const q of modele.quads) {
            if (q.cullface && cache(x, y, z, q.cullface, i)) continue;
            const rendu = atlas.rendu(q.texture);
            ajouterQuad(tampons[rendu], q, ox, oy, oz, q.teinte ? teintes[i] : blanc, atlas, uv, rendu !== 'opaque', venti);
          }
        }
        venti.mode = VENT_AUCUN;

        // Liquides : cube abaissé de 2 pixels, sauf sous le même liquide
        if (eau[i] || lave[i]) {
          const memeLiquide = (nx: number, ny: number, nz: number) => {
            if (nx < 0 || ny < 0 || nz < 0 || nx >= L || ny >= H || nz >= P) return false;
            const v = schem.blocs[index(nx, ny, nz)];
            return eau[i] ? eau[v] : lave[v];
          };
          const haut = memeLiquide(x, y + 1, z) ? 1 : 14 / 16;
          const texture = eau[i] ? TEXTURE_EAU : TEXTURE_LAVE;
          for (const [d, dx, dy, dz] of VOISINS) {
            if (memeLiquide(x + dx, y + dy, z + dz) || cache(x, y, z, d, -1)) continue;
            const rendu = atlas.rendu(texture);
            ajouterQuad(tampons[rendu], quadLiquide(d, haut, texture), ox, oy, oz, eau[i] ? teintes[i] : blanc, atlas, uv, false, venti);
          }
        }
      }
    }
  }

  const groupe = new THREE.Group();
  groupe.name = 'minecraft';
  for (const rendu of RENDUS) {
    const t = tampons[rendu];
    if (t.indices.longueur === 0) continue;
    const geometrie = new THREE.BufferGeometry();
    geometrie.setAttribute('position', new THREE.BufferAttribute(t.positions.contenu, 3));
    geometrie.setAttribute('normal', new THREE.BufferAttribute(t.normales.contenu, 3));
    geometrie.setAttribute('uv', new THREE.BufferAttribute(t.uvs.contenu, 2));
    geometrie.setAttribute('color', new THREE.BufferAttribute(t.couleurs.contenu, 3));
    if (t.ondule) geometrie.setAttribute('aVent', new THREE.BufferAttribute(t.vent.contenu, 4));
    geometrie.setIndex(new THREE.BufferAttribute(t.indices.contenu, 1));
    geometrie.computeBoundingSphere();
    const materiau = new THREE.MeshLambertMaterial({
      map: atlas.texture,
      vertexColors: true,
      alphaTest: rendu === 'decoupe' ? 0.5 : 0,
      transparent: rendu === 'translucide',
      depthWrite: rendu !== 'translucide',
    });
    const maillage = new THREE.Mesh(geometrie, materiau);
    maillage.name = `minecraft-${rendu}`;
    groupe.add(maillage);
  }
  return groupe;
}

/** Ondulation du bloc en cours de maillage. */
interface Vent {
  mode: number;
  x: number;
  pied: number;
  z: number;
}

function nouveauxTampons(): Tampons {
  return {
    positions: new Croissant(flottants), normales: new Croissant(flottants), uvs: new Croissant(flottants),
    couleurs: new Croissant(flottants), vent: new Croissant(flottants), indices: new Croissant((taille) => new Uint32Array(taille)),
    ondule: false,
  };
}

/** Le modèle d'un bloc fait d'une seule partie sans variante, sinon null. */
function modeleUnique(bloc: BlocCuit): ModeleCuit | null {
  return bloc.parties.length === 1 && bloc.parties[0].length === 1 ? bloc.parties[0][0].modele : null;
}

/** Variante pondérée, toujours la même pour une position donnée. */
function choisir(partie: readonly { readonly modele: ModeleCuit; readonly poids: number }[], x: number, y: number, z: number): ModeleCuit {
  if (partie.length === 1) return partie[0].modele;
  let h = Math.imul(x, 73856093) ^ Math.imul(y, 19349663) ^ Math.imul(z, 83492791);
  h = Math.imul(h ^ (h >>> 13), 0x5bd1e995);
  const total = partie.reduce((s, v) => s + v.poids, 0);
  let seuil = ((h >>> 0) / 0xffffffff) * total;
  for (const v of partie) {
    seuil -= v.poids;
    if (seuil < 0) return v.modele;
  }
  return partie[partie.length - 1].modele;
}

function ajouterQuad(
  t: Tampons,
  q: Quad,
  ox: number, oy: number, oz: number,
  couleur: THREE.Color,
  atlas: Atlas,
  uv: number[],
  decoller: boolean,
  vent: Vent,
): void {
  const base = t.positions.longueur / 3;
  const [nx, ny, nz] = q.normale;
  // Face découpée posée sur le bord du bloc (calque d'herbe…) : décollée d'un cheveu, sans scintillement
  let ex = 0;
  let ey = 0;
  let ez = 0;
  if (decoller) {
    const bord = (v: number) => v === 0 || v === 1;
    const p = q.positions;
    if ((Math.abs(nx) === 1 && bord(p[0])) || (Math.abs(ny) === 1 && bord(p[1])) || (Math.abs(nz) === 1 && bord(p[2]))) {
      ex = nx * 0.002;
      ey = ny * 0.002;
      ez = nz * 0.002;
    }
  }
  uv.length = 0;
  for (let c = 0; c < 4; c++) {
    t.positions.ajouter3(q.positions[c * 3] + ox + ex, q.positions[c * 3 + 1] + oy + ey, q.positions[c * 3 + 2] + oz + ez);
    if (q.ombre) t.normales.ajouter3(nx, ny, nz);
    else t.normales.ajouter3(0, 1, 0);
    t.couleurs.ajouter3(couleur.r, couleur.g, couleur.b);
    t.vent.ajouter4(vent.x, vent.pied, vent.z, vent.mode);
    atlas.uv(q.texture, q.uv[c * 2], q.uv[c * 2 + 1], uv);
  }
  for (let k = 0; k < 8; k += 2) t.uvs.ajouter2(uv[k], uv[k + 1]);
  if (vent.mode !== VENT_AUCUN) t.ondule = true;
  // Coins TL, TR, BR, BL vus de face : deux triangles dans le sens direct
  t.indices.ajouter3(base, base + 3, base + 2);
  t.indices.ajouter3(base, base + 2, base + 1);
}

/** Face d'un cube de liquide de hauteur `haut` (en blocs). */
function quadLiquide(d: Direction, haut: number, texture: string): Quad {
  const h = haut;
  const coins: Record<Direction, number[]> = {
    north: [1, h, 0, 0, h, 0, 0, 0, 0, 1, 0, 0],
    south: [0, h, 1, 1, h, 1, 1, 0, 1, 0, 0, 1],
    west: [0, h, 0, 0, h, 1, 0, 0, 1, 0, 0, 0],
    east: [1, h, 1, 1, h, 0, 1, 0, 0, 1, 0, 1],
    up: [0, h, 0, 1, h, 0, 1, h, 1, 0, h, 1],
    down: [0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0],
  };
  const normales: Record<Direction, [number, number, number]> = {
    down: [0, -1, 0], up: [0, 1, 0], north: [0, 0, -1], south: [0, 0, 1], west: [-1, 0, 0], east: [1, 0, 0],
  };
  const vHaut = d === 'up' || d === 'down' ? 0 : 16 - h * 16;
  return {
    positions: new Float32Array(coins[d]),
    uv: new Float32Array([0, vHaut, 16, vHaut, 16, 16, 0, 16]),
    texture,
    normale: normales[d],
    cullface: null,
    teinte: true,
    ombre: true,
  };
}
