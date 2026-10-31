/**
 * Mécanique « fissure » (Onde de choc) : le joueur frappe le sol, et des
 * fissures filent vers le plus gros groupe d'ennemis en soulevant les blocs
 * du sol (texture du sol de l'arène), qui jaillissent puis retombent. Chaque
 * ennemi n'est touché qu'une fois par onde. Un mur arrête la fissure.
 * Talents : Répliques (une 2e onde plus faible juste après), Séisme (une onde
 * dans 6 directions).
 */
import * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { blocDuSol } from '../donnees/arenes';
import { BLOCS } from '../donnees/blocs';
import { GERBES } from '../donnees/particules';
import type { CheminTexture } from '../donnees/types';
import type { Point } from '../monde/carte';
import { Pool } from '../utils/pool';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';
import { alea } from '../utils/aleatoire';

interface Fissure {
  arme: ArmeEquipee | null;
  x: number;
  z: number;
  dirX: number;
  dirZ: number;
  /** Distance parcourue (blocs) et prochaine distance où soulever des blocs. */
  avance: number;
  prochain: number;
  /** s avant de partir (réplique), et part des dégâts de l'arme. */
  attente: number;
  facteur: number;
}

interface BlocSouleve {
  x: number;
  z: number;
  age: number;
  rotation: number;
  inclinaison: number;
}

const groupe: Point = { x: 0, z: 0 };
const matrice = new THREE.Matrix4();
const position = new THREE.Vector3();
const orientation = new THREE.Quaternion();
const euler = new THREE.Euler();
const echelle = new THREE.Vector3();

export class Fissures {
  private readonly contexte: ContexteCombat;
  private readonly tampon = new Int32Array(64);
  private readonly fissures: Pool<Fissure>;
  private readonly blocs: Pool<BlocSouleve>;
  private readonly maillage: THREE.InstancedMesh;
  /** Texture des blocs soulevés : le côté du bloc du sol de l'arène. */
  private readonly texture: CheminTexture;

  constructor(contexte: ContexteCombat, textures: BibliothequeTextures, scene: THREE.Scene) {
    const c = CONFIG.armes.fissure;
    this.contexte = contexte;
    this.fissures = new Pool<Fissure>(c.capacite, () => ({ arme: null, x: 0, z: 0, dirX: 0, dirZ: 0, avance: 0, prochain: 0, attente: 0, facteur: 1 }));
    this.blocs = new Pool<BlocSouleve>(c.blocs, () => ({ x: 0, z: 0, age: 0, rotation: 0, inclinaison: 0 }));
    const sol = BLOCS[blocDuSol(contexte.etat.arene)];
    this.texture = sol.faces.cotes;
    const map = textures.obtenir(this.texture);
    this.maillage = new THREE.InstancedMesh(
      new THREE.BoxGeometry(1, 1, 1),
      new THREE.MeshLambertMaterial(map ? { map } : { color: sol.couleurSecours }),
      c.blocs,
    );
    this.maillage.name = 'fissures';
    this.maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.maillage.frustumCulled = false;
    this.maillage.count = 0;
    scene.add(this.maillage);
  }

  /** Lance les fissures de l'arme vers le plus gros groupe ; faux si personne n'est à portée. */
  frapper(arme: ArmeEquipee): boolean {
    const m = arme.definition.mecanique;
    if (m.type !== 'fissure') return false;
    const { etat, ennemis } = this.contexte;
    const j = etat.joueur;
    const s = arme.stats;
    if (!ennemis.plusGrosGroupe(j.x, j.z, s.portee, CONFIG.armes.fissure.rayonGroupe, groupe)) return false;
    const angle = Math.atan2(groupe.x - j.x, groupe.z - j.z);
    const talents = this.contexte.talents;
    const c = CONFIG.armes.fissure;
    // Séisme : 6 fissures en étoile au lieu de l'éventail
    const circulaire = talents.ondeCirculaire;
    const nombre = circulaire ? c.directionsSeisme : s.nombre;
    const replique = talents.valeur('repliques', c.repliques);
    for (let k = 0; k < nombre; k++) {
      const a = circulaire ? angle + (k / nombre) * Math.PI * 2 : angle + (k - (nombre - 1) / 2) * m.ecart;
      this.lancer(arme, j.x, j.z, a, 0, 1);
      if (replique > 0) this.lancer(arme, j.x, j.z, a, c.delaiReplique, replique);
    }
    this.contexte.declencherGesteJoueur(angle);
    this.contexte.retours.secousse(0.15);
    return true;
  }

  private lancer(arme: ArmeEquipee, x: number, z: number, angle: number, attente: number, facteur: number): void {
    const f = this.fissures.acquerir();
    if (!f) return;
    f.arme = arme;
    f.x = x;
    f.z = z;
    f.dirX = Math.sin(angle);
    f.dirZ = Math.cos(angle);
    f.avance = 0;
    f.prochain = CONFIG.armes.fissure.depart;
    f.attente = attente;
    f.facteur = facteur;
  }

  maj(dt: number): void {
    const c = CONFIG.armes.fissure;
    const { carte } = this.contexte;
    for (let i = this.fissures.nombre - 1; i >= 0; i--) {
      const f = this.fissures.elements[i];
      const arme = f.arme;
      const m = arme?.definition.mecanique;
      if (!arme || m?.type !== 'fissure') {
        this.fissures.liberer(i);
        continue;
      }
      if (f.attente > 0) {
        f.attente -= dt;
        continue;
      }
      f.avance += m.vitesse * dt;
      let finie = false;
      while (f.prochain <= f.avance) {
        if (f.prochain > arme.stats.portee) {
          finie = true;
          break;
        }
        const x = f.x + f.dirX * f.prochain;
        const z = f.z + f.dirZ * f.prochain;
        if (carte.bloqueMarche(x, z)) {
          finie = true;
          break;
        }
        this.soulever(arme, x, z, f.dirX, f.dirZ, f.facteur);
        f.prochain += c.espacement;
      }
      if (finie) {
        f.arme = null;
        this.fissures.liberer(i);
      }
    }
    for (let i = this.blocs.nombre - 1; i >= 0; i--) {
      const b = this.blocs.elements[i];
      b.age += dt;
      if (b.age >= c.dureeBloc) this.blocs.liberer(i);
    }
  }

  /** Soulève les blocs d'un segment de fissure et touche les ennemis dessus (une fois par onde). */
  private soulever(arme: ArmeEquipee, x: number, z: number, dirX: number, dirZ: number, facteur: number): void {
    const c = CONFIG.armes.fissure;
    const { ennemis, particules, etat } = this.contexte;
    const s = arme.stats;
    // Une rangée de blocs en travers de la fissure, assez large pour sa zone
    const largeur = Math.max(1, Math.round(s.zone / c.tailleBloc));
    for (let k = 0; k < largeur; k++) {
      const b = this.blocs.acquerir();
      if (!b) break;
      const decalage = (k - (largeur - 1) / 2) * c.tailleBloc;
      b.x = x + dirZ * decalage + (alea() - 0.5) * 0.25;
      b.z = z - dirX * decalage + (alea() - 0.5) * 0.25;
      b.age = 0;
      b.rotation = alea() * Math.PI;
      b.inclinaison = (alea() - 0.5) * 0.5;
    }
    particules.emettre(x, 0.4, z, GERBES.eboulis, this.texture);

    const n = ennemis.chercher(x, z, s.zone / 2, this.tampon);
    for (let k = 0; k < n; k++) {
      const e = ennemis.obtenir(this.tampon[k]);
      if (e.immunites[arme.index] > etat.tempsJeu) continue;
      e.immunites[arme.index] = etat.tempsJeu + c.immunite;
      ennemis.toucher(e, arme, x, z, arme.stats.recul, facteur);
    }
  }

  majVisuel(): void {
    const c = CONFIG.armes.fissure;
    let n = 0;
    for (let i = 0; i < this.blocs.nombre; i++) {
      const b = this.blocs.elements[i];
      // Jaillit (avec un léger dépassement), reste, puis s'enfonce
      const t = b.age;
      let y: number;
      if (t < c.montee) y = -1 + 1.15 * Math.sin((t / c.montee) * Math.PI * 0.5);
      else if (t < c.dureeBloc - c.descente) y = 0.15 - 0.1 * Math.min(1, (t - c.montee) / 0.08);
      else y = 0.05 - 1.05 * ((t - (c.dureeBloc - c.descente)) / c.descente);
      const cote = c.tailleBloc;
      position.set(b.x, y * cote, b.z);
      orientation.setFromEuler(euler.set(b.inclinaison, b.rotation, b.inclinaison * 0.5));
      matrice.compose(position, orientation, echelle.set(cote, cote, cote));
      this.maillage.setMatrixAt(n++, matrice);
    }
    this.maillage.count = n;
    this.maillage.instanceMatrix.needsUpdate = true;
  }
}
