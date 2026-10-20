/**
 * Chargement des textures vanilla.
 *
 * Chaque chemin est cherché dans les sources de SOURCES_TEXTURES, dans l'ordre
 * (distant, distant de secours, local). Si aucune ne répond, la texture est
 * absente et chaque système utilise sa couleur de secours : le chargement ne
 * lève jamais d'erreur.
 */
import * as THREE from 'three';
import { CONFIG, SOURCES_TEXTURES, type IdSourceTexture } from '../config';
import type { CheminTexture } from '../donnees/types';

export interface BilanTextures {
  readonly demandees: number;
  readonly chargees: number;
  readonly parSource: ReadonlyMap<IdSourceTexture, number>;
  readonly manquantes: readonly CheminTexture[];
}

export class BibliothequeTextures {
  private readonly textures = new Map<CheminTexture, THREE.Texture>();
  private readonly origines = new Map<CheminTexture, IdSourceTexture>();
  private readonly urls = new Map<CheminTexture, string>();
  private readonly manquantes: CheminTexture[] = [];

  /** Charge toutes les textures demandées en parallèle. */
  async charger(
    chemins: Iterable<CheminTexture>,
    surProgression?: (faites: number, total: number) => void,
  ): Promise<void> {
    const liste = [...new Set(chemins)].filter((chemin) => !this.textures.has(chemin));
    let faites = 0;
    surProgression?.(0, liste.length);

    await Promise.all(
      liste.map(async (chemin) => {
        const resultat = await chargerDepuisSources(chemin);
        if (resultat) {
          this.textures.set(chemin, creerTexture(resultat.image));
          this.origines.set(chemin, resultat.source);
          this.urls.set(chemin, resultat.image.src);
        } else {
          this.manquantes.push(chemin);
        }
        surProgression?.(++faites, liste.length);
      }),
    );
  }

  /** Texture prête à l'emploi, ou null si elle n'a pu être chargée. */
  obtenir(chemin: CheminTexture): THREE.Texture | null {
    return this.textures.get(chemin) ?? null;
  }

  /** Adresse d'où la texture a été chargée (pour l'interface HTML), ou null. */
  url(chemin: CheminTexture): string | null {
    return this.urls.get(chemin) ?? null;
  }

  /** Copie de la texture répétée `repetitions` fois sur chaque axe (même image, pour les grandes surfaces). */
  obtenirRepetee(chemin: CheminTexture, repetitions: number): THREE.Texture | null {
    const base = this.obtenir(chemin);
    if (!base) return null;
    const texture = base.clone();
    texture.wrapS = THREE.RepeatWrapping;
    texture.wrapT = THREE.RepeatWrapping;
    texture.repeat.set(repetitions, repetitions);
    texture.offset.set(0, 0);
    texture.needsUpdate = true;
    return texture;
  }

  get bilan(): BilanTextures {
    const parSource = new Map<IdSourceTexture, number>();
    for (const source of this.origines.values()) {
      parSource.set(source, (parSource.get(source) ?? 0) + 1);
    }
    return {
      demandees: this.textures.size + this.manquantes.length,
      chargees: this.textures.size,
      parSource,
      manquantes: this.manquantes,
    };
  }
}

/** Règles de rendu imposées à toutes les textures Minecraft. */
function creerTexture(image: HTMLImageElement): THREE.Texture {
  const texture = new THREE.Texture(image);
  // Pixels nets : sans ces deux filtres, les blocs sont lissés et flous
  texture.magFilter = THREE.NearestFilter;
  texture.minFilter = THREE.NearestFilter;
  texture.generateMipmaps = false;
  texture.colorSpace = THREE.SRGBColorSpace;

  // Texture animée vanilla = bande verticale de frames carrées : on n'affiche que la première
  const frames = image.height > image.width && image.height % image.width === 0
    ? image.height / image.width
    : 1;
  if (frames > 1) {
    texture.repeat.y = 1 / frames;
    texture.offset.y = 1 - 1 / frames;
  }

  texture.needsUpdate = true;
  return texture;
}

async function chargerDepuisSources(
  chemin: CheminTexture,
): Promise<{ image: HTMLImageElement; source: IdSourceTexture } | null> {
  for (const source of SOURCES_TEXTURES) {
    try {
      const image = await chargerImage(source.url(chemin), CONFIG.textures.delaiMaxMs);
      return { image, source: source.id };
    } catch {
      // Source indisponible ou fichier absent : on passe à la suivante
    }
  }
  return null;
}

function chargerImage(url: string, delaiMs: number): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    // CORS obligatoire pour WebGL : une image non autorisée échoue ici au lieu de planter au rendu
    image.crossOrigin = 'anonymous';
    const minuterie = window.setTimeout(() => {
      image.src = '';
      reject(new Error(`Délai dépassé : ${url}`));
    }, delaiMs);
    image.onload = () => {
      window.clearTimeout(minuterie);
      resolve(image);
    };
    image.onerror = () => {
      window.clearTimeout(minuterie);
      reject(new Error(`Introuvable : ${url}`));
    };
    image.src = url;
  });
}
