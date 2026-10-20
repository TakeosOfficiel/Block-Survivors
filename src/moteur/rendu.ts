/** Moteur de rendu : renderer WebGL, scène, caméra three.js et lumières. */
import * as THREE from 'three';
import { CONFIG } from '../config';
import type { DefinitionArene } from '../donnees/types';

export class Rendu {
  readonly renderer: THREE.WebGLRenderer;
  readonly scene = new THREE.Scene();
  readonly camera: THREE.PerspectiveCamera;
  private readonly ambiante: THREE.AmbientLight;
  private readonly soleil: THREE.DirectionalLight;

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
    this.ambiante = new THREE.AmbientLight(0xffffff, CONFIG.rendu.ambiante * Math.PI);
    this.soleil = new THREE.DirectionalLight(0xffffff, CONFIG.rendu.soleil * Math.PI);
    this.soleil.position.fromArray(CONFIG.rendu.directionSoleil);
    this.scene.add(this.ambiante, this.soleil);

    window.addEventListener('resize', this.redimensionner);
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
