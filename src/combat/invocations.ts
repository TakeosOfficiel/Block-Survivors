/**
 * Mécanique « invocation » : alliés (golems de fer…) qui vont frapper
 * l'ennemi le plus proche pendant un temps limité, puis disparaissent.
 */
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { ALLIES, IDS_ALLIES } from '../donnees/allies';
import { MODELES } from '../donnees/modeles';
import type { DefinitionAllie } from '../donnees/types';
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

interface Allie {
  indexType: number;        // index dans IDS_ALLIES
  definition: DefinitionAllie;
  arme: ArmeEquipee | null; // arme qui l'a invoqué (ses statistiques courantes)
  x: number;
  z: number;
  angle: number;
  vie: number;
  minuterie: number;        // s avant le prochain coup
  vitesseSol: number;
  animation: EtatAnimation;
}

const corrige: Point = { x: 0, z: 0 };

interface RenduType {
  readonly modele: ModeleCompile;
  readonly instances: ModeleInstancie;
  readonly pose: PoseModele;
  readonly couleurs: readonly number[];
}

export class Invocations implements SourceEntites {
  private readonly contexte: ContexteCombat;
  private readonly pool = new Pool<Allie>(CONFIG.armes.golems.capacite, () => ({
    indexType: 0, definition: ALLIES[IDS_ALLIES[0]], arme: null,
    x: 0, z: 0, angle: 0, vie: 0, minuterie: 0, vitesseSol: 0, animation: creerEtatAnimation(),
  }));
  private readonly rendus: RenduType[];

  constructor(contexte: ContexteCombat, scene: THREE.Scene, textures: BibliothequeTextures) {
    this.contexte = contexte;
    this.rendus = IDS_ALLIES.map((id) => {
      const def = ALLIES[id];
      const modele = compilerModele(MODELES[def.modele]);
      return {
        modele,
        instances: new ModeleInstancie(modele, def, textures, CONFIG.armes.golems.capacite, scene),
        pose: creerPoseModele(modele),
        couleurs: [0xffffff, def.couleurParticules],
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
    a.vie = arme.stats.duree;
    a.minuterie = 0;
    a.vitesseSol = 0;
    reinitialiserAnimation(a.animation);
    this.contexte.particules.emettre(a.x, 1, a.z, CONFIG.particules.invocation, this.rendus[a.indexType].couleurs);
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
        particules.emettre(a.x, 1, a.z, CONFIG.particules.invocation, this.rendus[a.indexType].couleurs);
        a.arme = null;
        this.pool.liberer(i);
        continue;
      }
      const s = arme.stats;
      if (a.minuterie > 0) a.minuterie -= dt;

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
        carte.resoudreCollision(a.x + versX * def.vitesse * dt, a.z + versZ * def.vitesse * dt, def.rayon, corrige);
        a.x = corrige.x;
        a.z = corrige.z;
        a.vitesseSol = def.vitesse;
      } else if (cible && a.minuterie <= 0) {
        ennemis.blesser(cible, s.degats, a.x, a.z, s.recul, arme.index);
        this.contexte.retours.son('golem_coup');
        declencherAttaque(a.animation, this.rendus[a.indexType].modele);
        a.minuterie = def.cadenceCoups;
      }
      a.angle += ecartAngle(a.angle, Math.atan2(versX, versZ)) * facteurLissage(10, dt);
      avancerAnimation(a.animation, this.rendus[a.indexType].modele, dt, a.vitesseSol);
    }
  }

  majVisuel(ombres: Ombres): void {
    for (let t = 0; t < this.rendus.length; t++) this.rendus[t].instances.commencer();
    for (let i = 0; i < this.pool.nombre; i++) {
      const a = this.pool.elements[i];
      const rendu = this.rendus[a.indexType];
      const pose = rendu.pose;
      pose.x = a.x;
      pose.z = a.z;
      pose.angle = a.angle;
      calculerRotations(rendu.modele, a.animation, pose.rotations);
      rendu.instances.ajouter(pose);
      ombres.ajouter(a.x, a.z, a.definition.rayonOmbre);
    }
    for (let t = 0; t < this.rendus.length; t++) this.rendus[t].instances.terminer();
  }
}
