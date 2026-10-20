/**
 * Moteur de rendu : renderer WebGPU de three.js (repli automatique sur WebGL2
 * si le navigateur ne gère pas WebGPU), scène, caméra et lumières.
 */
import * as THREE from 'three';
import { WebGPURenderer } from 'three/webgpu';
import { CONFIG } from '../config';
import type { DefinitionArene } from '../donnees/types';

/** Mémorise, pour la session, que WebGPU a lâché : les rechargements passent en WebGL2. */
const CLE_REPLI_WEBGL = 'block-survivors.repli-webgl';

export class Rendu {
  readonly renderer: WebGPURenderer;
  readonly scene = new THREE.Scene();
  readonly camera: THREE.PerspectiveCamera;
  private readonly ambiante: THREE.AmbientLight;
  private readonly soleil: THREE.DirectionalLight;

  /**
   * Crée le moteur : WebGPU si le navigateur le gère vraiment, sinon WebGL2.
   * Lève une erreur si aucun des deux n'est disponible.
   */
  static async creer(conteneur: HTMLElement): Promise<Rendu> {
    const repli = lireRepli() || !(await webgpuUtilisable());
    const rendu = new Rendu(conteneur, repli);
    await rendu.renderer.init();
    // GPU perdu en cours de route (pilote, mise en veille…) : plutôt qu'un écran
    // vide, on recharge la page en WebGL2
    rendu.renderer.onDeviceLost = () => {
      if (rendu.api !== 'WebGPU') return;
      try {
        window.sessionStorage.setItem(CLE_REPLI_WEBGL, '1');
      } catch {
        return;  // sans stockage, recharger reviendrait au même point
      }
      window.location.reload();
    };
    return rendu;
  }

  private constructor(conteneur: HTMLElement, forcerWebGL: boolean) {
    this.renderer = new WebGPURenderer({ antialias: true, forceWebGL: forcerWebGL });
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, CONFIG.rendu.pixelRatioMax));
    this.renderer.setSize(window.innerWidth, window.innerHeight);
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    conteneur.prepend(this.renderer.domElement);

    this.camera = new THREE.PerspectiveCamera(
      CONFIG.camera.fov, window.innerWidth / window.innerHeight, 0.5, 500,
    );

    // three.js calcule en unités physiques : une intensité de π = éclairage plein
    this.ambiante = new THREE.AmbientLight(0xffffff, CONFIG.rendu.ambiante * Math.PI);
    this.soleil = new THREE.DirectionalLight(0xffffff, CONFIG.rendu.soleil * Math.PI);
    this.soleil.position.fromArray(CONFIG.rendu.directionSoleil);
    this.scene.add(this.ambiante, this.soleil);

    window.addEventListener('resize', this.redimensionner);
  }

  /** « WebGPU » ou « WebGL2 » (repli). */
  get api(): string {
    return (this.renderer.backend as { isWebGPUBackend?: boolean }).isWebGPUBackend ? 'WebGPU' : 'WebGL2';
  }

  /** Ciel, brouillard et lumières propres à l'arène. */
  appliquerAmbiance(arene: DefinitionArene): void {
    this.scene.background = new THREE.Color(arene.ciel);
    this.scene.fog = new THREE.Fog(arene.ciel, arene.brouillard.debut, arene.brouillard.fin);
    const l = arene.lumiere;
    this.ambiante.color.set(l.couleurAmbiante);
    this.ambiante.intensity = l.ambiante * Math.PI;
    this.soleil.color.set(l.couleurSoleil);
    this.soleil.intensity = l.soleil * Math.PI;
  }

  dessiner(): void {
    this.renderer.render(this.scene, this.camera);
  }

  private readonly redimensionner = (): void => {
    this.camera.aspect = window.innerWidth / window.innerHeight;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(window.innerWidth, window.innerHeight);
  };
}

function lireRepli(): boolean {
  try {
    return window.sessionStorage.getItem(CLE_REPLI_WEBGL) === '1';
  } catch {
    return false;
  }
}

/**
 * Vrai si WebGPU est présent ET accepte ce que three.js lui demande. Certains
 * navigateurs exposent une version de WebGPU plus ancienne que celle visée par
 * three.js (vues de texture avec « swizzle ») : le jeu resterait vide. On
 * essaie donc une vraie vue de texture avant de choisir.
 */
async function webgpuUtilisable(): Promise<boolean> {
  type VueTexture = { createView(descripteur: object): unknown; destroy(): void };
  type Appareil = { createTexture(descripteur: object): VueTexture };
  type Gpu = { requestAdapter(): Promise<{ requestDevice(): Promise<Appareil> } | null> };
  const gpu = (navigator as { gpu?: Gpu }).gpu;
  if (!gpu) return false;
  try {
    const adaptateur = await gpu.requestAdapter();
    if (!adaptateur) return false;
    const appareil = await adaptateur.requestDevice();
    const texture = appareil.createTexture({ size: [1, 1], format: 'rgba8unorm', usage: 0x04 /* TEXTURE_BINDING */ });
    texture.createView({ swizzle: 'rgba' });
    texture.destroy();
    return true;
  } catch {
    return false;
  }
}
