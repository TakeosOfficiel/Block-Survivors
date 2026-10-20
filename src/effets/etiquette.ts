/**
 * Étiquette de nom au-dessus du joueur, comme celles du jeu : texte blanc en
 * police pixel sur un fond noir translucide, toujours lisible (par-dessus le décor).
 */
import * as THREE from 'three';
import { SpriteNodeMaterial } from 'three/webgpu';
import { NOM_POLICE } from '../assets/police';
import { CONFIG } from '../config';

/** Pixels de canvas par pixel de police (police de 8 px). */
const ECHELLE = 4;

export class Etiquette {
  private readonly sprite: THREE.Sprite;
  private readonly vide: boolean;

  constructor(scene: THREE.Scene, texte: string) {
    this.vide = texte.length === 0;
    const toile = document.createElement('canvas');
    const ctx = toile.getContext('2d')!;
    const police = `${8 * ECHELLE}px "${NOM_POLICE}", monospace`;
    ctx.font = police;
    const largeur = Math.ceil(ctx.measureText(texte).width) + 2 * ECHELLE;
    toile.width = largeur;
    toile.height = 9 * ECHELLE;
    ctx.font = police;  // le redimensionnement du canvas remet le contexte à zéro
    ctx.fillStyle = 'rgba(0, 0, 0, 0.25)';
    ctx.fillRect(0, 0, toile.width, toile.height);
    ctx.fillStyle = '#ffffff';
    ctx.textBaseline = 'alphabetic';
    ctx.fillText(texte, ECHELLE, 8 * ECHELLE);

    const image = new THREE.CanvasTexture(toile);
    image.magFilter = THREE.NearestFilter;
    image.minFilter = THREE.NearestFilter;
    image.generateMipmaps = false;
    image.colorSpace = THREE.SRGBColorSpace;
    const materiau = new SpriteNodeMaterial({ map: image, transparent: true, depthTest: false, depthWrite: false });
    this.sprite = new THREE.Sprite(materiau);
    this.sprite.name = 'etiquette';
    this.sprite.renderOrder = 9;
    const hauteur = CONFIG.joueur.etiquette.hauteurTexte;
    this.sprite.scale.set(hauteur * toile.width / toile.height, hauteur, 1);
    this.sprite.visible = false;
    scene.add(this.sprite);
  }

  /** Place l'étiquette au-dessus de la tête (`visible` faux : cachée, ex. à la mort). */
  placer(x: number, y: number, z: number, visible: boolean): void {
    this.sprite.position.set(x, y + CONFIG.joueur.etiquette.marge, z);
    this.sprite.visible = visible && !this.vide;
  }
}
