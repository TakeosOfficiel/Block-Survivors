/**
 * Barre de vie en cœurs, comme en jeu : sprites vanilla (conteneur, plein,
 * moitié). S'ils sont introuvables, les cœurs sont dessinés en pixels.
 */
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { element } from './dom';

// Cœur 9x9 de secours ; X = cœur
const MOTIF = [
  '.XX...XX.',
  'XXXX.XXXX',
  'XXXXXXXXX',
  'XXXXXXXXX',
  '.XXXXXXX.',
  '..XXXXX..',
  '...XXX...',
  '....X....',
  '.........',
];

export class BarreCoeurs {
  private readonly pleins: HTMLImageElement[] = [];
  private readonly urlPlein: string;
  private readonly urlMoitie: string;
  private affiche = -1;

  constructor(textures: BibliothequeTextures, pvMax: number) {
    const sprites = CONFIG.hud.coeurs;
    const urlConteneur = textures.url(sprites.conteneur) ?? dessinerCoeur('#1a1a1a', '#4a1010', 9);
    this.urlPlein = textures.url(sprites.plein) ?? dessinerCoeur(null, '#e01b1b', 9);
    this.urlMoitie = textures.url(sprites.moitie) ?? dessinerCoeur(null, '#e01b1b', 5);

    const conteneur = element('hud-coeurs');
    for (let i = 0; i < Math.ceil(pvMax / 2); i++) {
      const coeur = document.createElement('span');
      coeur.className = 'coeur';
      const fond = document.createElement('img');
      fond.src = urlConteneur;
      fond.alt = '';
      const plein = document.createElement('img');
      plein.alt = '';
      coeur.append(fond, plein);
      conteneur.append(coeur);
      this.pleins.push(plein);
    }
  }

  /** Met à jour l'affichage (ne touche le DOM que si les PV ont changé). */
  afficher(pv: number): void {
    const valeur = Math.ceil(pv);
    if (valeur === this.affiche) return;
    this.affiche = valeur;
    this.pleins.forEach((image, i) => {
      const reste = valeur - i * 2;
      image.style.visibility = reste > 0 ? 'visible' : 'hidden';
      if (reste > 0) image.src = reste >= 2 ? this.urlPlein : this.urlMoitie;
    });
  }
}

/** Dessine le motif en image : bord (ou rien), intérieur limité aux `colonnes` premières colonnes. */
function dessinerCoeur(bord: string | null, interieur: string, colonnes: number): string {
  const canvas = document.createElement('canvas');
  canvas.width = 9;
  canvas.height = 9;
  const ctx = canvas.getContext('2d');
  if (!ctx) return '';
  const plein = (x: number, y: number) => MOTIF[y]?.[x] === 'X';
  for (let y = 0; y < 9; y++) {
    for (let x = 0; x < 9; x++) {
      if (!plein(x, y)) continue;
      const auBord = !plein(x - 1, y) || !plein(x + 1, y) || !plein(x, y - 1) || !plein(x, y + 1);
      const couleur = auBord ? bord : x < colonnes ? interieur : null;
      if (!couleur) continue;
      ctx.fillStyle = couleur;
      ctx.fillRect(x, y, 1, 1);
    }
  }
  return canvas.toDataURL();
}
