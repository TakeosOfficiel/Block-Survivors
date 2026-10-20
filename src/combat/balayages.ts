/**
 * Mécanique « balayage » (épée) : frappe en arc de cercle devant le joueur,
 * avec la particule de balayage vanilla (sweep, 8 images) posée au sol.
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { ARMES, IDS_ARMES } from '../donnees/armes';
import { ecartAngle } from '../utils/maths';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';

const COULEURS_SECOURS = [0xffffff, 0xe0e0e0];

export class Balayages {
  private readonly contexte: ContexteCombat;
  private readonly tampon = new Int32Array(256);
  /** Images de l'effet, par index d'arme (vide si introuvables). */
  private readonly images: (THREE.Texture | null)[][];
  private readonly effet: THREE.Mesh;
  private readonly materiau: THREE.MeshBasicMaterial;
  private imagesEnCours: (THREE.Texture | null)[] = [];
  private temps = -1;

  constructor(contexte: ContexteCombat, textures: BibliothequeTextures, scene: THREE.Scene) {
    this.contexte = contexte;
    this.images = IDS_ARMES.map((id) => {
      const m = ARMES[id].mecanique;
      return m.type === 'balayage' ? m.effet.map((chemin) => textures.obtenir(chemin)) : [];
    });

    // Effet de particule (dégradé d'opacité) : translucide, comme en jeu
    // La première image disponible fixe le shader ; changer d'image ne le recompile pas
    const premiere = this.images.flat().find((t) => t !== null) ?? null;
    this.materiau = new THREE.MeshBasicMaterial({ map: premiere, transparent: true, depthWrite: false, side: THREE.DoubleSide });
    const geometrie = new THREE.PlaneGeometry(1, 1);
    geometrie.rotateX(-Math.PI / 2);
    this.effet = new THREE.Mesh(geometrie, this.materiau);
    this.effet.name = 'balayage';
    this.effet.visible = false;
    scene.add(this.effet);
  }

  frapper(arme: ArmeEquipee): boolean {
    const m = arme.definition.mecanique;
    if (m.type !== 'balayage') return false;
    const { etat, ennemis, particules } = this.contexte;
    const j = etat.joueur;
    const s = arme.stats;

    const n = ennemis.chercher(j.x, j.z, s.zone, this.tampon);
    for (let k = 0; k < n; k++) {
      const e = ennemis.obtenir(this.tampon[k]);
      const dx = e.x - j.x;
      const dz = e.z - j.z;
      const colle = dx * dx + dz * dz < 0.8 * 0.8;
      if (colle || Math.abs(ecartAngle(j.angle, Math.atan2(dx, dz))) <= m.angle / 2) {
        ennemis.blesser(e, s.degats, j.x, j.z, s.recul);
      }
    }
    this.contexte.declencherGesteJoueur();

    // Effet visuel devant le joueur
    const avantX = Math.sin(j.angle) * s.zone * 0.45;
    const avantZ = Math.cos(j.angle) * s.zone * 0.45;
    this.imagesEnCours = this.images[arme.index];
    if (this.imagesEnCours.some((t) => t !== null)) {
      this.temps = 0;
      this.effet.position.set(j.x + avantX, 0.6, j.z + avantZ);
      this.effet.rotation.y = j.angle + Math.PI;  // le haut de l'image vers l'avant
      this.effet.scale.setScalar(s.zone * 1.6);
    } else {
      particules.emettre(j.x + avantX, 1, j.z + avantZ, CONFIG.particules.mort, COULEURS_SECOURS);
    }
    return true;
  }

  majVisuel(dtJeu: number): void {
    if (this.temps < 0) return;
    const duree = CONFIG.armes.dureeBalayage;
    const index = Math.floor((this.temps / duree) * this.imagesEnCours.length);
    if (index >= this.imagesEnCours.length) {
      this.temps = -1;
      this.effet.visible = false;
      return;
    }
    const image = this.imagesEnCours[index];
    if (image) this.materiau.map = image;
    this.effet.visible = image !== null;
    this.temps += dtJeu;
  }
}
