/**
 * Mécanique « chute » (Lame colossale) : une arme géante tombe du ciel sur le
 * plus gros groupe d'ennemis, pointe en bas. Son ombre grandit au sol pendant
 * la chute ; à l'impact, dégâts de zone, éboulis et secousse ; elle reste
 * plantée un instant puis s'efface. Plusieurs lames tombent l'une après l'autre.
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { ARMES, IDS_ARMES } from '../donnees/armes';
import { blocDuSol } from '../donnees/arenes';
import { BLOCS } from '../donnees/blocs';
import { GERBES } from '../donnees/particules';
import type { Ombres } from '../effets/ombres';
import type { CheminTexture } from '../donnees/types';
import type { Point } from '../monde/carte';
import { Pool } from '../utils/pool';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';

interface Chute {
  arme: ArmeEquipee | null;
  x: number;
  z: number;
  /** s depuis le lancer (négatif : attend son tour). */
  t: number;
  tombee: boolean;
}

const groupe: Point = { x: 0, z: 0 };

export class Chutes {
  private readonly contexte: ContexteCombat;
  private readonly textures: BibliothequeTextures;
  private readonly camera: THREE.Camera;
  private readonly tampon = new Int32Array(128);
  private readonly chutes: Pool<Chute>;
  /** Une lame affichée par chute (un matériau chacune pour son fondu). */
  private readonly lames: THREE.Mesh<THREE.PlaneGeometry, THREE.MeshBasicMaterial>[];
  /** Éclats de l'impact : le côté du bloc du sol de l'arène. */
  private readonly eboulis: CheminTexture;

  constructor(contexte: ContexteCombat, textures: BibliothequeTextures, scene: THREE.Scene, camera: THREE.Camera) {
    const c = CONFIG.armes.chute;
    this.contexte = contexte;
    this.textures = textures;
    this.camera = camera;
    this.chutes = new Pool<Chute>(c.capacite, () => ({ arme: null, x: 0, z: 0, t: 0, tombee: false }));
    this.eboulis = BLOCS[blocDuSol(contexte.etat.arene)].faces.cotes;
    const geometrie = new THREE.PlaneGeometry(1, 1);
    // Texture posée dès le départ : le shader ne change pas à la première chute
    const premiere = IDS_ARMES.map((id) => ARMES[id].mecanique).find((m) => m.type === 'chute');
    const map = premiere?.type === 'chute' ? textures.obtenir(premiere.image) : null;
    this.lames = Array.from({ length: c.capacite }, () => {
      const lame = new THREE.Mesh(geometrie, new THREE.MeshBasicMaterial({ map, transparent: true, alphaTest: 0.1, side: THREE.DoubleSide, depthWrite: false }));
      lame.name = 'lame colossale';
      lame.visible = false;
      lame.renderOrder = 5;
      scene.add(lame);
      return lame;
    });
  }

  /** Fait tomber les lames de l'arme sur les plus gros groupes ; faux si personne n'est à portée. */
  frapper(arme: ArmeEquipee): boolean {
    const m = arme.definition.mecanique;
    if (m.type !== 'chute') return false;
    const { etat, ennemis } = this.contexte;
    const j = etat.joueur;
    const s = arme.stats;
    let lancee = false;
    for (let k = 0; k < s.nombre; k++) {
      if (!ennemis.plusGrosGroupe(j.x, j.z, s.portee, s.zone, groupe)) break;
      const c = this.chutes.acquerir();
      if (!c) break;
      c.arme = arme;
      c.x = groupe.x;
      c.z = groupe.z;
      c.t = -k * CONFIG.armes.chute.decalage;
      c.tombee = false;
      lancee = true;
    }
    return lancee;
  }

  maj(dt: number): void {
    const c = CONFIG.armes.chute;
    const { ennemis, particules, retours } = this.contexte;
    for (let i = this.chutes.nombre - 1; i >= 0; i--) {
      const ch = this.chutes.elements[i];
      const arme = ch.arme;
      ch.t += dt;
      if (!arme) {
        this.chutes.liberer(i);
        continue;
      }
      if (!ch.tombee && ch.t >= c.duree) {
        ch.tombee = true;
        const n = ennemis.chercher(ch.x, ch.z, arme.stats.zone, this.tampon);
        for (let k = 0; k < n; k++) ennemis.toucher(ennemis.obtenir(this.tampon[k]), arme, ch.x, ch.z);
        particules.emettre(ch.x, 0.3, ch.z, GERBES.impactLame);
        particules.emettre(ch.x, 0.3, ch.z, GERBES.eboulis, this.eboulis);
        retours.son('lame_colossale');
        retours.secousse(0.35);
      }
      if (ch.t >= c.duree + c.tenue + c.fondu) {
        ch.arme = null;
        this.chutes.liberer(i);
      }
    }
  }

  majVisuel(ombres: Ombres): void {
    const c = CONFIG.armes.chute;
    let n = 0;
    for (let i = 0; i < this.chutes.nombre; i++) {
      const ch = this.chutes.elements[i];
      const m = ch.arme?.definition.mecanique;
      if (!ch.arme || m?.type !== 'chute' || ch.t < 0) continue;
      const lame = this.lames[n++];
      const image = this.textures.obtenir(m.image);
      if (lame.material.map !== image) {
        lame.material.map = image;
        lame.material.needsUpdate = true;
      }
      // Chute accélérée, puis plantée un peu dans le sol
      const p = Math.min(1, ch.t / c.duree);
      const hauteur = ch.tombee ? -c.enfoncement * m.taille : c.hauteur * (1 - p * p);
      lame.visible = true;
      lame.scale.setScalar(m.taille);
      lame.position.set(ch.x, hauteur + m.taille * 0.35, ch.z);
      // Face à la caméra, pointe vers le bas (l'icône a la pointe en haut à droite)
      lame.quaternion.copy(this.camera.quaternion);
      lame.rotateZ(-Math.PI * 0.75);
      const fondu = ch.t - c.duree - c.tenue;
      lame.material.opacity = fondu > 0 ? Math.max(0, 1 - fondu / c.fondu) : 1;
      // L'ombre au sol annonce l'impact
      if (!ch.tombee) ombres.ajouter(ch.x, ch.z, ch.arme.stats.zone * (0.3 + 0.7 * p));
    }
    for (let i = n; i < this.lames.length; i++) this.lames[i].visible = false;
  }
}
