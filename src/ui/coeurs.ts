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
  private readonly conteneur = element('hud-coeurs');
  private readonly pleins: HTMLImageElement[] = [];
  private readonly urlConteneur: string;
  private readonly urlPlein: string;
  private readonly urlMoitie: string;
  private affiche = -1;
  private pvMaxAffiche = -1;
  private minuterieTremblement = 0;
  private tremble = false;

  constructor(textures: BibliothequeTextures) {
    const sprites = CONFIG.hud.coeurs;
    this.urlConteneur = textures.url(sprites.conteneur) ?? dessinerCoeur('#1a1a1a', '#4a1010', 9);
    this.urlPlein = textures.url(sprites.plein) ?? dessinerCoeur(null, '#e01b1b', 9);
    this.urlMoitie = textures.url(sprites.moitie) ?? dessinerCoeur(null, '#e01b1b', 5);
  }

  /** Met à jour l'affichage (ne touche le DOM que si les PV ont changé, ou pour le tremblement). */
  afficher(pv: number, pvMax: number, dt: number): void {
    if (pvMax !== this.pvMaxAffiche) this.construire(pvMax);
    this.trembler(pv > 0 && pv <= CONFIG.hud.tremblement.seuilPv, dt);
    const valeur = Math.ceil(pv);
    if (valeur === this.affiche) return;
    this.affiche = valeur;
    this.pleins.forEach((image, i) => {
      const reste = valeur - i * 2;
      image.style.visibility = reste > 0 ? 'visible' : 'hidden';
      if (reste > 0) image.src = reste >= 2 ? this.urlPlein : this.urlMoitie;
    });
  }

  /** Santé basse : chaque cœur saute d'un pixel au hasard, plusieurs fois par seconde. */
  private trembler(actif: boolean, dt: number): void {
    const t = CONFIG.hud.tremblement;
    if (!actif) {
      if (this.tremble) {
        this.tremble = false;
        for (const coeur of this.conteneur.children) (coeur as HTMLElement).style.transform = '';
      }
      return;
    }
    this.tremble = true;
    this.minuterieTremblement -= dt;
    if (this.minuterieTremblement > 0) return;
    this.minuterieTremblement = t.periode;
    for (const coeur of this.conteneur.children) {
      (coeur as HTMLElement).style.transform = Math.random() < 0.5 ? `translateY(-${t.amplitude}px)` : '';
    }
  }

  /** Un emplacement de cœur par tranche de 2 PV maximum. */
  private construire(pvMax: number): void {
    this.pvMaxAffiche = pvMax;
    this.affiche = -1;
    this.pleins.length = 0;
    this.conteneur.replaceChildren();
    for (let i = 0; i < Math.ceil(pvMax / 2); i++) {
      const coeur = document.createElement('span');
      coeur.className = 'coeur';
      const fond = document.createElement('img');
      fond.src = this.urlConteneur;
      fond.alt = '';
      const plein = document.createElement('img');
      plein.alt = '';
      coeur.append(fond, plein);
      this.conteneur.append(coeur);
      this.pleins.push(plein);
    }
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
