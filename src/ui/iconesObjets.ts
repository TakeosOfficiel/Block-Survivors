/**
 * Icônes des bagues et amulettes, dessinées en pixel art au lancement (16×16,
 * comme les objets du jeu) : anneau de métal et gemme sertie, ou chaîne et
 * pendentif, contour sombre. Enregistrées comme textures (`bs/objets/<id>`),
 * elles s'affichent partout comme les autres icônes.
 *
 * Aussi les armures teintes (cuir du mycélien) : texture du jeu multipliée par
 * la teinte, puis son calque non teint par-dessus (`bs/armures/…`).
 */
import type { BibliothequeTextures } from '../assets/textures';
import { ARMURES, iconeArmure, MONDES, PIECES_ARMURE, textureArmure } from '../donnees/armures';
import { IDS_OBJETS_EQUIPEMENT, OBJETS_EQUIPEMENT } from '../donnees/equipement';
import type { CheminTexture, IconeObjet, IdMonde, PieceArmure } from '../donnees/types';
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

/** Texture à poser sur le héros pour cette pièce (teinte : copie teinte enregistrée au lancement). */
export function texturePiece(monde: IdMonde, piece: PieceArmure): CheminTexture {
  const a = ARMURES[monde];
  return a.teinte !== undefined ? `bs/armures/${monde}/${piece === 'jambieres' ? 'jambieres' : 'corps'}` : textureArmure(a.materiau, piece);
}

/** Icône d'inventaire de la pièce (teinte comprise). */
export function iconePiece(monde: IdMonde, piece: PieceArmure): CheminTexture {
  const a = ARMURES[monde];
  return a.teinte !== undefined ? `bs/armures/${monde}/icone-${piece}` : iconeArmure(a.materiau, piece);
}

/** Textures du jeu dont les armures ont besoin (préchargement). */
export function texturesArmures(): CheminTexture[] {
  const chemins: CheminTexture[] = [];
  for (const monde of MONDES) {
    const a = ARMURES[monde];
    for (const piece of PIECES_ARMURE) {
      chemins.push(textureArmure(a.materiau, piece), iconeArmure(a.materiau, piece));
      if (a.teinte !== undefined) chemins.push(`${textureArmure(a.materiau, piece)}_overlay`, `${iconeArmure(a.materiau, piece)}_overlay`);
    }
  }
  return [...new Set(chemins)];
}

/** Copies teintes des armures en cuir (texture × teinte, puis calque non teint), une fois les textures chargées. */
export function enregistrerArmuresTeintes(textures: BibliothequeTextures): void {
  for (const monde of MONDES) {
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
    for (const piece of PIECES_ARMURE) teinter(iconeArmure(a.materiau, piece), `bs/armures/${monde}/icone-${piece}`);
  }
}
