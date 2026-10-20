/** Moteur de rendu : renderer WebGL, scène, caméra three.js et lumières. */
import * as THREE from 'three';
import { CONFIG } from '../config';

export class Rendu {
  readonly renderer: THREE.WebGLRenderer;
  readonly scene = new THREE.Scene();
  readonly camera: THREE.PerspectiveCamera;

  /** Lève une erreur si WebGL est indisponible. */
  constructor(conteneur: HTMLElement) {
    this.renderer = new THREE.WebGLRenderer({ antialias: true });
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, CONFIG.rendu.pixelRatioMax));
    this.renderer.setSize(window.innerWidth, window.innerHeight);
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    conteneur.prepend(this.renderer.domElement);

    this.camera = new THREE.PerspectiveCamera(
      CONFIG.camera.fov, window.innerWidth / window.innerHeight, 0.5, 500,
    );

    // three.js calcule en unités physiques : une intensité de π = éclairage plein
    const ambiante = new THREE.AmbientLight(0xffffff, CONFIG.rendu.ambiante * Math.PI);
    const soleil = new THREE.DirectionalLight(0xffffff, CONFIG.rendu.soleil * Math.PI);
    soleil.position.fromArray(CONFIG.rendu.directionSoleil);
    this.scene.add(ambiante, soleil);

    window.addEventListener('resize', this.redimensionner);
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
