/**
 * Caméra de jeu, deux vues (touche F5) :
 * - de dessus : inclinée, suit le joueur, zoom molette ;
 * - à la 3e personne (façon WoW) : en hauteur derrière le joueur, la souris
 *   capturée la fait tourner, la molette la rapproche ou l'éloigne.
 * Secousses dans les deux vues.
 */
import * as THREE from 'three';
import { CONFIG } from '../config';
import type { Entrees } from '../controles/entrees';
import type { DetecteurVisibilite } from '../entites/directeur';
import type { EtatPartie } from '../etat';
import type { ModeVue } from '../options';
import { borner, facteurLissage } from '../utils/maths';

const DEG = Math.PI / 180;

export class CameraJeu implements DetecteurVisibilite {
  private readonly camera: THREE.PerspectiveCamera;
  private readonly entrees: Entrees;
  private readonly etat: EtatPartie;
  private readonly sinInclinaison: number;
  private readonly cosInclinaison: number;
  private readonly champ = new THREE.Frustum();
  private readonly projection = new THREE.Matrix4();
  private readonly sphere = new THREE.Sphere();
  private readonly souris = { x: 0, y: 0 };

  private mode: ModeVue = 'dessus';
  private x: number;   // point suivi (lissé)
  private z: number;
  private distanceCible: number = CONFIG.camera.distance;
  /** Zoom de la vue à la 3e personne, gardé d'un changement de vue à l'autre. */
  private distanceTroisieme: number = CONFIG.camera.troisieme.distance;
  /** Orientation des vues à la souris (rad) : lacet autour de la verticale, tangage vers le bas. */
  private lacet = 0;
  private tangage = 0;
  /** Intensité de la secousse (0 à 1). */
  private secousse = 0;
  private tempsSecousse = 0;
  /** Point suivi à la place du joueur (multijoueur : spectateur d'un allié), null : le joueur. */
  cible: { readonly x: number; readonly z: number } | null = null;
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
    camera.rotation.order = 'YXZ';

    this.x = etat.joueur.x;
    this.z = etat.joueur.z;
    this.definirMode('dessus');
  }

  get vue(): ModeVue {
    return this.mode;
  }

  /** Vrai pour la vue tournée à la souris (3e personne). */
  get alaSouris(): boolean {
    return this.mode !== 'dessus';
  }

  /** Lacet (rad) qui oriente les touches de déplacement : 0 de dessus (haut de l'écran = −Z). */
  get lacetCommandes(): number {
    return this.mode === 'dessus' ? 0 : this.lacet;
  }

  definirMode(mode: ModeVue): void {
    const ancien = this.mode;
    this.mode = mode;
    const c = CONFIG.camera;
    // En quittant la vue de dessus : on regarde dans le sens où marche le joueur
    if (ancien === 'dessus' && mode !== 'dessus') this.lacet = this.etat.joueur.angle + Math.PI;
    if (mode === 'troisieme') this.tangage = c.troisieme.tangage * DEG;
    this.camera.fov = mode === 'dessus' ? c.fov : c.troisieme.fov;
    this.camera.rotation.set(mode === 'dessus' ? -Math.asin(this.sinInclinaison) : -this.tangage, mode === 'dessus' ? 0 : this.lacet, 0);
    this.camera.updateProjectionMatrix();
    this.x = this.etat.joueur.x;
    this.z = this.etat.joueur.z;
    this.distance = mode === 'troisieme' ? this.distanceTroisieme : this.distanceCible;
    this.placer();
  }

  /** dtJeu pilote le suivi (cohérent avec la vitesse du temps), dtReel le zoom (fluide même au ralenti). */
  majVisuel(dtJeu: number, dtReel: number): void {
    const c = CONFIG.camera;
    const molette = this.entrees.consommerMolette();
    this.entrees.consommerSouris(this.souris);
    const j = this.cible ?? this.etat.joueur;

    if (this.mode === 'dessus') {
      if (molette !== 0) {
        this.distanceCible = borner(this.distanceCible * Math.exp(molette * c.sensibiliteZoom), c.distanceMin, c.distanceMax);
      }
      const kSuivi = facteurLissage(c.suivi, dtJeu);
      this.x += (j.x - this.x) * kSuivi;
      this.z += (j.z - this.z) * kSuivi;
      this.distance += (this.distanceCible - this.distance) * facteurLissage(c.lissageZoom, dtReel);
    } else {
      // La souris tourne la caméra même en pause : on peut regarder autour de soi
      this.lacet -= this.souris.x * c.sensibiliteSouris;
      const v = c.troisieme;
      this.tangage = borner(this.tangage + this.souris.y * c.sensibiliteSouris, v.tangageMin * DEG, v.tangageMax * DEG);
      this.camera.rotation.set(-this.tangage, this.lacet, 0);
      if (molette !== 0) {
        this.distanceTroisieme = borner(this.distanceTroisieme * Math.exp(molette * c.sensibiliteZoom), v.distanceMin, v.distanceMax);
      }
      const kSuivi = facteurLissage(v.suivi, dtJeu);
      this.x += (j.x - this.x) * kSuivi;
      this.z += (j.z - this.z) * kSuivi;
      this.distance += (this.distanceTroisieme - this.distance) * facteurLissage(c.lissageZoom, dtReel);
    }

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
    const amplitude = this.secousse * this.secousse * s.amplitude * (this.mode === 'dessus' ? 1 : 0.35);
    const t = this.tempsSecousse * s.frequence;
    const decalageX = amplitude * (Math.sin(t) + Math.sin(t * 2.3 + 1.7)) * 0.5;
    const decalageZ = amplitude * (Math.sin(t * 1.3 + 4.1) + Math.sin(t * 3.1 + 0.6)) * 0.5;
    const position = this.camera.position;
    if (this.mode === 'dessus') {
      position.set(
        this.x + decalageX,
        CONFIG.camera.hauteurVisee + this.distance * this.sinInclinaison,
        this.z + decalageZ + this.distance * this.cosInclinaison,
      );
    } else {
      // Derrière le point visé, à l'opposé du regard
      const horizontal = Math.cos(this.tangage) * this.distance;
      position.set(
        this.x + Math.sin(this.lacet) * horizontal + decalageX,
        Math.max(0.3, CONFIG.camera.troisieme.hauteurVisee + Math.sin(this.tangage) * this.distance),
        this.z + Math.cos(this.lacet) * horizontal + decalageZ,
      );
    }
    this.camera.updateMatrixWorld();
    this.projection.multiplyMatrices(this.camera.projectionMatrix, this.camera.matrixWorldInverse);
    this.champ.setFromProjectionMatrix(this.projection);
  }
}
