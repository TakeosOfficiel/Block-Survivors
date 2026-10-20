/**
 * Compilation d'une DefinitionModele en structure indexée, prête pour
 * l'animation et le rendu instancié (aucune recherche par nom à chaque image).
 */
import type { Axe, CubeModele, DefinitionModele } from '../donnees/types';
import { PIXELS_PAR_BLOC } from '../moteur/geometrie';

export interface PartieCompilee {
  readonly nom: string;
  readonly cube: CubeModele;
  /** Articulation en blocs. */
  readonly pivot: readonly [number, number, number];
}

export interface MembreMarcheCompile {
  readonly partie: number;
  readonly axe: number;        // 0 = x, 1 = y, 2 = z
  readonly amplitude: number;
  readonly dephasage: number;
  readonly frequence: number;
  readonly absolu: boolean;
}

export interface MembreReposCompile {
  readonly partie: number;
  readonly signe: number;
}

export interface MembreTournoiementCompile {
  readonly partie: number;
  readonly vitesse: number;
}

export interface MembreAttaqueCompile {
  readonly partie: number;
  readonly amplitude: number;
}

export interface ModeleCompile {
  readonly definition: DefinitionModele;
  readonly parties: readonly PartieCompilee[];
  /** Pose de base : 3 rotations (x, y, z) par partie. */
  readonly poseBase: Float32Array;
  readonly marche: readonly MembreMarcheCompile[];
  readonly repos: readonly MembreReposCompile[];
  readonly attaque: readonly MembreAttaqueCompile[];
  readonly tournoiement: readonly MembreTournoiementCompile[];
  /** Hauteur du sommet du modèle en pose de base (blocs). */
  readonly hauteur: number;
}

const INDEX_AXE: Record<Axe, number> = { x: 0, y: 1, z: 2 };

export function compilerModele(definition: DefinitionModele): ModeleCompile {
  const parties: PartieCompilee[] = Object.entries(definition.parties).map(([nom, cube]) => ({
    nom,
    cube,
    pivot: [cube.pivot[0] / PIXELS_PAR_BLOC, cube.pivot[1] / PIXELS_PAR_BLOC, cube.pivot[2] / PIXELS_PAR_BLOC],
  }));

  const indexPartie = (nom: string): number => {
    const index = parties.findIndex((p) => p.nom === nom);
    if (index < 0) throw new Error(`Partie « ${nom} » absente du modèle`);
    return index;
  };

  const poseBase = new Float32Array(parties.length * 3);
  parties.forEach((p, i) => {
    if (p.cube.rotation) poseBase.set(p.cube.rotation, i * 3);
  });

  return {
    definition,
    parties,
    poseBase,
    marche: (definition.marche?.membres ?? []).map((m) => ({
      partie: indexPartie(m.partie),
      axe: INDEX_AXE[m.axe],
      amplitude: m.amplitude,
      dephasage: m.dephasage,
      frequence: m.frequence ?? 1,
      absolu: m.absolu ?? false,
    })),
    repos: (definition.repos?.membres ?? []).map((m) => ({ partie: indexPartie(m.partie), signe: m.signe })),
    attaque: (definition.attaque?.membres ?? []).map((m) => ({ partie: indexPartie(m.partie), amplitude: m.amplitude })),
    tournoiement: (definition.tournoiement ?? []).map((m) => ({ partie: indexPartie(m.partie), vitesse: m.vitesse })),
    hauteur: Math.max(...parties.map((p) => p.cube.pivot[1] + p.cube.centre[1] + p.cube.taille[1] / 2)) / PIXELS_PAR_BLOC,
  };
}
