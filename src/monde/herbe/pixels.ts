/**
 * Herbe en pixels 3D : de petites colonnes d'un pixel de côté (1/16 de bloc)
 * sortent du dessus des blocs d'herbe, avec les couleurs de sa texture, comme
 * les resource packs « 3D grass ». Le sommet des colonnes ondule à peine.
 */
import * as THREE from 'three';
import { MeshLambertNodeMaterial } from 'three/webgpu';
import { attribute, positionLocal, uniform, vec3 } from 'three/tsl';
import { lirePixels } from '../../assets/pixels';
import { CONFIG } from '../../config';
import { PIXELS_PAR_BLOC } from '../../moteur/geometrie';
import { creerAleatoire, graine } from '../../utils/aleatoire';
import type { TapisHerbe } from './herbe';
import { pliageVent } from './vent';

const PIXEL = 1 / PIXELS_PAR_BLOC;

export class Pixels implements TapisHerbe {
  readonly maillage: THREE.Mesh;
  readonly assombritSol = false;
  private readonly joueur = uniform(new THREE.Vector2());

  constructor(graineTexte: string, cases: readonly number[], dessus: THREE.Texture, scene: THREE.Scene) {
    this.maillage = new THREE.Mesh(creerGeometrie(graineTexte, cases, dessus), this.creerMateriau());
    this.maillage.name = 'herbe';
    this.maillage.frustumCulled = false;
    this.maillage.receiveShadow = true;  // ombres du soleil (shaders)
    scene.add(this.maillage);
  }

  majVisuel(x: number, z: number): void {
    this.joueur.value.set(x, z);
  }

  private creerMateriau(): MeshLambertNodeMaterial {
    const racine = attribute<'vec4'>('aRacine', 'vec4');
    const couleur = attribute<'vec3'>('aCouleur', 'vec3');
    const hauteur = racine.w;
    const { pliage, base } = pliageVent(this.joueur, racine.z);
    // Colonne : boîte unitaire posée au sol, ramenée à 1 pixel de côté ; seul son sommet bouge, très peu
    const t = positionLocal.y;
    const ondulation = pliage.mul(t.mul(hauteur).mul(0.6));
    const materiau = new MeshLambertNodeMaterial();
    materiau.positionNode = vec3(
      base.x.add(positionLocal.x.mul(PIXEL)).add(ondulation.x),
      t.mul(hauteur),
      base.y.add(positionLocal.z.mul(PIXEL)).add(ondulation.y),
    );
    materiau.colorNode = couleur;
    return materiau;
  }
}

function creerGeometrie(graineTexte: string, cases: readonly number[], dessus: THREE.Texture): THREE.InstancedBufferGeometry {
  const p = CONFIG.herbe.pixels;
  const boite = new THREE.BoxGeometry(1, 1, 1).translate(0, 0.5, 0);
  const geometrie = new THREE.InstancedBufferGeometry();
  geometrie.setIndex(boite.getIndex());
  for (const nom of ['position', 'normal'] as const) geometrie.setAttribute(nom, boite.getAttribute(nom));

  // Couleurs de la texture du dessus (en niveaux de gris dans le jeu), teintées comme l'herbe
  const image = dessus.image as HTMLImageElement;
  const donnees = lirePixels(image).data;
  const teinte = new THREE.Color(p.teinte);
  const couleurPixel = (px: number, py: number, eclat: number, sortie: number[]) => {
    const i = (py * image.width + px) * 4;
    // Texture sRGB → couleurs linéaires attendues par le matériau
    const c = new THREE.Color().setRGB(donnees[i] / 255, donnees[i + 1] / 255, donnees[i + 2] / 255, THREE.SRGBColorSpace);
    sortie.push(c.r * teinte.r * eclat, c.g * teinte.g * eclat, c.b * teinte.b * eclat);
  };

  const alea = creerAleatoire(graine(graineTexte));
  const racines: number[] = [];
  const couleurs: number[] = [];
  for (let c = 0; c < cases.length; c += 2) {
    for (let k = 0; k < p.parBloc; k++) {
      // Une colonne sur un pixel au hasard du bloc, de la couleur de ce pixel
      const px = Math.floor(alea() * PIXELS_PAR_BLOC);
      const py = Math.floor(alea() * PIXELS_PAR_BLOC);
      const pixels = p.hauteur[0] + Math.floor(alea() ** 1.8 * (p.hauteur[1] - p.hauteur[0] + 1));
      racines.push(cases[c] - 0.5 + (px + 0.5) * PIXEL, cases[c + 1] - 0.5 + (py + 0.5) * PIXEL, alea() * Math.PI * 2, pixels * PIXEL);
      couleurPixel(px % image.width, py % image.height, 1 + (pixels - 1) * 0.06, couleurs);
    }
  }
  geometrie.setAttribute('aRacine', new THREE.InstancedBufferAttribute(new Float32Array(racines), 4));
  geometrie.setAttribute('aCouleur', new THREE.InstancedBufferAttribute(new Float32Array(couleurs), 3));
  geometrie.instanceCount = racines.length / 4;
  return geometrie;
}
