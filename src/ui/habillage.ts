/**
 * Habillage de l'interface : relie les sprites vanilla aux variables CSS
 * (boutons, curseur, fonds) et dessine les panneaux d'inventaire et les cases,
 * dont les textures vanilla ne se découpent pas en neuf tranches.
 */
import type { BibliothequeTextures } from '../assets/textures';
import type { CheminTexture } from '../donnees/types';

/** Sprites de l'interface, par variable CSS. */
export const SPRITES_INTERFACE = {
  '--bouton': 'gui/sprites/widget/button',
  '--bouton-survol': 'gui/sprites/widget/button_highlighted',
  '--bouton-inactif': 'gui/sprites/widget/button_disabled',
  '--curseur': 'gui/sprites/widget/slider',
  '--curseur-survol': 'gui/sprites/widget/slider_highlighted',
  '--poignee': 'gui/sprites/widget/slider_handle',
  '--poignee-survol': 'gui/sprites/widget/slider_handle_highlighted',
  '--terre': 'block/dirt',
  '--logo': 'block/stone',
} as const satisfies Record<string, CheminTexture>;

// Panneau d'inventaire 16x16 : contour noir aux coins arrondis, reflet blanc
// en haut à gauche, ombre en bas à droite (B noir, W blanc, C gris, D ombre).
const PANNEAU = [
  '..BBBBBBBBBBBB..',
  '.BWWWWWWWWWWWWB.',
  'BWWWWWWWWWWWWCDB',
  'BWWCCCCCCCCCCDDB',
  'BWWCCCCCCCCCCDDB',
  'BWWCCCCCCCCCCDDB',
  'BWWCCCCCCCCCCDDB',
  'BWWCCCCCCCCCCDDB',
  'BWWCCCCCCCCCCDDB',
  'BWWCCCCCCCCCCDDB',
  'BWWCCCCCCCCCCDDB',
  'BWWCCCCCCCCCCDDB',
  'BWWCCCCCCCCCCDDB',
  'BWCDDDDDDDDDDDDB',
  '.BDDDDDDDDDDDDB.',
  '..BBBBBBBBBBBB..',
];

// Case d'inventaire 6x6 (bord de 1) : creusée, ombre en haut à gauche
const CASE = [
  'SSSSSF',
  'SFFFFW',
  'SFFFFW',
  'SFFFFW',
  'SFFFFW',
  'FWWWWW',
];

const COULEURS: Readonly<Record<string, string>> = {
  B: '#000000', W: '#ffffff', C: '#c6c6c6', D: '#555555',
  S: '#373737', F: '#8b8b8b',
};

export function appliquerHabillage(textures: BibliothequeTextures): void {
  const style = document.documentElement.style;
  let boutons = true;
  for (const [variable, chemin] of Object.entries(SPRITES_INTERFACE)) {
    const url = textures.url(chemin);
    if (url) style.setProperty(variable, `url("${url}")`);
    else if (variable.startsWith('--bouton') || variable.startsWith('--curseur') || variable.startsWith('--poignee')) boutons = false;
  }
  // Sans les sprites de boutons, les couleurs de secours restent visibles
  document.documentElement.classList.toggle('sprites-gui', boutons);
  style.setProperty('--panneau', `url("${dessiner(PANNEAU)}")`);
  style.setProperty('--case', `url("${dessiner(CASE)}")`);
}

function dessiner(motif: readonly string[]): string {
  const canvas = document.createElement('canvas');
  canvas.width = motif[0].length;
  canvas.height = motif.length;
  const ctx = canvas.getContext('2d');
  if (!ctx) return '';
  motif.forEach((ligne, y) => {
    [...ligne].forEach((code, x) => {
      const couleur = COULEURS[code];
      if (!couleur) return;
      ctx.fillStyle = couleur;
      ctx.fillRect(x, y, 1, 1);
    });
  });
  return canvas.toDataURL();
}
