/**
 * Bandeau de maîtrise d'une classe (menu, talents, fin de partie) : fiole
 * d'expérience, « Maîtrise 4 », barre d'XP vers le niveau suivant (verte
 * comme la barre d'expérience du jeu) et, si demandé, les points de talent
 * encore libres.
 */
import type { BibliothequeTextures } from '../assets/textures';
import { MAITRISE } from '../donnees/talents';
import type { EtatMaitrise } from '../progression/sauvegarde';

const FORMAT = new Intl.NumberFormat('fr-FR');

/** Part (0 à 1) du niveau en cours déjà remplie. */
export function partNiveau(m: EtatMaitrise): number {
  return (m.xp - m.debutNiveau) / Math.max(1, m.finNiveau - m.debutNiveau);
}

/** Bandeau ; `points` : points libres à afficher (null : pas de compteur). */
export function bandeauMaitrise(textures: BibliothequeTextures, titre: string, m: EtatMaitrise, points: number | null): HTMLElement {
  const bandeau = document.createElement('div');
  bandeau.className = 'maitrise';
  const url = textures.url(MAITRISE.icone);
  if (url) {
    const icone = document.createElement('img');
    icone.src = url;
    icone.alt = '';
    icone.className = 'maitrise-icone';
    bandeau.append(icone);
  }
  const textes = document.createElement('div');
  textes.className = 'maitrise-textes';
  const ligne = document.createElement('div');
  ligne.className = 'maitrise-ligne';
  const nom = document.createElement('span');
  nom.className = 'maitrise-nom';
  nom.textContent = `${titre} · Maîtrise ${m.niveau}`;
  ligne.append(nom);
  if (points !== null) {
    const libres = document.createElement('span');
    libres.className = points > 0 ? 'maitrise-points disponibles' : 'maitrise-points';
    libres.textContent = `${points} point${points > 1 ? 's' : ''}`;
    ligne.append(libres);
  }
  const barre = document.createElement('div');
  barre.className = 'maitrise-barre';
  const rempli = document.createElement('div');
  rempli.className = 'maitrise-rempli';
  rempli.style.width = `${Math.round(partNiveau(m) * 1000) / 10}%`;
  barre.append(rempli);
  barre.title = `${FORMAT.format(m.xp - m.debutNiveau)} / ${FORMAT.format(m.finNiveau - m.debutNiveau)} XP vers le niveau ${m.niveau + 1}`;
  textes.append(ligne, barre);
  bandeau.append(textes);
  return bandeau;
}
