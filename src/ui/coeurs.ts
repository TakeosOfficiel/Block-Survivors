/**
 * Barre de vie en cœurs, comme en jeu : sprites vanilla (conteneur, plein).
 * Chaque cœur se remplit pixel par pixel (7 colonnes intérieures), ce qui
 * rend visibles les petites régénérations ; les PV exacts s'affichent à côté.
 * Si les sprites sont introuvables, les cœurs sont dessinés en pixels.
 */
import type { BibliothequeTextures } from '../assets/textures';
import { CONFIG } from '../config';
import { element } from './dom';

const FORMAT = new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 1 });

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
  /** Colonnes de pixels remplies de chaque cœur (0 à 7), pour ne toucher le DOM qu'au changement. */
  private colonnes: number[] = [];
  private readonly texte = element('hud-pv');
  private readonly urlConteneur: string;
  private readonly urlPlein: string;
  private affiche = -1;
  private pvMaxAffiche = -1;
  private minuterieTremblement = 0;
  private tremble = false;

  constructor(textures: BibliothequeTextures) {
    const sprites = CONFIG.hud.coeurs;
    this.urlConteneur = textures.url(sprites.conteneur) ?? dessinerCoeur('#1a1a1a', '#4a1010', 9);
    this.urlPlein = textures.url(sprites.plein) ?? dessinerCoeur(null, '#e01b1b', 9);
  }

  /** Met à jour l'affichage (ne touche le DOM que si les PV ont changé, ou pour le tremblement). */
  afficher(pv: number, pvMax: number, dt: number): void {
    if (pvMax !== this.pvMaxAffiche) this.construire(pvMax);
    this.trembler(pv > 0 && pv <= CONFIG.hud.tremblement.seuilPv, dt);
    // PV exacts au dixième (« 17,4 / 24 »)
    const dixiemes = Math.ceil(pv * 10 - 1e-6);
    if (dixiemes !== this.affiche) {
      this.affiche = dixiemes;
      this.texte.textContent = `${FORMAT.format(dixiemes / 10)} / ${FORMAT.format(pvMax)}`;
    }
    // Chaque cœur vaut 2 PV ; son intérieur fait 7 pixels de large (bords exclus)
    for (let i = 0; i < this.pleins.length; i++) {
      const part = Math.min(1, Math.max(0, (pv - i * 2) / 2));
      let colonnes = Math.round(part * 7);
      if (part > 0 && colonnes === 0) colonnes = 1;
      if (colonnes === this.colonnes[i]) continue;
      this.colonnes[i] = colonnes;
      const image = this.pleins[i];
      image.style.visibility = colonnes > 0 ? 'visible' : 'hidden';
      // Coupe à droite : bord gauche + colonnes remplies sur les 9 pixels du sprite
      image.style.clipPath = colonnes >= 7 ? '' : `inset(0 ${((8 - colonnes) / 9) * 100}% 0 0)`;
    }
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
    this.colonnes = [];
    this.conteneur.replaceChildren();
    for (let i = 0; i < Math.ceil(pvMax / 2); i++) {
      const coeur = document.createElement('span');
      coeur.className = 'coeur';
      const fond = document.createElement('img');
      fond.src = this.urlConteneur;
      fond.alt = '';
      const plein = document.createElement('img');
      plein.alt = '';
      plein.src = this.urlPlein;
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
