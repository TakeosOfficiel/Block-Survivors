/**
 * Mécanique « invocation » : alliés (golems de fer, loups, vex) qui vont
 * frapper l'ennemi le plus proche pendant un temps limité, puis disparaissent.
 * Ils ont des PV : les ennemis au corps à corps qu'ils attirent les frappent,
 * et ils peuvent mourir avant la fin de leur temps.
 */
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { GERBES } from '../donnees/particules';
import { ALLIES, IDS_ALLIES } from '../donnees/allies';
import { MODELES } from '../donnees/modeles';
import type { DefinitionAllie } from '../donnees/types';
import type { BarresVie } from '../effets/barresVie';
import type { Ombres } from '../effets/ombres';
import type { SourceEntites } from '../entites/types';
import { avancerAnimation, calculerRotations, creerEtatAnimation, declencherAttaque, reinitialiserAnimation, type EtatAnimation } from '../modeles/animation';
import { compilerModele, type ModeleCompile } from '../modeles/compilation';
import { creerPoseModele, ModeleInstancie, type PoseModele } from '../modeles/rendu';
import { ecartAngle, facteurLissage, longueur } from '../utils/maths';
import { Pool } from '../utils/pool';
import type { Point } from '../monde/carte';
import type { ArmeEquipee } from './armeEquipee';
import type { ContexteCombat } from './contexte';

export interface Allie {
  /** Identifiant unique de l'invocation (les index du pool changent). */
  id: number;
  indexType: number;        // index dans IDS_ALLIES
  definition: DefinitionAllie;
  arme: ArmeEquipee | null; // arme qui l'a invoqué (ses statistiques courantes)
  x: number;
  z: number;
  angle: number;
  vie: number;              // s restantes
  pv: number;
  blessure: number;         // s de surcouche rouge restantes
  minuterie: number;        // s avant le prochain coup
  vitesseSol: number;
  animation: EtatAnimation;
}

const corrige: Point = { x: 0, z: 0 };

interface RenduType {
  readonly modele: ModeleCompile;
  readonly instances: ModeleInstancie;
  readonly pose: PoseModele;
}

export class Invocations implements SourceEntites {
  private readonly contexte: ContexteCombat;
  private readonly pool = new Pool<Allie>(CONFIG.armes.golems.capacite, () => ({
    id: 0, indexType: 0, definition: ALLIES[IDS_ALLIES[0]], arme: null,
    x: 0, z: 0, angle: 0, vie: 0, pv: 1, blessure: 0, minuterie: 0, vitesseSol: 0, animation: creerEtatAnimation(),
  }));
  private readonly rendus: RenduType[];
  private readonly barres: BarresVie;
  private prochainId = 1;

  constructor(contexte: ContexteCombat, scene: THREE.Scene, textures: BibliothequeTextures, barres: BarresVie) {
    this.contexte = contexte;
    this.barres = barres;
    this.rendus = IDS_ALLIES.map((id) => {
      const def = ALLIES[id];
      const modele = compilerModele(MODELES[def.modele]);
      return {
        modele,
        instances: new ModeleInstancie(modele, def, textures, CONFIG.armes.golems.capacite, scene),
        pose: creerPoseModele(modele),
      };
    });
  }

  get actifs(): number {
    return this.pool.nombre;
  }

  /** Invoque un allié près du joueur si l'arme n'a pas atteint son nombre maximal. */
  invoquer(arme: ArmeEquipee): boolean {
    const m = arme.definition.mecanique;
    if (m.type !== 'invocation') return false;
    let presents = 0;
    for (let i = 0; i < this.pool.nombre; i++) if (this.pool.elements[i].arme === arme) presents++;
    if (presents >= arme.stats.nombre) return false;

    const a = this.pool.acquerir();
    if (!a) return false;
    const j = this.contexte.etat.joueur;
    const angle = Math.random() * Math.PI * 2;
    a.indexType = IDS_ALLIES.indexOf(m.allie);
    a.definition = ALLIES[m.allie];
    a.arme = arme;
    a.x = j.x + Math.sin(angle) * 1.5;
    a.z = j.z + Math.cos(angle) * 1.5;
    a.angle = j.angle;
    a.id = this.prochainId++;
    a.vie = arme.stats.duree;
    a.pv = a.definition.pv;
    a.blessure = 0;
    a.minuterie = 0;
    a.vitesseSol = 0;
    reinitialiserAnimation(a.animation);
    this.contexte.particules.emettre(a.x, 1, a.z, GERBES.invocation);
    return true;
  }

  maj(dt: number): void {
    const { etat, carte, ennemis, particules } = this.contexte;
    const j = etat.joueur;

    for (let i = this.pool.nombre - 1; i >= 0; i--) {
      const a = this.pool.elements[i];
      const def = a.definition;
      const arme = a.arme;
      a.vie -= dt;
      if (!arme || a.vie <= 0) {
        particules.emettre(a.x, 1, a.z, GERBES.invocation);
        a.arme = null;
        this.pool.liberer(i);
        continue;
      }
      const s = arme.stats;
      if (a.minuterie > 0) a.minuterie -= dt;
      if (a.blessure > 0) a.blessure -= dt;

      // Cible : l'ennemi le plus proche ; sinon, rester près du joueur
      const cible = ennemis.plusProche(a.x, a.z, def.detection);
      let versX = (cible ? cible.x : j.x) - a.x;
      let versZ = (cible ? cible.z : j.z) - a.z;
      const distance = longueur(versX, versZ) || 1e-6;
      versX /= distance;
      versZ /= distance;
      const arret = cible ? def.rayon + cible.definition.rayon + s.zone : CONFIG.armes.golems.distanceSuivi;

      a.vitesseSol = 0;
      if (distance > arret) {
        const nx = a.x + versX * def.vitesse * dt;
        const nz = a.z + versZ * def.vitesse * dt;
        if (def.vol !== undefined) {
          // Volant : traverse le décor
          a.x = nx;
          a.z = nz;
        } else {
          carte.resoudreCollision(nx, nz, def.rayon, corrige);
          a.x = corrige.x;
          a.z = corrige.z;
        }
        a.vitesseSol = def.vitesse;
      } else if (cible && a.minuterie <= 0) {
        ennemis.blesser(cible, s.degats, a.x, a.z, s.recul, arme.index);
        this.contexte.retours.son(def.sonCoup);
        declencherAttaque(a.animation, this.rendus[a.indexType].modele);
        a.minuterie = def.cadenceCoups;
      }
      a.angle += ecartAngle(a.angle, Math.atan2(versX, versZ)) * facteurLissage(10, dt);
      avancerAnimation(a.animation, this.rendus[a.indexType].modele, dt, a.vitesseSol);
    }
  }

  /**
   * Allié qui détourne un ennemi en (x, z) : le plus proche à portée de sa
   * provocation, et plus proche que le joueur (à `distanceJoueur`). Null sinon.
   */
  provocateur(x: number, z: number, distanceJoueur: number): Allie | null {
    let meilleur: Allie | null = null;
    let meilleure = distanceJoueur;
    for (let i = 0; i < this.pool.nombre; i++) {
      const a = this.pool.elements[i];
      if (a.pv <= 0 || !a.arme) continue;
      const d = longueur(a.x - x, a.z - z);
      if (d < a.definition.provocation && d < meilleure) {
        meilleure = d;
        meilleur = a;
      }
    }
    return meilleur;
  }

  /** L'allié est-il toujours la même invocation, et en vie ? */
  vivant(a: Allie, id: number): boolean {
    return a.id === id && a.arme !== null && a.pv > 0;
  }

  blesser(a: Allie, degats: number): void {
    if (a.pv <= 0 || !a.arme) return;
    a.pv -= Math.max(1, degats - (a.definition.armure ?? 0));
    a.blessure = CONFIG.surcouche.blessure.duree;
    const retours = this.contexte.retours;
    if (a.pv > 0) {
      retours.son(a.definition.sonBlessure);
      return;
    }
    // Mort : disparaît au prochain pas (poof d'invocation)
    retours.son(a.definition.sonMort);
    a.vie = 0;
  }

  /** Explosion (creeper) : blesse les alliés dans le rayon, moins au bord. */
  blesserZone(x: number, z: number, rayon: number, degats: number): void {
    for (let i = 0; i < this.pool.nombre; i++) {
      const a = this.pool.elements[i];
      const d = longueur(a.x - x, a.z - z);
      if (d < rayon) this.blesser(a, Math.ceil(degats * (1 - d / rayon)));
    }
  }

  majVisuel(ombres: Ombres): void {
    for (let t = 0; t < this.rendus.length; t++) this.rendus[t].instances.commencer();
    this.barres.commencer();
    const blessure = CONFIG.surcouche.blessure;
    for (let i = 0; i < this.pool.nombre; i++) {
      const a = this.pool.elements[i];
      const rendu = this.rendus[a.indexType];
      const pose = rendu.pose;
      pose.x = a.x;
      pose.z = a.z;
      pose.angle = a.angle;
      // Les volants flottent en ondulant doucement
      pose.y = a.definition.vol !== undefined ? a.definition.vol + Math.sin(a.vie * 4) * 0.12 : 0;
      calculerRotations(rendu.modele, a.animation, pose.rotations);
      // Rouge quand il est touché, comme les mobs
      pose.surcouche[0] = blessure.couleur[0];
      pose.surcouche[1] = blessure.couleur[1];
      pose.surcouche[2] = blessure.couleur[2];
      pose.surcouche[3] = a.blessure > 0 ? blessure.force : 0;
      rendu.instances.ajouter(pose);
      ombres.ajouter(a.x, a.z, a.definition.rayonOmbre);
      // Barre de vie dès la première blessure
      if (a.pv < a.definition.pv) this.barres.ajouter(a.x, pose.y + rendu.modele.hauteur + 0.35, a.z, a.pv / a.definition.pv);
    }
    for (let t = 0; t < this.rendus.length; t++) this.rendus[t].instances.terminer();
    this.barres.terminer();
  }
}
