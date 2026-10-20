/**
 * Lecture du format NBT de Minecraft (fichiers .schem, .nbt, .litematic) :
 * gros-boutiste, compressé en gzip le plus souvent. Les tableaux d'octets,
 * d'entiers et de longs gardent leur forme typée, le reste devient des objets
 * et des listes JavaScript.
 */

export type ValeurNbt =
  | number
  | bigint
  | string
  | Int8Array
  | Int32Array
  | BigInt64Array
  | ValeurNbt[]
  | CompoundNbt;

export interface CompoundNbt {
  [nom: string]: ValeurNbt;
}

/** Décompresse si besoin (gzip ou zlib), puis lit l'étiquette racine. */
export async function lireNbt(donnees: ArrayBuffer): Promise<{ nom: string; valeur: CompoundNbt }> {
  const octets = new Uint8Array(donnees);
  let brut: Uint8Array = octets;
  if (octets[0] === 0x1f && octets[1] === 0x8b) brut = await decompresser(octets, 'gzip');
  else if (octets[0] === 0x78) brut = await decompresser(octets, 'deflate');
  const lecteur = new Lecteur(brut);
  const type = lecteur.octet();
  if (type !== 10) throw new Error('Fichier NBT invalide : la racine doit être un compound.');
  const nom = lecteur.chaine();
  return { nom, valeur: lecteur.compound() };
}

async function decompresser(octets: Uint8Array, format: 'gzip' | 'deflate'): Promise<Uint8Array<ArrayBuffer>> {
  const flux = new Blob([octets as BlobPart]).stream().pipeThrough(new DecompressionStream(format));
  return new Uint8Array(await new Response(flux).arrayBuffer());
}

class Lecteur {
  private readonly vue: DataView;
  private readonly octets: Uint8Array;
  private position = 0;
  private readonly decodeur = new TextDecoder();

  constructor(octets: Uint8Array) {
    this.octets = octets;
    this.vue = new DataView(octets.buffer, octets.byteOffset, octets.byteLength);
  }

  octet(): number {
    return this.vue.getInt8(this.position++);
  }

  private court(): number {
    const v = this.vue.getInt16(this.position);
    this.position += 2;
    return v;
  }

  private entier(): number {
    const v = this.vue.getInt32(this.position);
    this.position += 4;
    return v;
  }

  private long(): bigint {
    const v = this.vue.getBigInt64(this.position);
    this.position += 8;
    return v;
  }

  chaine(): string {
    const longueur = this.vue.getUint16(this.position);
    this.position += 2;
    const texte = this.decodeur.decode(this.octets.subarray(this.position, this.position + longueur));
    this.position += longueur;
    return texte;
  }

  compound(): CompoundNbt {
    const resultat: CompoundNbt = {};
    for (;;) {
      const type = this.octet();
      if (type === 0) return resultat;
      const nom = this.chaine();
      resultat[nom] = this.valeur(type);
    }
  }

  private valeur(type: number): ValeurNbt {
    switch (type) {
      case 1: return this.octet();
      case 2: return this.court();
      case 3: return this.entier();
      case 4: return this.long();
      case 5: {
        const v = this.vue.getFloat32(this.position);
        this.position += 4;
        return v;
      }
      case 6: {
        const v = this.vue.getFloat64(this.position);
        this.position += 8;
        return v;
      }
      case 7: {
        const n = this.entier();
        const v = new Int8Array(this.octets.buffer.slice(this.octets.byteOffset + this.position, this.octets.byteOffset + this.position + n));
        this.position += n;
        return v;
      }
      case 8: return this.chaine();
      case 9: {
        const typeElements = this.octet();
        const n = this.entier();
        const liste: ValeurNbt[] = [];
        for (let i = 0; i < n; i++) liste.push(this.valeur(typeElements));
        return liste;
      }
      case 10: return this.compound();
      case 11: {
        const n = this.entier();
        const v = new Int32Array(n);
        for (let i = 0; i < n; i++) v[i] = this.entier();
        return v;
      }
      case 12: {
        const n = this.entier();
        const v = new BigInt64Array(n);
        for (let i = 0; i < n; i++) v[i] = this.long();
        return v;
      }
      default:
        throw new Error(`Fichier NBT invalide : type d'étiquette ${type} inconnu.`);
    }
  }
}
