/**
 * Herbe posée sur les blocs d'herbe libres de l'arène, en plusieurs styles au
 * choix (option) : rien (Minecraft d'origine), touffes ondulantes, pixels 3D,
 * brins stylisés ou réalistes. Chaque style est créé à sa première sélection.
 */
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../../assets/textures';
import { CONFIG } from '../../config';
import type { StyleHerbe } from '../../options';
import type { MondeGenere } from '../generation';
import { Brins } from './brins';
import { Pixels } from './pixels';
import { Touffes } from './touffes';

/** Un style d'herbe affiché. */
export interface TapisHerbe {
  readonly maillage: THREE.Object3D;
  /** Vrai si le dessus des blocs doit être assombri sous le tapis (brins denses). */
  readonly assombritSol: boolean;
  /** Position du joueur, autour de laquelle les plantes se couchent. */
  majVisuel(x: number, z: number): void;
}

/** Touffes Minecraft en croix du décor, masquées quand un autre style les remplace. */
const DECOR_REMPLACE = ['short_grass'];
const BLOC_SOL = 'grass_block';
const INDEX_DESSUS = 1;  // matériaux d'un bloc : [côtés, dessus, dessous]

export class Herbe {
  private readonly tapis = new Map<StyleHerbe, TapisHerbe | null>();
  private actif: TapisHerbe | null = null;
  private couleurSol: THREE.Color | null = null;
  private readonly idArene: string;
  private readonly cases: readonly number[];
  private readonly scene: THREE.Scene;
  private readonly textures: BibliothequeTextures;

  /** Null si l'arène n'a aucun bloc d'herbe. */
  static creer(idArene: string, monde: MondeGenere, scene: THREE.Scene, textures: BibliothequeTextures): Herbe | null {
    const cases = casesHerbe(monde);
    return cases.length > 0 ? new Herbe(idArene, cases, scene, textures) : null;
  }

  private constructor(idArene: string, cases: readonly number[], scene: THREE.Scene, textures: BibliothequeTextures) {
    this.idArene = idArene;
    this.cases = cases;
    this.scene = scene;
    this.textures = textures;
  }

  definirStyle(style: StyleHerbe): void {
    if (!this.tapis.has(style)) this.tapis.set(style, this.creerTapis(style));
    for (const tapis of this.tapis.values()) if (tapis) tapis.maillage.visible = false;
    this.actif = this.tapis.get(style) ?? null;
    if (this.actif) this.actif.maillage.visible = true;

    for (const nom of DECOR_REMPLACE) {
      const decor = this.scene.getObjectByName(nom);
      if (decor) decor.visible = this.actif === null;
    }
    this.assombrirSol(this.actif?.assombritSol ?? false);
  }

  majVisuel(x: number, z: number): void {
    this.actif?.majVisuel(x, z);
  }

  private creerTapis(style: StyleHerbe): TapisHerbe | null {
    const graine = `herbe:${this.idArene}:${style}`;
    switch (style) {
      case 'minecraft':
        return null;
      case 'touffes': {
        const texture = this.textures.obtenir(CONFIG.herbe.touffes.texture);
        return texture ? new Touffes(graine, this.cases, texture, this.scene) : null;
      }
      case 'pixels': {
        const texture = this.textures.obtenir(CONFIG.herbe.pixels.texture);
        return texture ? new Pixels(graine, this.cases, texture, this.scene) : null;
      }
      case 'stylisee':
      case 'realiste':
        return new Brins(graine, this.cases, CONFIG.herbe.brins[style], this.scene);
    }
  }

  /** Sous un tapis dense, le dessus des blocs d'herbe est assombri : les interstices donnent de la profondeur. */
  private assombrirSol(actif: boolean): void {
    const sol = this.scene.getObjectByName(BLOC_SOL) as THREE.Mesh | undefined;
    const materiaux = sol && Array.isArray(sol.material) ? sol.material : [];
    const dessus = materiaux[INDEX_DESSUS] as THREE.MeshLambertMaterial | undefined;
    if (!dessus) return;
    this.couleurSol ??= dessus.color.clone();
    dessus.color.copy(this.couleurSol).multiplyScalar(actif ? CONFIG.herbe.assombrissementSol : 1);
  }
}

/** Centres (x, z) des blocs d'herbe du sol sur lesquels on peut marcher. */
function casesHerbe(monde: MondeGenere): number[] {
  const positions = monde.blocs.get('grass_block') ?? [];
  const cases: number[] = [];
  for (let i = 0; i < positions.length; i += 3) {
    const [x, y, z] = [positions[i], positions[i + 1], positions[i + 2]];
    if (y < 0 && !monde.carte.bloqueMarche(x, z)) cases.push(x, z);
  }
  return cases;
}
