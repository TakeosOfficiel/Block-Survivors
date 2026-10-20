/** Caméra de jeu : vue de dessus inclinée, suivi du joueur et zoom molette. */
import type * as THREE from 'three';
import { CONFIG } from '../config';
import type { Entrees } from '../controles/entrees';
import type { EtatPartie } from '../etat';
import { borner, facteurLissage } from '../utils/maths';

export class CameraJeu {
  private readonly camera: THREE.PerspectiveCamera;
  private readonly entrees: Entrees;
  private readonly etat: EtatPartie;
  private readonly sinInclinaison: number;
  private readonly cosInclinaison: number;

  private x: number;   // point suivi (lissé)
  private z: number;
  private distanceCible: number = CONFIG.camera.distance;
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

    this.placer();
  }

  private placer(): void {
    this.camera.position.set(
      this.x,
      CONFIG.camera.hauteurVisee + this.distance * this.sinInclinaison,
      this.z + this.distance * this.cosInclinaison,
    );
  }
}
