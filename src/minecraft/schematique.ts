/**
 * Schématiques Minecraft : .schem de WorldEdit / FAWE (formats Sponge 1, 2 et
 * 3) et .nbt des blocs de structure. Le résultat est un volume de blocs
 * indexés dans une palette d'états (« oak_stairs[facing=east,half=top] »),
 * avec le texte des panneaux (repères posés par le constructeur).
 */
import { lireNbt, type CompoundNbt, type ValeurNbt } from './nbt';

export interface Panneau {
  readonly x: number;
  readonly y: number;
  readonly z: number;
  readonly texte: string;
}

export interface Schematique {
  /** Blocs d'ouest en est (x), de bas en haut (y), du nord au sud (z). */
  readonly largeur: number;
  readonly hauteur: number;
  readonly profondeur: number;
  /** États de bloc sans espace de noms ; l'index 0 est toujours « air ». */
  readonly palette: readonly string[];
  /** Index dans la palette, case (x, y, z) en (y × profondeur + z) × largeur + x. */
  readonly blocs: Uint16Array;
  readonly panneaux: readonly Panneau[];
  /** Version des données Minecraft du fichier (3465 = 1.20.1). */
  readonly versionDonnees: number;
}

/** Blocs renommés depuis les anciennes versions → nom actuel. */
const RENOMMES: Readonly<Record<string, string>> = {
  grass: 'short_grass',          // 1.20.3
  chain: 'iron_chain',           // 1.21.9
};

const AIRS = new Set(['air', 'cave_air', 'void_air']);

export async function lireSchematique(donnees: ArrayBuffer): Promise<Schematique> {
  const { valeur: racine } = await lireNbt(donnees);
  if (racine.Schematic !== undefined) return lireSponge(racine.Schematic as CompoundNbt, 3);
  if (racine.BlockData !== undefined) return lireSponge(racine, 2);
  if (racine.blocks !== undefined && racine.size !== undefined) return lireStructure(racine);
  if (racine.Regions !== undefined) throw new Error('Fichier Litematica (.litematic) pas encore pris en charge : exporte la zone en .schem avec WorldEdit.');
  throw new Error('Format de schématique inconnu (attendu : .schem de WorldEdit ou .nbt de bloc de structure).');
}

/** Normalise un état : sans « minecraft: », avec les noms actuels, air unique. */
function normaliser(etat: string): string {
  const sansEspace = etat.replace(/^minecraft:/, '');
  const crochet = sansEspace.indexOf('[');
  const nom = crochet < 0 ? sansEspace : sansEspace.slice(0, crochet);
  if (AIRS.has(nom)) return 'air';
  const actuel = RENOMMES[nom] ?? nom;
  return crochet < 0 ? actuel : actuel + sansEspace.slice(crochet);
}

/** Palette normalisée (air en 0) et table de passage index du fichier → index normalisé. */
class Palette {
  readonly etats: string[] = ['air'];
  private readonly index = new Map<string, number>([['air', 0]]);

  ajouter(etat: string): number {
    const normalise = normaliser(etat);
    let i = this.index.get(normalise);
    if (i === undefined) {
      i = this.etats.length;
      this.etats.push(normalise);
      this.index.set(normalise, i);
    }
    return i;
  }
}

function lireSponge(s: CompoundNbt, version: number): Schematique {
  const largeur = s.Width as number;
  const hauteur = s.Height as number;
  const profondeur = s.Length as number;
  const blocsNbt = version >= 3 ? (s.Blocks as CompoundNbt) : s;
  const paletteNbt = blocsNbt.Palette as CompoundNbt;
  const donnees = (version >= 3 ? blocsNbt.Data : s.BlockData) as Int8Array;

  const palette = new Palette();
  const passage = new Uint16Array(Object.keys(paletteNbt).length + 1);
  for (const [etat, index] of Object.entries(paletteNbt)) passage[index as number] = palette.ajouter(etat);

  // Indices en varint, dans l'ordre (y × profondeur + z) × largeur + x
  const blocs = new Uint16Array(largeur * hauteur * profondeur);
  let lu = 0;
  for (let i = 0; i < blocs.length && lu < donnees.length; i++) {
    let valeur = 0;
    let decalage = 0;
    for (;;) {
      const octet = donnees[lu++] & 0xff;
      valeur |= (octet & 0x7f) << decalage;
      if ((octet & 0x80) === 0) break;
      decalage += 7;
    }
    blocs[i] = passage[valeur] ?? 0;
  }

  const entites = (blocsNbt.BlockEntities ?? s.BlockEntities ?? s.TileEntities ?? []) as CompoundNbt[];
  const panneaux: Panneau[] = [];
  for (const entite of entites) {
    const id = String(entite.Id ?? entite.id ?? '');
    if (!id.includes('sign')) continue;
    const pos = entite.Pos as Int32Array | undefined;
    if (!pos) continue;
    panneaux.push({ x: pos[0], y: pos[1], z: pos[2], texte: texteDe(entite.Data ?? entite) });
  }

  return {
    largeur, hauteur, profondeur,
    palette: palette.etats,
    blocs,
    panneaux,
    versionDonnees: (s.DataVersion as number | undefined) ?? 0,
  };
}

function lireStructure(s: CompoundNbt): Schematique {
  const taille = s.size as number[];
  const [largeur, hauteur, profondeur] = taille;
  const paletteNbt = ((s.palette ?? (s.palettes as ValeurNbt[][])[0]) as CompoundNbt[]);
  const palette = new Palette();
  const passage = paletteNbt.map((p) => {
    const proprietes = p.Properties as CompoundNbt | undefined;
    const suffixe = proprietes ? `[${Object.entries(proprietes).map(([k, v]) => `${k}=${String(v)}`).join(',')}]` : '';
    return palette.ajouter(String(p.Name) + suffixe);
  });

  const blocs = new Uint16Array(largeur * hauteur * profondeur);
  const panneaux: Panneau[] = [];
  for (const b of s.blocks as CompoundNbt[]) {
    const [x, y, z] = b.pos as number[];
    blocs[(y * profondeur + z) * largeur + x] = passage[b.state as number];
    const nbt = b.nbt as CompoundNbt | undefined;
    if (nbt && String(nbt.id ?? '').includes('sign')) panneaux.push({ x, y, z, texte: texteDe(nbt) });
  }
  return { largeur, hauteur, profondeur, palette: palette.etats, blocs, panneaux, versionDonnees: (s.DataVersion as number | undefined) ?? 0 };
}

/** Tout le texte d'un panneau (lignes JSON ou texte brut, faces avant et arrière). */
function texteDe(valeur: ValeurNbt | undefined): string {
  const morceaux: string[] = [];
  const parcourir = (v: ValeurNbt | undefined): void => {
    if (typeof v === 'string') {
      // Ligne au format JSON ({"text":"depart"}) ou texte simple
      const textes = [...v.matchAll(/"text"\s*:\s*"([^"]*)"/g)].map((m) => m[1]);
      morceaux.push(...(textes.length > 0 ? textes : [v]));
    } else if (Array.isArray(v)) {
      v.forEach(parcourir);
    } else if (v && typeof v === 'object' && !ArrayBuffer.isView(v)) {
      Object.values(v).forEach(parcourir);
    }
  };
  parcourir(valeur);
  return morceaux.join(' ').trim();
}
