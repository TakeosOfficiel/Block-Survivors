/**
 * Skin de joueur 64×64 dessiné à partir des traits de l'éditeur (donnees/traits.ts),
 * au format du jeu : chaque partie du modèle est une boîte dont les six faces
 * sont disposées en patron dans l'image. Un léger grain par pixel donne le
 * rendu « fait main » des skins vanilla.
 */
import { TRAITS, type ChoixCouleur, type ChoixStyle, type StyleBarbe, type StyleCoiffure, type Traits } from '../donnees/traits';

const TAILLE = 64;

/** Faces d'une boîte du patron : [x, y, largeur, hauteur]. */
type Rect = readonly [number, number, number, number];
interface Boite {
  readonly dessus: Rect;
  readonly dessous: Rect;
  /** Faces verticales dans l'ordre du patron : côté droit, avant, côté gauche, arrière. */
  readonly droite: Rect;
  readonly avant: Rect;
  readonly gauche: Rect;
  readonly arriere: Rect;
}

/** Patron d'une boîte (u, v) de largeur l, hauteur h, profondeur p, comme dans le jeu. */
function boite(u: number, v: number, l: number, h: number, p: number): Boite {
  return {
    dessus: [u + p, v, l, p],
    dessous: [u + p + l, v, l, p],
    droite: [u, v + p, p, h],
    avant: [u + p, v + p, l, h],
    gauche: [u + p + l, v + p, p, h],
    arriere: [u + 2 * p + l, v + p, l, h],
  };
}

class Toile {
  readonly pixels = new Uint8ClampedArray(TAILLE * TAILLE * 4);

  /** Pixel opaque, avec un grain stable (le même pixel garde la même nuance). */
  point(x: number, y: number, couleur: number, nuance = 1): void {
    if (x < 0 || y < 0 || x >= TAILLE || y >= TAILLE) return;
    const grain = nuance * (0.93 + 0.1 * bruit(x, y));
    const i = (y * TAILLE + x) * 4;
    this.pixels[i] = ((couleur >> 16) & 255) * grain;
    this.pixels[i + 1] = ((couleur >> 8) & 255) * grain;
    this.pixels[i + 2] = (couleur & 255) * grain;
    this.pixels[i + 3] = 255;
  }

  rect([x, y, l, h]: Rect, couleur: number, nuance = 1): void {
    for (let j = 0; j < h; j++) for (let i = 0; i < l; i++) this.point(x + i, y + j, couleur, nuance);
  }

  /** Pixel (col, ligne) d'une face. */
  dans([x, y]: Rect, col: number, ligne: number, couleur: number, nuance = 1): void {
    this.point(x + col, y + ligne, couleur, nuance);
  }

  /** Lignes [debut, fin[ des quatre faces verticales. */
  bandeau(b: Boite, debut: number, fin: number, couleur: number, nuance = 1): void {
    for (const face of [b.droite, b.avant, b.gauche, b.arriere]) {
      this.rect([face[0], face[1] + debut, face[2], fin - debut], couleur, nuance);
    }
  }

  boite(b: Boite, couleur: number): void {
    for (const face of [b.dessus, b.dessous, b.droite, b.avant, b.gauche, b.arriere]) this.rect(face, couleur);
  }
}

/** Valeur pseudo-aléatoire stable entre 0 et 1 pour un pixel. */
function bruit(x: number, y: number): number {
  const n = Math.sin(x * 12.9898 + y * 78.233) * 43758.5453;
  return n - Math.floor(n);
}

function melanger(a: number, b: number, t: number): number {
  const c = (d: number) => Math.round(((a >> d) & 255) * (1 - t) + ((b >> d) & 255) * t);
  return (c(16) << 16) | (c(8) << 8) | c(0);
}

const couleur = (trait: 'peau' | 'yeux' | 'cheveux' | 'couleurHaut' | 'pantalon' | 'chaussures', traits: Traits) =>
  (TRAITS[trait].choix as readonly ChoixCouleur[])[traits[trait]]?.couleur ?? 0x808080;

const style = <S extends string>(choix: readonly ChoixStyle<S>[], index: number): S => (choix[index] ?? choix[0]).style;

/** Dessine le skin et le renvoie dans un canvas 64×64. */
export function dessinerSkinPerso(traits: Traits, brasFins: boolean): HTMLCanvasElement {
  const t = new Toile();
  const peau = couleur('peau', traits);
  const cheveux = couleur('cheveux', traits);
  const haut = couleur('couleurHaut', traits);
  const pantalon = couleur('pantalon', traits);
  const chaussures = couleur('chaussures', traits);
  const bras = brasFins ? 3 : 4;

  const tete = boite(0, 0, 8, 8, 8);
  const chapeau = boite(32, 0, 8, 8, 8);
  const corps = boite(16, 16, 8, 12, 4);
  const veste = boite(16, 32, 8, 12, 4);
  const brasDroit = boite(40, 16, bras, 12, 4);
  const brasGauche = boite(32, 48, bras, 12, 4);
  const jambeDroite = boite(0, 16, 4, 12, 4);
  const jambeGauche = boite(16, 48, 4, 12, 4);

  // --- Tête et visage --------------------------------------------------------
  t.boite(tete, peau);
  const yeux = couleur('yeux', traits);
  t.dans(tete.avant, 1, 4, 0xffffff);
  t.dans(tete.avant, 2, 4, yeux);
  t.dans(tete.avant, 5, 4, yeux);
  t.dans(tete.avant, 6, 4, 0xffffff);
  t.dans(tete.avant, 3, 5, peau, 0.86);   // nez
  t.dans(tete.avant, 4, 5, peau, 0.86);
  t.dans(tete.avant, 3, 6, melanger(peau, 0x6a2a20, 0.55));  // bouche
  t.dans(tete.avant, 4, 6, melanger(peau, 0x6a2a20, 0.55));

  dessinerCoiffure(t, style(TRAITS.coiffure.choix, traits.coiffure), cheveux, tete, chapeau, veste);
  dessinerBarbe(t, style(TRAITS.barbe.choix, traits.barbe), cheveux, tete, veste);

  // --- Haut ------------------------------------------------------------------
  const styleHaut = style(TRAITS.haut.choix, traits.haut);
  t.boite(corps, haut);
  for (const b of [brasDroit, brasGauche]) {
    t.boite(b, peau);
    if (styleHaut === 'tshirt') {
      t.bandeau(b, 0, 4, haut);
      t.rect(b.dessus, haut);
    } else {
      t.bandeau(b, 0, 11, haut);
      t.rect(b.dessus, haut);
      if (styleHaut === 'sweat') t.bandeau(b, 9, 11, haut, 0.82);  // poignets resserrés
    }
  }
  const sombre = 0.72;
  switch (styleHaut) {
    case 'tshirt':
      for (let c = 2; c < 6; c++) t.dans(corps.avant, c, 0, peau);  // col rond
      break;
    case 'chemise':
      for (let c = 2; c < 6; c++) t.dans(corps.avant, c, 0, melanger(haut, 0xffffff, 0.45));  // col
      for (let l = 2; l < 11; l += 3) t.dans(corps.avant, 4, l, haut, sombre);          // boutons
      break;
    case 'sweat':
      t.rect([corps.avant[0] + 1, corps.avant[1] + 7, 6, 3], haut, 0.85);  // poche ventrale
      for (let c = 2; c < 6; c++) t.dans(corps.avant, c, 0, haut, sombre);
      t.dans(corps.avant, 3, 1, 0xe8e8e8);  // cordons de la capuche
      t.dans(corps.avant, 4, 1, 0xe8e8e8);
      break;
    case 'veste':
      // Veste ouverte sur un t-shirt clair
      for (let l = 0; l < 12; l++) {
        t.dans(corps.avant, 3, l, 0xe6e6e6);
        t.dans(corps.avant, 4, l, 0xe6e6e6);
        t.dans(corps.avant, 2, l, haut, sombre);
        t.dans(corps.avant, 5, l, haut, sombre);
      }
      break;
  }
  // Ceinture
  t.bandeau(corps, 11, 12, pantalon, 0.7);

  // --- Jambes ------------------------------------------------------------------
  for (const b of [jambeDroite, jambeGauche]) {
    t.boite(b, pantalon);
    t.bandeau(b, 9, 12, chaussures);
    t.bandeau(b, 11, 12, chaussures, 0.75);  // semelle
    t.rect(b.dessous, chaussures, 0.75);
  }

  const toile = document.createElement('canvas');
  toile.width = TAILLE;
  toile.height = TAILLE;
  toile.getContext('2d')!.putImageData(new ImageData(t.pixels, TAILLE, TAILLE), 0, 0);
  return toile;
}

/**
 * Cheveux : dessus de la tête, frange, côtés et arrière selon la coiffure ;
 * la seconde couche (chapeau, veste) donne du volume aux coiffures épaisses.
 * Sur les côtés, la colonne la plus proche du visage est la dernière du côté
 * droit et la première du côté gauche.
 */
function dessinerCoiffure(t: Toile, s: StyleCoiffure, c: number, tete: Boite, chapeau: Boite, veste: Boite): void {
  if (s === 'chauve') return;
  t.rect(tete.dessus, c);
  // Côtés sur `lignes` lignes ; la dernière ne couvre que l'arrière (dégagé autour de l'oreille)
  const cotes = (lignes: number) => {
    for (let l = 0; l < lignes; l++) {
      for (let col = 0; col < 8; col++) {
        const distanceAvantDroit = 7 - col;  // côté droit : colonne 7 = avant
        const distanceAvantGauche = col;     // côté gauche : colonne 0 = avant
        if (l < lignes - 1 || distanceAvantDroit > 2) t.dans(tete.droite, col, l, c);
        if (l < lignes - 1 || distanceAvantGauche > 2) t.dans(tete.gauche, col, l, c);
      }
    }
  };
  // Frange d'une ligne et arrière sur 4 lignes, communs à toutes les coiffures
  for (let col = 0; col < 8; col++) t.dans(tete.avant, col, 0, c);
  t.rect([tete.arriere[0], tete.arriere[1], 8, 4], c);

  switch (s) {
    case 'courte':
      cotes(3);
      break;
    case 'meche':
      cotes(3);
      for (let col = 0; col < 5; col++) t.dans(tete.avant, col, 1, c);
      t.dans(tete.avant, 0, 2, c);
      t.dans(tete.avant, 1, 2, c);
      break;
    case 'longue':
      cotes(8);
      t.rect(tete.arriere, c);
      for (let col = 0; col < 8; col++) t.dans(tete.avant, col, 1, c, 0.92);
      for (let l = 0; l < 8; l++) {
        t.dans(tete.avant, 0, l, c);
        t.dans(tete.avant, 7, l, c);
      }
      // Cheveux qui tombent dans le dos (seconde couche du corps)
      t.rect([veste.arriere[0], veste.arriere[1], 8, 5], c);
      break;
    case 'herissee':
      cotes(3);
      t.rect(chapeau.dessus, c);
      for (let col = 0; col < 8; col += 2) {
        t.dans(chapeau.avant, col, 0, c, 1.08);
        t.dans(chapeau.droite, col, 0, c, 1.08);
        t.dans(chapeau.gauche, col, 0, c, 1.08);
        t.dans(chapeau.arriere, col, 0, c, 1.08);
      }
      break;
    case 'queue':
      cotes(3);
      t.rect([tete.arriere[0], tete.arriere[1], 8, 5], c);
      // Queue de cheval : une mèche qui pend derrière (seconde couche)
      for (let l = 2; l < 8; l++) {
        t.dans(chapeau.arriere, 3, l, c, 0.9);
        t.dans(chapeau.arriere, 4, l, c, 0.9);
      }
      t.rect([veste.arriere[0] + 3, veste.arriere[1], 2, 3], c, 0.9);
      break;
    case 'bouclee':
      cotes(4);
      t.rect(chapeau.dessus, c);
      for (let l = 0; l < 5; l++) {
        for (let col = 0; col < 8; col++) {
          // Boucles : volume irrégulier sur les côtés et l'arrière
          if ((col + l) % 3 === 2 && l > 2) continue;
          t.dans(chapeau.droite, col, l, c, 0.95);
          t.dans(chapeau.gauche, col, l, c, 0.95);
          t.dans(chapeau.arriere, col, l + 1, c, 0.95);
        }
      }
      for (let col = 0; col < 8; col++) t.dans(chapeau.avant, col, 0, c, 0.95);
      break;
  }
}

/** Barbe et moustache, de la couleur des cheveux, sur l'avant (et le bas des côtés). */
function dessinerBarbe(t: Toile, s: StyleBarbe, c: number, tete: Boite, veste: Boite): void {
  if (s === 'aucune') return;
  const a = tete.avant;
  if (s === 'moustache' || s === 'bouc') {
    for (let col = 2; col < 6; col++) t.dans(a, col, 6, c);
    if (s === 'bouc') {
      t.dans(a, 3, 7, c);
      t.dans(a, 4, 7, c);
    }
    return;
  }
  // Barbe : joues, menton, et la mâchoire sur les côtés (près du visage)
  t.dans(a, 0, 5, c);
  t.dans(a, 1, 5, c);
  t.dans(a, 6, 5, c);
  t.dans(a, 7, 5, c);
  for (let col = 0; col < 8; col++) {
    if (col !== 3 && col !== 4) t.dans(a, col, 6, c);
    t.dans(a, col, 7, c);
  }
  for (let l = 5; l < 8; l++) {
    for (let k = 0; k < 3; k++) {
      t.dans(tete.droite, 7 - k, l, c);
      t.dans(tete.gauche, k, l, c);
    }
  }
  t.rect(tete.dessous, c, 0.9);
  if (s === 'longue') t.rect([veste.avant[0] + 2, veste.avant[1], 4, 3], c);  // tombe sur la poitrine
}
