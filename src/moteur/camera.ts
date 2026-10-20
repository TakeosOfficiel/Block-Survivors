/** Caméra de jeu : vue de dessus inclinée, suivi du joueur, zoom molette et secousses. */
import * as THREE from 'three';
import { CONFIG } from '../config';
import type { Entrees } from '../controles/entrees';
import type { DetecteurVisibilite } from '../entites/directeur';
import type { EtatPartie } from '../etat';
import { borner, facteurLissage } from '../utils/maths';

export class CameraJeu implements DetecteurVisibilite {
  private readonly camera: THREE.PerspectiveCamera;
  private readonly entrees: Entrees;
  private readonly etat: EtatPartie;
  private readonly sinInclinaison: number;
  private readonly cosInclinaison: number;
  private readonly champ = new THREE.Frustum();
  private readonly projection = new THREE.Matrix4();
  private readonly sphere = new THREE.Sphere();

  private x: number;   // point suivi (lissé)
  private z: number;
  private distanceCible: number = CONFIG.camera.distance;
  /** Intensité de la secousse (0 à 1). */
  private secousse = 0;
  private tempsSecousse = 0;
  /** Faux : les secousses sont ignorées (option du joueur). */
  secoussesActives = true;
  distance: number = CONFIG.camera.distance;

  constructor(camera: THREE.PerspectiveCamera, entrees: Entrees, etat: EtatPartie) {
    this.camera = camera;
    this.entrees = entrees;
    this.etat = etat;

    const inclinaison = (CONFIG.camera.inclinaisonDeg * Math.PI) / 180;
    this.sinInclinaison = Math.sin(inclinaison);
    this.cosInclinaison = Math.cos(inclinaison);
    // Pas de lacet : la caméra garde son inclinaison et ne fait que translater
    camera.rotation.set(-inclinaison, 0, 0);

    this.x = etat.joueur.x;
    this.z = etat.joueur.z;
    this.placer();
  }

  /** dtJeu pilote le suivi (cohérent avec la vitesse du temps), dtReel le zoom (fluide même au ralenti). */
  majVisuel(dtJeu: number, dtReel: number): void {
    const c = CONFIG.camera;

    const molette = this.entrees.consommerMolette();
    if (molette !== 0) {
      this.distanceCible = borner(
        this.distanceCible * Math.exp(molette * c.sensibiliteZoom),
        c.distanceMin,
        c.distanceMax,
      );
    }

    const kSuivi = facteurLissage(c.suivi, dtJeu);
    this.x += (this.etat.joueur.x - this.x) * kSuivi;
    this.z += (this.etat.joueur.z - this.z) * kSuivi;
    this.distance += (this.distanceCible - this.distance) * facteurLissage(c.lissageZoom, dtReel);

    // La secousse s'éteint en temps réel, mais reste figée pendant la pause
    if (this.secousse > 0 && dtJeu > 0) {
      this.secousse = Math.max(0, this.secousse - c.secousse.decroissance * dtReel);
      this.tempsSecousse += dtReel;
    }
    this.placer();
  }

  /** Ajoute de l'intensité à la secousse (0 à 1). */
  secouer(force: number): void {
    if (this.secoussesActives) this.secousse = Math.min(1, this.secousse + force);
  }

  /** Vrai si une sphère posée au sol en (x, z) apparaît à l'écran. */
  estVisible(x: number, z: number, rayon: number): boolean {
    this.sphere.center.set(x, 1, z);
    this.sphere.radius = rayon;
    return this.champ.intersectsSphere(this.sphere);
  }

  private placer(): void {
    // Bruit pseudo-aléatoire continu : somme de sinus de fréquences premières entre elles
    const s = CONFIG.camera.secousse;
    const amplitude = this.secousse * this.secousse * s.amplitude;
    const t = this.tempsSecousse * s.frequence;
    const decalageX = amplitude * (Math.sin(t) + Math.sin(t * 2.3 + 1.7)) * 0.5;
    const decalageZ = amplitude * (Math.sin(t * 1.3 + 4.1) + Math.sin(t * 3.1 + 0.6)) * 0.5;
    this.camera.position.set(
      this.x + decalageX,
      CONFIG.camera.hauteurVisee + this.distance * this.sinInclinaison,
      this.z + decalageZ + this.distance * this.cosInclinaison,
    );
    this.camera.updateMatrixWorld();
    this.projection.multiplyMatrices(this.camera.projectionMatrix, this.camera.matrixWorldInverse);
    this.champ.setFromProjectionMatrix(this.projection);
  }
}
