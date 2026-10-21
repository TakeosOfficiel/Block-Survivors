/**
 * Scène « La pause des mobs » de l'écran de chargement, côté page : prépare
 * les images (bibliothèque de textures du jeu, cache local compris) et le
 * modèle du feu de camp, puis confie le canvas à un worker qui dessine tout
 * (feuDeCamp.worker.ts). Le fil principal reste libre pour construire la
 * partie, et l'animation ne saccade pas pendant ce temps.
 */
import type { BibliothequeTextures } from '../assets/textures';
import { DECO_FEU_DE_CAMP, FEU_DE_CAMP, FUMEE_FEU_DE_CAMP, TEXTURES_FEU_DE_CAMP } from '../donnees/feuDeCamp';
import type { ApparenceEntite } from '../donnees/types';
import { BibliothequeModeles } from '../minecraft/modeles';
import type { HerosFeuDeCamp } from './feuDeCamp3d';
import type { ImageEnvoyee, MessageFeuDeCamp } from './feuDeCamp.worker';
import TravailleurFeuDeCamp from './feuDeCamp.worker?worker&inline';

/** Ce que la scène garde d'une apparence : modèle, skin, objet tenu. */
function herosDe(apparence: ApparenceEntite): HerosFeuDeCamp {
  return { modele: apparence.modele, skin: apparence.skin, objetEnMain: apparence.objetEnMain };
}

/** Images des chemins demandés, prêtes à partir vers le worker (celles qui manquent sont ignorées). */
async function images(textures: BibliothequeTextures, chemins: Iterable<string>): Promise<ImageEnvoyee[]> {
  const liste = await Promise.all([...new Set(chemins)].map(async (chemin) => {
    const source = textures.obtenir(chemin)?.image as ImageBitmapSource | undefined;
    if (!source) return null;
    try {
      return { chemin, image: await createImageBitmap(source) };
    } catch {
      return null;
    }
  }));
  return liste.filter((i): i is ImageEnvoyee => i !== null);
}

export class SceneChargement {
  private readonly travailleur: Worker;
  private readonly textures: BibliothequeTextures;
  private derniere = -1;

  private constructor(travailleur: Worker, conteneur: HTMLElement, textures: BibliothequeTextures) {
    this.travailleur = travailleur;
    this.textures = textures;
    new ResizeObserver(() => {
      const { width, height } = conteneur.getBoundingClientRect();
      this.envoyer({ type: 'taille', largeur: width, hauteur: height });
    }).observe(conteneur);
  }

  /**
   * Prépare la scène dans `conteneur` ; null si le navigateur ne sait pas
   * dessiner dans un worker : l'écran garde alors son fond de terre.
   */
  static async creer(conteneur: HTMLElement, textures: BibliothequeTextures, heros: ApparenceEntite): Promise<SceneChargement | null> {
    const canvas = document.createElement('canvas');
    if (typeof canvas.transferControlToOffscreen !== 'function') return null;

    // Le modèle du feu de camp et les textures de la scène en même temps
    const [[bloc]] = await Promise.all([
      new BibliothequeModeles().cuire([FEU_DE_CAMP.bloc]),
      textures.charger([
        ...Object.values(TEXTURES_FEU_DE_CAMP), ...DECO_FEU_DE_CAMP, ...FUMEE_FEU_DE_CAMP,
        heros.skin, ...(heros.objetEnMain ? [heros.objetEnMain] : []),
      ]),
    ]);
    const texturesFeu = BibliothequeModeles.textures([bloc]);
    await textures.charger(texturesFeu);
    const envoi = await images(textures, [
      ...Object.values(TEXTURES_FEU_DE_CAMP), ...DECO_FEU_DE_CAMP, ...FUMEE_FEU_DE_CAMP, ...texturesFeu,
      heros.skin, ...(heros.objetEnMain ? [heros.objetEnMain] : []),
    ]);

    canvas.className = 'chargement-scene';
    conteneur.prepend(canvas);
    const { width, height } = conteneur.getBoundingClientRect();
    const hors = canvas.transferControlToOffscreen();
    const travailleur = new TravailleurFeuDeCamp();
    const message: MessageFeuDeCamp = {
      type: 'init', canvas: hors, images: envoi, quads: bloc.parties[0]?.[0]?.modele.quads ?? [], heros: herosDe(heros), largeur: width, hauteur: height,
    };
    const prete = await new Promise<boolean>((resoudre) => {
      travailleur.onmessage = () => resoudre(true);
      travailleur.onerror = () => resoudre(false);
      travailleur.postMessage(message, [hors, ...envoi.map((i) => i.image)]);
    });
    if (!prete) {
      travailleur.terminate();
      canvas.remove();
      return null;
    }
    return new SceneChargement(travailleur, conteneur, textures);
  }

  /** Avancement du chargement (0 à 1) : le cookie du zombie dore avec lui. */
  definirProgression(part: number): void {
    if (Math.abs(part - this.derniere) < 0.01 && part < 1) return;
    this.derniere = part;
    this.envoyer({ type: 'progression', part });
  }

  /** Change le héros assis au coin du feu (classe ou skin choisies entre deux chargements). */
  async changerHeros(apparence: ApparenceEntite): Promise<void> {
    const chemins = [apparence.skin, ...(apparence.objetEnMain ? [apparence.objetEnMain] : [])];
    await this.textures.charger(chemins);
    const envoi = await images(this.textures, chemins);
    this.travailleur.postMessage({ type: 'heros', images: envoi, heros: herosDe(apparence) } satisfies MessageFeuDeCamp, envoi.map((i) => i.image));
  }

  demarrer(): void {
    this.envoyer({ type: 'demarrer' });
  }

  arreter(): void {
    this.envoyer({ type: 'arreter' });
  }

  private envoyer(message: MessageFeuDeCamp): void {
    this.travailleur.postMessage(message);
  }
}
