/**
 * Mécanique « foudre » : des éclairs frappent des ennemis au hasard autour du
 * joueur (dégâts dans une petite zone). L'éclair est une ligne brisée de
 * segments lumineux, rendue par un seul InstancedMesh.
 * Talents : Haute tension (un 2e ennemi proche), Conduction (rebonds sur un
 * ennemi ralenti), Tempête d'orage (éclairs de talent, `eclairTalent`).
 */
import * as THREE from 'three';
import { CONFIG } from '../config';
import { GERBES } from '../donnees/particules';
import { Pool } from '../utils/pool';
import type { Ennemi } from '../entites/ennemi';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';

interface Eclair {
  points: Float32Array;   // (segments + 1) points x, y, z
  vie: number;
  couleur: THREE.Color;
}

const AXE_Y = new THREE.Vector3(0, 1, 0);
const debut = new THREE.Vector3();
const fin = new THREE.Vector3();
const milieu = new THREE.Vector3();
const direction = new THREE.Vector3();
const echelle = new THREE.Vector3();
const orientation = new THREE.Quaternion();
const matrice = new THREE.Matrix4();

export class Foudre {
  private readonly contexte: ContexteCombat;
  private readonly tampon = new Int32Array(64);
  private readonly voisins = new Int32Array(64);
  /** id des ennemis déjà frappés par les arcs d'un même éclair. */
  private readonly frappes = new Int32Array(6);
  private readonly pool: Pool<Eclair>;
  private readonly maillage: THREE.InstancedMesh;

  constructor(contexte: ContexteCombat, scene: THREE.Scene) {
    this.contexte = contexte;
    const f = CONFIG.armes.foudre;
    this.pool = new Pool<Eclair>(f.capacite, () => ({
      points: new Float32Array((f.segments + 1) * 3),
      vie: 0,
      couleur: new THREE.Color(),
    }));
    this.maillage = new THREE.InstancedMesh(
      new THREE.BoxGeometry(1, 1, 1),
      new THREE.MeshBasicMaterial({ color: 0xffffff, fog: false }),  // lumineux, non éclairé
      f.capacite * f.segments,
    );
    this.maillage.name = 'foudre';
    this.maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.maillage.setColorAt(0, new THREE.Color(0xffffff));  // crée l'attribut de couleur
    this.maillage.frustumCulled = false;
    this.maillage.count = 0;
    scene.add(this.maillage);
  }

  /** Lance les éclairs de l'arme ; faux si aucun ennemi n'est à portée. */
  frapper(arme: ArmeEquipee): boolean {
    const m = arme.definition.mecanique;
    if (m.type !== 'foudre') return false;
    const { etat, ennemis, particules } = this.contexte;
    const j = etat.joueur;
    const s = arme.stats;
    let frappe = false;

    const talents = this.contexte.talents;
    const degats = s.degats * talents.facteurDeclenchement;
    const tension = talents.valeur('haute_tension', CONFIG.armes.foudre.hauteTension);
    const conduction = talents.valeur('conduction', CONFIG.armes.foudre.conduction.voisins);
    for (let k = 0; k < s.nombre; k++) {
      const cible = ennemis.auHasard(j.x, j.z, s.portee);
      if (!cible) break;
      frappe = true;
      // Conduction : la cible était ralentie avant le coup (le coup lui-même peut la ralentir)
      const ralentie = cible.ralentiTemps > 0 || cible.enracine > 0;
      const x = cible.x;
      const z = cible.z;
      const n = ennemis.chercher(x, z, s.zone, this.tampon);
      for (let i = 0; i < n; i++) ennemis.blesser(ennemis.obtenir(this.tampon[i]), degats, x, z, s.recul, arme.index);
      this.creerEclair(x, z, m.couleur);
      particules.emettre(x, 0.3, z, GERBES.impactFoudre);
      // Haute tension : un 2e ennemi proche, plus faiblement (réaction)
      if (tension > 0) this.arcs(arme.index, x, z, cible, 1, degats * tension, m.couleur);
      if (conduction > 0 && ralentie) this.arcs(arme.index, x, z, cible, conduction, degats * CONFIG.armes.foudre.conduction.degats, m.couleur);
    }
    return frappe;
  }

  /** Arcs vers les `nombre` ennemis les plus proches de (x, z), sauf `exclu` : réactions, sans rebond. */
  private arcs(arme: number, x: number, z: number, exclu: Ennemi, nombre: number, degats: number, couleur: number): void {
    const { ennemis, talents, particules } = this.contexte;
    const n = ennemis.chercher(x, z, CONFIG.armes.foudre.porteeArc, this.voisins);
    this.frappes[0] = exclu.id;
    for (let fait = 0; fait < nombre && fait < this.frappes.length - 1; fait++) {
      let meilleur: Ennemi | null = null;
      let meilleureD2 = Infinity;
      for (let k = 0; k < n; k++) {
        const e = ennemis.obtenir(this.voisins[k]);
        const d2 = (e.x - x) ** 2 + (e.z - z) ** 2;
        if (e.mort >= 0 || d2 >= meilleureD2) continue;
        let deja = false;
        for (let q = 0; q <= fait; q++) if (this.frappes[q] === e.id) deja = true;
        if (deja) continue;
        meilleur = e;
        meilleureD2 = d2;
      }
      if (!meilleur) return;
      this.frappes[fait + 1] = meilleur.id;
      talents.blesser(meilleur, degats, x, z, 0, arme);
      this.creerArc(x, z, meilleur.x, meilleur.z, couleur);
      particules.emettre(meilleur.x, 0.8, meilleur.z, GERBES.impactFoudre);
    }
  }

  /** Éclair d'un talent (Tempête d'orage) en (x, z) : réaction, critiques compris. */
  eclairTalent(arme: number, x: number, z: number, degats: number, zone: number, couleur: number): void {
    const { ennemis, talents, particules } = this.contexte;
    const n = ennemis.chercher(x, z, zone, this.tampon);
    for (let i = 0; i < n; i++) talents.blesser(ennemis.obtenir(this.tampon[i]), degats, x, z, 1, arme);
    this.creerEclair(x, z, couleur);
    particules.emettre(x, 0.3, z, GERBES.impactFoudre);
    this.contexte.retours.son('tonnerre');
  }

  maj(dt: number): void {
    for (let i = this.pool.nombre - 1; i >= 0; i--) {
      const e = this.pool.elements[i];
      e.vie -= dt;
      if (e.vie <= 0) this.pool.liberer(i);
    }
  }

  majVisuel(): void {
    const f = CONFIG.armes.foudre;
    let n = 0;
    for (let i = 0; i < this.pool.nombre; i++) {
      const e = this.pool.elements[i];
      for (let s = 0; s < f.segments; s++) {
        debut.fromArray(e.points, s * 3);
        fin.fromArray(e.points, s * 3 + 3);
        direction.subVectors(fin, debut);
        const longueur = direction.length();
        direction.divideScalar(longueur);
        milieu.addVectors(debut, fin).multiplyScalar(0.5);
        orientation.setFromUnitVectors(AXE_Y, direction);
        echelle.set(f.largeur, longueur + f.largeur, f.largeur);
        matrice.compose(milieu, orientation, echelle);
        this.maillage.setMatrixAt(n, matrice);
        this.maillage.setColorAt(n, e.couleur);
        n++;
      }
    }
    this.maillage.count = n;
    this.maillage.instanceMatrix.needsUpdate = true;
    if (this.maillage.instanceColor) this.maillage.instanceColor.needsUpdate = true;
  }

  /** Arc électrique presque horizontal d'un ennemi à un autre. */
  private creerArc(x1: number, z1: number, x2: number, z2: number, couleur: number): void {
    const e = this.pool.acquerir();
    if (!e) return;
    const f = CONFIG.armes.foudre;
    e.vie = f.duree;
    e.couleur.setHex(couleur);
    for (let s = 0; s <= f.segments; s++) {
      const t = s / f.segments;
      const ecart = s === 0 || s === f.segments ? 0 : 0.35;
      e.points[s * 3] = x1 + (x2 - x1) * t + (Math.random() - 0.5) * 2 * ecart;
      e.points[s * 3 + 1] = 1 + Math.sin(t * Math.PI) * 0.6 + (Math.random() - 0.5) * ecart;
      e.points[s * 3 + 2] = z1 + (z2 - z1) * t + (Math.random() - 0.5) * 2 * ecart;
    }
  }

  /** Ligne brisée du ciel jusqu'au sol en (x, z), plus déviée en hauteur. */
  private creerEclair(x: number, z: number, couleur: number): void {
    const e = this.pool.acquerir();
    if (!e) return;
    const f = CONFIG.armes.foudre;
    e.vie = f.duree;
    e.couleur.setHex(couleur);
    for (let s = 0; s <= f.segments; s++) {
      const t = s / f.segments;               // 0 = en haut, 1 = au sol
      const ecart = s === f.segments ? 0 : f.ecart * (1 - t * 0.7);
      e.points[s * 3] = x + (Math.random() - 0.5) * 2 * ecart;
      e.points[s * 3 + 1] = f.hauteur * (1 - t);
      e.points[s * 3 + 2] = z + (Math.random() - 0.5) * 2 * ecart;
    }
  }
}
