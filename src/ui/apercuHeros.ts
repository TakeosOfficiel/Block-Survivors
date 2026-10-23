/**
 * Aperçu 3D du héros dans le menu (et l'écran d'apparence) : le vrai modèle
 * du jeu, sa skin et son objet en main, qui respire doucement. Glisser à la
 * souris le fait tourner. Un petit moteur séparé, sur son propre canvas,
 * déplacé d'un écran à l'autre ; il ne dessine que lorsqu'il est attaché.
 */
import { ArmureHeros } from '../modeles/armure';
import type { IdMonde, PieceArmure } from '../donnees/types';
import * as THREE from 'three';
import { WebGPURenderer } from 'three/webgpu';
import type { BibliothequeTextures } from '../assets/textures';
import { MODELES } from '../donnees/modeles';
import type { ApparenceEntite } from '../donnees/types';
import { avancerAnimation, calculerRotations, creerEtatAnimation } from '../modeles/animation';
import { compilerModele, type ModeleCompile } from '../modeles/compilation';
import { creerPoseModele, ModeleInstancie, type PoseModele } from '../modeles/rendu';

/** Orientation de départ : de trois quarts, comme sur un écran de personnage. */
const ANGLE_INITIAL = -0.45;
/** Radians par pixel glissé. */
const SENSIBILITE = 0.012;

interface Heros {
  readonly scene: THREE.Scene;
  readonly modele: ModeleCompile;
  readonly rendu: ModeleInstancie;
  readonly pose: PoseModele;
  readonly armure: ArmureHeros;
}

export class ApercuHeros {
  private readonly renderer: WebGPURenderer;
  private readonly camera = new THREE.PerspectiveCamera(30, 1, 0.1, 50);
  private readonly textures: BibliothequeTextures;
  private readonly animation = creerEtatAnimation();
  private heros: Heros | null = null;
  private cle = '';
  /** Armure de la Collection à faire porter au héros (branchée au lancement). */
  armure: () => Partial<Record<PieceArmure, IdMonde>> = () => ({});
  private conteneur: HTMLElement | null = null;
  private angle = ANGLE_INITIAL;
  private glisse: number | null = null;
  private dernier = 0;
  private boucle = 0;

  private constructor(renderer: WebGPURenderer, textures: BibliothequeTextures) {
    this.renderer = renderer;
    this.textures = textures;
    const canvas = renderer.domElement;
    canvas.className = 'apercu-heros';
    canvas.addEventListener('pointerdown', (e) => {
      this.glisse = e.clientX;
      canvas.setPointerCapture(e.pointerId);
    });
    canvas.addEventListener('pointermove', (e) => {
      if (this.glisse === null) return;
      this.angle += (e.clientX - this.glisse) * SENSIBILITE;
      this.glisse = e.clientX;
    });
    const lacher = () => { this.glisse = null; };
    canvas.addEventListener('pointerup', lacher);
    canvas.addEventListener('pointercancel', lacher);
  }

  /** Même moteur que le jeu (WebGPU ou repli WebGL2) ; null s'il ne démarre pas. */
  static async creer(textures: BibliothequeTextures, webgl: boolean): Promise<ApercuHeros | null> {
    try {
      const renderer = new WebGPURenderer({ antialias: true, alpha: true, forceWebGL: webgl });
      renderer.outputColorSpace = THREE.SRGBColorSpace;
      renderer.setClearColor(0x000000, 0);
      await renderer.init();
      return new ApercuHeros(renderer, textures);
    } catch {
      return null;
    }
  }

  /** Place le canvas dans `conteneur` (à sa taille) et lance l'animation. */
  attacher(conteneur: HTMLElement): void {
    this.conteneur = conteneur;
    conteneur.append(this.renderer.domElement);
    this.redimensionner();
    if (!this.boucle) {
      this.dernier = performance.now();
      this.boucle = requestAnimationFrame(this.image);
    }
  }

  /** Arrête de dessiner (écran masqué, partie lancée). */
  detacher(): void {
    cancelAnimationFrame(this.boucle);
    this.boucle = 0;
    this.renderer.domElement.remove();
    this.conteneur = null;
  }

  /** Change le personnage affiché (classe, skin, gabarit). */
  afficher(apparence: ApparenceEntite): void {
    const armure = this.armure();
    const cle = `${apparence.modele}|${apparence.skin}|${apparence.objetEnMain ?? ''}|${this.textures.url(apparence.skin) ?? ''}|${JSON.stringify(armure)}`;
    if (cle === this.cle) return;
    this.cle = cle;
    if (this.heros) liberer(this.heros.scene);

    const scene = new THREE.Scene();
    // Lumière du jour : ambiante douce et soleil de face, un peu en hauteur
    scene.add(new THREE.AmbientLight(0xffffff, 0.55 * Math.PI));
    const soleil = new THREE.DirectionalLight(0xffffff, 0.6 * Math.PI);
    soleil.position.set(2, 4, 5);
    scene.add(soleil);
    const modele = compilerModele(MODELES[apparence.modele]);
    const rendu = new ModeleInstancie(modele, apparence, this.textures, 1, scene);
    this.heros = { scene, modele, rendu, pose: creerPoseModele(modele), armure: new ArmureHeros(modele, armure, this.textures, scene) };
    // Cadrage : tout le modèle, centré à mi-hauteur
    const h = modele.hauteur;
    this.camera.position.set(0, h * 0.55, h * 2.9);
    this.camera.lookAt(0, h * 0.5, 0);
  }

  private redimensionner(): void {
    const c = this.conteneur;
    if (!c) return;
    const largeur = Math.max(1, c.clientWidth);
    const hauteur = Math.max(1, c.clientHeight);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.renderer.setSize(largeur, hauteur, false);
    this.camera.aspect = largeur / hauteur;
    this.camera.updateProjectionMatrix();
  }

  private readonly image = (maintenant: number): void => {
    this.boucle = requestAnimationFrame(this.image);
    const dt = Math.min(0.1, (maintenant - this.dernier) / 1000);
    this.dernier = maintenant;
    const h = this.heros;
    if (!h || !this.conteneur) return;
    // Le conteneur peut changer de taille (zoom d'interface, fenêtre)
    const canvas = this.renderer.domElement;
    if (canvas.width !== Math.round(this.conteneur.clientWidth * this.renderer.getPixelRatio())) this.redimensionner();

    avancerAnimation(this.animation, h.modele, dt, 0);
    h.pose.angle = this.angle;
    calculerRotations(h.modele, this.animation, h.pose.rotations);
    h.rendu.commencer();
    h.rendu.ajouter(h.pose);
    h.rendu.terminer();
    h.armure.poser(h.pose);
    this.renderer.render(h.scene, this.camera);
  };
}

/** Libère les ressources du héros précédent (géométries et matériaux ; les textures restent partagées). */
function liberer(scene: THREE.Scene): void {
  scene.traverse((objet) => {
    const maillage = objet as THREE.Mesh;
    if (!maillage.isMesh) return;
    maillage.geometry.dispose();
    const materiaux = Array.isArray(maillage.material) ? maillage.material : [maillage.material];
    for (const m of materiaux) m.dispose();
  });
}
