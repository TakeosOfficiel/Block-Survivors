/**
 * Moteur de rendu : renderer WebGPU de three.js (repli automatique sur WebGL2
 * si le navigateur ne gère pas WebGPU), scène, caméra, lumières, et ombres du
 * soleil quand les shaders sont actifs. Les textures restent celles du jeu.
 */
import * as THREE from 'three';
import { WebGPURenderer } from 'three/webgpu';
import { CONFIG } from '../config';
import type { DefinitionArene } from '../donnees/types';
import type { NiveauShaders } from '../options';

/** Mémorise, pour la session, que WebGPU a lâché : les rechargements passent en WebGL2. */
const CLE_REPLI_WEBGL = 'block-survivors.repli-webgl';

/** Objets qui ne projettent pas d'ombre (effets translucides, sol lointain). */
const SANS_OMBRE_PORTEE = new Set([
  'ombres', 'balayage', 'foudre', 'exterieur', 'particules', 'chiffres', 'etiquette', 'barres-vie',
  // Le feu et le magma de la tornade éclairent ; ses brûlures sont collées au sol
  'tornades:feu', 'tornades:magma', 'tornades:brûlures',
]);

export class Rendu {
  readonly renderer: WebGPURenderer;
  readonly scene = new THREE.Scene();
  readonly camera: THREE.PerspectiveCamera;
  private readonly ambiante: THREE.AmbientLight;
  private readonly soleil: THREE.DirectionalLight;
  private niveau: NiveauShaders = 'aucun';
  /** Plus grand côté de l'arène (la carte d'ombres la couvre en entier). */
  private tailleArene = 60;

  /**
   * Crée le moteur : WebGPU si le navigateur le gère vraiment, sinon WebGL2.
   * Lève une erreur si aucun des deux n'est disponible.
   */
  static async creer(conteneur: HTMLElement): Promise<Rendu> {
    const repli = lireRepli() || !(await webgpuUtilisable());
    const rendu = new Rendu(conteneur, repli);
    await rendu.renderer.init();
    partagerShadersInstances(rendu.renderer);
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
    // Ombres toujours branchées dans les matériaux : les (dés)activer ne recompile rien,
    // on change seulement leur intensité et leur mise à jour (voir definirShaders)
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFShadowMap;
    conteneur.prepend(this.renderer.domElement);

    this.camera = new THREE.PerspectiveCamera(
      CONFIG.camera.fov, window.innerWidth / window.innerHeight, 0.5, 500,
    );

    // three.js calcule en unités physiques : une intensité de π = éclairage plein
    this.ambiante = new THREE.AmbientLight(0xffffff, CONFIG.rendu.ambiante * Math.PI);
    this.soleil = new THREE.DirectionalLight(0xffffff, CONFIG.rendu.soleil * Math.PI);
    this.soleil.position.fromArray(CONFIG.rendu.directionSoleil).normalize().multiplyScalar(80);
    this.soleil.castShadow = true;
    this.soleil.shadow.intensity = 0;
    this.soleil.shadow.autoUpdate = false;
    this.soleil.shadow.bias = CONFIG.shaders.biaisOmbres;
    this.soleil.shadow.normalBias = CONFIG.shaders.biaisNormalOmbres;
    this.scene.add(this.ambiante, this.soleil, this.soleil.target);

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
    this.tailleArene = Math.max(arene.largeur, arene.profondeur);
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
   * Shaders : « aucun » = rendu d'origine ; « legers » et « complets » = ombres
   * du soleil (plus fines en « complets »). Une seule carte d'ombres couvre
   * toute l'arène : elle est petite et la caméra ne tourne pas. Désactivées,
   * les ombres sont invisibles et ne sont plus recalculées : aucun shader à
   * recompiler, le changement est immédiat.
   */
  definirShaders(niveau: NiveauShaders): void {
    if (niveau === this.niveau) return;
    this.niveau = niveau;
    const s = CONFIG.shaders;
    const actifs = niveau !== 'aucun';
    const ombre = this.soleil.shadow;
    ombre.intensity = actifs ? 1 : 0;
    ombre.autoUpdate = actifs;
    if (niveau !== 'aucun') {
      const taille = s.tailleOmbres[niveau];
      ombre.mapSize.setScalar(taille);
      ombre.map?.setSize(taille, taille);
      ombre.radius = s.flouOmbres[niveau];
      const d = this.tailleArene / 2 + s.margeOmbres;
      Object.assign(ombre.camera, { left: -d, right: d, top: d, bottom: -d, near: 1, far: 200 });
      ombre.camera.updateProjectionMatrix();
    }
    // Les ombres rondes sous les entités font double emploi avec les vraies ombres
    const ombresRondes = this.scene.getObjectByName('ombres');
    if (ombresRondes) ombresRondes.visible = !actifs;
  }

  dessiner(): void {
    this.renderer.render(this.scene, this.camera);
  }

  /**
   * Photo de la scène vue par `camera`, réduite à `largeur` pixels de large
   * (image de l'écran de chargement). Dessinée sur le canvas du jeu, caché
   * sous l'écran de chargement, puis recopiée tout de suite ; null si elle ne
   * peut pas être relue ou si elle est vide.
   */
  photographier(camera: THREE.PerspectiveCamera, largeur: number): HTMLCanvasElement | null {
    const toile = this.renderer.domElement;
    camera.aspect = toile.width / toile.height;
    camera.updateProjectionMatrix();
    this.renderer.render(this.scene, camera);
    const photo = document.createElement('canvas');
    photo.width = Math.min(largeur, toile.width);
    photo.height = Math.round((photo.width * toile.height) / toile.width);
    const ctx = photo.getContext('2d', { willReadFrequently: true });
    if (!ctx || photo.height < 1) return null;
    try {
      ctx.drawImage(toile, 0, 0, photo.width, photo.height);
      // Une image toute noire ou transparente : le dessin n'a pas pu être relu
      const centre = ctx.getImageData(photo.width >> 1, photo.height >> 1, 1, 1).data;
      if (centre[3] === 0 || centre[0] + centre[1] + centre[2] === 0) return null;
    } catch {
      return null;
    }
    return photo;
  }

  /**
   * Prépare la carte graphique pendant le chargement : chaque maillage encore
   * vide (boss, mobs pas encore apparus, golem, particules…) reçoit une
   * instance, puis tous les shaders sont compilés d'un coup, en parallèle.
   * Sans cela, la première apparition d'un boss ou d'un nouveau mob fige le
   * jeu un instant. `surProgression(faites, total)` suit la préparation.
   */
  async prechauffer(surProgression?: (faites: number, total: number) => void): Promise<void> {
    const vides: THREE.InstancedMesh[] = [];
    const maillages: THREE.Mesh[] = [];
    this.scene.traverse((objet) => {
      const maillage = objet as THREE.InstancedMesh;
      if (maillage.isInstancedMesh && maillage.count === 0 && maillage.visible) vides.push(maillage);
      if ((objet as THREE.Mesh).isMesh && objet.visible) maillages.push(objet as THREE.Mesh);
    });
    // Un premier dessin sans aucun modèle met en place la lumière et les ombres :
    // les shaders compilés ensuite sont exactement ceux que le jeu utilisera
    this.renderer.setScissorTest(true);
    this.renderer.setScissor(0, 0, 1, 1);
    for (const m of maillages) m.visible = false;
    this.renderer.render(this.scene, this.camera);
    for (const m of maillages) m.visible = true;
    // Tous les modèles encore vides reçoivent une instance le temps de la préparation
    for (const m of vides) m.count = 1;
    // Toutes les compilations sont lancées d'un coup : la carte graphique les mène en
    // parallèle (au lieu d'attendre la fin de chacune avant la suivante)
    // (la barre en montre les trois quarts : le dernier quart est le dessin final)
    const total = maillages.length * 4;
    let faites = 0;
    await Promise.all(maillages.map((m) => this.renderer.compileAsync(m, this.camera, this.scene).then(() => {
      surProgression?.(++faites * 3, total);
    })));
    await new Promise((resoudre) => requestAnimationFrame(resoudre));  // laisse la barre s'afficher
    // Un dessin d'un seul pixel compile ce qui reste (passes d'ombre, que le moteur ne sait
    // compiler qu'en dessinant) sans rien coûter à l'écran
    this.renderer.render(this.scene, this.camera);
    surProgression?.(total, total);
    this.renderer.setScissorTest(false);
    for (const m of vides) m.count = 0;
  }

  private readonly redimensionner = (): void => {
    this.camera.aspect = window.innerWidth / window.innerHeight;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(window.innerWidth, window.innerHeight);
  };
}

/**
 * Les matrices des instances passent toujours par des attributs de sommets.
 * Sinon, three range celles des petits maillages (moins de ~256 instances)
 * dans un tableau uniforme propre à chacun : son nom et sa taille entrent
 * dans le code du shader, et chaque morceau de chaque modèle demande ses
 * propres shaders (≈ 290 à compiler au chargement au lieu d'une vingtaine).
 * Cette limite ne sert qu'à ce choix (instances, squelettes, plages).
 */
function partagerShadersInstances(renderer: WebGPURenderer): void {
  const capacites = (renderer.backend as { capabilities?: { getUniformBufferLimit(): number } }).capabilities;
  if (capacites) capacites.getUniformBufferLimit = () => 0;
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
