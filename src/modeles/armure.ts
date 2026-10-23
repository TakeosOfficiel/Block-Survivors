/**
 * Armure de la Collection portée par le héros (en partie et dans le menu) : un
 * modèle par pièce (donnees/modeles.ts, `armure_*`) et par texture du monde,
 * posé exactement comme le joueur à chaque image — ses parties reprennent les
 * rotations des parties du joueur du même nom (table calculée une fois).
 */
import type * as THREE from 'three';
import type { BibliothequeTextures } from '../assets/textures';
import { PIECES_ARMURE } from '../donnees/armures';
import { MODELES } from '../donnees/modeles';
import type { IdMonde, PieceArmure } from '../donnees/types';
import { texturePiece } from '../ui/iconesObjets';
import { compilerModele, type ModeleCompile } from './compilation';
import { creerPoseModele, ModeleInstancie, type PoseModele } from './rendu';

interface PieceRendue {
  readonly rendu: ModeleInstancie;
  readonly pose: PoseModele;
  /** Index, dans le modèle du joueur, de chaque partie de la pièce (−1 : absente). */
  readonly correspondance: Int32Array;
}

/** Modèle de chaque pièce visible (le talisman ne se voit pas). */
const MODELES_PIECES = {
  casque: 'armure_casque',
  plastron: 'armure_plastron',
  jambieres: 'armure_jambieres',
  bottes: 'armure_bottes',
  bouclier: 'armure_bouclier',
  cape: 'armure_cape',
  talisman: null,
} as const satisfies Record<PieceArmure, keyof typeof MODELES | null>;

export class ArmureHeros {
  private readonly pieces: PieceRendue[] = [];

  /** `portees` : monde de la pièce portée à chaque emplacement (absent : rien). */
  constructor(joueur: ModeleCompile, portees: Partial<Record<PieceArmure, IdMonde>>, textures: BibliothequeTextures, scene: THREE.Scene) {
    for (const piece of PIECES_ARMURE) {
      const monde = portees[piece];
      if (!monde) continue;
      const id = MODELES_PIECES[piece];
      if (!id) continue;
      const modele = compilerModele(MODELES[id]);
      const rendu = new ModeleInstancie(modele, { modele: id, skin: texturePiece(monde, piece), couleursSecours: {}, couleurSecoursParDefaut: 0x9a9aa0, rayonOmbre: 0 }, textures, 1, scene);
      const correspondance = Int32Array.from(modele.parties, (p) => joueur.parties.findIndex((q) => q.nom === p.nom));
      this.pieces.push({ rendu, pose: creerPoseModele(modele), correspondance });
    }
  }

  get vide(): boolean {
    return this.pieces.length === 0;
  }

  /** Dessine l'armure sur le joueur posé par `pose` (même place, mêmes rotations, même surcouche). */
  poser(pose: PoseModele): void {
    for (let i = 0; i < this.pieces.length; i++) {
      const p = this.pieces[i];
      const cible = p.pose;
      cible.x = pose.x;
      cible.y = pose.y;
      cible.z = pose.z;
      cible.angle = pose.angle;
      cible.inclinaison = pose.inclinaison;
      cible.echelleXZ = pose.echelleXZ;
      cible.echelleY = pose.echelleY;
      cible.surcouche[0] = pose.surcouche[0];
      cible.surcouche[1] = pose.surcouche[1];
      cible.surcouche[2] = pose.surcouche[2];
      cible.surcouche[3] = pose.surcouche[3];
      for (let k = 0; k < p.correspondance.length; k++) {
        const source = p.correspondance[k];
        if (source < 0) continue;
        cible.rotations[k * 3] = pose.rotations[source * 3];
        cible.rotations[k * 3 + 1] = pose.rotations[source * 3 + 1];
        cible.rotations[k * 3 + 2] = pose.rotations[source * 3 + 2];
      }
      p.rendu.commencer();
      p.rendu.ajouter(cible);
      p.rendu.terminer();
    }
  }
}
