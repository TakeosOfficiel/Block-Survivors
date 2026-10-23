/**
 * Icônes des bagues et amulettes, dessinées en pixel art au lancement (16×16,
 * comme les objets du jeu) : anneau de métal et gemme sertie, ou chaîne et
 * pendentif, contour sombre. Enregistrées comme textures (`bs/objets/<id>`),
 * elles s'affichent partout comme les autres icônes.
 *
 * Aussi les armures de la Collection (`bs/armures/…`) : cuir teint (texture du
 * jeu multipliée par la teinte, puis son calque non teint), et bouclier, cape
 * et talisman dessinés aux couleurs de leur monde.
 */
import type { BibliothequeTextures } from '../assets/textures';
import { ARMURES, estPieceJeu, iconeArmure, MONDES, PIECES_JEU, textureArmure } from '../donnees/armures';
import { IDS_OBJETS_EQUIPEMENT, OBJETS_EQUIPEMENT } from '../donnees/equipement';
import type { CheminTexture, DefinitionArmure, IconeObjet, IdMonde, PieceArmure } from '../donnees/types';
import { cheminIconeObjet } from '../progression/equipement';

const TAILLE = 16;
const CONTOUR = '#1a1206';
/** Métal : clair, moyen, sombre. */
const METAUX: Readonly<Record<IconeObjet['metal'], readonly [string, string, string]>> = {
  fer: ['#f0f0f0', '#b8b8b8', '#6a6a6a'],
  or: ['#fff2a0', '#e8b830', '#946a10'],
  cuivre: ['#ffc08a', '#d0784a', '#7a3a20'],
  netherite: ['#8a7a7a', '#4e4444', '#262020'],
};

function melanger(couleur: number, vers: number, part: number): string {
  const canal = (decalage: number) => {
    const a = (couleur >> decalage) & 0xff;
    const b = (vers >> decalage) & 0xff;
    return Math.round(a + (b - a) * part);
  };
  return `rgb(${canal(16)}, ${canal(8)}, ${canal(0)})`;
}

/** Grille 16×16 de couleurs (null : transparent), puis contour sombre autour de ce qui est dessiné. */
function dessiner(icone: IconeObjet): HTMLCanvasElement {
  const grille: (string | null)[] = new Array(TAILLE * TAILLE).fill(null);
  const poser = (x: number, y: number, c: string) => {
    if (x >= 0 && y >= 0 && x < TAILLE && y < TAILLE) grille[y * TAILLE + x] = c;
  };
  const [clair, moyen, sombre] = METAUX[icone.metal];
  const gemme = [melanger(icone.gemme, 0xffffff, 0.45), melanger(icone.gemme, 0, 0), melanger(icone.gemme, 0, 0.45)];

  if (icone.forme === 'bague') {
    // Anneau vu de trois quarts
    for (let y = 0; y < TAILLE; y++) {
      for (let x = 0; x < TAILLE; x++) {
        const d = ((x + 0.5 - 8) / 5.5) ** 2 + ((y + 0.5 - 10.5) / 4.5) ** 2;
        if (d > 1 || d <= 0.5) continue;
        poser(x, y, y < 9 ? clair : x > 8 && y > 11 ? sombre : moyen);
      }
    }
    // Chaton et gemme en losange
    for (let x = 6; x <= 9; x++) poser(x, 7, moyen);
    losange(poser, 7.5, 4, 3, gemme);
  } else {
    // Chaîne en V, maillons alternés
    for (let k = 0; k <= 7; k++) {
      poser(2 + Math.round(k * 0.7), 1 + k, k % 2 ? sombre : clair);
      poser(13 - Math.round(k * 0.7), 1 + k, k % 2 ? sombre : clair);
    }
    // Monture de métal et pendentif
    for (let y = 8; y < TAILLE; y++) {
      for (let x = 0; x < TAILLE; x++) {
        const d = Math.abs(x + 0.5 - 8) / 3.6 + Math.abs(y + 0.5 - 12) / 4;
        if (d > 1 || d <= 0.7) continue;
        poser(x, y, y < 11 ? clair : y > 13 ? sombre : moyen);
      }
    }
    losange(poser, 7.5, 11.5, 2, gemme);
  }

  // Contour : chaque case vide qui touche un pixel dessiné
  const sortie = grille.slice();
  for (let y = 0; y < TAILLE; y++) {
    for (let x = 0; x < TAILLE; x++) {
      if (grille[y * TAILLE + x]) continue;
      const voisin = (dx: number, dy: number) => {
        const vx = x + dx;
        const vy = y + dy;
        return vx >= 0 && vy >= 0 && vx < TAILLE && vy < TAILLE && grille[vy * TAILLE + vx] !== null;
      };
      if (voisin(1, 0) || voisin(-1, 0) || voisin(0, 1) || voisin(0, -1)) sortie[y * TAILLE + x] = CONTOUR;
    }
  }
  const toile = document.createElement('canvas');
  toile.width = TAILLE;
  toile.height = TAILLE;
  const ctx = toile.getContext('2d');
  if (ctx) {
    for (let i = 0; i < sortie.length; i++) {
      const c = sortie[i];
      if (!c) continue;
      ctx.fillStyle = c;
      ctx.fillRect(i % TAILLE, Math.floor(i / TAILLE), 1, 1);
    }
  }
  return toile;
}

/** Gemme en losange de rayon `r` : reflet en haut à gauche, ombre en bas à droite, éclat blanc. */
function losange(poser: (x: number, y: number, c: string) => void, cx: number, cy: number, r: number, [clair, moyen, sombre]: string[]): void {
  for (let y = Math.floor(cy - r); y <= Math.ceil(cy + r); y++) {
    for (let x = Math.floor(cx - r); x <= Math.ceil(cx + r); x++) {
      const dx = x + 0.5 - cx - 0.5;
      const dy = y + 0.5 - cy;
      if (Math.abs(dx) + Math.abs(dy) > r) continue;
      poser(x, y, dx + dy < -0.9 ? clair : dx + dy > 0.9 ? sombre : moyen);
    }
  }
  poser(Math.round(cx - 1), Math.round(cy - 1), '#ffffff');
}

/** Dessine et enregistre l'icône de chaque objet d'équipement. */
export function enregistrerIconesObjets(textures: BibliothequeTextures): void {
  for (const id of IDS_OBJETS_EQUIPEMENT) textures.enregistrer(cheminIconeObjet(id), dessiner(OBJETS_EQUIPEMENT[id].icone));
}

// ---------------------------------------------------------------------------
// Armures de la Collection
// ---------------------------------------------------------------------------

/** Texture à poser sur le héros pour cette pièce (teinte ou pièce dessinée : copie enregistrée au lancement). */
export function texturePiece(monde: IdMonde, piece: PieceArmure): CheminTexture {
  const a = ARMURES[monde];
  if (!estPieceJeu(piece)) return `bs/armures/${monde}/${piece}`;
  return a.teinte !== undefined ? `bs/armures/${monde}/${piece === 'jambieres' ? 'jambieres' : 'corps'}` : textureArmure(a.materiau, piece);
}

/** Icône d'inventaire de la pièce (teinte comprise). */
export function iconePiece(monde: IdMonde, piece: PieceArmure): CheminTexture {
  const a = ARMURES[monde];
  return a.teinte !== undefined || !estPieceJeu(piece) ? `bs/armures/${monde}/icone-${piece}` : iconeArmure(a.materiau, piece);
}

/** Textures du jeu dont les armures ont besoin (préchargement). */
export function texturesArmures(): CheminTexture[] {
  const chemins: CheminTexture[] = [];
  for (const monde of MONDES) {
    const a = ARMURES[monde];
    for (const piece of PIECES_JEU) {
      chemins.push(textureArmure(a.materiau, piece), iconeArmure(a.materiau, piece));
      if (a.teinte !== undefined) chemins.push(`${textureArmure(a.materiau, piece)}_overlay`, `${iconeArmure(a.materiau, piece)}_overlay`);
    }
  }
  return [...new Set(chemins)];
}

/** Toutes les pièces à enregistrer au lancement : cuir teint, puis bouclier, cape et talisman de chaque monde. */
export function enregistrerArmures(textures: BibliothequeTextures): void {
  enregistrerArmuresTeintes(textures);
  for (const monde of MONDES) {
    const c = ARMURES[monde].couleurs;
    textures.enregistrer(`bs/armures/${monde}/bouclier`, textureBouclier(c));
    textures.enregistrer(`bs/armures/${monde}/cape`, textureCape(c));
    textures.enregistrer(`bs/armures/${monde}/icone-bouclier`, iconeBouclier(c));
    textures.enregistrer(`bs/armures/${monde}/icone-cape`, iconeCape(c));
    textures.enregistrer(`bs/armures/${monde}/icone-talisman`, dessiner({ forme: 'pendentif', metal: 'or', gemme: c.embleme }));
  }
}

type Couleurs = DefinitionArmure['couleurs'];
const hex = (c: number) => `#${c.toString(16).padStart(6, '0')}`;

function toile(largeur: number, hauteur: number): [HTMLCanvasElement, CanvasRenderingContext2D | null] {
  const t = document.createElement('canvas');
  t.width = largeur;
  t.height = hauteur;
  return [t, t.getContext('2d')];
}

/** Une face de blason : bordure, champ, emblème en losange (+ reflet), dans le rectangle donné. */
function blason(ctx: CanvasRenderingContext2D, x: number, y: number, l: number, h: number, c: Couleurs): void {
  ctx.fillStyle = hex(c.bordure);
  ctx.fillRect(x, y, l, h);
  ctx.fillStyle = hex(c.champ);
  ctx.fillRect(x + 1, y + 1, l - 2, h - 2);
  ctx.fillStyle = hex(c.embleme);
  const cx = x + l / 2;
  const cy = y + h / 2;
  const r = Math.max(1, Math.floor(Math.min(l, h) / 4));
  for (let dy = -r; dy <= r; dy++) {
    const demi = r - Math.abs(dy);
    ctx.fillRect(Math.floor(cx - demi - 0.5), Math.floor(cy + dy - 0.5), demi * 2 + 1, 1);
  }
}

/**
 * Bouclier : boîte de 1 × 12 × 10 px sur le côté du bras gauche (modèle armure_bouclier), planche
 * 64×32 dépliée comme le jeu (faces de 10 × 12 en (0, 10) et (11, 10)).
 */
function textureBouclier(c: Couleurs): HTMLCanvasElement {
  const [t, ctx] = toile(64, 32);
  if (!ctx) return t;
  ctx.fillStyle = hex(c.bordure);
  ctx.fillRect(0, 0, 32, 32);
  blason(ctx, 0, 10, 10, 12, c);
  blason(ctx, 11, 10, 10, 12, c);
  return t;
}

/** Cape : boîte de 10 × 16 × 1 px dans le dos (modèle armure_cape), faces en (1, 1) et (12, 1). */
function textureCape(c: Couleurs): HTMLCanvasElement {
  const [t, ctx] = toile(64, 32);
  if (!ctx) return t;
  ctx.fillStyle = hex(c.champ);
  ctx.fillRect(0, 0, 32, 32);
  for (const x of [1, 12]) {
    blason(ctx, x, 1, 10, 16, c);
    // Col et ourlet de la couleur de bordure
    ctx.fillStyle = hex(c.bordure);
    ctx.fillRect(x, 1, 10, 2);
    ctx.fillRect(x, 15, 10, 2);
  }
  return t;
}

function iconeBouclier(c: Couleurs): HTMLCanvasElement {
  const [t, ctx] = toile(16, 16);
  if (!ctx) return t;
  // Écu : rectangle à pointe arrondie, contour sombre
  for (let y = 1; y < 15; y++) {
    const retrait = y > 10 ? (y - 10) * 1.2 : 0;
    const x0 = Math.round(2 + retrait);
    const x1 = Math.round(14 - retrait);
    ctx.fillStyle = CONTOUR;
    ctx.fillRect(x0 - 1, y, x1 - x0 + 2, 1);
    ctx.fillStyle = hex(c.bordure);
    ctx.fillRect(x0, y, x1 - x0, 1);
    if (y > 1 && y < 13) {
      ctx.fillStyle = hex(c.champ);
      ctx.fillRect(x0 + 1, y, Math.max(0, x1 - x0 - 2), 1);
    }
  }
  ctx.fillStyle = CONTOUR;
  ctx.fillRect(2, 0, 12, 1);
  ctx.fillStyle = hex(c.embleme);
  for (let dy = -2; dy <= 2; dy++) ctx.fillRect(8 - (2 - Math.abs(dy)), 6 + dy, (2 - Math.abs(dy)) * 2, 1);
  return t;
}

function iconeCape(c: Couleurs): HTMLCanvasElement {
  const [t, ctx] = toile(16, 16);
  if (!ctx) return t;
  // Cape qui s'évase, col en haut
  for (let y = 2; y < 15; y++) {
    const demi = 3 + Math.round((y - 2) * 0.35);
    ctx.fillStyle = CONTOUR;
    ctx.fillRect(8 - demi - 1, y, demi * 2 + 2, 1);
    ctx.fillStyle = y < 4 || y > 12 ? hex(c.bordure) : hex(c.champ);
    ctx.fillRect(8 - demi, y, demi * 2, 1);
  }
  ctx.fillStyle = CONTOUR;
  ctx.fillRect(4, 1, 8, 1);
  ctx.fillStyle = hex(c.embleme);
  for (let dy = -2; dy <= 2; dy++) ctx.fillRect(8 - (2 - Math.abs(dy)), 8 + dy, (2 - Math.abs(dy)) * 2, 1);
  return t;
}

/** Copies teintes des armures en cuir (texture × teinte, puis calque non teint), une fois les textures chargées. */
function enregistrerArmuresTeintes(textures: BibliothequeTextures): void {  for (const monde of MONDES) {
    const a = ARMURES[monde];
    if (a.teinte === undefined) continue;
    const teinter = (base: CheminTexture, cible: CheminTexture) => {
      const image = textures.image(base);
      if (!image) return;
      const toile = document.createElement('canvas');
      toile.width = image.width;
      toile.height = image.height;
      const ctx = toile.getContext('2d');
      if (!ctx) return;
      ctx.drawImage(image, 0, 0);
      ctx.globalCompositeOperation = 'multiply';
      ctx.fillStyle = `#${(a.teinte ?? 0).toString(16).padStart(6, '0')}`;
      ctx.fillRect(0, 0, toile.width, toile.height);
      // Garde la transparence de la texture
      ctx.globalCompositeOperation = 'destination-in';
      ctx.drawImage(image, 0, 0);
      ctx.globalCompositeOperation = 'source-over';
      const calque = textures.image(`${base}_overlay`);
      if (calque) ctx.drawImage(calque, 0, 0);
      textures.enregistrer(cible, toile);
    };
    teinter(textureArmure(a.materiau, 'casque'), `bs/armures/${monde}/corps`);
    teinter(textureArmure(a.materiau, 'jambieres'), `bs/armures/${monde}/jambieres`);
    for (const piece of PIECES_JEU) teinter(iconeArmure(a.materiau, piece), `bs/armures/${monde}/icone-${piece}`);
  }
}
