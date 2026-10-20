/**
 * Moteur de rendu : renderer WebGPU de three.js (repli automatique sur WebGL2
 * si le navigateur ne gère pas WebGPU), scène, caméra, lumières et shaders,
 * réglables par niveau : ombres du soleil, relief des textures, lumière du
 * ciel, lueurs, étalonnage des couleurs, occlusion ambiante.
 */
import * as THREE from 'three';
import { RenderPipeline, WebGPURenderer } from 'three/webgpu';
import {
  float, length, mrt, normalView, output, packNormalToRGB, pass, sample, saturation, screenUV, smoothstep,
  unpackRGBToNormal, vec4,
} from 'three/tsl';
import { bloom } from 'three/addons/tsl/display/BloomNode.js';
import { ao } from 'three/addons/tsl/display/GTAONode.js';
import { lirePixels } from '../assets/pixels';
import { CONFIG } from '../config';
import type { DefinitionArene } from '../donnees/types';
import type { NiveauShaders } from '../options';

/** Objets qui ne projettent pas d'ombre (effets translucides, sol lointain). */
const SANS_OMBRE_PORTEE = new Set(['ombres', 'balayage', 'foudre', 'exterieur', 'particules']);

/** Mémorise, pour la session, que WebGPU a lâché : les rechargements passent en WebGL2. */
const CLE_REPLI_WEBGL = 'block-survivors.repli-webgl';

export class Rendu {
  readonly renderer: WebGPURenderer;
  readonly scene = new THREE.Scene();
  readonly camera: THREE.PerspectiveCamera;
  private readonly ambiante: THREE.AmbientLight;
  private readonly soleil: THREE.DirectionalLight;
  /** Lumière du ciel (shaders) : bleutée par-dessus, chaude par-dessous. */
  private readonly ciel = new THREE.HemisphereLight(0xffffff, 0xffffff, 0);
  private lumiere: DefinitionArene['lumiere'] | null = null;
  /** Relief (normal map) généré pour chaque texture, à la demande. */
  private readonly reliefs = new Map<THREE.Texture, THREE.DataTexture | null>();
  private pipeline: RenderPipeline | null = null;
  private niveau: NiveauShaders = 'aucun';
  private tailleArene = 60;

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
    this.soleil.position.fromArray(CONFIG.rendu.directionSoleil).normalize().multiplyScalar(80);
    this.scene.add(this.ambiante, this.soleil, this.soleil.target, this.ciel);

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
    this.tailleArene = arene.taille;
    this.lumiere = l;
    this.ciel.color.set(arene.ciel);
    this.ciel.groundColor.set(CONFIG.shaders.lumiere.couleurSol);
  }

  /** À appeler une fois la scène construite : qui projette et reçoit les ombres. */
  preparerOmbres(): void {
    this.scene.traverse((objet) => {
      if (!(objet as THREE.Mesh).isMesh) return;
      objet.receiveShadow = true;
      objet.castShadow = !SANS_OMBRE_PORTEE.has(objet.name);
    });
  }

  /**
   * Shaders : « aucun » = rendu Minecraft d'origine ; « legers » = ombres du
   * soleil, blocs lumineux qui rayonnent, hautes lumières adoucies ;
   * « complets » = idem avec des ombres plus fines et l'occlusion ambiante
   * (recoins et pieds de murs assombris).
   */
  definirShaders(niveau: NiveauShaders): void {
    if (niveau === this.niveau && (this.pipeline !== null) === (niveau !== 'aucun')) return;
    this.niveau = niveau;
    const s = CONFIG.shaders;
    const actifs = niveau !== 'aucun';

    // Ombres du soleil : une seule carte couvre toute l'arène (petite, caméra fixe)
    this.renderer.shadowMap.enabled = actifs;
    this.renderer.shadowMap.type = THREE.PCFShadowMap;
    this.soleil.castShadow = actifs;
    if (niveau !== 'aucun') {
      const ombre = this.soleil.shadow;
      ombre.mapSize.setScalar(s.tailleOmbres[niveau]);
      ombre.radius = s.flouOmbres[niveau];
      ombre.bias = s.biaisOmbres;
      ombre.normalBias = s.biaisNormalOmbres;
      const d = this.tailleArene / 2 + s.margeOmbres;
      Object.assign(ombre.camera, { left: -d, right: d, top: d, bottom: -d, near: 1, far: 200 });
      ombre.camera.updateProjectionMatrix();
      ombre.map?.dispose();
      ombre.map = null;
    }

    // Lumière : en partie remplacée par celle du ciel, soleil plus franc
    const l = this.lumiere;
    if (l) {
      const lu = s.lumiere;
      this.ambiante.intensity = l.ambiante * Math.PI * (actifs ? lu.ambiante : 1);
      this.ciel.intensity = actifs ? l.ambiante * Math.PI * lu.ciel : 0;
      this.soleil.intensity = l.soleil * Math.PI * (actifs ? lu.soleil : 1);
    }

    // Les blocs lumineux dépassent le blanc : la lueur les fait rayonner
    this.renderer.toneMapping = actifs ? THREE.NeutralToneMapping : THREE.NoToneMapping;
    this.scene.traverse((objet) => {
      const materiaux = (objet as THREE.Mesh).material;
      if (!materiaux) return;
      for (const materiau of Array.isArray(materiaux) ? materiaux : [materiaux]) {
        this.appliquerRelief(materiau, actifs);
        if (materiau.userData['lumineux']) {
          const m = materiau as THREE.MeshBasicMaterial;
          const base = (m.userData['couleurBase'] ??= m.color.clone()) as THREE.Color;
          m.color.copy(base).multiplyScalar(actifs ? s.intensiteLumineux : 1);
        }
        materiau.needsUpdate = true;  // recompile avec ou sans ombres
      }
    });
    // Les ombres rondes sous les entités font double emploi avec les vraies ombres
    const ombresRondes = this.scene.getObjectByName('ombres');
    if (ombresRondes) ombresRondes.visible = !actifs;

    this.pipeline?.dispose();
    this.pipeline = actifs ? this.creerPipeline(niveau) : null;
  }

  dessiner(): void {
    if (this.pipeline) this.pipeline.render();
    else this.renderer.render(this.scene, this.camera);
  }

  /** Rendu de la scène, puis occlusion ambiante (complets) et lueur des zones les plus vives. */
  private creerPipeline(niveau: NiveauShaders): RenderPipeline {
    const s = CONFIG.shaders;
    const scene = pass(this.scene, this.camera);
    if (niveau === 'complets') scene.setMRT(mrt({ output, normal: packNormalToRGB(normalView) }));
    const couleur = scene.getTextureNode('output');
    const image = niveau === 'complets' ? this.occlusion(scene, couleur) : couleur;
    const lueur = bloom(image, s.lueur.force, s.lueur.rayon, s.lueur.seuil);
    // Étalonnage : couleurs plus saturées, bords de l'écran assombris
    const e = s.etalonnage;
    const vignette = float(1).sub(smoothstep(0.35, 0.85, length(screenUV.sub(0.5))).mul(e.vignette));
    const final = image.add(lueur);
    return new RenderPipeline(this.renderer, vec4(saturation(final.rgb, e.saturation).mul(vignette), final.a));
  }

  /**
   * Relief des textures (comme les normal maps des packs de ressources) :
   * calculé depuis la luminosité de chaque pixel, il accroche la lumière du
   * soleil. Textures animées (bandes) et matériaux sans texture exclus.
   */
  private appliquerRelief(materiau: THREE.Material, actif: boolean): void {
    const m = materiau as THREE.MeshLambertMaterial;
    if (!('normalMap' in m) || !m.map || m.transparent || m.userData['lumineux']) return;
    const relief = actif ? this.relief(m.map) : null;
    m.normalMap = relief;
    if (relief) m.normalScale.setScalar(CONFIG.shaders.relief);
  }

  private relief(carte: THREE.Texture): THREE.DataTexture | null {
    if (this.reliefs.has(carte)) return this.reliefs.get(carte) ?? null;
    let relief: THREE.DataTexture | null = null;
    const image = carte.image as HTMLImageElement | undefined;
    if (image?.width && carte.repeat.y === 1 && carte.offset.y === 0) {
      try {
        relief = creerRelief(lirePixels(image));
        relief.wrapS = carte.wrapS;
        relief.wrapT = carte.wrapT;
        relief.repeat.copy(carte.repeat);
      } catch {
        relief = null;  // image sans CORS
      }
    }
    this.reliefs.set(carte, relief);
    return relief;
  }

  /** Occlusion ambiante (GTAO) calculée à partir de la profondeur et des normales de la scène. */
  private occlusion(scene: ReturnType<typeof pass>, couleur: ReturnType<ReturnType<typeof pass>['getTextureNode']>) {
    const s = CONFIG.shaders;
    const normales = scene.getTextureNode('normal');
    const normale = sample((uv) => unpackRGBToNormal(normales.sample(uv)));
    const occlusion = ao(scene.getTextureNode('depth'), normale, this.camera);
    occlusion.resolutionScale = s.occlusion.resolution;
    occlusion.radius.value = s.occlusion.rayon;
    return vec4(couleur.rgb.mul(occlusion.getTextureNode().r), couleur.a);
  }

  private readonly redimensionner = (): void => {
    this.camera.aspect = window.innerWidth / window.innerHeight;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(window.innerWidth, window.innerHeight);
  };
}

/** Normal map d'une texture : pentes de sa luminosité (pixels transparents ignorés). */
function creerRelief(image: ImageData): THREE.DataTexture {
  const { width: l, height: h, data } = image;
  const hauteur = (x: number, y: number): number => {
    const i = (((y + h) % h) * l + ((x + l) % l)) * 4;
    return data[i + 3] < 128 ? 0.5 : (data[i] * 0.299 + data[i + 1] * 0.587 + data[i + 2] * 0.114) / 255;
  };
  const sortie = new Uint8Array(l * h * 4);
  const n = new THREE.Vector3();
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < l; x++) {
      n.set(-(hauteur(x + 1, y) - hauteur(x - 1, y)) * 2, (hauteur(x, y + 1) - hauteur(x, y - 1)) * 2, 1).normalize();
      // Lignes inversées : la texture d'origine est retournée à l'envoi au GPU (flipY)
      const i = ((h - 1 - y) * l + x) * 4;
      sortie[i] = (n.x * 0.5 + 0.5) * 255;
      sortie[i + 1] = (n.y * 0.5 + 0.5) * 255;
      sortie[i + 2] = (n.z * 0.5 + 0.5) * 255;
      sortie[i + 3] = 255;
    }
  }
  const relief = new THREE.DataTexture(sortie, l, h);
  relief.magFilter = THREE.NearestFilter;
  relief.minFilter = THREE.NearestFilter;
  relief.needsUpdate = true;
  return relief;
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
