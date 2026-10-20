/**
 * Écran de montée de niveau : 3 cartes d'amélioration, choisies à la souris
 * ou avec les touches 1, 2, 3 (rangée du haut ou pavé numérique).
 */
import type { BibliothequeTextures } from '../assets/textures';
import type { Carte } from '../progression/ameliorations';
import { element } from './dom';

const TOUCHES: Readonly<Record<string, number>> = {
  Digit1: 0, Digit2: 1, Digit3: 2,
  Numpad1: 0, Numpad2: 1, Numpad3: 2,
};

export class EcranCartes {
  private readonly racine = element('cartes');
  private readonly titre = element('cartes-titre');
  private readonly liste = element('cartes-liste');
  private readonly textures: BibliothequeTextures;
  private cartes: readonly Carte[] = [];
  private surChoix: ((carte: Carte) => void) | null = null;

  constructor(textures: BibliothequeTextures) {
    this.textures = textures;
    window.addEventListener('keydown', this.surTouche);
  }

  get ouvert(): boolean {
    return this.surChoix !== null;
  }

  ouvrir(niveau: number, cartes: readonly Carte[], surChoix: (carte: Carte) => void): void {
    this.cartes = cartes;
    this.surChoix = surChoix;
    this.titre.textContent = `Niveau ${niveau} !`;
    this.liste.replaceChildren(...cartes.map((carte, i) => this.creerCarte(carte, i)));
    this.racine.classList.remove('cache');
  }

  private fermer(): void {
    this.surChoix = null;
    this.racine.classList.add('cache');
  }

  private choisir(index: number): void {
    const carte = this.cartes[index];
    const rappel = this.surChoix;
    if (!carte || !rappel) return;
    this.fermer();
    rappel(carte);
  }

  private readonly surTouche = (e: KeyboardEvent): void => {
    if (!this.ouvert) return;
    const index = TOUCHES[e.code];
    if (index === undefined) return;
    e.preventDefault();
    this.choisir(index);
  };

  private creerCarte(carte: Carte, index: number): HTMLElement {
    const bouton = document.createElement('button');
    bouton.className = 'carte';
    bouton.addEventListener('click', () => this.choisir(index));

    const touche = document.createElement('span');
    touche.className = 'carte-touche';
    touche.textContent = String(index + 1);

    const icone = document.createElement('div');
    icone.className = 'carte-icone';
    const url = this.textures.url(carte.icone);
    if (url) {
      const image = document.createElement('img');
      image.src = url;
      image.alt = '';
      icone.append(image);
    } else {
      icone.textContent = carte.nom.charAt(0);
    }

    const nom = document.createElement('div');
    nom.className = 'carte-nom';
    nom.textContent = carte.nom;

    const etiquette = document.createElement('div');
    etiquette.className = 'carte-etiquette';
    etiquette.textContent = carte.etiquette;

    const description = document.createElement('div');
    description.className = 'carte-description';
    description.textContent = carte.description;

    const lignes = document.createElement('ul');
    lignes.className = 'carte-stats';
    for (const ligne of carte.lignes) {
      const item = document.createElement('li');
      item.textContent = ligne;
      lignes.append(item);
    }

    bouton.append(touche, icone, nom, etiquette, description, lignes);
    return bouton;
  }
}
