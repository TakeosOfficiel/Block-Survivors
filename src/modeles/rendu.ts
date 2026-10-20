/**
 * Rendu instancié d'un modèle cubique : un InstancedMesh par partie du modèle,
 * partagé par toutes les entités qui l'utilisent. Chaque image, les entités
 * ajoutent leur pose ; aucune allocation.
 */
import * as THREE from 'three';
import { lirePixels } from '../assets/pixels';
import type { BibliothequeTextures } from '../assets/textures';
import type { ApparenceEntite, MainModele } from '../donnees/types';
import { creerGeometrieCube, creerGeometrieObjet, PIXELS_PAR_BLOC } from '../moteur/geometrie';
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
const matriceMain = new THREE.Matrix4();

/** Objet tenu : un maillage instancié de plus, accroché à une partie du modèle. */
interface ObjetTenu {
  readonly maillage: THREE.InstancedMesh;
  readonly partie: number;
  /** Position et orientation de l'objet dans le repère de la partie. */
  readonly local: THREE.Matrix4;
}

export class ModeleInstancie {
  private readonly modele: ModeleCompile;
  private readonly maillages: THREE.InstancedMesh[];
  private readonly surcouche: THREE.InstancedBufferAttribute;
  private readonly objet: ObjetTenu | null;
  private readonly capacite: number;
  private compte = 0;

  constructor(
    modele: ModeleCompile,
    apparence: ApparenceEntite,
    textures: BibliothequeTextures,
    capacite: number,
    scene: THREE.Scene,
  ) {
    const skin = textures.obtenir(apparence.skin);
    this.modele = modele;
    this.capacite = capacite;
    this.surcouche = creerAttributSurcouche(capacite);

    // Découpe comme en jeu (rendu « entity cutout no cull ») : les pixels
    // transparents (côtes du squelette, yeux de l'araignée) sont percés et
    // l'intérieur reste visible à travers, au lieu d'apparaître en noir
    const materiauSkin = skin ? creerMateriau({ map: skin, alphaTest: 0.5, side: THREE.DoubleSide }) : null;
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

    const main = modele.definition.main;
    const texture = apparence.objetEnMain ? textures.obtenir(apparence.objetEnMain) : null;
    this.objet = main && texture ? this.creerObjet(main, texture, capacite, scene) : null;
  }

  private creerObjet(main: MainModele, texture: THREE.Texture, capacite: number, scene: THREE.Scene): ObjetTenu | null {
    const image = texture.image as HTMLImageElement;
    let pixels: ImageData;
    try {
      pixels = lirePixels(image);
    } catch {
      return null;  // image illisible (sans CORS) : pas d'objet plutôt qu'une erreur
    }
    const geometrie = creerGeometrieObjet(pixels.data, image.width, image.height, main.taille / image.width / PIXELS_PAR_BLOC);
    geometrie.setAttribute(NOM_ATTRIBUT_SURCOUCHE, this.surcouche);
    const maillage = new THREE.InstancedMesh(
      geometrie,
      creerMateriau({ map: texture, alphaTest: 0.5, side: THREE.DoubleSide }),
      capacite,
    );
    maillage.name = 'objet en main';
    maillage.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    maillage.frustumCulled = false;
    maillage.count = 0;
    scene.add(maillage);

    const [x, y, z] = main.position;
    const local = new THREE.Matrix4()
      .makeRotationFromEuler(new THREE.Euler(main.rotation[0], main.rotation[1], main.rotation[2], 'XYZ'))
      .setPosition(x / PIXELS_PAR_BLOC, y / PIXELS_PAR_BLOC, z / PIXELS_PAR_BLOC);
    const partie = this.modele.parties.findIndex((p) => p.nom === main.partie);
    if (partie < 0) throw new Error(`Partie « ${main.partie} » absente du modèle`);
    return { maillage, partie, local };
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
      if (this.objet?.partie === p) matriceMain.copy(matrice);
    }
    if (this.objet) this.objet.maillage.setMatrixAt(i, matrice.multiplyMatrices(matriceMain, this.objet.local));

    const s = pose.surcouche;
    this.surcouche.setXYZW(i, s[0], s[1], s[2], s[3]);
  }

  /** Publie les instances ajoutées depuis commencer(). */
  terminer(): void {
    for (let p = 0; p < this.maillages.length; p++) {
      const maillage = this.maillages[p];
      maillage.count = this.compte;
      maillage.visible = this.compte > 0;  // pas d'appel de dessin pour un type absent
      maillage.instanceMatrix.needsUpdate = true;
    }
    if (this.objet) {
      this.objet.maillage.count = this.compte;
      this.objet.maillage.visible = this.compte > 0;
      this.objet.maillage.instanceMatrix.needsUpdate = true;
    }
    this.surcouche.needsUpdate = true;
  }
}

function creerMateriau(parametres: THREE.MeshLambertMaterialParameters): THREE.MeshLambertMaterial {
  const materiau = new THREE.MeshLambertMaterial(parametres);
  activerSurcouche(materiau);
  return materiau;
}
