/**
 * Animation d'un ModeleCubique selon les données `marche` et `repos` de sa
 * définition (balancement des membres façon Minecraft).
 */
import type * as THREE from 'three';
import type { DefinitionModele } from '../donnees/types';
import { facteurLissage } from '../utils/maths';
import type { ModeleCubique } from './modeleCubique';

interface MembreMarcheResolu {
  readonly articulation: THREE.Object3D;
  readonly amplitude: number;
  readonly dephasage: number;
}

interface MembreReposResolu {
  readonly articulation: THREE.Object3D;
  readonly signe: number;
}

const TOUR = Math.PI * 2;

export class AnimateurModele {
  private readonly definition: DefinitionModele;
  private readonly articulationsAnimees: THREE.Object3D[];
  private readonly membresMarche: MembreMarcheResolu[];
  private readonly membresRepos: MembreReposResolu[];

  private phase = 0;      // cycle de marche (rad)
  private intensite = 0;  // 0 = immobile, 1 = pleine allure
  private temps = 0;      // horloge du balancement au repos (s)

  constructor(modele: ModeleCubique, definition: DefinitionModele) {
    this.definition = definition;
    this.membresMarche = (definition.marche?.membres ?? []).flatMap((m) => {
      const articulation = modele.articulations.get(m.partie);
      return articulation ? [{ articulation, amplitude: m.amplitude, dephasage: m.dephasage }] : [];
    });
    this.membresRepos = (definition.repos?.membres ?? []).flatMap((m) => {
      const articulation = modele.articulations.get(m.partie);
      return articulation ? [{ articulation, signe: m.signe }] : [];
    });
    this.articulationsAnimees = [
      ...new Set([...this.membresMarche, ...this.membresRepos].map((m) => m.articulation)),
    ];
  }

  /** dt en temps de jeu (s), vitesseSol en blocs/s. */
  maj(dt: number, vitesseSol: number): void {
    this.temps += dt;

    for (let i = 0; i < this.articulationsAnimees.length; i++) {
      this.articulationsAnimees[i].rotation.set(0, 0, 0);
    }

    const marche = this.definition.marche;
    if (marche) {
      this.phase = (this.phase + vitesseSol * dt * marche.frequence) % TOUR;
      const cible = Math.min(vitesseSol / marche.vitesseReference, 1);
      this.intensite += (cible - this.intensite) * facteurLissage(marche.lissage, dt);
      for (let i = 0; i < this.membresMarche.length; i++) {
        const m = this.membresMarche[i];
        m.articulation.rotation.x += Math.cos(this.phase + m.dephasage) * m.amplitude * this.intensite;
      }
    }

    const repos = this.definition.repos;
    if (repos) {
      const ecart = Math.cos(this.temps * repos.frequenceZ) * repos.amplitude + repos.amplitude;
      const avant = Math.sin(this.temps * repos.frequenceX) * repos.amplitude;
      for (let i = 0; i < this.membresRepos.length; i++) {
        const m = this.membresRepos[i];
        m.articulation.rotation.z -= m.signe * ecart;  // écarte le bras du corps
        m.articulation.rotation.x += m.signe * avant;
      }
    }
  }
}
