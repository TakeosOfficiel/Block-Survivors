/**
 * Rendu instancié d'un modèle cubique : un InstancedMesh par partie du modèle,
 * partagé par toutes les entités qui l'utilisent. Chaque image, les entités
 * ajoutent leur pose ; aucune allocation.
 */
import * as THREE from 'three';
import type { ApparenceEntite } from '../donnees/types';
import { creerGeometrieCube } from '../moteur/geometrie';
import { activerSurcouche, creerAttributSurcouche, NOM_ATTRIBUT_SURCOUCHE } from '../moteur/surcouche';
import type { ModeleCompile } from './compilation';

/** Pose d'une instance, remplie par l'appelant (objet réutilisé, pas d'allocation). */
export interface PoseModele {
  x: number;
  y: number;
  z: number;
  angle: number;        // lacet (rad), 0 = face vers +Z
  inclinaison: number;  // chute sur le côté (rad)
  echelleXZ: number;
  echelleY: number;
  /** Rotations (x, y, z) de chaque partie, calculées par calculerRotations. */
  rotations: Float32Array;
  /** Surcouche de couleur (r, g, b, force) ; force 0 = aucune. */
  surcouche: [number, number, number, number];
}

export function creerPoseModele(modele: ModeleCompile): PoseModele {
  return {
    x: 0, y: 0, z: 0,
    angle: 0,
    inclinaison: 0,
    echelleXZ: 1,
    echelleY: 1,
    rotations: new Float32Array(modele.parties.length * 3),
    surcouche: [0, 0, 0, 0],
  };
}

// Temporaires réutilisés
const AXE_Y = new THREE.Vector3(0, 1, 0);
const AXE_Z = new THREE.Vector3(0, 0, 1);
const position = new THREE.Vector3();
const echelle = new THREE.Vector3();
const lacet = new THREE.Quaternion();
const chute = new THREE.Quaternion();
const orientation = new THREE.Quaternion();
const euler = new THREE.Euler(0, 0, 0, 'ZYX');
const matriceRacine = new THREE.Matrix4();
const matricePartie = new THREE.Matrix4();
const matrice = new THREE.Matrix4();

export class ModeleInstancie {
  private readonly modele: ModeleCompile;
  private readonly maillages: THREE.InstancedMesh[];
  private readonly surcouche: THREE.InstancedBufferAttribute;
  private readonly capacite: number;
  private compte = 0;

  constructor(
    modele: ModeleCompile,
    apparence: ApparenceEntite,
    skin: THREE.Texture | null,
    capacite: number,
    scene: THREE.Scene,
  ) {
    this.modele = modele;
    this.capacite = capacite;
    this.surcouche = creerAttributSurcouche(capacite);

    const materiauSkin = skin ? creerMateriau({ map: skin }) : null;
    this.maillages = modele.parties.map((partie) => {
      const geometrie = creerGeometrieCube(partie.cube, modele.definition.tailleTexture);
      geometrie.setAttribute(NOM_ATTRIBUT_SURCOUCHE, this.surcouche);
      const couleur = apparence.couleursSecours[partie.nom] ?? apparence.couleurSecoursParDefaut ?? 0x808080;
      const maillage = new THREE.InstancedMesh(geometrie, materiauSkin ?? creerMateriau({ color: couleur }), capacite);
      maillage.name = `${apparence.modele}:${partie.nom}`;
      maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
      maillage.frustumCulled = false;  // les instances couvrent toute l'arène
      maillage.count = 0;
      scene.add(maillage);
      return maillage;
    });
  }

  /** À appeler avant d'ajouter les poses de l'image. */
  commencer(): void {
    this.compte = 0;
  }

  ajouter(pose: PoseModele): void {
    if (this.compte >= this.capacite) return;
    const i = this.compte++;

    position.set(pose.x, pose.y, pose.z);
    lacet.setFromAxisAngle(AXE_Y, pose.angle);
    chute.setFromAxisAngle(AXE_Z, pose.inclinaison);
    orientation.multiplyQuaternions(lacet, chute);
    echelle.set(pose.echelleXZ, pose.echelleY, pose.echelleXZ);
    matriceRacine.compose(position, orientation, echelle);

    const parties = this.modele.parties;
    for (let p = 0; p < parties.length; p++) {
      const pivot = parties[p].pivot;
      euler.set(pose.rotations[p * 3], pose.rotations[p * 3 + 1], pose.rotations[p * 3 + 2]);
      matricePartie.makeRotationFromEuler(euler).setPosition(pivot[0], pivot[1], pivot[2]);
      matrice.multiplyMatrices(matriceRacine, matricePartie);
      this.maillages[p].setMatrixAt(i, matrice);
    }

    const s = pose.surcouche;
    this.surcouche.setXYZW(i, s[0], s[1], s[2], s[3]);
  }

  /** Publie les instances ajoutées depuis commencer(). */
  terminer(): void {
    for (let p = 0; p < this.maillages.length; p++) {
      const maillage = this.maillages[p];
      maillage.count = this.compte;
      maillage.instanceMatrix.needsUpdate = true;
    }
    this.surcouche.needsUpdate = true;
  }
}

function creerMateriau(parametres: THREE.MeshLambertMaterialParameters): THREE.MeshLambertMaterial {
  const materiau = new THREE.MeshLambertMaterial(parametres);
  activerSurcouche(materiau);
  return materiau;
}
